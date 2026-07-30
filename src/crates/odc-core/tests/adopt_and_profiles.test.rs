use odc_core::{
    AdoptOptions, adopt_workspace, known_profiles, load_workspace, standard_profile_catalog,
};
use odc_test_support::temp_workspace;
use std::fs;

#[test]
fn standard_catalog_includes_core_profiles() {
    let cat = standard_profile_catalog();
    for name in [
        "note",
        "feature",
        "guide",
        "decision",
        "policy",
        "meeting",
        "index",
        "faq",
        "checklist",
        "api",
        "architecture",
        "sop",
    ] {
        assert!(
            cat.definitions.contains_key(name),
            "missing standard profile {name}"
        );
    }
}

#[test]
fn adopt_infers_feature_from_headings() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# R\n\n- [f.md](f.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("f.md"),
        "# F\n\n## Goal\n\n## Requirements\n\n## Acceptance Criteria\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let report = adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    assert_eq!(report.written.len(), 1);
    let text = fs::read_to_string(dir.join("f.md")).unwrap();
    assert!(text.contains("profile: feature"), "{text}");
    assert!(text.contains("status: draft"));
}

#[test]
fn adopt_infers_guide_and_policy() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# R\n\n- [g.md](g.md)\n- [p.md](p.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("g.md"),
        "# G\n\n## Prerequisites\n\n## Steps\n\n## Troubleshooting\n",
    )
    .unwrap();
    fs::write(
        dir.join("p.md"),
        "# P\n\n## Purpose\n\n## Rules\n\n## Exceptions\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    assert!(
        fs::read_to_string(dir.join("g.md"))
            .unwrap()
            .contains("profile: guide")
    );
    assert!(
        fs::read_to_string(dir.join("p.md"))
            .unwrap()
            .contains("profile: policy")
    );
}

#[test]
fn known_profiles_lists_standards() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# R\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let names = known_profiles(&ws);
    assert!(names.iter().any(|n| n == "feature"));
    assert!(names.iter().any(|n| n == "guide"));
}
