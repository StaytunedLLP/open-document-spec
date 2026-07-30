//! CLI smoke for export + service unit rendering (no live systemd required).
use odc_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn export_writes_graph_md() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\nid: b\ndepends:\n  - a\n---\n\n# B\n",
    )
    .unwrap();
    let out = dir.join("my-graph.md");
    let status = Command::new(ods_bin())
        .args(["export", root, "--out", out.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    let body = fs::read_to_string(&out).unwrap();
    assert!(body.contains("ODS workspace graph"));
    assert!(body.contains("depends"));
    assert!(body.contains("`b`") || body.contains("| `b`"));
}

#[test]
fn help_lists_export_start_stop_serve() {
    let out = Command::new(ods_bin()).arg("help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for cmd in ["export", "start", "stop", "serve", "watch", "init"] {
        assert!(stdout.contains(cmd), "missing {cmd} in help: {stdout}");
    }
    assert!(
        !stdout.contains("ods-lsp"),
        "help must not mention removed ods-lsp: {stdout}"
    );
    assert!(
        !stdout.to_lowercase().contains("zed extension"),
        "help must not mention Zed extension: {stdout}"
    );
}
