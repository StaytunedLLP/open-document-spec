// In-memory tree fingerprinting for live rename detection (e.g. `ods watch`).
//
// Session-only: compare previous scan to current disk. No on-disk snapshot product.
// Does not use Git.

use crate::fs::{load_workspace, path_matches_workspace_ignore, should_ignore_name};
use crate::mv::PathChange;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};

/// Content fingerprint map: workspace-relative path → hash of file bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TreeSnapshot {
    /// Relative paths of non-index `.md` files under the workspace root.
    pub files: BTreeMap<PathBuf, u64>,
}

/// Live watch state: last full scan plus paths that disappeared without a
/// matching create yet (FS often delivers delete and create in separate
/// debounced batches — without this, renames are lost).
#[derive(Debug, Clone, Default)]
pub struct WatchTree {
    pub snapshot: TreeSnapshot,
    /// Recently removed path → content hash, kept for rename pairing.
    pub pending_removed: BTreeMap<PathBuf, u64>,
}

impl WatchTree {
    pub fn from_scan(snapshot: TreeSnapshot) -> Self {
        Self {
            snapshot,
            pending_removed: BTreeMap::new(),
        }
    }

    /// Previous tree for [`observe_renames`]: live files + unpaired removals.
    pub fn effective_previous(&self) -> TreeSnapshot {
        let mut files = self.snapshot.files.clone();
        for (path, hash) in &self.pending_removed {
            files.entry(path.clone()).or_insert(*hash);
        }
        TreeSnapshot { files }
    }

    /// After a scan (and optional apply), update snapshot and pending removals.
    ///
    /// `paired_from` = relative paths that were rename sources this tick (drop
    /// them from pending; they are not true deletes).
    pub fn commit_scan(&mut self, current: TreeSnapshot, paired_from: &[PathBuf]) {
        let paired: BTreeSet<_> = paired_from.iter().cloned().collect();

        // Paths that vanished this tick (or were already pending) and were not rename sources.
        let mut next_pending = BTreeMap::new();
        for (path, hash) in self.effective_previous().files {
            if current.files.contains_key(&path) {
                continue;
            }
            // File move exact match, or under a DirMoved source prefix.
            if paired.iter().any(|p| path == *p || path.starts_with(p)) {
                continue;
            }
            next_pending.insert(path, hash);
        }
        // Cap pending size — extreme churn should not grow forever.
        if next_pending.len() > 10_000 {
            next_pending.clear();
        }
        self.pending_removed = next_pending;
        self.snapshot = current;
    }
}

/// Relative `from` paths for file/dir moves in `changes` (for pending cleanup).
pub fn paired_from_paths(changes: &[PathChange]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for change in changes {
        match change {
            PathChange::FileMoved { from, .. } => out.push(from.clone()),
            PathChange::DirMoved { from, .. } => out.push(from.clone()),
        }
    }
    out
}

/// Scan workspace markdown files plus explicitly referenced code files.
///
/// Markdown scan skips `index.md`, ignored names, and workspace `ignore:` prefixes.
/// Code files are added from parsed `code[].path` entries even when they live
/// under ignored implementation trees such as `src/`. When a declared code path
/// disappears, same-named candidates are scanned so rename pairing can see the
/// newly moved file.
pub fn scan_markdown_tree(
    root: impl AsRef<Path>,
    workspace_ignore: &[String],
) -> io::Result<TreeSnapshot> {
    let root = root.as_ref();
    let code_paths = load_workspace(root).map(|w| w.code_paths).unwrap_or_default();
    scan_markdown_tree_with_code_paths(root, workspace_ignore, &code_paths)
}

/// Same as [`scan_markdown_tree`], but takes already-known `code_paths` instead
/// of loading a fresh `Workspace` internally — for callers (e.g. the watch/serve
/// tick loop) that already have a `Workspace` in scope, avoiding a redundant
/// full reload+reparse of the whole tree just to read one field off it.
pub fn scan_markdown_tree_with_code_paths(
    root: impl AsRef<Path>,
    workspace_ignore: &[String],
    code_paths: &HashSet<PathBuf>,
) -> io::Result<TreeSnapshot> {
    let root = root.as_ref();
    let mut files = BTreeMap::new();
    collect_md(root, root, workspace_ignore, &mut files)?;
    for path in code_paths {
        if path.is_file() {
            insert_hashed_path(root, path, &mut files)?;
        } else if let Some(name) = path.file_name() {
            collect_same_name_candidates(root, root, name, &mut files)?;
        }
    }
    Ok(TreeSnapshot { files })
}

