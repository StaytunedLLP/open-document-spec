use odc_core::{
    LintLevel, Severity, current_ods_spec_version, lint_workspace, lint_workspace_with_level,
    load_workspace,
};
use odc_test_support::temp_workspace;
use std::fs;
use std::path::Path;

fn write_root(dir: impl AsRef<Path>, extra: &str) {
    let dir = dir.as_ref();
    fs::write(
        dir.join("index.md"),
        format!("---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n\n{extra}"),
    )
    .unwrap();
}

#[test]
fn duplicate_tag_warns() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - billing\n  - Billing\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    // Both normalize to billing → duplicate after normalize
    assert!(
        ws.documents
            .iter()
            .any(|d| matches!(&d.frontmatter, odc_core::FrontmatterState::Parsed(fm) if fm.tags == vec!["billing".to_string(), "billing".to_string()])),
        "expected normalized duplicate tags"
    );
    let diags = lint_workspace_with_level(&ws, LintLevel::Level1);
    assert!(
        diags
            .iter()
            .any(|d| d.severity == Severity::Warning && d.message.contains("duplicate tag")),
        "{diags:?}"
    );
}

#[test]
fn tag_index_builds_from_workspace() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n- [b.md](b.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - oncall\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - oncall\n  - billing\n---\n\n# B\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    assert_eq!(ws.tag_index.get("oncall").map(|v| v.len()), Some(2));
    assert_eq!(ws.tag_index.get("billing").map(|v| v.len()), Some(1));
    let tags = odc_core::completion_tags(&ws);
    assert!(tags.iter().any(|t| t == "oncall"));
    assert!(tags.iter().any(|t| t == "security")); // builtin unused
}

#[test]
fn invalid_status_errors() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: WIP\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags.iter().any(|d| d.message.contains("invalid status")),
        "{diags:?}"
    );
}

#[test]
fn stale_root_ods_version_errors() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: draft-1\n---\n\n# Root\n\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Level1);
    assert!(
        diags.iter().any(|d| {
            d.severity == Severity::Error
                && d.message.contains("root ods spec version mismatch")
                && d.message.contains(current_ods_spec_version())
        }),
        "{diags:?}"
    );
}

#[test]
fn invalid_or_missing_odc_errors() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=999.0.0\"\n---\n\n# Root\n\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Level1);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("root odc requirement not satisfied")),
        "{diags:?}"
    );

    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Level1);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("root index.md missing odc")),
        "{diags:?}"
    );
}

#[test]
fn unknown_profile_warns() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: not-a-real-profile\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| { d.severity == Severity::Warning && d.message.contains("unknown profile") })
    );
}

#[test]
fn dangling_reference_errors() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/doc\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("dangling reference"))
    );
}

#[test]
fn depends_cycle_errors() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n- [b.md](b.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - b\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# B\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags.iter().any(|d| d.message.contains("cycle")),
        "{diags:?}"
    );
}

#[test]
fn missing_resource_errors() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nresources:\n  - path: ./nope.csv\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(diags.iter().any(|d| d.message.contains("missing resource")));
}

#[test]
fn code_paths_are_validated_at_level3() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/a.ts"), "export function a() {}\n").unwrap();
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: ./src/a.ts\n    symbol: a\n    role: implementation\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("missing code path")),
        "{diags:?}"
    );

    fs::remove_file(dir.join("src/a.ts")).unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("missing code path")),
        "{diags:?}"
    );
}

#[test]
fn invalid_code_role_is_a_level1_error() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: ./src/a.ts\n    role: controller\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Level1);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("frontmatter parse error")
                && d.message.contains("invalid code role")),
        "{diags:?}"
    );
}

#[test]
fn level1_skips_dangling() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - gone\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Level1);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("dangling reference"))
    );
}

#[test]
fn aliases_on_non_root_warn() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\naliases:\n  Goal:\n    - Mission\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(diags.iter().any(|d| d.message.contains("root index")));
}

#[test]
fn missing_expected_section_warns_for_feature() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: feature\nstatus: draft\n---\n\n# A\n\n## Only One\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("missing expected section")),
        "{diags:?}"
    );
}

#[test]
fn complete_feature_profile_has_no_missing_section_warn() {
    let dir = temp_workspace();
    write_root(&dir, "- [feat.md](feat.md)\n");
    fs::write(
        dir.join("feat.md"),
        r#"---
profile: feature
status: draft
---

# Feat

## Goal

## Scope

## Requirements

## Acceptance Criteria

## Risks
"#,
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        !diags
            .iter()
            .any(|d| d.message.contains("missing expected section")),
        "{diags:?}"
    );
}

#[test]
fn lint_canonical_edge_cases() {
    let dir = temp_workspace();
    // Missing root index.md
    let ws_no_root = load_workspace(&dir).unwrap();
    let diags_no_root = odc_core::lint_workspace(&ws_no_root);
    assert!(diags_no_root.iter().any(|d| d.message.contains("missing root index.md")));

    write_root(&dir, "- [a.md](a.md)\n");
    // Non-root declaring ods/odc
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nods: 0.1\ncontext:\n  load:\n    - missing_res.csv\n    - dangling_id\n  ignore:\n    - nonexistent_target\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(diags.iter().any(|d| d.message.contains("ods and odc should be declared only in root index.md")));
    assert!(diags.iter().any(|d| d.message.contains("missing context resource")));
    assert!(diags.iter().any(|d| d.message.contains("dangling context reference")));
    assert!(diags.iter().any(|d| d.message.contains("context ignore target not found")));
}
