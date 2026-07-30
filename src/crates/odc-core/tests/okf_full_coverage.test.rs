//! Drive OKF modules toward full line coverage: audit, model, init, parse, load.
use odc_core::{
    ActorEvent, OkfAuditClass, OkfBundle, OkfDocument, OkfFrontmatter, OkfFrontmatterState,
    OkfInitOptions, OkfStatus, OkfTrustTier, audit_okf_bundle, concept_id_for_path,
    current_okf_version, derive_trust_tier, init_okf_bundle, load_okf_bundle, okf_enabled,
    okf_version_from_root, parse_okf_frontmatter_block, render_okf_audit_markdown,
};
use std::path::PathBuf;
use tempfile::tempdir;

fn doc(path: &str, fm: OkfFrontmatterState, reserved: bool) -> OkfDocument {
    OkfDocument {
        path: PathBuf::from(path),
        concept_id: path.trim_end_matches(".md").into(),
        body: String::new(),
        frontmatter: fm,
        is_reserved: reserved,
    }
}

#[test]
fn audit_all_classes_and_markdown_render() {
    let ok_fm = OkfFrontmatter {
        type_name: Some("Metric".into()),
        ..Default::default()
    };
    let no_type = OkfFrontmatter::default();
    let attested = OkfFrontmatter {
        type_name: Some("Attested Computation".into()),
        ..Default::default()
    };

    let bundle = OkfBundle {
        root: PathBuf::from("/bundle"),
        okf_version: Some("0.2".into()),
        documents: vec![
            doc("index.md", OkfFrontmatterState::Absent, true),
            doc("plain.md", OkfFrontmatterState::Absent, false),
            doc(
                "bad.md",
                OkfFrontmatterState::Invalid("parse error".into()),
                false,
            ),
            doc("partial.md", OkfFrontmatterState::Parsed(no_type), false),
            doc("attested.md", OkfFrontmatterState::Parsed(attested), false),
            doc("ok.md", OkfFrontmatterState::Parsed(ok_fm), false),
        ],
    };
    let report = audit_okf_bundle(&bundle);
    assert_eq!(report.total_md, 6);
    assert_eq!(report.skipped, 1);
    assert_eq!(report.plain, 1);
    assert_eq!(report.invalid, 1);
    assert_eq!(report.partial, 2);
    assert_eq!(report.compliant, 1);

    let md = render_okf_audit_markdown(PathBuf::from("/bundle").as_path(), &report);
    assert!(md.contains("compliant"));
    assert!(md.contains("Plain Markdown"));
    assert!(md.contains("Invalid Frontmatter"));
    assert!(md.contains("Partial"));
    assert!(md.contains("odc okf adopt"));
    assert!(md.contains("ok.md") || md.contains("plain.md"));
}

#[test]
fn audit_empty_shows_none_sections() {
    let report = audit_okf_bundle(&OkfBundle {
        root: PathBuf::from("/b"),
        okf_version: Some("0.2".into()),
        documents: vec![],
    });
    let md = render_okf_audit_markdown(PathBuf::from("/b").as_path(), &report);
    assert!(md.contains("_None._"));
    assert_eq!(report.items.len(), 0);
    let _ = OkfAuditClass::Compliant;
}

#[test]
fn model_helpers_status_trust_ids() {
    assert_eq!(current_okf_version(), "0.2");
    assert_eq!(OkfStatus::Draft.as_str(), "draft");
    assert_eq!(OkfStatus::Stable.as_str(), "stable");
    assert_eq!(OkfStatus::Deprecated.as_str(), "deprecated");
    assert_eq!(OkfStatus::parse(" draft "), Some(OkfStatus::Draft));
    assert_eq!(OkfStatus::parse("stable"), Some(OkfStatus::Stable));
    assert_eq!(OkfStatus::parse("deprecated"), Some(OkfStatus::Deprecated));
    assert_eq!(OkfStatus::parse("other"), None);

    assert_eq!(derive_trust_tier(&[]), OkfTrustTier::Unverified);
    assert_eq!(OkfTrustTier::Unverified.as_str(), "unverified");
    assert_eq!(
        derive_trust_tier(&[ActorEvent {
            by: "bot".into(),
            at: None
        }]),
        OkfTrustTier::MachineConfirmed
    );
    assert_eq!(OkfTrustTier::MachineConfirmed.as_str(), "machine-confirmed");
    assert_eq!(
        derive_trust_tier(&[ActorEvent {
            by: "human:x".into(),
            at: Some("t".into())
        }]),
        OkfTrustTier::HumanReviewed
    );
    assert_eq!(OkfTrustTier::HumanReviewed.as_str(), "human-reviewed");

    let root = PathBuf::from("/ws");
    assert_eq!(concept_id_for_path(&root, &root.join("a/b.md")), "a/b");
    assert_eq!(
        concept_id_for_path(&root, PathBuf::from("x.md").as_path()),
        "x"
    );
}