fn collect_md(
    root: &Path,
    dir: &Path,
    workspace_ignore: &[String],
    out: &mut BTreeMap<PathBuf, u64>,
) -> io::Result<()> {
    let mut entries = match fs::read_dir(dir) {
        Ok(rd) => rd.collect::<Result<Vec<_>, _>>()?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        if should_ignore_name(&name) {
            continue;
        }
        if path_matches_workspace_ignore(root, &path, workspace_ignore) {
            continue;
        }
        let ft = entry.file_type()?;
        if ft.is_dir() {
            collect_md(root, &path, workspace_ignore, out)?;
        } else if ft.is_file() && path.extension().is_some_and(|e| e == "md") {
            // Generated indexes are not rename-paired; they refresh from the tree.
            if name.to_str() == Some("index.md") {
                continue;
            }
            insert_hashed_path(root, &path, out)?;
        }
    }
    Ok(())
}

fn collect_same_name_candidates(
    root: &Path,
    dir: &Path,
    name: &std::ffi::OsStr,
    out: &mut BTreeMap<PathBuf, u64>,
) -> io::Result<()> {
    let mut entries = match fs::read_dir(dir) {
        Ok(rd) => rd.collect::<Result<Vec<_>, _>>()?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        let entry_name = entry.file_name();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            if should_skip_code_candidate_dir(&entry_name) {
                continue;
            }
            collect_same_name_candidates(root, &path, name, out)?;
        } else if ft.is_file() && entry_name == name {
            insert_hashed_path(root, &path, out)?;
        }
    }

    Ok(())
}

fn should_skip_code_candidate_dir(name: &std::ffi::OsStr) -> bool {
    let text = name.to_string_lossy();
    matches!(
        text.as_ref(),
        "target"
            | "node_modules"
            | "dist"
            | "build"
            | ".artifacts"
            | ".git"
            | ".hg"
            | ".svn"
            | ".jj"
            | "__pycache__"
            | ".venv"
            | "venv"
            | "vendor"
    )
}

fn insert_hashed_path(
    root: &Path,
    path: &Path,
    out: &mut BTreeMap<PathBuf, u64>,
) -> io::Result<()> {
    let bytes = fs::read(path)?;
    let key = path.strip_prefix(root).unwrap_or(path).to_path_buf();
    out.insert(key, hash_bytes(&bytes));
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

/// Diff two scans and emit path changes for unique content-hash renames/moves.
///
/// Ambiguous hashes (one hash shared by multiple removed or multiple added paths)
/// are skipped — caller relies on lint to surface dangling refs.
pub fn observe_renames(previous: &TreeSnapshot, current: &TreeSnapshot) -> Vec<PathChange> {
    let mut removed: Vec<(PathBuf, u64)> = Vec::new();
    let mut added: Vec<(PathBuf, u64)> = Vec::new();

    for (path, hash) in &previous.files {
        if !current.files.contains_key(path) {
            removed.push((path.clone(), *hash));
        }
    }
    for (path, hash) in &current.files {
        if !previous.files.contains_key(path) {
            added.push((path.clone(), *hash));
        }
    }

    if removed.is_empty() || added.is_empty() {
        return Vec::new();
    }

    let mut removed_by_hash: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (path, hash) in &removed {
        removed_by_hash.entry(*hash).or_default().push(path.clone());
    }
    let mut added_by_hash: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (path, hash) in &added {
        added_by_hash.entry(*hash).or_default().push(path.clone());
    }

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut used_added: BTreeSet<PathBuf> = BTreeSet::new();

    for (hash, froms) in &removed_by_hash {
        let Some(tos) = added_by_hash.get(hash) else {
            continue;
        };
        if froms.len() != 1 || tos.len() != 1 {
            continue;
        }
        let from = &froms[0];
        let to = &tos[0];
        if used_added.contains(to) {
            continue;
        }
        used_added.insert(to.clone());
        pairs.push((from.clone(), to.clone()));
    }

    if let Some(dir_change) = try_collapse_dir_move(&pairs) {
        return vec![dir_change];
    }

    pairs
        .into_iter()
        .map(|(from, to)| PathChange::FileMoved {
            from,
            to,
            disk_already_moved: true,
        })
        .collect()
}

/// If every pair is `old_dir/rel` → `new_dir/rel` with the same parents, emit DirMoved.
fn try_collapse_dir_move(pairs: &[(PathBuf, PathBuf)]) -> Option<PathChange> {
    if pairs.len() < 2 {
        return None;
    }

    let mut old_dir: Option<PathBuf> = None;
    let mut new_dir: Option<PathBuf> = None;

    for (from, to) in pairs {
        let from_parent = from.parent()?.to_path_buf();
        let to_parent = to.parent()?.to_path_buf();
        let from_name = from.file_name()?;
        let to_name = to.file_name()?;
        if from_name != to_name {
            return None;
        }
        match (&old_dir, &new_dir) {
            (None, None) => {
                old_dir = Some(from_parent);
                new_dir = Some(to_parent);
            }
            (Some(o), Some(n)) => {
                if o != &from_parent || n != &to_parent {
                    return None;
                }
            }
            _ => return None,
        }
    }

    let from = old_dir?;
    let to = new_dir?;
    if from == to {
        return None;
    }
    Some(PathChange::DirMoved {
        from,
        to,
        disk_already_moved: true,
    })
}
