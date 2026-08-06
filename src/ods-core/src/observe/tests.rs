#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn two_phase_delete_then_create_still_pairs() {
        let dir = tempfile_dir();
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        fs::write(&a, "unique-body-xyz\n").unwrap();
        let mut watch = WatchTree::from_scan(scan_markdown_tree(&dir, &[]).unwrap());

        // Phase 1: only delete (as if create not seen yet).
        fs::remove_file(&a).unwrap();
        let mid = scan_markdown_tree(&dir, &[]).unwrap();
        let changes = observe_renames(&watch.effective_previous(), &mid);
        assert!(changes.is_empty(), "no create yet");
        watch.commit_scan(mid, &[]);
        assert!(watch.pending_removed.contains_key(Path::new("a.md")));

        // Phase 2: create with same content.
        fs::write(&b, "unique-body-xyz\n").unwrap();
        let cur = scan_markdown_tree(&dir, &[]).unwrap();
        let changes = observe_renames(&watch.effective_previous(), &cur);
        assert_eq!(changes.len(), 1, "should pair across batches: {changes:?}");
        let paired = paired_from_paths(&changes);
        watch.commit_scan(cur, &paired);
        assert!(watch.pending_removed.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unique_hash_rename_is_paired() {
        let dir = tempfile_dir();
        let a = dir.join("a.md");
        let b = dir.join("b.md");
        fs::write(&a, "same body\n").unwrap();
        let prev = scan_markdown_tree(&dir, &[]).unwrap();
        fs::rename(&a, &b).unwrap();
        let cur = scan_markdown_tree(&dir, &[]).unwrap();
        let changes = observe_renames(&prev, &cur);
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            PathChange::FileMoved {
                from,
                to,
                disk_already_moved,
            } => {
                assert_eq!(from, Path::new("a.md"));
                assert_eq!(to, Path::new("b.md"));
                assert!(*disk_already_moved);
            }
            other => panic!("expected FileMoved, got {other:?}"),
        }
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rename_apply_rewrites_depends() {
        use crate::mv::apply_path_changes;
        use crate::{InitOptions, init_workspace, lint_workspace, load_workspace};

        let dir = tempfile_dir();
        init_workspace(&dir, InitOptions::default()).unwrap();
        fs::create_dir_all(dir.join("products")).unwrap();
        fs::write(
            dir.join("products/a.md"),
            "---\nprofile: note\nstatus: draft\nid: products/a\n---\n\n# A\n",
        )
        .unwrap();
        fs::write(
            dir.join("ref.md"),
            "---\nprofile: note\nstatus: draft\ndepends:\n  - products/a\n---\n\n# Ref\n",
        )
        .unwrap();
        let prev = scan_markdown_tree(&dir, &[]).unwrap();
        fs::rename(dir.join("products/a.md"), dir.join("products/b.md")).unwrap();
        let cur = scan_markdown_tree(&dir, &[]).unwrap();
        let changes = observe_renames(&prev, &cur);
        assert_eq!(changes.len(), 1);
        apply_path_changes(&dir, &changes).unwrap();
        let ref_body = fs::read_to_string(dir.join("ref.md")).unwrap();
        assert!(
            ref_body.contains("products/b"),
            "depends should rewrite: {ref_body}"
        );
        assert!(!ref_body.contains("products/a"));
        let ws = load_workspace(&dir).unwrap();
        let diags = lint_workspace(&ws);
        assert!(
            diags.iter().all(|d| !d.message.contains("dangling")),
            "{diags:?}"
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn ambiguous_identical_files_not_paired() {
        let dir = tempfile_dir();
        fs::write(dir.join("a.md"), "dup\n").unwrap();
        fs::write(dir.join("c.md"), "dup\n").unwrap();
        let prev = scan_markdown_tree(&dir, &[]).unwrap();
        fs::remove_file(dir.join("a.md")).unwrap();
        fs::remove_file(dir.join("c.md")).unwrap();
        fs::write(dir.join("x.md"), "dup\n").unwrap();
        let cur = scan_markdown_tree(&dir, &[]).unwrap();
        let changes = observe_renames(&prev, &cur);
        assert!(changes.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[allow(deprecated)]
    fn tempfile_dir() -> PathBuf {
        tempfile::tempdir().unwrap().into_path()
    }
}
