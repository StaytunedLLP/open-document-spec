use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn help_lists_serve_modes_and_setup() {
    let out = Command::new(ods_bin()).arg("--help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("setup [path]"), "{stdout}");
    assert!(stdout.contains("serve --mode poll"), "{stdout}");
    assert!(stdout.contains("ODS_LOW_MEMORY=1"), "{stdout}");
}

#[test]
fn invalid_serve_mode_is_usage_error() {
    let out = Command::new(ods_bin())
        .args(["serve", "--mode", "tiny"])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("invalid") && (stderr.contains("--mode") || stderr.contains("mode")),
        "{stderr}"
    );
    assert!(stderr.contains("Next:"), "{stderr}");
}
