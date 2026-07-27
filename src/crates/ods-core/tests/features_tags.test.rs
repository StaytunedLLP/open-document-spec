use ods_core::{
    docs_with_any_tag, load_workspace, normalize_tag_list, rename_tag_in_workspace,
    tag_usage_with_builtins,
};
use std::fs;

#[test]
fn tags_normalize_find_and_rename() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("index.md"),
        "---\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("a.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - Billing\n  - Old-CX\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("b.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - billing\n---\n\n# B\n",
    )
    .unwrap();
    assert_eq!(
        normalize_tag_list([" Billing ", "billing", "Old-CX"]),
        vec!["billing", "old-cx"]
    );
    let workspace = load_workspace(dir.path()).unwrap();
    assert_eq!(
        docs_with_any_tag(&workspace, &["billing".to_string()]).len(),
        2
    );
    let rows = tag_usage_with_builtins(&workspace, true);
    assert!(
        rows.iter()
            .any(|(tag, count, _)| tag == "billing" && *count == 2)
    );
    let report = rename_tag_in_workspace(&workspace, "old-cx", "customer-care", true).unwrap();
    assert_eq!(report.rewritten_files.len(), 1);
    let next = fs::read_to_string(dir.path().join("a.md")).unwrap();
    assert!(next.contains("customer-care"), "{next}");
    assert!(!next.contains("old-cx"), "{next}");
}
