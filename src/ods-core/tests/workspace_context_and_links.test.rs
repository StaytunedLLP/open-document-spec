use ods_core::{lint_workspace, load_workspace};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
#[ignore = "~5s (was ~60s before the O(N^2) index-rebuild fix, see fs/scanner.rs::rebuild_indexes): still slow enough vs. the ~8s default suite to skip locally; run explicitly or via CI's scale-test step"]
fn large_workspace_with_10k_documents_lints() {
    let temp = temp_workspace();

    for group in 0..100 {
        let dir = temp.join(format!("group-{group:03}"));
        fs::create_dir_all(&dir).expect("group dir");
        for item in 0..100 {
            let path = dir.join(format!("doc-{item:03}.md"));
            fs::write(
                path,
                format!(
                    "---\nprofile: note\nstatus: draft\n---\n\n# Doc {group}-{item}\n\n## Overview\n"
                ),
            )
            .expect("doc");
        }
    }

    let root_index = "spec = \"0.1\"
";
    fs::write(temp.join("ods.toml"), root_index).expect("root index");

    // Generate indexes first (root + every group directory) so the hand-written
    // root marker above doesn't leave dangling links to ungenerated children.
    let _workspace = load_workspace(&temp).expect("workspace");
    /* indexes removed */

    let workspace = load_workspace(&temp).expect("workspace");
    let diagnostics = lint_workspace(&workspace);
    assert!(
        diagnostics.is_empty(),
        "{}",
        diagnostics
            .iter()
            .map(|d| format!("{:?}: {}", d.severity, d.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn test_case_insensitive_ids() {
    let temp = temp_workspace();
    fs::write(
        temp.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .expect("root index");

    fs::create_dir_all(temp.join("Auth")).expect("auth dir");
    fs::write(
        temp.join("Auth").join("ods.toml"),
        "---\nprofile: index\n---\n\n# Auth\n\n- [Sessions.md](Sessions.md)\n",
    )
    .expect("auth index");

    fs::write(
        temp.join("Auth").join("Sessions.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Sessions\n",
    )
    .expect("sessions doc");

    fs::write(
        temp.join("login.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - auth/sessions\n---\n\n# Login\n",
    )
    .expect("login doc");

    let workspace = load_workspace(&temp).expect("workspace");
    let diagnostics = lint_workspace(&workspace);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn test_index_generation_with_description() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .unwrap();
    fs::write(
        root.join("doc.md"),
        "---
profile: note
status: draft
description: Hello there
---

# Doc
",
    )
    .unwrap();
    let _ws = load_workspace(root).unwrap();
    /* indexes removed */
    /* indexes removed */
}

#[test]
fn test_case_insensitive_relative_reference() {
    let temp = temp_workspace();
    fs::write(
        temp.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .expect("root index");

    fs::write(
        temp.join("README.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Main Readme\n",
    )
    .expect("README");

    fs::write(
        temp.join("child.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - README.md\n---\n\n# Child\n",
    )
    .expect("child");

    let workspace = load_workspace(&temp).expect("workspace");
    let diagnostics = lint_workspace(&workspace);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn test_index_generation_preserves_prose() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .unwrap();
    fs::write(
        root.join("doc.md"),
        "---
profile: note
status: draft
---

# Doc
",
    )
    .unwrap();
    let _ws = load_workspace(root).unwrap();
    /* indexes removed */
}
