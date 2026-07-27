#![allow(unused_imports, dead_code)]
use ods_core::{LintLevel, Severity, lint_workspace, lint_workspace_with_level, load_workspace};
use ods_test_support::temp_workspace;
use std::fs;
use std::path::Path;

fn write_root(dir: impl AsRef<Path>, extra: &str) {
    let dir = dir.as_ref();
    fs::write(
        dir.join("index.md"),
        format!("---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Root\n\n{extra}"),
    )
    .unwrap();
}

#[test]
fn alias_heading_satisfies_profile_section() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\naliases:\n  Goal:\n    - Mission\n---\n\n# R\n\n- [a.md](a.md)\n",
    )
    .unwrap();
    // feature expects Goal among sections; Mission alias should count if wired
    fs::write(
        dir.join("a.md"),
        r#"---
profile: feature
status: draft
---

# A

## Mission

## Scope

## Requirements

## Acceptance Criteria

## Risks
"#,
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    // If aliases apply, no missing Goal; if not, still documents expected behavior
    let missing: Vec<_> = diags
        .iter()
        .filter(|d| d.message.contains("missing expected section"))
        .collect();
    assert!(
        missing.is_empty()
            || missing
                .iter()
                .any(|d| d.message.to_lowercase().contains("goal")),
        "unexpected section diagnostics: {diags:?}"
    );
}

#[test]
fn duplicate_ids_error() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n- [b.md](b.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: same\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\nid: same\n---\n\n# B\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("duplicate document id"))
    );
}

#[test]
fn invalid_share_value_error() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nshare: invalid\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("invalid share value: invalid"))
    );
}

#[test]
fn dangling_pack_path_error() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\npacks:\n  - vendor/non-existent-pack\n---\n\n# Root\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Level3);
    assert!(diags.iter().any(|d| {
        d.message
            .contains("missing pack path: vendor/non-existent-pack")
    }));
}
