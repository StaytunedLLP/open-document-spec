use ods_core::{LintLevel, lint_workspace, lint_workspace_with_level, load_workspace};
use ods_test_support::temp_workspace;
use std::fs;
use std::path::Path;

fn write_root(dir: impl AsRef<Path>, extra: &str) {
    let dir = dir.as_ref();
    fs::write(
        dir.join("index.md"),
        format!("---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n{extra}"),
    )
    .unwrap();
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
    let ws_no_root = load_workspace(&dir).unwrap();
    let diags_no_root = ods_core::lint_workspace(&ws_no_root);
    assert!(
        diags_no_root
            .iter()
            .any(|d| d.message.contains("missing root index.ods.md"))
    );

    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nods: 0.1\ncontext:\n  load:\n    - missing_res.csv\n    - dangling_id\n  ignore:\n    - nonexistent_target\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(diags.iter().any(|d| {
        d.message
            .contains("ods and ods should be declared only in root index.ods.md")
    }));
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("missing context resource"))
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("dangling context reference"))
    );
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("context ignore target not found"))
    );
}
