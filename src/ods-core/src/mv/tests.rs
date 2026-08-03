#[cfg(test)]
mod tests_move_b {
    use super::*;
    use std::fs;

    #[test]
    fn deep_folder_rename() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("a/b/c")).unwrap();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(
            dir.join("a/b/c/doc.md"),
            "---\nprofile: note\nstatus: draft\nid: a/b/c/doc\n---\n\n# D\n",
        )
        .unwrap();
        fs::write(
            dir.join("ref.md"),
            "---\nprofile: note\nstatus: draft\ndepends:\n  - a/b/c/doc\ncontext:\n  load:\n    - a/b/c/doc\n---\n\n# R\n\n[link](a/b/c/doc.md)\n",
        )
        .unwrap();
        move_document_and_rewrite_refs(&dir, "a/b/c", "a/b/z").unwrap();
        let r = fs::read_to_string(dir.join("ref.md")).unwrap();
        assert!(r.contains("  - a/b/z/doc\n"), "{r}");
        assert!(r.contains("    - a/b/z/doc\n"), "{r}");
        assert!(r.contains("](a/b/z/doc.md)"), "{r}");
        let d = fs::read_to_string(dir.join("a/b/z/doc.md")).unwrap();
        assert!(d.contains("id: a/b/z/doc"), "{d}");
    }

    #[test]
    fn ignore_prefix_rewritten_on_folder_rename() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("vendor/pkg")).unwrap();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nignore:\n  - vendor\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(
            dir.join("vendor/pkg/x.md"),
            "---\nprofile: note\nstatus: draft\n---\n\n# X\n",
        )
        .unwrap();
        // vendor is ignored so may not be in workspace docs for rewrite of content,
        // but root ignore list should still update via path_prefix on index.md
        // Moving vendor requires it to exist; load may skip ignored docs.
        // Use a non-ignored path listed in ignore for the rewrite of the list entry:
        fs::create_dir_all(dir.join("legacy")).unwrap();
        fs::write(
            dir.join("legacy/a.md"),
            "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
        )
        .unwrap();
        // Fix index to ignore legacy
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nignore:\n  - legacy\n---\n\n# R\n\n- [legacy/](legacy/index.md)\n",
        )
        .unwrap();
        fs::write(
            dir.join("legacy/index.md"),
            "---\nprofile: index\n---\n\n# L\n",
        )
        .unwrap();
        // Remove ignore temporarily so mv can see files — actually ignore only affects load.
        // move_document uses load after rename; for DirMoved disk rename uses collect_md_files
        // which does NOT respect ignore. Good.
        move_document_and_rewrite_refs(&dir, "legacy", "archive").unwrap();
        let index = fs::read_to_string(dir.join("index.md")).unwrap();
        assert!(
            index.contains("- archive") || index.contains("  - archive"),
            "{index}"
        );
    }

    #[test]
    fn heal_orphan_path_id_matches_filename() {
        let dir = tempfile_dir();
        fs::create_dir_all(dir.join("products")).unwrap();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        // File path is clay-mask.md but id still says clay-mask-new (missed rename rewrite).
        fs::write(
            dir.join("products/clay-mask.md"),
            "---\nprofile: product\nid: products/clay-mask-new\nstatus: draft\n---\n\n# Clay\n",
        )
        .unwrap();
        fs::write(
            dir.join("ref.md"),
            "---\nprofile: note\nstatus: draft\ndepends:\n  - products/clay-mask-new\n---\n\n# R\n",
        )
        .unwrap();
        let report = heal_orphan_path_ids(&dir).unwrap();
        assert!(
            !report.rewritten_files.is_empty(),
            "should rewrite id drift: {report:?}"
        );
        let body = fs::read_to_string(dir.join("products/clay-mask.md")).unwrap();
        assert!(
            body.contains("id: products/clay-mask\n"),
            "id must match path: {body}"
        );
        assert!(!body.contains("clay-mask-new"));
        let ref_body = fs::read_to_string(dir.join("ref.md")).unwrap();
        assert!(
            ref_body.contains("products/clay-mask\n") || ref_body.contains("products/clay-mask"),
            "{ref_body}"
        );
        assert!(!ref_body.contains("clay-mask-new"));
        let _ = fs::remove_dir_all(&dir);
    }


    #[allow(dead_code)]
    fn count_blanks_after_frontmatter(text: &str) -> usize {
        let lines: Vec<&str> = text.lines().collect();
        let mut end = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                end = Some(i);
                break;
            }
        }
        let end = end.expect("closing frontmatter");
        let mut blanks = 0;
        for line in lines.iter().skip(end + 1) {
            if line.is_empty() {
                blanks += 1;
            } else {
                break;
            }
        }
        blanks
    }

    fn tempfile_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ods-mv-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }
}
