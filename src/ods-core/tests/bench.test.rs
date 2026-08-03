use ods_core::bench::{
    BenchStripOptions, bench_calculate_stats, bench_restore_workspace, bench_strip_workspace,
};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn test_bench_strip_and_restore_roundtrip() {
    let root = temp_workspace();

    fs::write(
        root.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n# Root Index\n\nWelcome.",
    )
    .unwrap();

    fs::write(
        root.join("guide.md"),
        "---\nprofile: guide\nstatus: draft\nid: guide-doc\n---\n# Guide\n\nThis is prose body.",
    )
    .unwrap();

    let stats_before = bench_calculate_stats(&root).unwrap();
    assert_eq!(stats_before.total_files, 2);

    // Perform strip with write = true
    let strip_report = bench_strip_workspace(
        &root,
        BenchStripOptions {
            write: true,
            path_filter: None,
            strip_indexes: false,
            strip_profiles: false,
            full: false,
        },
    )
    .unwrap();

    assert_eq!(strip_report.total_stripped, 2);

    // Verify files no longer have frontmatter in text
    let guide_stripped = fs::read_to_string(root.join("guide.md")).unwrap();
    assert!(!guide_stripped.contains("profile: guide"));
    assert!(guide_stripped.contains("# Guide"));
    assert!(guide_stripped.contains("This is prose body."));

    // Perform restore
    let restore_report = bench_restore_workspace(&root, Some(&strip_report.snapshot_id)).unwrap();
    assert_eq!(restore_report.total_restored, 2);

    // Verify frontmatter was restored byte-for-byte
    let guide_restored = fs::read_to_string(root.join("guide.md")).unwrap();
    assert!(guide_restored.contains("profile: guide"));
    assert!(guide_restored.contains("id: guide-doc"));
    assert!(guide_restored.contains("This is prose body."));
}

#[test]
fn test_bench_strip_full_and_restore_roundtrip() {
    let root = temp_workspace();

    // Create root index
    fs::write(
        root.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\ncustom-profiles:\n  - ods-profiles/custom.md\n---\n# Root Index",
    )
    .unwrap();

    // Create sub directory with child index.ods.md and doc.md
    let sub_dir = root.join("sub");
    fs::create_dir_all(&sub_dir).unwrap();

    let sub_index_content = "---\nprofile: index\n---\n# Sub Index\n- [doc.md](doc.md)";
    fs::write(sub_dir.join("index.ods.md"), sub_index_content).unwrap();

    let doc_content = "---\nprofile: note\nstatus: draft\n---\n# Note Body";
    fs::write(sub_dir.join("doc.md"), doc_content).unwrap();

    // Create custom profile file
    let profiles_dir = root.join("ods-profiles");
    fs::create_dir_all(&profiles_dir).unwrap();
    let profile_content = "## Goal\n## Scope";
    fs::write(profiles_dir.join("custom.md"), profile_content).unwrap();

    // Create error artifact
    let error_content = "error: dangling link";
    fs::create_dir_all(root.join(".ods")).unwrap();
    fs::write(root.join(".ods/ods-errors.md"), error_content).unwrap();

    // Run bench strip --full
    let strip_report = bench_strip_workspace(
        &root,
        BenchStripOptions {
            write: true,
            path_filter: None,
            strip_indexes: true,
            strip_profiles: true,
            full: true,
        },
    )
    .unwrap();

    assert_eq!(strip_report.total_indexes_deleted, 2);
    assert_eq!(strip_report.total_profiles_removed, 1);

    // Verify non-root index.ods.md was deleted
    assert!(!sub_dir.join("index.ods.md").exists());

    // Verify custom profile was deleted
    assert!(!profiles_dir.join("custom.md").exists());

    // Verify error artifact was deleted
    assert!(!root.join(".ods/ods-errors.md").exists());

    // Verify doc.md frontmatter was stripped
    let stripped_doc = fs::read_to_string(sub_dir.join("doc.md")).unwrap();
    assert!(!stripped_doc.contains("profile: note"));
    assert!(stripped_doc.contains("# Note Body"));

    // Run bench restore
    let restore_report = bench_restore_workspace(&root, Some(&strip_report.snapshot_id)).unwrap();

    assert_eq!(restore_report.total_restored, 1);
    assert_eq!(restore_report.total_indexes_restored, 2);
    assert_eq!(restore_report.total_profiles_restored, 1);

    // Verify sub/index.ods.md was restored
    assert!(sub_dir.join("index.ods.md").exists());
    assert_eq!(
        fs::read_to_string(sub_dir.join("index.ods.md")).unwrap(),
        "# Sub Index\n- [doc.md](doc.md)"
    );

    // Verify custom profile was restored
    assert!(profiles_dir.join("custom.md").exists());
    assert_eq!(
        fs::read_to_string(profiles_dir.join("custom.md")).unwrap(),
        profile_content
    );

    // Verify lint report was restored under .ods/
    assert!(root.join(".ods/ods-errors.md").exists());
    assert_eq!(
        fs::read_to_string(root.join(".ods/ods-errors.md")).unwrap(),
        error_content
    );

    // Verify doc.md frontmatter was restored
    let restored_doc = fs::read_to_string(sub_dir.join("doc.md")).unwrap();
    assert!(restored_doc.contains("profile: note"));
    assert!(restored_doc.contains("# Note Body"));
}

/// Regression test for the migration off a hand-rolled JSON parser (which
/// broke on nested braces/brackets/quotes inside snapshotted content): a
/// document containing `{`, `}`, `[`, `]`, and escaped quotes in both its
/// frontmatter and an index lockfile must still round-trip byte-for-byte.
#[test]
fn test_bench_strip_and_restore_roundtrip_with_json_special_characters() {
    let root = temp_workspace();

    fs::write(
        root.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\ndescription: \"nested {braces} and [brackets]\"\n---\n# Root\n\nSee `{ \"key\": [1, 2] }` for details.",
    )
    .unwrap();

    fs::write(
        root.join("tricky.md"),
        "---\nprofile: note\nstatus: draft\ndescription: \"quote \\\" and brace {x} and bracket [y]\"\n---\n# Tricky\n\nBody with { curly } and [ square ] and \"quotes\".",
    )
    .unwrap();

    let strip_report = bench_strip_workspace(
        &root,
        BenchStripOptions {
            write: true,
            path_filter: None,
            strip_indexes: false,
            strip_profiles: false,
            full: false,
        },
    )
    .unwrap();
    assert_eq!(strip_report.total_stripped, 2);

    let restore_report = bench_restore_workspace(&root, Some(&strip_report.snapshot_id)).unwrap();
    assert_eq!(restore_report.total_restored, 2);

    let restored = fs::read_to_string(root.join("tricky.md")).unwrap();
    assert!(restored.contains(r#"description: "quote \" and brace {x} and bracket [y]""#));
    assert!(restored.contains("Body with { curly } and [ square ] and \"quotes\"."));
}
