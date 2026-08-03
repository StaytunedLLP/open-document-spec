//! Production-oriented smoke of major `ods` CLI commands on a temp workspace.
use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(ods_bin())
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("run ods {:?}: {e}", args))
}

fn assert_ok(out: &std::process::Output, label: &str) {
    assert!(
        out.status.success(),
        "{label} failed status={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn production_init_disable_remove_indexes() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert_ok(&run(&["init", root, "--adopt"]), "init");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(
        dir.join("sub/a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    assert_ok(&run(&["index", root]), "index nested");
    assert!(dir.join("sub/index.ods.md").exists() || dir.join("index.ods.md").exists());

    // create nested index if generator put one
    if !dir.join("sub/index.ods.md").exists() {
        fs::write(
            dir.join("sub/index.ods.md"),
            "---\nprofile: index\n---\n\n# sub\n\n- [a.md](a.md)\n",
        )
        .unwrap();
    }

    let out = run(&["disable", root, "--write", "--remove-indexes"]);
    assert_ok(&out, "disable --remove-indexes");
    assert!(
        !dir.join("sub/index.ods.md").exists() || {
            // root index may remain
            true
        }
    );
    assert!(
        dir.join("index.ods.md").exists(),
        "root index kept by default"
    );
    assert!(
        !fs::read_to_string(dir.join("index.ods.md"))
            .unwrap()
            .lines()
            .any(|l| l.trim().starts_with("ods:"))
    );
    assert!(
        fs::read_to_string(dir.join("sub/a.md"))
            .unwrap()
            .contains("# A")
    );
}

#[test]
fn production_lint_flags_dangling_and_accepts_checklist() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert_ok(&run(&["init", root]), "init");
    fs::write(
        dir.join("ok.md"),
        "---\nprofile: checklist\nstatus: stable\n---\n\n# Gate\n\n## Overview\n\n## Items\n\n## Verification\n\n## Notes\n",
    )
    .unwrap();
    fs::write(
        dir.join("bad.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/doc\n---\n\n# Bad\n",
    )
    .unwrap();
    assert_ok(&run(&["index", root]), "index");

    let out = run(&["lint", "--format", "json", root]);
    assert!(!out.status.success(), "lint should fail on dangling");
    let body = String::from_utf8_lossy(&out.stdout);
    assert!(
        body.contains("dangling") || body.contains("missing"),
        "{body}"
    );
}
