use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn help_exits_zero() {
    let out = Command::new(ods_bin()).arg("help").output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("lint") || stdout.contains("init"));
}

#[test]
fn init_lint_index_graph_context() {
    let dir = temp_workspace();
    let status = Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\ndescription: D\n---\n\n# Doc\n",
    )
    .unwrap();

    let status = Command::new(ods_bin())
        .args(["index", dir.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    let out = Command::new(ods_bin())
        .args(["lint", "--format", "json", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .args(["index", "--check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .args(["graph", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());

    let out = Command::new(ods_bin())
        .args(["profiles", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("note") || out.status.success());

    let out = Command::new(ods_bin())
        .args(["context", dir.to_str().unwrap(), "doc"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    // Single-arg id from inside the workspace must work (not treat id as root).
    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["context", "doc"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "context single-arg id failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("doc.md"),
        "expected doc.md in context output: {stdout}"
    );

    // Missing id must fail (not silent empty success — agents dump the tree otherwise).
    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["context", "does-not-exist-xyz"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "missing context id should be non-zero"
    );
}

#[test]
fn context_path_shaped_id_from_cwd() {
    let dir = temp_workspace();
    assert!(Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .status()
        .unwrap()
        .success());
    fs::create_dir_all(dir.join("specs/ods")).unwrap();
    fs::write(
        dir.join("specs/ods/core.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - specs/ods/intro.md\n---\n\n# Core\n",
    )
    .unwrap();
    fs::write(
        dir.join("specs/ods/intro.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Intro\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["context", "specs/ods/core"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "path-shaped context id failed: status={:?} stderr={} stdout={}",
        out.status,
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("core.md"), "{stdout}");
    assert!(stdout.contains("intro.md"), "{stdout}");
}

#[test]
fn mv_file() {
    let dir = temp_workspace();
    Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .status()
        .unwrap();
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# B\n",
    )
    .unwrap();
    Command::new(ods_bin())
        .args(["index", dir.to_str().unwrap()])
        .status()
        .unwrap();

    let status = Command::new(ods_bin())
        .args(["mv", dir.to_str().unwrap(), "a.md", "c.md"])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(dir.join("c.md").exists());
    let b = fs::read_to_string(dir.join("b.md")).unwrap();
    assert!(b.contains("c"), "{b}");
}

#[test]
fn lint_discovers_workspace_root_from_nested_dir() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("nested")).unwrap();
    Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .status()
        .unwrap();
    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n",
    )
    .unwrap();
    Command::new(ods_bin())
        .args(["index", dir.to_str().unwrap()])
        .status()
        .unwrap();

    let out = Command::new(ods_bin())
        .current_dir(dir.join("nested"))
        .arg("lint")
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn doctor_reports_workspace_health() {
    let dir = temp_workspace();
    Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .status()
        .unwrap();

    let out = Command::new(ods_bin())
        .args(["doctor", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("workspace:"), "{stdout}");

    let out = Command::new(ods_bin())
        .args(["doctor", "--format", "json", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"workspace\""), "{stdout}");
    assert!(stdout.contains("\"ok\""), "{stdout}");
}

#[test]
fn start_status_accepts_status_flag() {
    let dir = temp_workspace();
    let out = Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .args(["start", "--status", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("installed=") || stderr.contains("installed="),
        "expected service status output, stdout={stdout}, stderr={stderr}"
    );
}
