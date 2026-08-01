//! Logic-level e2e for watch rename pairing + id heal (no live notify).
use odc_core::{
    InitOptions, WatchTree, apply_path_changes, heal_orphan_path_ids, init_workspace,
    lint_workspace, load_workspace, observe_renames, paired_from_paths, scan_markdown_tree,
};
use std::fs;
use std::sync::Mutex;

static WATCH_RENAME_TEST_LOCK: Mutex<()> = Mutex::new(());

fn tmp() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ods-watch-e2e-{}-{}",
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

#[test]
fn same_batch_rename_rewrites_depends_and_id() {
    let _guard = WATCH_RENAME_TEST_LOCK.lock().unwrap();
    let dir = tmp();
    init_workspace(&dir, InitOptions::default()).unwrap();
    fs::create_dir_all(dir.join("products")).unwrap();
    fs::write(
        dir.join("products/clay-mask.md"),
        "---\nprofile: note\nstatus: draft\nid: products/clay-mask\ndescription: Clay\n---\n\n# Clay\n",
    )
    .unwrap();
    fs::write(
        dir.join("ref.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - products/clay-mask\n---\n\n# R\n\n[c](products/clay-mask.md)\n",
    )
    .unwrap();
    let mut watch = WatchTree::from_scan(scan_markdown_tree(&dir, &[]).unwrap());
    fs::rename(
        dir.join("products/clay-mask.md"),
        dir.join("products/clay-mask-new.md"),
    )
    .unwrap();
    let cur = scan_markdown_tree(&dir, &[]).unwrap();
    let changes = observe_renames(&watch.effective_previous(), &cur);
    assert_eq!(changes.len(), 1);
    apply_path_changes(&dir, &changes).unwrap();
    let paired = paired_from_paths(&changes);
    let after = scan_markdown_tree(&dir, &[]).unwrap();
    watch.commit_scan(after, &paired);

    let body = fs::read_to_string(dir.join("products/clay-mask-new.md")).unwrap();
    assert!(
        body.contains("id: products/clay-mask-new"),
        "id must follow path: {body}"
    );
    let ref_body = fs::read_to_string(dir.join("ref.md")).unwrap();
    assert!(ref_body.contains("products/clay-mask-new"), "{ref_body}");
    assert!(!ref_body.contains("products/clay-mask\n") || ref_body.contains("clay-mask-new"));
    let diags = lint_workspace(&load_workspace(&dir).unwrap());
    assert!(
        diags.iter().all(|d| !d.message.contains("dangling")),
        "{diags:?}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn referenced_code_rename_is_observed_and_rewritten() {
    let _guard = WATCH_RENAME_TEST_LOCK.lock().unwrap();
    let dir = tmp();
    init_workspace(&dir, InitOptions::default()).unwrap();
    fs::create_dir_all(dir.join("src/old")).unwrap();
    fs::create_dir_all(dir.join("src/new")).unwrap();
    fs::write(dir.join("src/old/login.ts"), "export function login() {}\n").unwrap();
    fs::write(
        dir.join("feature.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: src/old/login.ts\n    symbol: login\n    role: implementation\n---\n\n# Feature\n",
    )
    .unwrap();

    let mut watch = WatchTree::from_scan(scan_markdown_tree(&dir, &[]).unwrap());
    fs::rename(dir.join("src/old/login.ts"), dir.join("src/new/login.ts")).unwrap();
    let cur = scan_markdown_tree(&dir, &[]).unwrap();
    let changes = observe_renames(&watch.effective_previous(), &cur);
    assert_eq!(changes.len(), 1, "{changes:?}");
    apply_path_changes(&dir, &changes).unwrap();
    let paired = paired_from_paths(&changes);
    watch.commit_scan(cur, &paired);

    let feature = fs::read_to_string(dir.join("feature.md")).unwrap();
    assert!(feature.contains("path: src/new/login.ts"), "{feature}");
    assert!(watch.pending_removed.is_empty());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn split_batch_delete_then_create_pairs() {
    let _guard = WATCH_RENAME_TEST_LOCK.lock().unwrap();
    let dir = tmp();
    init_workspace(&dir, InitOptions::default()).unwrap();
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A unique body 42\n",
    )
    .unwrap();
    fs::write(
        dir.join("ref.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# R\n",
    )
    .unwrap();
    let mut watch = WatchTree::from_scan(scan_markdown_tree(&dir, &[]).unwrap());

    fs::remove_file(dir.join("a.md")).unwrap();
    let mid = scan_markdown_tree(&dir, &[]).unwrap();
    assert!(observe_renames(&watch.effective_previous(), &mid).is_empty());
    watch.commit_scan(mid, &[]);

    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A unique body 42\n",
    )
    .unwrap();
    let cur = scan_markdown_tree(&dir, &[]).unwrap();
    let changes = observe_renames(&watch.effective_previous(), &cur);
    assert_eq!(changes.len(), 1, "{changes:?}");
    apply_path_changes(&dir, &changes).unwrap();
    let ref_body = fs::read_to_string(dir.join("ref.md")).unwrap();
    assert!(
        ref_body.contains("  - b\n") || ref_body.contains("- b"),
        "{ref_body}"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn heal_orphan_id_when_filename_already_new() {
    let _guard = WATCH_RENAME_TEST_LOCK.lock().unwrap();
    let dir = tmp();
    init_workspace(&dir, InitOptions::default()).unwrap();
    fs::create_dir_all(dir.join("products")).unwrap();
    fs::write(
        dir.join("products/clay-mask.md"),
        "---\nprofile: note\nstatus: draft\nid: products/clay-mask-new\n---\n\n# Clay\n",
    )
    .unwrap();
    fs::write(
        dir.join("ref.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - products/clay-mask-new\n---\n\n# R\n",
    )
    .unwrap();
    heal_orphan_path_ids(&dir).unwrap();
    let body = fs::read_to_string(dir.join("products/clay-mask.md")).unwrap();
    assert!(body.contains("id: products/clay-mask\n"), "{body}");
    let ref_body = fs::read_to_string(dir.join("ref.md")).unwrap();
    assert!(ref_body.contains("products/clay-mask"), "{ref_body}");
    assert!(!ref_body.contains("clay-mask-new"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn stable_id_without_slash_not_healed() {
    let _guard = WATCH_RENAME_TEST_LOCK.lock().unwrap();
    let dir = tmp();
    init_workspace(&dir, InitOptions::default()).unwrap();
    fs::write(
        dir.join("note.md"),
        "---\nprofile: note\nstatus: draft\nid: stable-handle\n---\n\n# N\n",
    )
    .unwrap();
    let report = heal_orphan_path_ids(&dir).unwrap();
    assert!(report.rewritten_files.is_empty());
    let body = fs::read_to_string(dir.join("note.md")).unwrap();
    assert!(body.contains("id: stable-handle"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn heal_orphan_path_ids_edge_cases() {
    let _guard = WATCH_RENAME_TEST_LOCK.lock().unwrap();
    let dir = tmp();
    init_workspace(&dir, InitOptions::default()).unwrap();
    fs::create_dir_all(dir.join("sub")).unwrap();
    // Document a.md claims id sub/b, but sub/b.md actually exists (owned_path_ids collision)
    fs::write(
        dir.join("sub/a.md"),
        "---\nprofile: note\nstatus: draft\nid: sub/b\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("sub/b.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# B\n",
    )
    .unwrap();

    // Plain doc with no frontmatter
    fs::write(dir.join("plain.md"), "# Plain\n").unwrap();

    let report = heal_orphan_path_ids(&dir).unwrap();
    // sub/a.md shouldn't be healed because sub/b is owned by real sub/b.md
    assert!(!report.rewritten_files.iter().any(|p| p.ends_with("sub/a.md")));
    let _ = fs::remove_dir_all(&dir);
}
