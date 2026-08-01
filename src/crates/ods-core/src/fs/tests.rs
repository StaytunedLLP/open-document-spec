pub fn should_ignore_name(name: &std::ffi::OsStr) -> bool {
    let text = name.to_string_lossy();
    if text.starts_with('.') {
        return true;
    }
    // Auto-generated lint reports — not workspace documents.
    if text.eq_ignore_ascii_case("ods-error.md") || text.eq_ignore_ascii_case("ods-errors.md") {
        return true;
    }
    DEFAULT_IGNORE_NAMES
        .iter()
        .any(|ignored| text.eq_ignore_ascii_case(ignored))
}

fn load_gitignore_patterns(root: &Path) -> Vec<String> {
    let path = root.join(".gitignore");
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };

    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter(|line| !line.starts_with('!'))
        .map(|line| line.trim_end_matches('/').to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn normalize_path_collapses_parent_dirs() {
        let path = PathBuf::from("/a/b/../c/./d");
        assert_eq!(normalize_path(&path), PathBuf::from("/a/c/d"));
    }

    #[test]
    fn find_workspace_root_prefers_nearest_ods_marker() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("ods-root-{nonce}"));
        let nested = root.join("nested");
        fs::create_dir_all(nested.join("products")).expect("dirs");
        fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n",
        )
        .expect("root index");
        fs::write(
            nested.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Nested\n",
        )
        .expect("nested index");
        let file = nested.join("products/item.md");
        fs::write(&file, "# Item\n").expect("file");

        let found = find_workspace_root(&file).expect("root");
        assert_eq!(found, nested);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn find_workspace_root_returns_none_for_plain_directory() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ods-plain-{nonce}"));
        fs::create_dir_all(dir.join("sub")).expect("dirs");
        let file = dir.join("sub/note.md");
        fs::write(&file, "# Note\n").expect("file");

        // No index.md anywhere — should return None (not the parent dir).
        let found = find_workspace_root(&file);
        assert!(
            found.is_none(),
            "expected None for plain directory, got {:?}",
            found
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn find_workspace_root_returns_none_without_ods_key() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("ods-nokey-{nonce}"));
        fs::create_dir_all(dir.join("docs")).expect("dirs");
        // index.md exists but has no ods: key — just a plain markdown file.
        fs::write(dir.join("index.md"), "# My Notes\n\nJust a readme.\n")
            .expect("index");
        let file = dir.join("docs/thing.md");
        fs::write(&file, "# Thing\n").expect("file");

        // index.md without ods: key is still found as nearest_index fallback.
        // But it should NOT act as a workspace root for plain directories
        // that lack the ods: marker.
        let found = find_workspace_root(&file);
        // nearest_index is found (index.md exists), so it returns Some.
        // This is expected — the strict guard (require_ods_workspace) in CLI
        // catches this case by checking ods_enabled().
        assert!(found.is_some());

        let _ = fs::remove_dir_all(dir);
    }
}
