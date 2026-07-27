use ods_core::{known_profiles, lint_workspace, load_workspace};
use std::fs;

#[test]
fn profile_paths_and_conflicts_are_loaded() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("ods-profiles")).unwrap();
    fs::create_dir_all(dir.path().join("more-profiles")).unwrap();
    fs::write(
        dir.path().join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\nprofiles:\n  - ods-profiles\n  - more-profiles\n---\n\n# Root\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("ods-profiles/custom.md"),
        "# Custom\n\n## One\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("more-profiles/custom.md"),
        "# Custom\n\n## Two\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("doc.md"),
        "---\nprofile: custom\nstatus: draft\n---\n\n# Doc\n\n## One\n",
    )
    .unwrap();
    let workspace = load_workspace(dir.path()).unwrap();
    let profiles = known_profiles(&workspace);
    assert!(profiles.contains(&"custom".to_string()), "{profiles:?}");
    assert_eq!(workspace.profiles.conflicts.len(), 1);
    let diags = lint_workspace(&workspace);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("duplicate profile definition")),
        "{diags:?}"
    );
}
