use std::process::Command;
use tempfile::tempdir;

fn odc_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_odc"))
}

#[test]
fn bare_lint_requires_namespace() {
    let out = Command::new(odc_bin()).args(["lint"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2), "{:?}", out);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("odc ods") || err.contains("namespace"),
        "{err}"
    );
}

#[test]
fn okf_init_lint_audit() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let st = Command::new(odc_bin())
        .args(["okf", "init", path, "--attested"])
        .status()
        .unwrap();
    assert!(st.success());
    let out = Command::new(odc_bin())
        .args(["okf", "lint", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let out = Command::new(odc_bin())
        .args(["okf", "audit", path, "--write-report"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.path().join(".odc/odc-errors.md").exists());
}

#[test]
fn ods_namespace_init_lint() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let st = Command::new(odc_bin())
        .args(["ods", "init", path])
        .status()
        .unwrap();
    assert!(st.success());
    let out = Command::new(odc_bin())
        .args(["ods", "lint", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn okf_index_export_fmt_context() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["okf", "init", path])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(odc_bin())
            .args(["okf", "index", path])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(odc_bin())
            .args(["okf", "export", path, "--out", &format!("{path}/g.md")])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(odc_bin())
            .args(["okf", "fmt", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(odc_bin())
        .args(["okf", "context", path, "sample-metric"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn agents_sync_writes_agents_md() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["ods", "init", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(odc_bin())
        .args(["agents", "sync", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.path().join("AGENTS.md").exists());
}
