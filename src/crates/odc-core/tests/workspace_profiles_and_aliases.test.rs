#![allow(unused_imports, dead_code)]
use odc_core::{
    AdoptOptions, adopt_workspace, generate_indexes, lint_workspace, load_workspace,
    move_document_and_rewrite_refs, profile_section_labels, resolve_context,
    workspace_alias_suggestions, workspace_aliases,
};
use odc_test_support::{copy_fixture_to_temp, temp_workspace};
use std::fs;

fn fixture_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join("ods-test/ecommerce")
        .canonicalize()
        .expect("fixture root")
}

#[test]
fn sample_workspace_lints_cleanly() {
    let workspace = load_workspace(fixture_root()).expect("workspace");
    let diags = lint_workspace(&workspace);
    assert!(
        diags.is_empty(),
        "expected clean sample workspace, got: {diags:?}"
    );
}

#[test]
fn lint_document_in_workspace_unparsed_frontmatter() {
    let dir = odc_test_support::temp_workspace();
    std::fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n",
    )
    .unwrap();
    std::fs::write(dir.join("plain.md"), "# Plain\n").unwrap();

    let ws = load_workspace(&dir).unwrap();
    let diags = odc_core::lint_document_in_workspace(
        &ws,
        &dir.join("plain.md"),
        odc_core::LintLevel::Level3,
    );
    assert!(diags.is_empty());
}

#[test]
fn context_resolution_follows_depends_chain_deterministically() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n\n- [checkout.md](checkout.md)\n- [pricing.md](pricing.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("checkout.md"),
        "---\nprofile: feature\nstatus: stable\nid: checkout\ndepends:\n  - pricing\n---\n\n# Checkout\n",
    )
    .expect("checkout");
    fs::write(
        temp.join("pricing.md"),
        "---\nprofile: decision\nstatus: stable\nid: pricing\n---\n\n# Pricing\n",
    )
    .expect("pricing");

    let workspace = load_workspace(&temp).expect("workspace");
    let resolved = resolve_context(&workspace, "checkout", true);
    let names = resolved
        .iter()
        .map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap()
                .to_string()
        })
        .collect::<Vec<_>>();
    assert_eq!(names, vec!["checkout.md", "pricing.md"]);
}

#[test]
fn index_generation_matches_workspace_children() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n",
    )
    .expect("root index");
    for folder in ["alpha", "beta", "gamma"] {
        fs::create_dir_all(temp.join(folder)).expect("child dir");
        fs::write(
            temp.join(folder).join("index.md"),
            "---\nprofile: index\n---\n\n# child\n",
        )
        .expect("child index");
        // A folder containing only an index.md (no real content) is an
        // orphan and gets pruned rather than listed as a child.
        fs::write(
            temp.join(folder).join("doc.md"),
            "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n",
        )
        .expect("child doc");
    }

    let workspace = load_workspace(&temp).expect("workspace");
    let _generated = generate_indexes(&workspace).expect("generate");
    // Write-if-changed may return empty when indexes are already correct.
    assert!(
        odc_core::indexes_are_current(&load_workspace(&temp).expect("reload")).expect("check"),
        "indexes should be current after generate"
    );
    let rendered = fs::read_to_string(temp.join("index.md")).expect("read index");
    let mut children = rendered
        .lines()
        .filter_map(odc_core::split_markdown_link_target)
        .collect::<Vec<_>>();
    children.sort();
    assert_eq!(
        children,
        vec![
            "alpha/index.md".to_string(),
            "beta/index.md".to_string(),
            "gamma/index.md".to_string(),
        ]
    );
}

#[test]
fn mv_rewrites_references() {
    let temp = copy_fixture_to_temp();
    fs::write(
        temp.join("test-doc-a.md"),
        "---\nprofile: note\nstatus: stable\ndepends:\n  - test-doc-b\n---\n\nLink: [B](test-doc-b.md)\n",
    )
    .unwrap();
    fs::write(
        temp.join("test-doc-b.md"),
        "---\nprofile: note\nstatus: stable\n---\n",
    )
    .unwrap();

    move_document_and_rewrite_refs(&temp, "test-doc-b.md", "test-doc-b-new.md").expect("mv");

    let content_a = fs::read_to_string(temp.join("test-doc-a.md")).expect("read A");
    assert!(content_a.contains("test-doc-b-new"));
    assert!(content_a.contains("test-doc-b-new.md"));
}

