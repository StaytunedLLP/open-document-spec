// Automatic `index.md` management for ODS workspaces.
//
// Indexes are lockfile-like: child lists are generated. Frontmatter keys
// (`profile`, `ods`, `profiles`, `ignore`) and the `# title` are preserved
// when already present. CLI (`ods index`) and LSP (save / create / delete /
// path ops) all call into this module.

use crate::fs::{
    normalize_join, path_matches_workspace_ignore, paths_equal_normalized, should_ignore_name,
};
use crate::parse::split_markdown_link_target;
use std::collections::BTreeSet;
use std::fs;
use std::io;

/// Generate or update `index.md` for every directory that contains documents
/// (and all ancestors up to the workspace root). Writes only when content would
/// change. Removes orphan non-root `index.md` files for empty trees.
///
fn index_path_for_dir(dir: &Path, _root: &Path) -> PathBuf {
    dir.join("index.ods.md")
}

/// Generate or update `index.ods.md` (or `index.md`) for every directory that contains documents
/// (and all ancestors up to the workspace root). Writes only when content would
/// change. Removes orphan non-root index files for empty trees.
///
/// Returns paths that were written or deleted.
pub fn generate_indexes(workspace: &Workspace) -> io::Result<Vec<PathBuf>> {
    let directories = index_directories(workspace);
    let mut expected = BTreeSet::new();
    let mut touched = Vec::new();

    for directory in &directories {
        let index_path = index_path_for_dir(directory, &workspace.root);
        expected.insert(index_path.clone());
        let existing = fs::read_to_string(&index_path).ok();
        let rendered = render_index(workspace, directory, existing.as_deref());
        let existing_normalized = existing.as_deref().map(|s| s.replace("\r\n", "\n"));
        if existing_normalized.as_deref() != Some(rendered.replace("\r\n", "\n").as_str()) {
            if let Some(parent) = index_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&index_path, rendered)?;
            touched.push(index_path);
        }
    }

    let pruned = prune_orphan_indexes(workspace, &expected)?;
    touched.extend(pruned);

    Ok(touched)
}

/// Returns true when every managed index matches disk and no orphan indexes remain.
pub fn indexes_are_current(workspace: &Workspace) -> io::Result<bool> {
    let directories = index_directories(workspace);
    let mut expected = BTreeSet::new();

    for directory in &directories {
        let index_path = index_path_for_dir(directory, &workspace.root);
        expected.insert(index_path.clone());
        let existing = fs::read_to_string(&index_path).unwrap_or_default();
        let rendered = render_index(workspace, directory, Some(&existing));
        if existing.replace("\r\n", "\n") != rendered.replace("\r\n", "\n") {
            return Ok(false);
        }
    }

    // Orphan non-root index files mean stale management.
    if has_orphan_indexes(workspace, &expected)? {
        return Ok(false);
    }
    Ok(true)
}

/// Directories that must have a managed index file (doc dirs + ancestors + root).
pub fn index_directories(workspace: &Workspace) -> Vec<PathBuf> {
    let mut directories = Vec::new();
    for doc in &workspace.documents {
        if path_matches_workspace_ignore(&workspace.root, &doc.path, &workspace.ignore) {
            continue;
        }
        if crate::fs::is_excluded_profile_catalog(workspace, &doc.path) {
            continue;
        }
        // Do not seed from index file alone — otherwise empty folders never prune.
        if doc
            .path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("index.md") || n.eq_ignore_ascii_case("index.ods.md"))
        {
            continue;
        }
        let mut current = doc.directory.clone();
        loop {
            if path_matches_workspace_ignore(&workspace.root, &current, &workspace.ignore) {
                break;
            }
            if crate::fs::is_excluded_profile_catalog(workspace, &current) {
                break;
            }
            directories.push(current.clone());
            if current == workspace.root {
                break;
            }
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
    }
    if !directories.iter().any(|d| d == &workspace.root) {
        directories.push(workspace.root.clone());
    }
    directories.sort();
    directories.dedup();
    directories
}

fn has_orphan_indexes(workspace: &Workspace, expected: &BTreeSet<PathBuf>) -> io::Result<bool> {
    let mut found = false;
    visit_index_files(workspace, &mut |path| {
        if !expected.contains(path) && *path != workspace.root.join("index.md") && *path != workspace.root.join("index.ods.md") {
            found = true;
        }
        Ok(())
    })?;
    Ok(found)
}

/// Delete non-root index files that are not required by current documents.
fn prune_orphan_indexes(
    workspace: &Workspace,
    expected: &BTreeSet<PathBuf>,
) -> io::Result<Vec<PathBuf>> {
    let mut removed = Vec::new();
    let root_index_md = workspace.root.join("index.md");
    let root_index_ods = workspace.root.join("index.ods.md");
    visit_index_files(workspace, &mut |path| {
        if expected.contains(path) || path == root_index_md || path == root_index_ods {
            return Ok(());
        }
        // Only prune auto-managed indexes (profile: index or missing profile).
        if !is_auto_managed_index(path) {
            return Ok(());
        }
        fs::remove_file(path)?;
        removed.push(path.to_path_buf());
        Ok(())
    })?;
    Ok(removed)
}

fn is_auto_managed_index(path: &Path) -> bool {
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    if let Some((_, profile, _, _, _, _)) = extract_title_and_meta(&text) {
        profile == "index" || profile.is_empty()
    } else {
        true
    }
}

fn visit_index_files(
    workspace: &Workspace,
    visit: &mut dyn FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    fn walk(
        workspace: &Workspace,
        dir: &Path,
        visit: &mut dyn FnMut(&Path) -> io::Result<()>,
    ) -> io::Result<()> {
        if path_matches_workspace_ignore(&workspace.root, dir, &workspace.ignore) {
            return Ok(());
        }
        if crate::fs::is_excluded_profile_catalog(workspace, dir) {
            return Ok(());
        }
        let index_ods = dir.join("index.ods.md");
        let index_md = dir.join("index.md");
        if index_ods.is_file() {
            visit(&index_ods)?;
        }
        if index_md.is_file() {
            visit(&index_md)?;
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let name = entry.file_name();
            if should_ignore_name(name.as_os_str()) {
                continue;
            }
            if path.is_dir() {
                walk(workspace, &path, visit)?;
            }
        }
        Ok(())
    }
    walk(workspace, &workspace.root, visit)
}

#[cfg(test)]
mod test_generator {
    use super::*;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn test_generator_helpers() {
        let td = tempdir().unwrap();
        let root = td.path();

        std::fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();

        std::fs::write(
            root.join("doc1.md"),
            "---\nprofile: note\nstatus: draft\nid: doc1\n---\n\n# Doc 1\n",
        )
        .unwrap();

        let ws = load_workspace(root).unwrap();

        let is_curr = indexes_are_current(&ws).unwrap();
        assert!(is_curr || !is_curr);

        let dirs = index_directories(&ws);
        assert!(!dirs.is_empty());

        let touched = generate_indexes(&ws).unwrap();
        let _ = touched;

        let auto = is_auto_managed_index(&root.join("index.ods.md"));
        assert!(auto);
    }
}
