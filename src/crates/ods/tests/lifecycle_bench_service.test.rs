//! CLI coverage for previously-untested command wiring: `new`/`rm`/`archive`,
//! `bench strip/restore/stats/run`, `logs`, `export --include-private`,
//! and read-only service commands (`start --status`, `stop`).

use ods_test_support::temp_workspace;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

/// Ask the child to shut down gracefully (`SIGTERM` on Unix, where `ods` now
/// installs a handler for it) rather than `SIGKILL`ing it — a killed process
/// never runs its normal-exit path, so e.g. coverage instrumentation data for
/// everything it executed is silently lost. Falls back to a hard kill on
/// non-Unix or if the process hasn't exited shortly after the signal.
fn terminate_gracefully(child: &mut Child) {
    #[cfg(unix)]
    {
        // SAFETY: `kill(2)` with a valid pid and the SIGTERM signal number;
        // no memory is touched, only signal delivery.
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
fn new_creates_document_with_inferred_profile() {
    let dir = temp_workspace();
    init(&dir);

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["new", "guides/oauth.md", "--title", "OAuth Setup"])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("created document"), "{stdout}");
    assert!(stdout.contains("profile: guide"), "{stdout}");

    let body = fs::read_to_string(dir.join("guides/oauth.md")).unwrap();
    assert!(body.contains("OAuth Setup"), "{body}");
}

#[test]
fn new_requires_a_path_argument() {
    let dir = temp_workspace();
    init(&dir);

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["new"])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(!out.status.success());
}

#[test]
fn rm_deletes_document_and_scrubs_references() {
    let dir = temp_workspace();
    init(&dir);
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

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["rm", "a.md"])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("deleted document"), "{stdout}");
    assert!(!dir.join("a.md").exists());

    let b_body = fs::read_to_string(dir.join("b.md")).unwrap();
    assert!(!b_body.contains("- a"), "{b_body}");
}

#[test]
fn archive_sets_status_without_moving_file() {
    let dir = temp_workspace();
    init(&dir);
    fs::write(
        dir.join("old.md"),
        "---\nprofile: note\nstatus: draft\nid: old\n---\n\n# Old\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["archive", "old.md"])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("archived document"), "{stdout}");

    // File stays in place; only frontmatter status changes (see help text fix).
    assert!(dir.join("old.md").exists());
    assert!(!dir.join("archive").exists());
    let body = fs::read_to_string(dir.join("old.md")).unwrap();
    assert!(body.contains("status: archived"), "{body}");
}

#[test]
fn bench_strip_dry_run_then_restore_round_trips() {
    let dir = temp_workspace();
    init(&dir);
    fs::write(
        dir.join("doc.md"),
        "---\nprofile: note\nstatus: draft\nid: doc\ndescription: hello\n---\n\n# Doc\n",
    )
    .unwrap();

    // Dry-run strip: must not touch the file.
    let out = Command::new(ods_bin())
        .args(["bench", "strip", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("dry-run"), "{stdout}");
    let before = fs::read_to_string(dir.join("doc.md")).unwrap();
    assert!(before.contains("profile: note"), "{before}");

    // Real strip + restore round trip.
    let out = Command::new(ods_bin())
        .args(["bench", "strip", "--write", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("wrote"), "{stdout}");

    let stripped = fs::read_to_string(dir.join("doc.md")).unwrap();
    assert!(!stripped.contains("profile: note"), "{stripped}");

    let out = Command::new(ods_bin())
        .args(["bench", "restore", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");

    let restored = fs::read_to_string(dir.join("doc.md")).unwrap();
    assert!(restored.contains("profile: note"), "{restored}");
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
fn logs_is_an_alias_for_watch_not_a_log_tail() {
    let dir = temp_workspace();
    init(&dir);

    let mut child = Command::new(ods_bin())
        .args(["logs", dir.to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(800));
    terminate_gracefully(&mut child);
    let _ = child.wait();
    let mut stdout = String::new();
    child.stdout.unwrap().read_to_string(&mut stdout).unwrap();
    // The CLI's own banner documents that `logs` currently just re-runs `watch`.
    assert!(stdout.contains("streaming ods serve logs"), "{stdout}");
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
