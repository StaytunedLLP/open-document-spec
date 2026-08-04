//! Discover Markdown paths under a workspace root (effectful walk, pure ordering).

use crate::fs::{path_matches_workspace_ignore, should_ignore_name};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Walk `root` and return sorted absolute paths of `.md` files (excluding ignored names/paths).
pub fn discover_markdown_paths(
    root: &Path,
    excluded_roots: &[PathBuf],
    gitignore: &[String],
    workspace_ignore: &[String],
) -> io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_markdown_paths(
        root,
        root,
        &mut paths,
        excluded_roots,
        gitignore,
        workspace_ignore,
    )?;
    paths.sort();
    Ok(paths)
}

fn collect_markdown_paths(
    root: &Path,
    dir: &Path,
    out: &mut Vec<PathBuf>,
    excluded_roots: &[PathBuf],
    gitignore: &[String],
    workspace_ignore: &[String],
) -> io::Result<()> {
    let mut entries = match fs::read_dir(dir) {
        Ok(rd) => rd.collect::<Result<Vec<_>, _>>()?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_type = entry.file_type()?;

        if should_ignore_name(&file_name) {
            continue;
        }

        if is_gitignored(root, &path, gitignore) {
            continue;
        }

        if path_matches_workspace_ignore(root, &path, workspace_ignore) {
            continue;
        }

        if excluded_roots.iter().any(|excl| is_within(&path, excl)) {
            continue;
        }

        if file_type.is_dir() {
            collect_markdown_paths(
                root,
                &path,
                out,
                excluded_roots,
                gitignore,
                workspace_ignore,
            )?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }

    Ok(())
}

fn is_within(path: &Path, root: &Path) -> bool {
    path == root || path.strip_prefix(root).is_ok()
}

fn is_gitignored(root: &Path, path: &Path, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let relative = relative.to_string_lossy().replace('\\', "/");
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();

    patterns.iter().any(|pattern| {
        let pattern = pattern.trim_start_matches('/');
        if pattern.contains('/') {
            relative == pattern
                || relative.starts_with(&format!("{pattern}/"))
                || relative.contains(&format!("/{pattern}/"))
        } else {
            name == pattern
                || relative == pattern
                || relative.ends_with(&format!("/{pattern}"))
                || relative.contains(&format!("/{pattern}/"))
        }
    })
}

#[cfg(test)]
mod test_discover {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_discover_markdown_paths_with_gitignore_and_excluded() {
        let td = tempdir().unwrap();
        let root = td.path();

        let sub = root.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        let excl = root.join("excl");
        std::fs::create_dir_all(&excl).unwrap();

        std::fs::write(root.join("index.md"), "# Root").unwrap();
        std::fs::write(sub.join("doc.md"), "# Doc").unwrap();
        std::fs::write(sub.join("ignored.md"), "# Ignored").unwrap();
        std::fs::write(excl.join("excluded.md"), "# Excluded").unwrap();

        let gitignore = vec!["sub/ignored.md".to_string()];
        let excluded_roots = vec![excl];
        let workspace_ignore = vec![];

        let paths =
            discover_markdown_paths(root, &excluded_roots, &gitignore, &workspace_ignore).unwrap();
        assert!(paths.contains(&root.join("index.md")));
        assert!(paths.contains(&sub.join("doc.md")));
        assert!(!paths.contains(&sub.join("ignored.md")));
        assert!(!paths.contains(&root.join("excl/excluded.md")));
    }
}