#[test]
fn init_create_and_skip_all_options() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let _ = OkfInitOptions::default();
    let r1 = init_okf_bundle(
        root,
        OkfInitOptions {
            write_sample_concept: true,
            write_attested_stub: true,
            write_log: true,
        },
    )
    .unwrap();
    assert!(!r1.created.is_empty());
    assert!(root.join("index.md").is_file());
    assert!(root.join("metrics/sample-metric.md").is_file());
    assert!(root.join("computations/sample-computation.md").is_file());
    assert!(root.join("log.md").is_file());

    // Default options path (sample only)
    let dir2 = tempdir().unwrap();
    let r_def = init_okf_bundle(dir2.path(), OkfInitOptions::default()).unwrap();
    assert!(!r_def.created.is_empty());

    let r2 = init_okf_bundle(
        root,
        OkfInitOptions {
            write_sample_concept: true,
            write_attested_stub: true,
            write_log: true,
        },
    )
    .unwrap();
    assert!(
        r2.skipped.len() >= 4,
        "expected all artifacts skipped: {:?}",
        r2.skipped
    );
    assert!(okf_enabled(root));
    assert_eq!(okf_version_from_root(root).as_deref(), Some("0.2"));
    let bundle = load_okf_bundle(root).unwrap();
    assert!(bundle.okf_version.as_deref() == Some("0.2"));
    let _ = audit_okf_bundle(&bundle);

    // Test with write_sample_concept = false
    let dir3 = tempdir().unwrap();
    let r_no_sample = init_okf_bundle(
        dir3.path(),
        OkfInitOptions {
            write_sample_concept: false,
            write_attested_stub: false,
            write_log: false,
        },
    )
    .unwrap();
    assert_eq!(r_no_sample.created.len(), 1); // index.md only
}

#[test]
fn parse_okf_frontmatter_exhaustive() {
    let metric = r#"
type: Metric
title: T
description: D
tags: [a, b]
status: stable
stale_after: 2099-12-31
timestamp: 2020-01-01T00:00:00Z
custom_ext: hello
generated: { by: agent/v1, at: 2026-06-20T22:53:05Z }
verified: { by: human:alice, at: 2026-06-25T09:00:00Z }
sources:
  - id: rev-policy
    resource: https://example.com/policy
    title: Policy
    author: team:finance
    last_modified: 2026-04-02
"#;
    let fm = parse_okf_frontmatter_block(metric).unwrap();
    assert_eq!(fm.type_name.as_deref(), Some("Metric"));
    assert_eq!(fm.status, Some(OkfStatus::Stable));
    assert!(fm.tags.len() >= 2);
    assert_eq!(fm.sources.len(), 1);
    assert_eq!(fm.verified.len(), 1);
    assert_eq!(
        fm.unknown.get("custom_ext").map(String::as_str),
        Some("hello")
    );

    let attested = r#"
type: Attested Computation
runtime: bigquery
parameters:
  - { name: year, type: integer, required: true }
executor:
  resource: references/skills/run-on-bq.md
  receipt: [job_id, executed_sql, result]
attester:
  resource: references/attesters/sql-equality.py
"#;
    let fm = parse_okf_frontmatter_block(attested).unwrap();
    assert_eq!(fm.parameters.len(), 1);
    assert!(fm.executor.resource.is_some());
    assert!(fm.attester.resource.is_some());
    assert!(!fm.executor.receipt.is_empty());
}

#[test]
fn parse_draft_and_deprecated_status() {
    let d = parse_okf_frontmatter_block("type: X\nstatus: draft\n").unwrap();
    assert_eq!(d.status, Some(OkfStatus::Draft));
    let d = parse_okf_frontmatter_block("type: X\nstatus: deprecated\n").unwrap();
    assert_eq!(d.status, Some(OkfStatus::Deprecated));
}

#[test]
fn okf_bundle_scan_and_version_edge_cases() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // empty okf_version
    std::fs::write(root.join("index.md"), "---\nokf_version: \"\"\n---\n").unwrap();
    assert_eq!(okf_version_from_root(root), None);

    // missing okf_version line
    std::fs::write(root.join("index.md"), "---\ntitle: No Version\n---\n").unwrap();
    assert_eq!(okf_version_from_root(root), None);

    // subdirs node_modules, target, .hidden, and .txt files
    std::fs::create_dir_all(root.join("node_modules")).unwrap();
    std::fs::write(root.join("node_modules/a.md"), "# NM\n").unwrap();
    std::fs::create_dir_all(root.join("target")).unwrap();
    std::fs::write(root.join("target/b.md"), "# T\n").unwrap();
    std::fs::create_dir_all(root.join(".hidden")).unwrap();
    std::fs::write(root.join(".hidden/c.md"), "# H\n").unwrap();
    std::fs::write(root.join("file.txt"), "text").unwrap();

    let bundle = load_okf_bundle(root).unwrap();
    assert!(
        !bundle
            .documents
            .iter()
            .any(|d| d.path.to_string_lossy().contains("node_modules"))
    );
    assert!(
        !bundle
            .documents
            .iter()
            .any(|d| d.path.to_string_lossy().contains("target"))
    );
    assert!(
        !bundle
            .documents
            .iter()
            .any(|d| d.path.to_string_lossy().contains(".hidden"))
    );
    assert!(
        !bundle
            .documents
            .iter()
            .any(|d| d.path.ends_with("file.txt"))
    );
}

#[test]
fn scaffold_and_remove_profile_templates_and_errors() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    std::fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
    )
    .unwrap();

    // Scaffold with explicit profiles
    for (prof, file) in [
        ("decision", "decisions/adr.md"),
        ("sop", "sop/ops.md"),
        ("api", "api/endpoint.md"),
        ("meeting", "meetings/sync.md"),
        ("faq", "faq/q.md"),
    ] {
        let rep = odc_core::scaffold_new_document(
            root,
            std::path::Path::new(file),
            odc_core::NewDocumentOptions {
                profile: Some(prof.to_string()),
                title: None,
            },
        )
        .unwrap();
        assert_eq!(rep.profile, prof);
    }

    // AlreadyExists error
    let err_exist = odc_core::scaffold_new_document(
        root,
        std::path::Path::new("decisions/adr.md"),
        odc_core::NewDocumentOptions::default(),
    );
    assert!(err_exist.is_err());

    // NotFound error in delete
    let err_nf = odc_core::atomic_delete_document(
        root,
        std::path::Path::new("nonexistent.md"),
        odc_core::RemoveDocumentOptions::default(),
    );
    assert!(err_nf.is_err());
}
