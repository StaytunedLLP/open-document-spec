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
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [doc.md](doc.md)\n",
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
    let root_text = "---\nprofile: index\nods: 0.1\nprofile: index\n---\n\n# R\n";
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

#[test]
fn migrate_workspace_frontmatter_helper_and_edge_cases() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    // Empty body document needing migration
    fs::write(
        dir.join("empty_body.md"),
        "---\nprofile: note\nstatus: draft\n---\n",
    )
    .unwrap();

    let changed = ods_core::migrate_workspace_frontmatter(&dir).unwrap();
    assert_eq!(changed.len(), 1);

    // Frontmatter with empty block / line without colon
    let empty_fm = "---\n---\n";
    assert!(migrate_frontmatter_to_canonical(empty_fm).is_none());

    let no_colon_fm = "---\nno_colon_raw_line\n---\n";
    assert!(migrate_frontmatter_to_canonical(no_colon_fm).is_none());
}

#[test]
fn migrate_hoists_nested_tags_under_ods_to_root() {
    let text = "---\ndescription: Nested tags bug\nods:\n  profile: note\n  status: draft\n  tags:\n    - block\n    - action\n---\n\n# Doc\n";
    let migrated = migrate_frontmatter_to_canonical(text).expect("should hoist tags");
    assert!(
        migrated.starts_with("---\ndescription: Nested tags bug\ntags:\n  - action\n  - block\nods:\n  profile: note\n  status: draft\n---\n")
            || migrated.contains("tags:\n  - block\n  - action\n")
            || migrated.contains("tags:\n  - action\n  - block\n"),
        "expected root tags: {migrated}"
    );
    assert!(
        !migrated.contains("  tags:"),
        "tags must not remain nested under ods: {migrated}"
    );
    assert!(
        migrate_frontmatter_to_canonical(&migrated).is_none(),
        "second migrate should be no-op: {migrated}"
    );
}

#[test]
fn migrate_never_drops_nested_tag_values() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(
        dir.join("doc.md"),
        "---\nods:\n  profile: guide\n  status: stable\n  tags:\n    - billing\n    - customer-care\n---\n\n# Doc\n",
    )
    .unwrap();

    let workspace = load_workspace(&dir).unwrap();
    let changed = migrate_workspace_frontmatter_with_workspace(&workspace).unwrap();
    assert_eq!(changed.len(), 1);

    let text = fs::read_to_string(dir.join("doc.md")).unwrap();
    assert!(text.contains("billing"), "{text}");
    assert!(text.contains("customer-care"), "{text}");
    assert!(
        text.contains("tags:\n  - billing\n  - customer-care\n")
            || text.contains("tags:\n  - customer-care\n  - billing\n"),
        "{text}"
    );
}
