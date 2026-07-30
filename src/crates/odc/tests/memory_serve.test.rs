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

#[test]
fn poll_serve_prints_memory_report() {
    let dir = tempfile::tempdir().unwrap();
    let init = Command::new(ods_bin())
        .args(["init", dir.path().to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let mut child = Command::new(ods_bin())
        .args([
            "serve",
            "--mode",
            "poll",
            "--memory-report",
            "--poll-secs",
            "60",
            "--root",
            dir.path().to_str().unwrap(),
        ])
        .env("ODS_AUTO_UPDATE", "0")
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(2));
    terminate_gracefully(&mut child);
    let _ = child.wait();
    let mut stderr = String::new();
    child.stderr.unwrap().read_to_string(&mut stderr).unwrap();
    assert!(stderr.contains("mode=poll"), "{stderr}");
    assert!(stderr.contains("rss_kb="), "{stderr}");

    let rss_kb: u64 = stderr
        .lines()
        .find_map(|line| line.split("rss_kb=").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("could not parse rss_kb from: {stderr}"));

    // Regression canary, not a strict product SLA: docs/guide/faq.md measures
    // ~7.5-17MB on an optimized macOS build; this generous ceiling catches
    // gross regressions (e.g. loading full document bodies unnecessarily)
    // without being flaky across debug builds / CI machines / platforms.
    assert!(
        rss_kb > 0,
        "rss_kb should be a real positive sample: {rss_kb}"
    );
    assert!(
        rss_kb < 100_000,
        "ods serve RSS ({rss_kb} KB) exceeded the 100MB regression ceiling — investigate before raising this limit"
    );
}
