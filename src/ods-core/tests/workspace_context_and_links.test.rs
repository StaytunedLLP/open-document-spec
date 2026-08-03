use ods_core::{generate_indexes, lint_workspace, load_workspace};
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

    let root_index = "---\nprofile: index\nods: 0.1\n---\n\n# Large Workspace\n";
    fs::write(temp.join("index.md"), root_index).expect("root index");

    // Generate indexes first (root + every group directory) so the hand-written
    // root marker above doesn't leave dangling links to ungenerated children.
    let workspace = load_workspace(&temp).expect("workspace");
    generate_indexes(&workspace).expect("generate indexes");

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
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [Auth/](Auth/index.md)\n- [login.md](login.md)\n",
    )
    .expect("root index");

    fs::create_dir_all(temp.join("Auth")).expect("auth dir");
    fs::write(
        temp.join("Auth").join("index.md"),
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
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [doc.md](doc.md)\n",
    )
    .expect("root index");

    fs::write(
        temp.join("doc.md"),
        "---\nprofile: note\nstatus: draft\ndescription: A simple feature description.\n---\n\n# Doc\n",
    )
    .expect("doc");

    let workspace = load_workspace(&temp).expect("workspace");
    let generated = generate_indexes(&workspace).expect("generate");
    assert!(generated.iter().any(|path| path.ends_with("index.md")));

    let rendered = fs::read_to_string(temp.join("index.md")).expect("read index");
    assert!(rendered.contains("- [doc.md](doc.md) - A simple feature description."));
}

#[test]
fn test_case_insensitive_relative_reference() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [README.md](README.md)\n- [child.md](child.md)\n",
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
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        r#"---
profile: index
ods: 0.1
ods: ">=0.0.1"
---

# Root Index

Welcome to this workspace.
Here are the active guidelines:
1. Always test.
2. Keep docs clean.

- [doc.md](doc.md)

This is the footer prose.
It explains where to report issues.
"#,
    )
    .expect("root index");

    fs::write(
        temp.join("doc.md"),
        "---\nprofile: note\nstatus: draft\ndescription: A simple feature description.\n---\n\n# Doc\n",
    )
    .expect("doc");

    let workspace = load_workspace(&temp).expect("workspace");
    let generated = generate_indexes(&workspace).expect("generate");
    assert!(generated.iter().any(|path| path.ends_with("index.md")));

    let rendered = fs::read_to_string(temp.join("index.md")).expect("read index");

    assert!(rendered.contains("profile: index"));
    assert!(rendered.contains("# Root Index"));
    assert!(rendered.contains("Welcome to this workspace."));
    assert!(rendered.contains("1. Always test."));
    assert!(rendered.contains("- [doc.md](doc.md) - A simple feature description."));
    assert!(rendered.contains("This is the footer prose."));
    assert!(rendered.contains("It explains where to report issues."));
}
