// Path moves/renames: rewrite document references and regenerate indexes.
//
// Used by `ods mv`, `ods watch`, and `ods serve` / `ods start` (background watch).

use crate::fs::load_workspace;
use crate::index::generate_indexes;
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

/// A single path-level change to apply (disk may already reflect the move).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathChange {
    /// File rename or move. If `disk_already_moved`, skip `fs::rename`.
    FileMoved {
        from: PathBuf,
        to: PathBuf,
        disk_already_moved: bool,
    },
    /// Directory rename or move (all markdown descendants).
    DirMoved {
        from: PathBuf,
        to: PathBuf,
        disk_already_moved: bool,
    },
}

#[derive(Debug, Clone, Default)]
pub struct PathChangeReport {
    pub rewritten_files: Vec<PathBuf>,
    pub indexes: Vec<PathBuf>,
    pub moves: Vec<(PathBuf, PathBuf)>,
    /// Non-fatal issues (skipped file, cross-root note, etc.).
    pub warnings: Vec<String>,
    /// Serious but non-aborting failures (I/O on one path); other work continues.
    pub errors: Vec<String>,
}

impl PathChangeReport {
    /// Human-readable one-line summary for CLI / LSP notifications.
    pub fn summary(&self) -> String {
        let mut parts = vec![format!(
            "rewrote {} file(s), {} index(es), {} move(s)",
            self.rewritten_files.len(),
            self.indexes.len(),
            self.moves.len()
        )];
        if !self.warnings.is_empty() {
            parts.push(format!("{} warning(s)", self.warnings.len()));
        }
        if !self.errors.is_empty() {
            parts.push(format!("{} error(s)", self.errors.len()));
        }
        parts.join("; ")
    }

    pub fn has_issues(&self) -> bool {
        !self.warnings.is_empty() || !self.errors.is_empty()
    }
}

/// Move a file or directory (relative to root) and rewrite refs + indexes.
pub fn move_document_and_rewrite_refs(
    root: impl AsRef<Path>,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> io::Result<()> {
    let _ = move_document_and_rewrite_refs_report(root, from, to)?;
    Ok(())
}

/// Same as [`move_document_and_rewrite_refs`] but returns a report.
pub fn move_document_and_rewrite_refs_report(
    root: impl AsRef<Path>,
    from: impl AsRef<Path>,
    to: impl AsRef<Path>,
) -> io::Result<PathChangeReport> {
    let root = root
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root.as_ref().to_path_buf());
    let root = root.as_path();
    let from = from.as_ref();
    let to = to.as_ref();
    // Classify against absolutized copies, but hand `apply_path_changes` the
    // original (possibly root-relative) paths — it absolutizes internally,
    // and joining `root` twice here (once now, once there) silently doubles
    // a relative root into a nonexistent nested path.
    let abs_from = absolutize(root, from);
    let abs_to = absolutize(root, to);
    let change = if abs_from.is_dir() || (!abs_from.exists() && abs_to.is_dir()) {
        PathChange::DirMoved {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            disk_already_moved: false,
        }
    } else {
        PathChange::FileMoved {
            from: from.to_path_buf(),
            to: to.to_path_buf(),
            disk_already_moved: false,
        }
    };
    apply_path_changes(root, &[change])
}

/// Apply one or more path changes: filesystem move (unless already done),
/// rewrite references across the workspace, regenerate indexes.
///
/// Fail-soft: individual I/O errors are recorded on the report and work continues.
/// Returns `Err` only when the workspace cannot be loaded at all.
pub fn apply_path_changes(
    root: impl AsRef<Path>,
    changes: &[PathChange],
) -> io::Result<PathChangeReport> {
    let root = root
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root.as_ref().to_path_buf());
    let root = root.as_path();
    let (mut report, edits) = compute_path_change_edits(root, changes)?;
    for (path, text) in edits {
        match fs::write(&path, &text) {
            Ok(()) => {
                if !report.rewritten_files.iter().any(|p| p == &path) {
                    report.rewritten_files.push(path);
                }
            }
            Err(err) => report
                .errors
                .push(format!("write {}: {err}", path.display())),
        }
    }
    report.rewritten_files.sort();
    report.rewritten_files.dedup();

    // Heal path-shaped `id:` fields that still don't match on-disk paths (missed
    // rename batches, partial rewrites, etc.), then regenerate indexes once.
    // `heal_orphan_path_ids` already regenerates indexes against current state
    // as its last step, so on success `report.indexes` reflects reality even
    // when empty (nothing needed writing) — only retry below if `heal` itself
    // failed, to avoid a redundant same-state reload/regenerate in the common
    // "everything already current" case.
    let mut heal_failed = false;
    match heal_orphan_path_ids(root) {
        Ok(heal) => {
            for p in heal.rewritten_files {
                if !report.rewritten_files.iter().any(|x| x == &p) {
                    report.rewritten_files.push(p);
                }
            }
            report.warnings.extend(heal.warnings);
            report.errors.extend(heal.errors);
            report.indexes = heal.indexes;
        }
        Err(err) => {
            report.errors.push(format!("heal path ids: {err}"));
            heal_failed = true;
        }
    }

    if heal_failed && report.indexes.is_empty() {
        match load_workspace(root) {
            Ok(workspace) => match generate_indexes(&workspace) {
                Ok(indexes) => report.indexes = indexes,
                Err(err) => report.errors.push(format!("regenerate indexes: {err}")),
            },
            Err(err) => report
                .errors
                .push(format!("reload workspace for index: {err}")),
        }
    }

    Ok(report)
}
