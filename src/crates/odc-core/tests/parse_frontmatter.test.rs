use odc_core::{
    CodeRole, FrontmatterState, document_id, extract_heading_groups, extract_headings,
    parse_document_text, split_frontmatter, split_markdown_link_target,
};
use std::path::PathBuf;

#[test]
fn split_frontmatter_absent() {
    let (fm, body) = split_frontmatter("# Hi\n");
    assert!(fm.is_none());
    assert!(body.starts_with("# Hi"));
}

#[test]
fn split_frontmatter_present() {
    let text = "---\nprofile: note\n---\n\n# Title\n";
    let (fm, body) = split_frontmatter(text);
    assert_eq!(fm.unwrap().trim(), "profile: note");
    assert!(body.contains("# Title"));
}

#[test]
fn parse_resources_path_only_ignores_type() {
    let root = PathBuf::from("/ws");
    let path = root.join("doc.md");
    let text = r#"---
profile: note
status: draft
resources:
  - path: ./data.csv
    type: csv
  - path: ./pic.png
---

# Doc
"#;
    let doc = parse_document_text(&root, path, text, true);
    match doc.frontmatter {
        FrontmatterState::Parsed(fm) => {
            assert_eq!(fm.resources.len(), 2);
            assert!(fm.resources[0].path.ends_with("data.csv"));
        }
        other => panic!("expected parsed: {other:?}"),
    }
}

#[test]
fn parse_context_block() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: note
status: draft
context:
  max-depth: 2
  load:
    - a/b
  ignore:
    - archive/
---

