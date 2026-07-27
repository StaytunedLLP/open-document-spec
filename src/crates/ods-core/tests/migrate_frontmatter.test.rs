use ods_core::{
    load_workspace, migrate_frontmatter_to_canonical, migrate_workspace_frontmatter_with_workspace,
};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn migrate_flat_legacy_document_to_nested_ods_block() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n\n- [doc.md](doc.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("doc.md"),
        "---\ndescription: Refund flow\ntags:\n  - billing\nprofile: guide\nstatus: stable\ndepends:\n  - website/checkout.md\n---\n\n# Doc\n",
    )
    .unwrap();

    let workspace = load_workspace(&dir).unwrap();
    let changed = migrate_workspace_frontmatter_with_workspace(&workspace).unwrap();
    assert_eq!(changed.len(), 1);

    let text = fs::read_to_string(dir.join("doc.md")).unwrap();
    assert!(
        text.starts_with("---\ndescription: Refund flow\ntags:\n  - billing\nods:\n  profile: guide\n  status: stable\n  depends:\n    - website/checkout.md\n---\n"),
        "{text}"
    );
}

#[test]
fn migrate_reorders_out_of_order_nested_ods_subkeys() {
    let text = "---\ndescription: Doc\nods:\n  status: stable\n  profile: guide\n---\n\n# Doc\n";
    let migrated = migrate_frontmatter_to_canonical(text).expect("should reorder");
    assert!(
        migrated.contains("ods:\n  profile: guide\n  status: stable"),
        "{migrated}"
    );
}

#[test]
fn migrate_is_idempotent() {
    let text = "---\ndescription: Doc\nprofile: guide\nstatus: stable\n---\n\n# Doc\n";
    let once = migrate_frontmatter_to_canonical(text).expect("first run should change text");
    let twice = migrate_frontmatter_to_canonical(&once);
    assert!(twice.is_none(), "second run should be a no-op: {once}");
}

#[test]
fn migrate_skips_root_index_md() {
    let dir = temp_workspace();
    let root_text =
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\nprofile: index\n---\n\n# R\n";
    fs::write(dir.join("index.md"), root_text).unwrap();

    let workspace = load_workspace(&dir).unwrap();
    let changed = migrate_workspace_frontmatter_with_workspace(&workspace).unwrap();
    assert!(changed.is_empty(), "{changed:?}");
    assert_eq!(fs::read_to_string(dir.join("index.md")).unwrap(), root_text);
}

#[test]
fn migrate_skips_scalar_ods_marker_anywhere() {
    let text = "---\nprofile: guide\nods: 0.1\n---\n\n# Doc\n";
    assert!(migrate_frontmatter_to_canonical(text).is_none());
}

#[test]
fn migrate_skips_documents_with_no_engine_keys() {
    let text = "---\ndescription: Just a note\ntags:\n  - misc\n---\n\n# Doc\n";
    assert!(migrate_frontmatter_to_canonical(text).is_none());
}

#[test]
fn migrate_skips_documents_with_no_frontmatter() {
    let text = "# Doc\n\nJust prose.\n";
    assert!(migrate_frontmatter_to_canonical(text).is_none());
}

#[test]
fn migrate_later_key_wins_on_duplicate() {
    let text = "---\nprofile: note\nods:\n  profile: guide\n  status: draft\n---\n\n# Doc\n";
    let migrated = migrate_frontmatter_to_canonical(text).expect("should migrate");
    assert!(migrated.contains("profile: guide"), "{migrated}");
    assert!(!migrated.contains("profile: note"), "{migrated}");

    let reversed = "---\nods:\n  profile: guide\n  status: draft\nprofile: note\n---\n\n# Doc\n";
    let migrated_reversed = migrate_frontmatter_to_canonical(reversed).expect("should migrate");
    assert!(
        migrated_reversed.contains("profile: note"),
        "{migrated_reversed}"
    );
    assert!(
        !migrated_reversed.contains("profile: guide"),
        "{migrated_reversed}"
    );
}

#[test]
fn migrate_preserves_universal_top_level_owner_list_formatting() {
    let text = "---\nowner:\n  - a\n  - b\nprofile: note\nstatus: draft\n---\n\n# Doc\n";
    let migrated = migrate_frontmatter_to_canonical(text).expect("should migrate");
    assert!(migrated.contains("owner:\n  - a\n  - b\n"), "{migrated}");
}
