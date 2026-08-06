use std::process::Command;
use tempfile::tempdir;

fn ods_bin() -> std::path::PathBuf {
    eprintln!("ods_bin: {}", env!("CARGO_BIN_EXE_ods"));
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn bare_lint_auto_detects_or_explains() {
    let dir = tempdir().unwrap();
    let out = Command::new(ods_bin())
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
    let st = Command::new(ods_bin())
        .args(["init", "--okf", path, "--attested"])
        .status()
        .unwrap();
    assert!(st.success());
    let out = Command::new(ods_bin())
        .args(["lint", "--okf", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let out = Command::new(ods_bin())
        .args(["audit", "--okf", path, "--write-report"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.path().join(".ods/ods-errors.md").exists());
}

#[test]
fn ods_namespace_init_lint() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    let st = Command::new(ods_bin())
        .args(["init", path])
        .status()
        .unwrap();
    assert!(st.success());
    let out = Command::new(ods_bin())
        .args(["lint", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn okf_index_export_fmt_context() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", "--okf", path])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(ods_bin())
            .args(["index", "--okf", path])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(ods_bin())
            .args(["export", "--okf", path, "--out", &format!("{path}/g.md")])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new(ods_bin())
            .args(["fmt", "--okf", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(ods_bin())
        .args(["context", "--okf", path, "sample-metric"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn agents_sync_writes_agents_md() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(ods_bin())
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
        Command::new(ods_bin())
            .args(["init", "--okf", path, "--log"])
            .status()
            .unwrap()
            .success()
    );

    // doctor text and json
    let doc_text = Command::new(ods_bin())
        .args(["doctor", "--okf", path])
        .output()
        .unwrap();
    assert!(doc_text.status.success());
    let doc_json = Command::new(ods_bin())
        .args(["doctor", "--okf", path, "--format", "json"])
        .output()
        .unwrap();
    assert!(doc_json.status.success());

    // audit json
    let aud_json = Command::new(ods_bin())
        .args(["audit", "--okf", path, "--format", "json"])
        .output()
        .unwrap();
    assert!(aud_json.status.success());

    // adopt dry-run and write
    std::fs::write(dir.path().join("plain.md"), "# Plain\n").unwrap();
    let adopt_dry = Command::new(ods_bin())
        .args(["adopt", "--okf", path])
        .output()
        .unwrap();
    assert!(adopt_dry.status.success());
    let adopt_write = Command::new(ods_bin())
        .args(["adopt", "--okf", "--write", path])
        .output()
        .unwrap();
    assert!(adopt_write.status.success());

    // index and index check
    let idx_gen = Command::new(ods_bin())
        .args(["index", "--okf", path])
        .output()
        .unwrap();
    assert!(idx_gen.status.success());
    let idx_check = Command::new(ods_bin())
        .args(["index", "--okf", "--check", path])
        .output()
        .unwrap();
    assert!(idx_check.status.success());
}

#[test]
fn hybrid_bare_lint_runs_both_engines() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    // Add OKF root marker alongside ODS
    let index = dir.path().join("index.ods.md");
    std::fs::write(
        &index,
        "---\nods: 0.1\nokf_version: \"0.2\"\n---\n\n# Root\n",
    )
    .unwrap();
    let out = Command::new(ods_bin())
        .args(["lint", path])
        .output()
        .unwrap();
    // May warn but should not fail with hybrid-ambiguous error
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("Bare `ods lint` is ambiguous"),
        "{combined}"
    );
}

#[test]
fn hybrid_bare_index_requires_explicit_namespace() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let index = dir.path().join("index.ods.md");
    std::fs::write(
        &index,
        "---\nods: 0.1\nokf_version: \"0.2\"\n---\n\n# Root\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .args(["index", path])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{:?}", out);
}

#[test]
fn coverage_write_report_goes_under_dot_ods() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(ods_bin())
        .args(["coverage", path, "--write-report"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.path().join(".ods/coverage.md").exists());
    assert!(!dir.path().join("ods-report.md").exists());
}

#[test]
fn archive_updates_nested_ods_status() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let doc = dir.path().join("note.md");
    std::fs::write(
        &doc,
        "---\nods:\n  profile: note\n  status: draft\n---\n\n# Note\n",
    )
    .unwrap();
    let out = Command::new(ods_bin())
        .current_dir(dir.path())
        .args(["archive", "note.md"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let body = std::fs::read_to_string(&doc).unwrap();
    assert!(
        body.contains("status: archived"),
        "expected archived status: {body}"
    );
    // nested form preserved under ods:
    assert!(
        body.contains("ods:") && body.lines().any(|l| l.trim() == "status: archived"),
        "{body}"
    );
}
