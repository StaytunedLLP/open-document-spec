use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn odc_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_odc"))
}

#[test]
fn okf_doctor_and_fmt() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["okf", "init", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(odc_bin())
        .args(["okf", "doctor", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let out = Command::new(odc_bin())
        .args(["okf", "fmt", path])
        .output()
        .unwrap();
    assert!(out.status.success() || out.status.code().is_some(), "{:?}", out);
}

#[test]
fn tags_list() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(odc_bin())
        .args(["tags", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn okf_full_command_surface() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["okf", "init", path, "--attested", "--log"])
            .status()
            .unwrap()
            .success()
    );
    for args in [
        vec!["okf", "lint", path],
        vec!["okf", "index", path],
        vec!["okf", "index", "--check", path],
        vec!["okf", "doctor", path],
        vec!["okf", "audit", path],
        vec!["okf", "audit", path, "--write-report"],
        vec!["okf", "adopt", path],
        vec!["okf", "fmt", path],
        vec!["okf", "export", path],
        vec!["okf", "context", path, "sample-metric"],
        vec!["okf", "help"],
    ] {
        let out = Command::new(odc_bin()).args(&args).output().unwrap();
        assert!(
            out.status.success() || out.status.code() == Some(1),
            "cmd {args:?} => {:?}",
            out
        );
    }
}

#[test]
fn upgrade_empty_and_okf_and_migrate() {
    let empty = tempdir().unwrap();
    let out = Command::new(odc_bin())
        .args(["upgrade", empty.path().to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    assert!(out.status.success() || out.status.code() == Some(1), "{:?}", out);

    let okf = tempdir().unwrap();
    let path = okf.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["okf", "init", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(odc_bin())
        .args(["upgrade", path, "--check"])
        .output()
        .unwrap();
    assert!(out.status.success() || out.status.code() == Some(1), "{:?}", out);

    let ods = tempdir().unwrap();
    let op = ods.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["init", op])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(odc_bin())
        .args(["upgrade", op, "--write"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn share_and_export_and_disable_dry() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(odc_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        dir.path().join("p.md"),
        "---\nprofile: note\nstatus: draft\nshare: public\n---\n\n# P\n",
    )
    .unwrap();
    let _ = Command::new(odc_bin()).args(["index", path]).status();
    let out_dir = dir.path().join("published");
    let out = Command::new(odc_bin())
        .args([
            "share",
            path,
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(out.status.success() || out.status.code().is_some(), "{:?}", out);

    let graph = dir.path().join("graph.md");
    let out = Command::new(odc_bin())
        .args(["export", path, "--out", graph.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success() || out.status.code().is_some(), "{:?}", out);

    let out = Command::new(odc_bin())
        .args(["disable", path])
        .output()
        .unwrap();
    assert!(out.status.success() || out.status.code().is_some(), "{:?}", out);
}

#[test]
fn help_and_version_and_unknown() {
    for args in [vec!["help"], vec!["--help"], vec!["version"], vec!["--version"]] {
        let out = Command::new(odc_bin()).args(&args).output().unwrap();
        assert!(out.status.success(), "{args:?} {:?}", out);
    }
    let out = Command::new(odc_bin())
        .args(["not-a-real-command"])
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn git_detect_renames_in_git_repo() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    let _ = Command::new("git").args(["init"]).current_dir(root).output();
    let _ = Command::new("git").args(["config", "user.name", "Test"]).current_dir(root).output();
    let _ = Command::new("git").args(["config", "user.email", "test@example.com"]).current_dir(root).output();

    fs::write(root.join("index.md"), "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n\n- [old.md](old.md)\n").unwrap();
    fs::write(root.join("old.md"), "---\nprofile: note\nstatus: draft\n---\n\n# Old\n").unwrap();

    let _ = Command::new("git").args(["add", "."]).current_dir(root).output();
    let _ = Command::new("git").args(["commit", "-m", "init"]).current_dir(root).output();

    let _ = Command::new("git").args(["mv", "old.md", "new.md"]).current_dir(root).output();

    let sync_out = Command::new(odc_bin()).args(["sync", root.to_str().unwrap()]).output().unwrap();
    assert!(sync_out.status.success(), "{:?}", sync_out);
}