#[test]
fn root_aliases_allow_workspace_vocab_variants() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\naliases:\n  Goal:\n    - Mission\n---\n\n# Root\n\n- [feature.md](feature.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("feature.md"),
        "---\nprofile: feature\nstatus: draft\n---\n\n# Feature\n\n## Mission\n## Scope\n## Requirements\n## Acceptance Criteria\n## Risks\n",
    )
    .expect("feature");

    let workspace = load_workspace(&temp).expect("workspace");
    assert!(workspace_aliases(&workspace).contains_key("Goal"));
    let diagnostics = lint_workspace(&workspace);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert!(
        profile_section_labels(&workspace, "feature")
            .iter()
            .any(|label| label == "Mission")
    );
}

#[test]
fn custom_profiles_are_loaded_from_workspace_catalogs() {
    let temp = temp_workspace();
    fs::create_dir_all(temp.join("ods-profiles")).expect("catalog dir");
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\nprofiles:\n  - ods-profiles\n---\n\n# Root\n\n- [doc.md](doc.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("ods-profiles").join("custom.md"),
        "---\naliases:\n  Overview:\n    - Summary\n---\n\n# Custom Profile\n\n## Overview\n## Details\n",
    )
    .expect("profile");
    fs::write(
        temp.join("doc.md"),
        "---\nprofile: custom\nstatus: draft\n---\n\n# Doc\n\n## Summary\n## Details\n",
    )
    .expect("doc");

    let workspace = load_workspace(&temp).expect("workspace");
    assert!(workspace.profiles.definitions.contains_key("custom"));
    assert!(
        workspace
            .documents
            .iter()
            .all(|document| !document.path.starts_with(temp.join("ods-profiles")))
    );

    let diagnostics = lint_workspace(&workspace);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    let _generated = generate_indexes(&workspace).expect("generate");
    let rendered = fs::read_to_string(temp.join("index.md")).expect("read index");
    assert!(rendered.contains("[doc.md](doc.md)"), "{rendered}");
    assert!(rendered.contains("profiles:"), "{rendered}");
    assert!(rendered.contains("- ods-profiles"), "{rendered}");
    assert!(
        profile_section_labels(&workspace, "custom")
            .iter()
            .any(|label| label == "Summary")
    );
}

#[test]
fn duplicate_profile_names_are_reported() {
    let temp = temp_workspace();
    fs::create_dir_all(temp.join("ods-profiles")).expect("catalog dir");
    fs::create_dir_all(temp.join("more-profiles")).expect("extra catalog dir");
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\nprofiles:\n  - more-profiles\n---\n\n# Root\n",
    )
    .expect("root index");
    fs::write(
        temp.join("ods-profiles").join("custom.md"),
        "# Custom\n\n## Overview\n## Details\n",
    )
    .expect("default profile");
    fs::write(
        temp.join("more-profiles").join("custom.md"),
        "# Custom Override\n\n## Overview\n## Details\n",
    )
    .expect("override profile");

    let workspace = load_workspace(&temp).expect("workspace");
    let diagnostics = lint_workspace(&workspace);
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("duplicate profile definition")),
        "{diagnostics:#?}"
    );
    assert_eq!(
        workspace
            .profiles
            .definitions
            .get("custom")
            .expect("custom")
            .source,
        temp.join("more-profiles")
            .join("custom.md")
            .canonicalize()
            .unwrap()
    );
}

#[test]
fn adopt_suggests_workspace_aliases_from_unmatched_headings() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n\n- [feature.md](feature.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("feature.md"),
        "---\nprofile: feature\nstatus: draft\n---\n\n# Feature\n\n## Mission\n## Scope\n## Requirements\n## Acceptance Criteria\n## Risks\n",
    )
    .expect("feature");

    let workspace = load_workspace(&temp).expect("workspace");
    let suggestions = workspace_alias_suggestions(&workspace);

    assert!(suggestions.contains_key("Goal"), "{suggestions:#?}");
    assert!(
        suggestions.get("Goal").expect("goal").contains("Mission"),
        "{suggestions:#?}"
    );
}
