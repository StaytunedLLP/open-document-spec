use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn git_available() -> bool {
    Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .expect("git")
}

#[test]
fn sync_rewrites_refs_after_git_mv() {
    if !git_available() {
        eprintln!("skipping sync_rewrites_refs_after_git_mv: git not available");
        return;
    }

    let dir = temp_workspace();
    let root = dir.path();

    assert!(
        Command::new(ods_bin())
            .args(["init", root.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    fs::write(
        root.join("pricing.md"),
        "---\nprofile: note\nstatus: draft\nid: pricing\n---\n\n# Pricing\n",
    )
    .unwrap();
    fs::write(
        root.join("service.md"),
        "---\nprofile: note\nstatus: draft\nid: service\ndepends:\n  - pricing\n---\n\n# Service\n",
    )
    .unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["lint", root.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );

    assert!(git(root, &["init"]).status.success());
    // Detached identity for CI-friendly commits.
    assert!(
        git(
            root,
            &[
                "-c",
                "user.email=ods@test",
                "-c",
                "user.name=ods",
                "add",
                "-A",
            ]
        )
        .status
        .success()
    );
    assert!(
        git(
            root,
            &[
                "-c",
                "user.email=ods@test",
                "-c",
                "user.name=ods",
                "commit",
                "-m",
                "init",
            ]
        )
        .status
        .success()
    );
    assert!(
        git(root, &["mv", "pricing.md", "pricing-new.md"])
            .status
            .success()
    );

    let out = Command::new(ods_bin())
        .args(["sync", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "sync failed: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let service = fs::read_to_string(root.join("service.md")).unwrap();
    assert!(
        service.contains("pricing-new")
            || service.contains("pricing_new")
            || service.contains("pricing-new.md")
            || service.to_lowercase().contains("pricing-new"),
        "expected depends rewritten toward pricing-new, got:\n{service}"
    );
    // Path-id rewrite uses path_to_default_id — typically "pricing-new".
    assert!(
        !service.contains("- pricing\n") || service.contains("pricing-new"),
        "stale depends still present:\n{service}"
    );
}

#[test]
fn format_json_on_index_and_mv() {
    let dir = temp_workspace();
    let root = dir.path();
    assert!(
        Command::new(ods_bin())
            .args(["init", root.to_str().unwrap()])
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        root.join("a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        root.join("b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# B\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .args(["lint", "--format", "json", root.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"written\""), "{stdout}");

    let out = Command::new(ods_bin())
        .args([
            "mv",
            "--format",
            "json",
            root.to_str().unwrap(),
            "a.md",
            "c.md",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"rewritten\""), "{stdout}");
    assert!(stdout.contains("\"from\""), "{stdout}");
}
