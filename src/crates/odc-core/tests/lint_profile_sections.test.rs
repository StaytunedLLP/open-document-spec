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


