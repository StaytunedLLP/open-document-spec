use std::process::Command;
use tempfile::tempdir;

fn odc_bin() -> std::path::PathBuf {
    eprintln!(
        "odc_bin: {}, ods_bin: {}",
        env!("CARGO_BIN_EXE_odc"),
        env!("CARGO_BIN_EXE_ods")
    );
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_odc"))
}

#[test]
fn bare_lint_auto_detects_or_explains() {
    let dir = tempdir().unwrap();
    let out = Command::new(odc_bin())
        .args(["lint", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{:?}", out);
    let err = String::from_utf8_lossy(&out.stderr);
    let combined = format!("{}{}", err, String::from_utf8_lossy(&out.stdout));
    assert!(
        combined.contains("ODS")
            || combined.contains("OKF")
            || combined.contains("init")
            || combined.contains("workspace"),
        "{combined}"
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
}

#[test]
fn okf_cli_subcommands_exhaustive() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["okf", "init", path, "--log"])
            .status()
            .unwrap()
            .success()
    );

    // doctor text and json
    let doc_text = Command::new(odc_bin())
        .args(["okf", "doctor", path])
        .output()
        .unwrap();
    assert!(doc_text.status.success());
    let doc_json = Command::new(odc_bin())
        .args(["okf", "doctor", path, "--format", "json"])
        .output()
        .unwrap();
    assert!(doc_json.status.success());

    // audit json
    let aud_json = Command::new(odc_bin())
        .args(["okf", "audit", path, "--format", "json"])
        .output()
        .unwrap();
    assert!(aud_json.status.success());

    // adopt dry-run and write
    std::fs::write(dir.path().join("plain.md"), "# Plain\n").unwrap();
    let adopt_dry = Command::new(odc_bin())
        .args(["okf", "adopt", path])
        .output()
        .unwrap();
    assert!(adopt_dry.status.success());
    let adopt_write = Command::new(odc_bin())
        .args(["okf", "adopt", "--write", path])
        .output()
        .unwrap();
    assert!(adopt_write.status.success());

    // index and index check
    let idx_gen = Command::new(odc_bin())
        .args(["okf", "index", path])
        .output()
        .unwrap();
    assert!(idx_gen.status.success());
    let idx_check = Command::new(odc_bin())
        .args(["okf", "index", "--check", path])
        .output()
        .unwrap();
    assert!(idx_check.status.success());
}
