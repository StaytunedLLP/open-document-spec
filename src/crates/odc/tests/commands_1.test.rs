use odc_test_support::temp_workspace;
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

#[test]
fn help_lists_new_commands() {
    let out = Command::new(ods_bin()).arg("help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for cmd in [
        "setup",
        "doctor",
        "sync",
        "watch",
        "tags",
        "find",
        "tag rename",
        "init",
        "disable",
        "update",
        "workspaces",
    ] {
        assert!(stdout.contains(cmd), "help missing {cmd}: {stdout}");
    }
}

#[test]
fn setup_help_lists_setup_behavior() {
    let out = Command::new(ods_bin())
        .args(["setup", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("odc setup") || stdout.contains("ods setup"),
        "{stdout}"
    );
    assert!(stdout.contains("doctor"), "{stdout}");
}

#[test]
fn workspaces_help_lists_subcommands() {
    let out = Command::new(ods_bin())
        .args(["workspaces", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("odc workspaces") || stdout.contains("ods workspaces"),
        "{stdout}"
    );
    assert!(stdout.contains("add"), "{stdout}");
    assert!(stdout.contains("remove"), "{stdout}");
    assert!(stdout.contains("list"), "{stdout}");
}

#[test]
fn setup_outside_workspace_prompts_to_run_init() {
    let dir = tempfile::Builder::new()
        .prefix("ods-setup-outside-")
        .tempdir()
        .unwrap();
    let out = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("no ODS workspace found"), "{stdout}");
    assert!(
        stdout.contains("run 'ods init") || stdout.contains("run 'odc init") || stdout.contains("odc init"),
        "{stdout}"
    );
    assert!(!dir.path().join("index.md").exists());
}

#[test]
fn setup_inside_workspace_runs_doctor_without_test_service_start() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    let out = Command::new(ods_bin())
        .args(["init", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("workspace"), "{stdout}");
    assert!(stdout.contains("service"), "{stdout}");
    assert!(stdout.contains("doctor"), "{stdout}");
    assert!(
        stdout.contains("odc version") || stdout.contains("ods cli version"),
        "{stdout}"
    );
    assert!(stdout.contains("root ods spec"), "{stdout}");
    assert!(stdout.contains("root odc"), "{stdout}");
}

#[test]
fn setup_updates_stale_root_ods_version() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: draft-1\n---\n\n# Root\n\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let root = fs::read_to_string(dir.join("index.md")).unwrap();
    assert!(root.contains("ods: 0.1"), "{root}");
    assert!(
        root.contains(&format!("odc: \">={}\"", env!("CARGO_PKG_VERSION"))),
        "{root}"
    );
    assert!(!root.contains("ods: draft-1"), "{root}");
}

#[test]
fn doctor_reports_stale_root_ods_version() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: draft-1\n---\n\n# Root\n\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .args(["doctor", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root ods spec: draft-1"), "{stdout}");
    assert!(stdout.contains("0.1"), "{stdout}");
}
