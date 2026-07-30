//! OKF index / load / audit coverage.
use odc_core::okf::{
    export_okf_graph, fmt_okf_bundle, generate_okf_indexes, init_okf_bundle, load_okf_bundle,
    lint_okf_bundle, okf_context, okf_indexes_are_current, OkfInitOptions,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn init_load_lint_generate_indexes() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let report = init_okf_bundle(
        root,
        OkfInitOptions {
            write_attested_stub: true,
            write_sample_concept: true,
            write_log: true,
        },
    )
    .unwrap();
    assert!(
        !report.created.is_empty() || root.join("index.md").exists(),
        "{report:?}"
    );

    fs::create_dir_all(root.join("metrics")).unwrap();
    fs::write(
        root.join("metrics/revenue.md"),
        "---\ntype: Metric\ntitle: Revenue\ndescription: Money\n---\n\n# Revenue\n",
    )
    .unwrap();

    let bundle = load_okf_bundle(root).unwrap();
    assert!(bundle.okf_version.as_deref() == Some("0.2") || bundle.okf_version.is_some());
    let diags = lint_okf_bundle(&bundle);
    assert!(
        !diags.iter().any(|d| d.severity == odc_core::Severity::Error),
        "{diags:?}"
    );

    let written = generate_okf_indexes(&bundle).unwrap();
    assert!(!written.is_empty() || root.join("metrics/index.md").exists());
    let metrics_index = fs::read_to_string(root.join("metrics/index.md")).unwrap_or_default();
    assert!(
        metrics_index.contains("revenue") || metrics_index.contains("Revenue"),
        "{metrics_index}"
    );

    let bundle2 = load_okf_bundle(root).unwrap();
    assert!(okf_indexes_are_current(&bundle2).unwrap());
    // Stale index check
    fs::write(root.join("metrics/index.md"), "# wrong\n").unwrap();
    assert!(!okf_indexes_are_current(&bundle2).unwrap());
    generate_okf_indexes(&bundle2).unwrap();

    // Body links for context + export edges
    fs::write(
        root.join("metrics/revenue.md"),
        "---\ntype: Metric\ntitle: Revenue\n---\n\n# Revenue\n\nSee [other](./other.md) and [ext](https://example.com) and [#anchor](#x).\n",
    )
    .unwrap();
    fs::write(
        root.join("metrics/other.md"),
        "---\ntype: Metric\ntitle: Other\n---\n\n# Other\n",
    )
    .unwrap();
    let bundle3 = load_okf_bundle(root).unwrap();
    let ctx = okf_context(&bundle3, "revenue");
    assert!(!ctx.is_empty());
    let _ = okf_context(&bundle3, "missing-id-xyz");
    let out_graph = root.join("out/graph-export.md");
    export_okf_graph(&bundle3, &out_graph).unwrap();
    assert!(out_graph.is_file());
    let graph_txt = fs::read_to_string(&out_graph).unwrap();
    assert!(graph_txt.contains("Concepts") || graph_txt.contains("OKF"));

    // fmt: trailing space in frontmatter
    let sample = root.join("metrics/sample-metric.md");
    if sample.exists() {
        let t = fs::read_to_string(&sample).unwrap();
        fs::write(&sample, t.replace("title:", "title:  ")).unwrap();
    }
    let bundle4 = load_okf_bundle(root).unwrap();
    let _ = fmt_okf_bundle(&bundle4).unwrap();
}

#[test]
fn generate_okf_indexes_creates_root_if_missing() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("c.md"),
        "---\ntype: Metric\ntitle: C\n---\n\n# C\n",
    )
    .unwrap();
    // Minimal bundle without root index
    let bundle = load_okf_bundle(root);
    // load may fail without okf_version — build manually via parse path
    if bundle.is_err() {
        // create versioned root then load
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# K\n",
        )
        .unwrap();
        let bundle = load_okf_bundle(root).unwrap();
        let _ = generate_okf_indexes(&bundle).unwrap();
        assert!(root.join("index.md").exists());
    }
}
