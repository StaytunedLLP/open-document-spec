use ods_test_support::ChildGuard;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
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
    let mut guard = ChildGuard::new(
        Command::new(ods_bin())
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
            .unwrap(),
    );
    std::thread::sleep(Duration::from_secs(2));
    let mut stderr_pipe = guard
        .child_mut()
        .expect("child")
        .stderr
        .take()
        .expect("stderr");
    let _ = guard.terminate();
    let mut stderr = String::new();
    stderr_pipe.read_to_string(&mut stderr).unwrap();
    assert!(stderr.contains("mode=poll"), "{stderr}");
    assert!(stderr.contains("rss_kb="), "{stderr}");

    let rss_kb: u64 = stderr
        .lines()
        .find_map(|line| line.split("rss_kb=").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or_else(|| panic!("could not parse rss_kb from: {stderr}"));

    assert!(
        rss_kb > 0,
        "rss_kb should be a real positive sample: {rss_kb}"
    );
    // Product SLA: service.max_rss_mb = 10. Debug builds may be larger; allow
    // ODS_MEM_TEST_RELAXED=1 or non-release to use a 32MB soft cap.
    let limit_kb: u64 = if cfg!(debug_assertions) || std::env::var("ODS_MEM_TEST_RELAXED").is_ok() {
        32_768
    } else {
        10_240
    };
    assert!(
        rss_kb < limit_kb,
        "ods serve RSS ({rss_kb} KB) exceeded {limit_kb} KB budget — investigate before raising service.max_rss_mb"
    );
}
