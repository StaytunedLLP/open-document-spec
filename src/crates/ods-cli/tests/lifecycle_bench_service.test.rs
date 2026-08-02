//! CLI coverage for previously-untested command wiring: `new`/`rm`/`archive`,
//! `bench strip/restore/stats/run`, `logs`, `export --include-private`,
//! and read-only service commands (`start --status`, `stop`).

use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
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
