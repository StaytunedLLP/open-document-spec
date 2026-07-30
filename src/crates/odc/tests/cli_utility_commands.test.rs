use odc_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn lint_clean_prints_ok_message() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    let out = Command::new(ods_bin())
        .args(["init", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    fs::write(
        dir.join("note.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Note\n",
    )
    .unwrap();
    let out = Command::new(ods_bin())
        .args(["index", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let out = Command::new(ods_bin())
        .args(["lint", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Everything is fine"),
        "expected green message: {stdout}"
    );
    assert!(!dir.join(".odc/odc-errors.md").exists());
    assert!(!dir.join("ods-error.md").exists());
}

#[test]
fn lint_broken_writes_odc_error_report() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        dir.join("broken.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/doc\n---\n\n# Broken\n",
    )
    .unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["index", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let out = Command::new(ods_bin())
        .args(["lint", root])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "lint should fail on dangling depends"
    );
    let report = dir.join(".odc/odc-errors.md");
    assert!(report.is_file(), "expected .odc/odc-errors.md");
    let body = fs::read_to_string(&report).unwrap();
    assert!(body.contains("missing/doc") || body.contains("dangling") || body.contains("error"));
}

#[test]
fn init_and_disable_cli() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    fs::write(dir.join("plain.md"), "# Plain\n\nBody stays.\n").unwrap();

    let out = Command::new(ods_bin())
        .args(["init", root, "--adopt"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(
        fs::read_to_string(dir.join("index.md"))
            .unwrap()
            .contains("ods:"),
        "root should be initialized"
    );
    let plain = fs::read_to_string(dir.join("plain.md")).unwrap();
    assert!(plain.contains("profile:"));
    assert!(plain.contains("Body stays."));

    let out = Command::new(ods_bin())
        .args(["disable", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("dry-run")
            || String::from_utf8_lossy(&out.stdout).contains("would_edit"),
        "{:?}",
        out
    );

    let out = Command::new(ods_bin())
        .args(["disable", root, "--write"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let plain = fs::read_to_string(dir.join("plain.md")).unwrap();
    assert!(!plain.contains("profile:"));
    assert!(plain.contains("Body stays."));
    assert!(
        !fs::read_to_string(dir.join("index.md"))
            .unwrap()
            .lines()
            .any(|l| l.trim().starts_with("ods:"))
    );
}

#[test]
fn tags_find_and_rename() {
    let dir = temp_workspace();
    Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .status()
        .unwrap();
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - Billing\n  - old-cx\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - billing\n---\n\n# B\n",
    )
    .unwrap();
    Command::new(ods_bin())
        .args(["index", dir.to_str().unwrap()])
        .status()
        .unwrap();

    let out = Command::new(ods_bin())
        .args(["tags", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("billing"), "{stdout}");
    assert!(stdout.contains("old-cx"), "{stdout}");

    let out = Command::new(ods_bin())
        .args(["find", dir.to_str().unwrap(), "--tag", "billing"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a") || stdout.contains("b"), "{stdout}");

    let out = Command::new(ods_bin())
        .args([
            "tag",
            "rename",
            dir.to_str().unwrap(),
            "old-cx",
            "customer-care",
            "--write",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let a = fs::read_to_string(dir.join("a.md")).unwrap();
    assert!(a.contains("customer-care"), "{a}");
    assert!(!a.contains("old-cx"), "{a}");
}


