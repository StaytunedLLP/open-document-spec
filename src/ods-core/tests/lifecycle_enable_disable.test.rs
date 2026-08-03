use ods_core::{
    DisableOptions, InitOptions, current_ods_spec_version, disable_workspace, init_workspace,
    load_workspace, ods_enabled,
};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn init_then_disable_round_trip_preserves_body() {
    let dir = temp_workspace();
    fs::write(dir.join("note.md"), "# Hello\n\nKeep this prose.\n").unwrap();

    assert!(!ods_enabled(&dir));

    let en = init_workspace(&dir, InitOptions { adopt: true }).unwrap();
    assert!(ods_enabled(&dir));
    assert!(en.initialized || en.already_initialized);
    assert!(!en.adopted.is_empty());

    let note = fs::read_to_string(dir.join("note.md")).unwrap();
    assert!(note.contains("profile:"));
    assert!(note.contains("Keep this prose."));

    let dry = disable_workspace(&dir, DisableOptions::default()).unwrap();
    assert!(!dry.already_disabled);
    assert!(dry.dry_run);
    assert!(!dry.would_edit.is_empty());
    // dry-run must not write
    assert!(
        fs::read_to_string(dir.join("note.md"))
            .unwrap()
            .contains("profile:")
    );

    let applied = disable_workspace(
        &dir,
        DisableOptions {
            write: true,
            ..DisableOptions::default()
        },
    )
    .unwrap();
    assert!(!applied.edited.is_empty());
    assert!(!ods_enabled(&dir));

    let note = fs::read_to_string(dir.join("note.md")).unwrap();
    assert!(!note.contains("profile:"));
    assert!(note.contains("Keep this prose."));
    assert!(!note.contains("---") || note.find("---").is_none());
}

#[test]
fn disable_keep_frontmatter_only_drops_ods_marker() {
    let dir = temp_workspace();
    init_workspace(&dir, InitOptions { adopt: false }).unwrap();
    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# D\n",
    )
    .unwrap();

    disable_workspace(
        &dir,
        DisableOptions {
            write: true,
            strip_frontmatter: false,
            strip_root_policy: true,
            remove_indexes: false,
            remove_root_index: false,
        },
    )
    .unwrap();

    assert!(!ods_enabled(&dir));
    let doc = fs::read_to_string(dir.join("doc.md")).unwrap();
    assert!(doc.contains("profile: note"));
    let root = fs::read_to_string(dir.join("index.ods.md")).unwrap();
    assert!(!root.lines().any(|l| l.trim().starts_with("ods:")));
}

#[test]
fn disable_remove_indexes_deletes_non_root_index() {
    let dir = temp_workspace();
    init_workspace(&dir, InitOptions { adopt: false }).unwrap();
    fs::create_dir_all(dir.join("nested")).unwrap();
    fs::write(
        dir.join("nested/doc.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n",
    )
    .unwrap();
    fs::write(
        dir.join("nested/index.ods.md"),
        "---\nprofile: index\n---\n\n# nested\n\n- [doc.md](doc.md)\n",
    )
    .unwrap();

    disable_workspace(
        &dir,
        DisableOptions {
            write: true,
            strip_frontmatter: true,
            strip_root_policy: true,
            remove_indexes: true,
            remove_root_index: false,
        },
    )
    .unwrap();

    assert!(!ods_enabled(&dir));
    assert!(dir.join("index.ods.md").exists());
    assert!(!dir.join("nested/index.ods.md").exists());
    assert!(dir.join("nested/doc.md").exists());
    assert!(
        fs::read_to_string(dir.join("nested/doc.md"))
            .unwrap()
            .contains("# Doc")
    );
}

#[test]
fn ods_enabled_false_without_marker() {
    let dir = temp_workspace();
    fs::write(dir.join("readme.md"), "# R\n").unwrap();
    assert!(!ods_enabled(&dir));
    assert!(!ods_core::ods_enabled_for_path(dir.join("readme.md")));
}

#[test]
fn init_on_existing_index_injects_ods() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\n---\n\n# Docs\n\n",
    )
    .unwrap();
    assert!(!ods_enabled(&dir));
    init_workspace(&dir, InitOptions::default()).unwrap();
    assert!(ods_enabled(&dir));
    let root = fs::read_to_string(dir.join("index.ods.md")).unwrap();
    assert!(
        root.contains(&format!("ods: {}", current_ods_spec_version())),
        "{root}"
    );
    let _ = load_workspace(&dir).unwrap();
}

#[test]
fn init_on_existing_index_updates_stale_ods_version() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: draft-1\n---\n\n# Docs\n\n",
    )
    .unwrap();

    init_workspace(&dir, InitOptions::default()).unwrap();

    let root = fs::read_to_string(dir.join("index.ods.md")).unwrap();
    assert!(
        root.contains(&format!("ods: {}", current_ods_spec_version())),
        "{root}"
    );
    assert!(!root.contains("ods: draft-1"), "{root}");
}

#[test]
fn disable_workspace_remove_root_index_and_already_disabled() {
    let dir = temp_workspace();
    init_workspace(&dir, InitOptions::default()).unwrap();
    assert!(ods_enabled(&dir));

    let rep = disable_workspace(
        &dir,
        DisableOptions {
            write: true,
            remove_root_index: true,
            ..DisableOptions::default()
        },
    )
    .unwrap();
    assert!(!rep.deleted.is_empty());
    assert!(!dir.join("index.md").exists());

    // Second call: already disabled
    let rep2 = disable_workspace(&dir, DisableOptions::default()).unwrap();
    assert!(rep2.already_disabled);
}
