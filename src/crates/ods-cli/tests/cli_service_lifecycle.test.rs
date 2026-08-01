use ods_test_support::temp_workspace;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn terminate_gracefully(child: &mut Child) {
    #[cfg(unix)]
    {
        unsafe {
            libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
        }
        std::thread::sleep(Duration::from_millis(300));
    }
    let _ = child.kill();
}

fn init(dir: &std::path::Path) {
    let status = Command::new(ods_bin())
        .args(["init", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .status()
        .unwrap();
    assert!(status.success());
}

#[test]
fn bench_stats_reports_token_estimate() {
    let dir = temp_workspace();
    init(&dir);
    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\nid: doc\n---\n\n# Doc\n\nSome body text.\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .args(["bench", "stats", "--format", "json", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"total_files\""), "{stdout}");
    assert!(
        stdout.contains("\"token_reduction_percentage\""),
        "{stdout}"
    );
}

#[test]
fn bench_run_prints_simulated_estimate_without_calling_any_api() {
    let dir = temp_workspace();
    init(&dir);
    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\nid: doc\n---\n\n# Doc\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .args([
            "bench",
            "run",
            "--prompt",
            "Refactor the checkout flow",
            dir.to_str().unwrap(),
        ])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Simulated estimate"), "{stdout}");
    assert!(stdout.contains("no live LLM API call"), "{stdout}");
}

#[test]
fn export_omits_private_docs_by_default_and_includes_with_flag() {
    let dir = temp_workspace();
    init(&dir);
    fs::write(
        dir.join("public.md"),
        "---\nprofile: note\nstatus: draft\nid: public\n---\n\n# Public\n",
    )
    .unwrap();
    fs::write(
        dir.join("secret.md"),
        "---\nprofile: note\nstatus: draft\nid: secret\nshare: private\n---\n\n# Secret\n",
    )
    .unwrap();

    let out_path = dir.join("graph.md");
    let out = Command::new(ods_bin())
        .args([
            "export",
            dir.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
        ])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("were omitted"), "{stdout}");
    let body = fs::read_to_string(&out_path).unwrap();
    assert!(body.contains("`public`"), "{body}");
    assert!(!body.contains("`secret`"), "{body}");

    let out = Command::new(ods_bin())
        .args([
            "export",
            dir.to_str().unwrap(),
            "--out",
            out_path.to_str().unwrap(),
            "--include-private",
        ])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let body = fs::read_to_string(&out_path).unwrap();
    assert!(body.contains("`secret`"), "{body}");
}

#[test]
fn logs_shows_service_log_path_not_watch_alias() {
    let dir = temp_workspace();
    init(&dir);

    // `logs` is not a watch alias: it reports missing service logs or prints log contents.
    let out = Command::new(ods_bin())
        .args(["logs"])
        .env("ODS_AUTO_UPDATE", "0")
        .env("HOME", dir.to_str().expect("utf8 temp path"))
        .env("USERPROFILE", dir.to_str().expect("utf8 temp path"))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("no service logs found") || stdout.contains("ods-serve.log"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("streaming ods serve logs"),
        "must not claim fake log streaming: {stdout}"
    );
}

#[test]
fn watch_does_not_print_fake_log_stream_banner() {
    let dir = temp_workspace();
    init(&dir);

    let mut child = Command::new(ods_bin())
        .args(["watch", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(800));
    terminate_gracefully(&mut child);
    let _ = child.wait();
    let mut stdout = String::new();
    child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
    assert!(
        !stdout.contains("streaming ods serve logs")
            && !stdout.contains("streaming ods serve logs"),
        "watch must not print logs banner: {stdout}"
    );
}

#[test]
fn start_status_is_read_only_and_reports_state() {
    let dir = temp_workspace();
    init(&dir);

    let out = Command::new(ods_bin())
        .args(["start", "--status", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("installed="), "{stdout}");
    assert!(stdout.contains("running="), "{stdout}");
}

#[test]
fn stop_on_an_unregistered_workspace_does_not_error_out() {
    let dir = temp_workspace();
    init(&dir);

    let out = Command::new(ods_bin())
        .args(["stop", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
}
