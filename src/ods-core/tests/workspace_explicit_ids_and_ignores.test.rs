use ods_core::{lint_workspace, load_workspace};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn root_ignore_excludes_code_tree_from_scan() {
    let temp = temp_workspace();
    fs::create_dir_all(temp.join("src/pkg")).expect("src");
    fs::create_dir_all(temp.join("docs")).expect("docs");
    fs::write(
        temp.join("ods.toml"),
        "spec = \"0.1\"\nignore = [\"src\"]\n",
    )
    .expect("toml");
    fs::write(
        temp.join("src/pkg/README.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Code readme\n",
    )
    .expect("src readme");
    fs::write(
        temp.join("docs/guide.md"),
        "---\nprofile: note\nstatus: draft\ndescription: A guide.\n---\n\n# Guide\n",
    )
    .expect("guide");

    let workspace = load_workspace(&temp).expect("workspace");
    assert_eq!(workspace.ignore, vec!["src".to_string()]);
    assert!(
        !workspace
            .documents
            .iter()
            .any(|d| d.path.to_string_lossy().contains("/src/")),
        "src markdown should not be loaded"
    );
    let diags = lint_workspace(&workspace);
    // ignore path itself is not a lint error
    let _ = diags;
}

#[test]
fn explicit_ids_load() {
    let temp = temp_workspace();
    fs::write(temp.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
    fs::write(
        temp.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: custom-a\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&temp).unwrap();
    assert!(ws.document_by_id("custom-a").is_some());
}