# Doc
"#;
    let doc = parse_document_text(&root, root.join("d.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    let ctx = fm.context.expect("context");
    assert_eq!(ctx.max_depth, Some(2));
    assert_eq!(ctx.load, vec!["a/b".to_string()]);
    assert_eq!(ctx.ignore, vec!["archive/".to_string()]);
}

#[test]
fn parse_context_ignores_legacy_max_depth_key() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: note
context:
  max_depth: 2
---

# Doc
"#;
    let doc = parse_document_text(&root, root.join("d.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.context.expect("context").max_depth, None);
}

#[test]
fn parse_code_refs_with_fixed_roles() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: feature
status: draft
code:
  - path: src/routes/login.tsx
    symbol: LoginRoute
    role: Entrypoint
  - path: src/auth/session.rs
    symbol: create_session
    role: implementation
  - path: src/auth/session.test.ts
    role: test
  - path: src/schema/user.ts
    role: schema
  - path: db/migrations/001.sql
    role: migration
  - path: src/flags.ts
    role: config
  - path: infra/main.tf
    role: infrastructure
  - path: .github/workflows/ci.yml
    role: pipeline
---

# Feature
"#;
    let doc = parse_document_text(&root, root.join("feature.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.code.len(), 8);
    assert_eq!(fm.code[0].role, CodeRole::Entrypoint);
    assert_eq!(fm.code[0].symbol.as_deref(), Some("LoginRoute"));
    assert_eq!(fm.code[1].role.as_str(), "implementation");
    assert_eq!(fm.code[7].role, CodeRole::Pipeline);
}

#[test]
fn parse_code_refs_reject_missing_path_missing_role_and_invalid_role() {
    let root = PathBuf::from("/ws");
    for (text, expected) in [
        (
            "---\ncode:\n  - role: implementation\n---\n\n# D\n",
            "code entry missing path",
        ),
        (
            "---\ncode:\n  - path: src/a.ts\n---\n\n# D\n",
            "code entry missing role",
        ),
        (
            "---\ncode:\n  - path: src/a.ts\n    role: controller\n---\n\n# D\n",
            "invalid code role: controller",
        ),
    ] {
        let doc = parse_document_text(&root, root.join("d.md"), text, true);
        match doc.frontmatter {
            FrontmatterState::Invalid(message) => assert!(
                message.contains(expected),
                "expected {expected}, got {message}"
            ),
            other => panic!("expected invalid frontmatter: {other:?}"),
        }
    }
}

#[test]
fn parse_aliases_map() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: index
aliases:
  Goal:
    - Mission
    - Objective
---

# Root
"#;
    let doc = parse_document_text(&root, root.join("index.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(
        fm.aliases.get("Goal").map(|v| v.as_slice()),
        Some(["Mission".to_string(), "Objective".to_string()].as_slice())
    );
}

#[test]
fn parse_ignore_and_profiles_lists() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: index
ods: 0.1
odc: ">=0.0.1"
profiles:
  - ods-profiles
ignore:
  - src
  - apps/web/
---

# Root
"#;
    let doc = parse_document_text(&root, root.join("index.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.ods.as_deref(), Some("0.1"));
    assert_eq!(fm.odc.as_deref(), Some(">=0.0.1"));
    assert_eq!(fm.profiles, vec!["ods-profiles".to_string()]);
    assert_eq!(fm.ignore, vec!["src".to_string(), "apps/web".to_string()]);
}

#[test]
fn document_id_path_and_explicit() {
    let root = PathBuf::from("/ws");
    let path = root.join("features/login.md");
    let id = document_id(&root, &path, None);
    assert_eq!(id, "features/login");

    let text = "---\nid: stable-login\nprofile: note\n---\n\n# X\n";
    let doc = parse_document_text(&root, path.clone(), text, true);
    let FrontmatterState::Parsed(fm) = &doc.frontmatter else {
        panic!();
    };
    assert_eq!(document_id(&root, &path, Some(fm)), "stable-login");
}

#[test]
fn extract_headings_and_groups() {
    let body = "# T\n\n## Goal | Objective\n\n## Scope\n";
    assert_eq!(extract_headings(body), vec!["Goal", "Scope"]);
    let groups = extract_heading_groups(body);
    assert_eq!(groups[0], vec!["Goal", "Objective"]);
}

#[test]
fn markdown_link_target() {
    assert_eq!(
        split_markdown_link_target("- [x](foo/bar.md)"),
        Some("foo/bar.md".to_string())
    );
    assert!(split_markdown_link_target("no link").is_none());
}

#[test]
fn status_normalized_lowercase() {
    let root = PathBuf::from("/ws");
    let text = "---\nprofile: Note\nstatus: Draft\n---\n\n# D\n";
    let doc = parse_document_text(&root, root.join("d.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!();
    };
    assert_eq!(fm.profile.as_deref(), Some("note"));
    assert_eq!(fm.status.as_deref(), Some("draft"));
}

#[test]
fn include_body_false_still_has_headings() {
    let root = PathBuf::from("/ws");
    let text = "---\nprofile: note\n---\n\n# T\n\n## Overview\n";
    let doc = parse_document_text(&root, root.join("d.md"), text, false);
    assert!(doc.body.is_empty());
    assert_eq!(doc.headings, vec!["Overview"]);
}

#[test]
fn test_parse_pattern_b_nested_ods_map() {
    let root = PathBuf::from("/ws");
    let text = r#"---
description: Refund processing guide
tags:
  - billing
  - support
owner:
  - support-team
  - billing-ops

ods:
  profile: guide
  status: stable
  id: refund-flow
  share: public
  depends:
    - ../checkout/cart.md
  related:
    - ../policy/faq.md
  resources:
    - path: docs/flow.pdf
  code:
    - path: apps/web/src/refund.ts
      role: implementation
      symbol:
        - processRefund
        - validateRefund
  context:
    max-depth: 2
    load:
      - ../checkout/cart.md
    ignore:
      - archive/
---

# Refund Processing Guide
"#;
    let doc = parse_document_text(&root, root.join("refund.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("expected parsed frontmatter, got {:?}", doc.frontmatter);
    };

    assert_eq!(fm.description.as_deref(), Some("Refund processing guide"));
    assert_eq!(fm.tags, vec!["billing", "support"]);
    assert_eq!(fm.owner.as_deref(), Some("support-team, billing-ops"));
    assert_eq!(fm.profile.as_deref(), Some("guide"));
    assert_eq!(fm.status.as_deref(), Some("stable"));
    assert_eq!(fm.id.as_deref(), Some("refund-flow"));
    assert_eq!(fm.share.as_deref(), Some("public"));
    assert_eq!(fm.depends, vec!["../checkout/cart.md"]);
    assert_eq!(fm.related, vec!["../policy/faq.md"]);
    assert_eq!(fm.resources.len(), 1);
    assert_eq!(fm.code.len(), 1);
    assert_eq!(
        fm.code[0].symbol.as_deref(),
        Some("processRefund, validateRefund")
    );
    assert_eq!(fm.context.expect("context").max_depth, Some(2));
}

#[test]
fn nested_ods_block_tolerates_key_order() {
    let root = PathBuf::from("/ws");
    let text = "---\nods:\n  status: stable\n  depends:\n    - ../a.md\n  profile: guide\n  share: public\n---\n\n# Doc\n";
    let doc = parse_document_text(&root, root.join("doc.md"), text, false);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("expected parsed frontmatter, got {:?}", doc.frontmatter);
    };
    assert_eq!(fm.profile.as_deref(), Some("guide"));
    assert_eq!(fm.status.as_deref(), Some("stable"));
    assert_eq!(fm.share.as_deref(), Some("public"));
    assert_eq!(fm.depends, vec!["../a.md"]);
}

#[test]
fn test_frontmatter_title_prohibited() {
    let root = PathBuf::from("/ws");
    let text = "---\ntitle: Invalid Title Key\nprofile: guide\n---\n\n# Document Header\n";
    let doc = parse_document_text(&root, root.join("doc.md"), text, true);
    match doc.frontmatter {
        FrontmatterState::Invalid(err) => {
            assert!(err.contains("MUST NOT contain a title field"));
        }
        other => panic!("expected invalid frontmatter due to title key, got {other:?}"),
    }
}
