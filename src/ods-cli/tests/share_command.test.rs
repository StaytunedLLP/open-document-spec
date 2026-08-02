//! CLI smoke tests for `ods share`.
use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn share_writes_filtered_directory_by_default() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .status()
            .unwrap()
            .success()
    );
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
    fs::write(
        dir.join("internal.md"),
        "---\nprofile: note\nstatus: draft\nid: internal\nshare: org\n---\n\n# Internal\n",
    )
    .unwrap();

    let out = dir.join("dist");
    let status = Command::new(ods_bin())
        .args(["share", root, "--out", out.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    assert!(out.join("public.md").exists());
    assert!(!out.join("secret.md").exists());
    assert!(!out.join("internal.md").exists());
    assert!(out.join("index.md").exists());
}

#[test]
fn share_include_org_and_private_flags_widen_output() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        dir.join("secret.md"),
        "---\nprofile: note\nstatus: draft\nid: secret\nshare: private\n---\n\n# Secret\n",
    )
    .unwrap();
    fs::write(
        dir.join("internal.md"),
        "---\nprofile: note\nstatus: draft\nid: internal\nshare: org\n---\n\n# Internal\n",
    )
    .unwrap();

    let out = dir.join("dist-all");
    let status = Command::new(ods_bin())
        .args([
            "share",
            root,
            "--out",
            out.to_str().unwrap(),
            "--include-org",
            "--include-private",
        ])
        .status()
        .unwrap();
    assert!(status.success());

    assert!(out.join("secret.md").exists());
    assert!(out.join("internal.md").exists());
}

#[test]
fn share_requires_out_flag() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .status()
            .unwrap()
            .success()
    );

    let status = Command::new(ods_bin())
        .args(["share", root])
        .status()
        .unwrap();
    assert!(!status.success());
}
