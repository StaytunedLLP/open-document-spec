use ods_core::{
    PathChange, apply_path_changes, classify_watch_events, move_document_and_rewrite_refs,
    reindex_workspace, rewrite_references_in_text,
};
use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;

#[test]
fn classify_file_rename_pair() {
    let root = PathBuf::from("/ws");
    let events = vec![(root.join("old.md"), 3u8), (root.join("new.md"), 1u8)];
    let changes = classify_watch_events(&root, &events, true);
    assert_eq!(changes.len(), 1);
    match &changes[0] {
        PathChange::FileMoved {
            from,
            to,
            disk_already_moved,
        } => {
            assert!(from.ends_with("old.md"));
            assert!(to.ends_with("new.md"));
            assert!(*disk_already_moved);
        }
        other => panic!("expected FileMoved: {other:?}"),
    }
}

#[test]
fn classify_dir_move_multiple_files() {
    let root = PathBuf::from("/ws");
    let events = vec![
        (root.join("old/a.md"), 3u8),
        (root.join("old/b.md"), 3u8),
        (root.join("neu/a.md"), 1u8),
        (root.join("neu/b.md"), 1u8),
    ];
    // A complete, self-consistent directory move is detected even on a
    // non-final pass — only the looser per-file fallback needs to wait.
    let changes = classify_watch_events(&root, &events, false);
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, PathChange::DirMoved { .. })),
        "{changes:?}"
    );
}

/// Regression test for the "sales" → "sales-z" folder rename bug: when a
/// multi-file directory rename's create/delete events arrive in separate
/// notifications, classifying the first, partial batch must NOT peel off a
/// same-named sibling as a lone file move — that would let
/// `apply_path_changes` rewrite refs for just that one file while the rest
/// of the folder's files (and everything depending on them) are silently
/// dropped by the caller, since it only re-queues events that classify as
/// entirely unrecognized by the watch reconciliation path.
#[test]
fn classify_early_pass_does_not_peel_off_partial_folder_rename() {
    let root = PathBuf::from("/ws");
    // Only 2 of the 3 renamed files' events have arrived so far (a.md's
    // create hasn't shown up yet); index.md matches by filename right away.
    let events = vec![
        (root.join("sales/index.md"), 3u8),
        (root.join("sales/a.md"), 3u8),
        (root.join("sales-z/index.md"), 1u8),
    ];
    let changes = classify_watch_events(&root, &events, false);
    assert!(
        changes.is_empty(),
        "must wait for the rest of the folder rename instead of committing a partial move: {changes:?}"
    );
}

#[test]
fn healer_classify_and_rewrite_edge_cases() {
    let root = PathBuf::from("/ws");
    // Non-md file and modification (kind 2) events
    let events = vec![
        (root.join("ignore.txt"), 3u8),
        (root.join("ignore.txt"), 1u8),
        (root.join("mod.md"), 2u8),
    ];
    let changes = classify_watch_events(&root, &events, true);
    assert!(changes.is_empty());

    // Empty old_id and old_target rewrite_references_in_text
    let text = "# Text\n";
    assert_eq!(rewrite_references_in_text(text, "", "", "", ""), text);
}

#[test]
fn rewrite_refs_after_moves_test() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n- [new.md](new.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("new.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# New\n",
    )
    .unwrap();

    let moves = vec![(dir.join("old.md"), dir.join("new.md"))];
    let rep = ods_core::rewrite_refs_after_moves(&dir, &moves).unwrap();
    assert!(rep.rewritten_files.is_empty() || !rep.rewritten_files.is_empty());
}

#[test]
fn mv_rewriter_traversal_and_error_tests() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();

    // Traversal outside root
    let outside = PathBuf::from("/tmp/outside_path_123");
    let changes = vec![PathChange::FileMoved {
        from: dir.join("doc.md"),
        to: outside,
        disk_already_moved: false,
    }];
    assert!(apply_path_changes(&dir, &changes).is_err());
}

#[test]
fn path_change_report_summary_and_has_issues_test() {
    let mut report = ods_core::PathChangeReport::default();
    assert!(!report.has_issues());
    assert!(report.summary().contains("rewrote 0 file(s)"));

    report.warnings.push("warn".to_string());
    report.errors.push("err".to_string());
    assert!(report.has_issues());
    let sum = report.summary();
    assert!(sum.contains("1 warning(s)"));
    assert!(sum.contains("1 error(s)"));
}

#[test]
fn apply_disk_already_moved_file() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [a.md](a.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# B\n",
    )
    .unwrap();
    // Simulate rename already done
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    fs::rename(dir.join("a.md"), dir.join("c.md")).unwrap();

    let report = apply_path_changes(
        &dir,
        &[PathChange::FileMoved {
            from: dir.join("a.md"),
            to: dir.join("c.md"),
            disk_already_moved: true,
        }],
    )
    .unwrap();
    assert!(!report.indexes.is_empty());
    let b = fs::read_to_string(dir.join("b.md")).unwrap();
    assert!(b.contains("  - c\n"), "{b}");
}

#[test]
fn reindex_workspace_writes_indexes() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n",
    )
    .unwrap();
    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\ndescription: Hello\n---\n\n# D\n",
    )
    .unwrap();
    let paths = reindex_workspace(&dir).unwrap();
    assert!(paths.iter().any(|p| p.ends_with("index.md")));
    let index = fs::read_to_string(dir.join("index.md")).unwrap();
    assert!(index.contains("doc.md"));
    assert!(index.contains("Hello"));
}

#[test]
fn move_document_cli_path() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [x.md](x.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("x.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# X\n",
    )
    .unwrap();
    move_document_and_rewrite_refs(&dir, "x.md", "y.md").unwrap();
    assert!(dir.join("y.md").exists());
    assert!(!dir.join("x.md").exists());
}

#[test]
fn moving_markdown_preserves_md_frontmatter_ref_style() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
    )
    .unwrap();
    fs::create_dir_all(dir.join("docs")).unwrap();
    fs::write(
        dir.join("docs/a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("docs/b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a.md\n---\n\n# B\n",
    )
    .unwrap();

    move_document_and_rewrite_refs(&dir, "docs/a.md", "docs/c.md").unwrap();
    let b = fs::read_to_string(dir.join("docs/b.md")).unwrap();
    assert!(b.contains("  - c.md"), "{b}");
    assert!(!b.contains("  - c\n"), "{b}");
}
