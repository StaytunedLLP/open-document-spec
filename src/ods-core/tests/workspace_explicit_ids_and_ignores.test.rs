use ods_core::{generate_indexes, lint_workspace, load_workspace};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn root_ignore_excludes_code_tree_from_scan_and_index() {
    let temp = temp_workspace();
    fs::create_dir_all(temp.join("src/pkg")).expect("src");
    fs::create_dir_all(temp.join("docs")).expect("docs");
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nignore:\n  - src\n---\n\n# Root\n\n- [docs/](docs/index.md)\n",
    )
    .expect("root");
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
            .any(|d| d.path.to_string_lossy().contains("src")),
        "src markdown should not be loaded"
    );
    let root = temp.path().canonicalize().unwrap();
    assert!(workspace.children.get(root.as_path()).is_some_and(|c| {
        c.iter().any(|e| e == "docs/index.md") && !c.iter().any(|e| e.contains("src"))
    }));

    let diagnostics = lint_workspace(&workspace);
    let index_errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.path.ends_with("index.md") && d.message.contains("src"))
        .collect();
    assert!(
        index_errors.is_empty(),
        "unexpected index diagnostics about src: {index_errors:#?}"
    );

    generate_indexes(&workspace).expect("index");
    let root_index = fs::read_to_string(temp.join("index.md")).expect("read root");
    assert!(root_index.contains("ignore:\n  - src\n"));
    assert!(!root_index.contains("src/index.md"));
    assert!(!temp.join("src/index.md").exists());
}

#[test]
fn index_lint_ignores_prose_links_outside_list() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\nSee [elsewhere](../outside.md) for notes.\n\n- [doc.md](doc.md)\n",
    )
    .expect("root");
    fs::write(
        temp.join("doc.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n",
    )
    .expect("doc");

    let workspace = load_workspace(&temp).expect("workspace");
    let diagnostics = lint_workspace(&workspace);
    let extras: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("extra entries"))
        .collect();
    assert!(
        extras.is_empty(),
        "prose links should not count as index children: {extras:#?}"
    );
}

#[test]
fn workspace_load_reports_explicit_ids() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [a.md](a.md)\n- [b.md](b.md)\n- [c.md](c.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("a.md"),
        "---\nprofile: decision\nstatus: stable\nid: pricing/subscription\n---\n\n# A\n",
    )
    .expect("a");
    fs::write(
        temp.join("b.md"),
        "---\nprofile: feature\nstatus: stable\nid: website/subscription-service\n---\n\n# B\n",
    )
    .expect("b");
    fs::write(
        temp.join("c.md"),
        "---\nprofile: product\nstatus: stable\n---\n\n# C\n",
    )
    .expect("c");

    let ws = load_workspace(&temp).expect("load");
    assert!(ws.documents.len() > 3);
    assert!(ws.by_id.contains_key("pricing/subscription"));
    assert!(ws.by_id.contains_key("website/subscription-service"));
}

#[test]
fn explicit_id_resolves_for_depends_context_and_lint() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [impl.md](impl.md)\n- [spec.md](spec.md)\n",
    )
    .expect("root");
    fs::write(
        temp.join("impl.md"),
        "---\nprofile: note\nstatus: draft\nid: stable/handle\n---\n\n# Impl\n",
    )
    .expect("impl");
    fs::write(
        temp.join("spec.md"),
        "---\nprofile: feature\nstatus: draft\ndepends:\n  - stable/handle\n---\n\n# Spec\n\n## Goal\n\n## Scope\n\n## Requirements\n\n## Acceptance Criteria\n\n## Risks\n",
    )
    .expect("spec");

    let ws = load_workspace(&temp).expect("load");
    assert!(ws.document_by_id("stable/handle").is_some());
    assert!(ws.document_by_id("impl").is_none() || ws.by_id.contains_key("stable/handle"));

    let paths = ods_core::resolve_context(&ws, "stable/handle", true);
    assert!(
        paths.iter().any(|p| p.ends_with("impl.md")),
        "context by explicit id: {paths:?}"
    );

    let diags = lint_workspace(&ws);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("dangling reference")),
        "depends on explicit id should resolve: {diags:?}"
    );
}
