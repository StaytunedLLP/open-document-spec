use ods_core::{
    PathChange, apply_path_changes, classify_watch_events, move_document_and_rewrite_refs,
    normalize_frontmatter_body_spacing, reindex_workspace, rewrite_references_in_text,
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
fn apply_disk_already_moved_file() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n\n- [a.md](a.md)\n",
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
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n\n",
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
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n\n- [x.md](x.md)\n",
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
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Root\n",
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

#[test]
fn classify_single_file_dir_when_parent_gone() {
    let root = temp_workspace();
    let old = root.join("old");
    let neu = root.join("neu");
    fs::create_dir_all(&neu).unwrap();
    fs::write(neu.join("only.md"), "---\nprofile: note\n---\n\n# O\n").unwrap();
    // Source parent does not exist (already renamed).
    let events = vec![(old.join("only.md"), 3u8), (neu.join("only.md"), 1u8)];
    let changes = classify_watch_events(&root, &events, false);
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, PathChange::DirMoved { .. })),
        "{changes:?}"
    );
}

#[test]
fn classify_single_file_stays_file_move_when_parent_exists() {
    let root = temp_workspace();
    fs::create_dir_all(root.join("old")).unwrap();
    fs::create_dir_all(root.join("neu")).unwrap();
    fs::write(root.join("old/keep.md"), "---\nprofile: note\n---\n\n# K\n").unwrap();
    fs::write(
        root.join("neu/moved.md"),
        "---\nprofile: note\n---\n\n# M\n",
    )
    .unwrap();
    let events = vec![
        (root.join("old/moved.md"), 3u8),
        (root.join("neu/moved.md"), 1u8),
    ];
    let changes = classify_watch_events(&root, &events, true);
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, PathChange::FileMoved { .. })),
        "{changes:?}"
    );
    assert!(
        !changes
            .iter()
            .any(|c| matches!(c, PathChange::DirMoved { .. })),
        "{changes:?}"
    );
}

#[test]
fn sales_like_folder_rename_end_to_end() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("sales")).unwrap();
    fs::create_dir_all(dir.join("support")).unwrap();
    fs::create_dir_all(dir.join("products")).unwrap();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Shop\n\n- [sales/](sales/index.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("sales/index.md"),
        "---\nprofile: index\n---\n\n# sales\n",
    )
    .unwrap();
    fs::write(
        dir.join("sales/subscription-pricing.md"),
        "---\nprofile: decision\nid: sales/subscription-pricing\nstatus: draft\n---\n\n# Pricing\n",
    )
    .unwrap();
    fs::write(
        dir.join("products/glow.md"),
        "---\nprofile: product\nstatus: draft\ndepends:\n  - sales/subscription-pricing\n---\n\n# Glow\n",
    )
    .unwrap();
    fs::write(
        dir.join("support/faq.md"),
        "---\nprofile: guide\nstatus: draft\n---\n\n# FAQ\n\nSee [pricing](../sales/subscription-pricing.md).\n",
    )
    .unwrap();

    move_document_and_rewrite_refs(&dir, "sales", "revenue").unwrap();

    let glow = fs::read_to_string(dir.join("products/glow.md")).unwrap();
    assert!(
        glow.contains("  - revenue/subscription-pricing\n"),
        "{glow}"
    );
    let faq = fs::read_to_string(dir.join("support/faq.md")).unwrap();
    assert!(
        faq.contains("](../revenue/subscription-pricing.md)"),
        "{faq}"
    );
    let pricing = fs::read_to_string(dir.join("revenue/subscription-pricing.md")).unwrap();
    assert!(
        pricing.contains("id: revenue/subscription-pricing"),
        "{pricing}"
    );
    assert_eq!(count_blanks(&pricing), 1);
    assert_eq!(count_blanks(&glow), 1);
}

#[test]
fn code_file_move_rewrites_code_path() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("src/old")).unwrap();
    fs::create_dir_all(dir.join("src/new")).unwrap();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(dir.join("src/old/login.ts"), "export function login() {}\n").unwrap();
    fs::write(
        dir.join("feature.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: src/old/login.ts\n    symbol: login\n    role: implementation\n---\n\n# Feature\n",
    )
    .unwrap();

    move_document_and_rewrite_refs(&dir, "src/old/login.ts", "src/new/login.ts").unwrap();
    let feature = fs::read_to_string(dir.join("feature.md")).unwrap();
    assert!(feature.contains("path: src/new/login.ts"), "{feature}");
    assert!(!feature.contains("src/old/login.ts"), "{feature}");
}

#[test]
fn code_folder_move_rewrites_descendant_code_paths() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("apps/web/src/features/checkout")).unwrap();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(
        dir.join("apps/web/src/features/checkout/pricing.ts"),
        "export function price() {}\n",
    )
    .unwrap();
    fs::write(
        dir.join("feature.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: apps/web/src/features/checkout/pricing.ts\n    symbol: price\n    role: implementation\n---\n\n# Feature\n",
    )
    .unwrap();

    move_document_and_rewrite_refs(&dir, "apps/web/src/features", "apps/web/src/modules").unwrap();
    let feature = fs::read_to_string(dir.join("feature.md")).unwrap();
    assert!(
        feature.contains("path: apps/web/src/modules/checkout/pricing.ts"),
        "{feature}"
    );
}

#[test]
fn moving_markdown_document_recalculates_relative_code_paths() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("docs/features")).unwrap();
    fs::create_dir_all(dir.join("notes")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(dir.join("src/login.ts"), "export function login() {}\n").unwrap();
    fs::write(
        dir.join("docs/features/login.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: ../../src/login.ts\n    symbol: login\n    role: implementation\n---\n\n# Login\n",
    )
    .unwrap();

    move_document_and_rewrite_refs(&dir, "docs/features/login.md", "notes/login.md").unwrap();
    let moved = fs::read_to_string(dir.join("notes/login.md")).unwrap();
    assert!(moved.contains("path: ../src/login.ts"), "{moved}");
}

#[test]
fn rewrite_collapses_blank_lines() {
    let dirty = "---\nprofile: note\n---\n\n\n\n\n\n\n\n\n\n\n\n\n\n# T\n";
    let fixed = normalize_frontmatter_body_spacing(dirty);
    assert_eq!(count_blanks(&fixed), 1, "{fixed}");
    let again = rewrite_references_in_text(&fixed, "a", "b", "a.md", "b.md");
    assert_eq!(count_blanks(&again), 1);
    assert_eq!(again, normalize_frontmatter_body_spacing(&again));
}

fn count_blanks(text: &str) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let mut end = None;
    for (i, line) in lines.iter().enumerate().skip(1) {
        if line.trim() == "---" {
            end = Some(i);
            break;
        }
    }
    let end = end.expect("fm");
    lines
        .iter()
        .skip(end + 1)
        .take_while(|l| l.is_empty())
        .count()
}

#[test]
fn test_path_traversal_move_blocked() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Root\n- [a.md](a.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    let err = move_document_and_rewrite_refs(&dir, "a.md", "../outside.md");
    assert!(err.is_err());
    let err_msg = err.unwrap_err().to_string();
    assert!(
        err_msg.contains("Path traversal attempt blocked"),
        "{}",
        err_msg
    );
}
