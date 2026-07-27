#![allow(unused_imports, dead_code)]
use ods_core::{
    AdoptOptions, adopt_workspace, generate_indexes, lint_workspace, load_workspace,
    move_document_and_rewrite_refs, profile_section_labels, resolve_context,
    workspace_alias_suggestions, workspace_aliases,
};
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

    let root_index =
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Large Workspace\n";
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
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n\n- [Auth/](Auth/index.md)\n- [login.md](login.md)\n",
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
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n\n- [doc.md](doc.md)\n",
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
fn test_body_link_validation() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n\n- [doc.md](doc.md)\n",
    )
    .expect("root index");

    // doc.md has a valid body link pointing to index.md and an invalid one pointing to missing.md
    fs::write(
        temp.join("doc.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n\n- [good link](index.md)\n- [bad link](missing.md)\n",
    )
    .expect("doc");

    let workspace = load_workspace(&temp).expect("workspace");
    let diagnostics = lint_workspace(&workspace);

    let dangling_errors = diagnostics
        .iter()
        .filter(|d| d.message.contains("dangling markdown link in body"))
        .collect::<Vec<_>>();

    assert_eq!(dangling_errors.len(), 1);
    assert!(dangling_errors[0].message.contains("missing.md"));
}

#[test]
fn context_ignore_skips_matching_paths() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n\n- [main.md](main.md)\n- [archive/](archive/index.md)\n",
    )
    .expect("root");
    fs::create_dir_all(temp.join("archive")).expect("archive");
    fs::write(
        temp.join("archive/index.md"),
        "---\nprofile: index\n---\n\n# Archive\n\n- [old.md](old.md)\n",
    )
    .expect("archive index");
    fs::write(
        temp.join("archive/old.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Old\n",
    )
    .expect("old");
    fs::write(
        temp.join("main.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - archive/old\ncontext:\n  max-depth: 2\n  ignore:\n    - archive/\n---\n\n# Main\n",
    )
    .expect("main");

    let workspace = load_workspace(&temp).expect("workspace");
    let resolved = resolve_context(&workspace, "main", true);
    let names: Vec<_> = resolved
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert_eq!(names, vec!["main.md".to_string()]);
    assert!(!names.iter().any(|n| n == "old.md"));
}

#[test]
fn depends_and_related_references_resolve_without_dangling() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n\n- [product.md](product.md)\n- [pricing.md](pricing.md)\n- [service.md](service.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("product.md"),
        "---\nprofile: product\nstatus: stable\ndepends:\n  - pricing\nrelated:\n  - service\n---\n\n# Product\n",
    )
    .expect("product");
    fs::write(
        temp.join("pricing.md"),
        "---\nprofile: decision\nstatus: stable\nid: pricing\n---\n\n# Pricing\n",
    )
    .expect("pricing");
    fs::write(
        temp.join("service.md"),
        "---\nprofile: feature\nstatus: stable\nid: service\n---\n\n# Service\n",
    )
    .expect("service");

    let workspace = load_workspace(&temp).expect("workspace");
    let path = temp.join("product.md");
    let diagnostics =
        ods_core::lint_document_in_workspace(&workspace, &path, ods_core::LintLevel::Level3);
    let dangling: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("dangling reference"))
        .collect();
    assert!(
        dangling.is_empty(),
        "unexpected dangling refs: {dangling:#?}"
    );
}

#[test]
fn adopt_write_adds_minimal_frontmatter() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n\n- [plain.md](plain.md)\n",
    )
    .expect("root");
    fs::write(temp.join("plain.md"), "# Plain\n\nJust prose.\n").expect("plain");

    let workspace = load_workspace(&temp).expect("workspace");
    let report = adopt_workspace(&workspace, AdoptOptions { write: true }).expect("adopt");
    assert_eq!(report.written.len(), 1);
    let text = fs::read_to_string(temp.join("plain.md")).expect("read");
    assert!(text.starts_with("---\nods:\n  profile: note\n  status: draft\n---\n"));
    assert!(text.contains("# Plain"));
}

#[test]
fn test_context_share_private_filtering() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.0.1\"\n---\n\n# Root\n",
    )
    .expect("root");
    fs::write(
        temp.join("public.md"),
        "---\nprofile: note\nstatus: stable\ndepends:\n  - private-doc\n---\n\n# Public Doc\n",
    )
    .expect("public");
    fs::write(
        temp.join("private-doc.md"),
        "---\nprofile: note\nstatus: stable\nid: private-doc\nshare: private\n---\n\n# Private Doc\n",
    )
    .expect("private");

    let workspace = load_workspace(&temp).expect("workspace");

    // When include_private = false, public.md context should exclude private-doc.md
    let paths_excluded = resolve_context(&workspace, "public", false);
    let names_excluded: Vec<_> = paths_excluded
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert_eq!(names_excluded, vec!["public.md".to_string()]);

    // When include_private = true, public.md context should include private-doc.md
    let paths_included = resolve_context(&workspace, "public", true);
    let names_included: Vec<_> = paths_included
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert_eq!(
        names_included,
        vec!["public.md".to_string(), "private-doc.md".to_string()]
    );
}
