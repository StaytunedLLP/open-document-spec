use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn ods_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn test_coverage_listing_non_compliant_files() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create ODS index.md
    fs::write(
        root.join("ods.toml"), "spec = \"0.1\"
",
    )
    .unwrap();

    // Valid compliant doc
    fs::write(
        root.join("doc1.md"),
        "---\nprofile: note\nods: 0.1\n---\n\n# Doc 1\n",
    )
    .unwrap();

    // Non-compliant doc (invalid YAML frontmatter)
    fs::write(
        root.join("broken.md"),
        "---\nprofile: note\nods: invalid_yaml: [:\n---\n\n# Broken\n",
    )
    .unwrap();

    // Test text coverage output
    let out = Command::new(ods_bin())
        .args(["coverage", root.to_str().unwrap()])
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Documentation Health:"));
    assert!(stdout.contains("Non-Compliant Documents:"));
    assert!(stdout.contains("broken.md"));

    // Test JSON coverage output
    let out_json = Command::new(ods_bin())
        .args(["coverage", root.to_str().unwrap(), "--format", "json"])
        .output()
        .unwrap();

    assert!(out_json.status.success());
    let stdout_json = String::from_utf8_lossy(&out_json.stdout);
    assert!(stdout_json.contains("non_compliant_files"));
    assert!(stdout_json.contains("broken.md"));

    // Test write report
    let out_report = Command::new(ods_bin())
        .args(["coverage", root.to_str().unwrap(), "--write-report"])
        .output()
        .unwrap();

    assert!(out_report.status.success());
    assert!(root.join(".ods/coverage.md").exists());
    let report_text = fs::read_to_string(root.join(".ods/coverage.md")).unwrap();
    assert!(report_text.contains("broken.md"));
}

#[test]
fn test_bench_agent_subcommand() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(
        root.join("ods.toml"), "spec = \"0.1\"
",
    )
    .unwrap();

    fs::write(
        root.join("api.md"),
        "---\nprofile: note\nods: 0.1\n---\n\n# API Specs\n",
    )
    .unwrap();

    // Test bench agent subcommand
    let out = Command::new(ods_bin())
        .args([
            "bench",
            "agent",
            root.to_str().unwrap(),
            "--agent",
            "antigravity",
            "--prompt",
            "Refactor endpoints",
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "bench agent failed:\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("ODS AI / Agent Benchmark Report"));
    assert!(stdout.contains("Agent Profile Target: antigravity"));
    assert!(stdout.contains("Agent Prompt Fitness:"));

    // Test bench agent JSON format
    let out_json = Command::new(ods_bin())
        .args([
            "bench",
            "run",
            root.to_str().unwrap(),
            "--agent",
            "claude",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout_json = String::from_utf8_lossy(&out_json.stdout);
    let stderr_json = String::from_utf8_lossy(&out_json.stderr);
    assert!(
        out_json.status.success(),
        "bench run json failed:\nstdout: {stdout_json}\nstderr: {stderr_json}"
    );
    assert!(stdout_json.contains("agent_profile"));
    assert!(stdout_json.contains("agent_fitness_score"));
}

#[test]
fn test_friction_free_ods_mv_and_rm_dry_run() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    fs::write(
        root.join("ods.toml"), "spec = \"0.1\"
",
    )
    .unwrap();

    fs::write(
        root.join("doc_a.md"),
        "---\nprofile: note\nods: 0.1\n---\n\n# Doc A\n",
    )
    .unwrap();

    // Test ods mv --dry-run
    let out_mv = Command::new(ods_bin())
        .args([
            "mv",
            root.to_str().unwrap(),
            "doc_a.md",
            "doc_b.md",
            "--dry-run",
        ])
        .output()
        .unwrap();

    let stdout_mv = String::from_utf8_lossy(&out_mv.stdout);
    let stderr_mv = String::from_utf8_lossy(&out_mv.stderr);
    assert!(
        out_mv.status.success(),
        "ods mv dry run failed:\nstdout: {stdout_mv}\nstderr: {stderr_mv}"
    );
    assert!(stdout_mv.contains("(dry-run) would move document"));
    assert!(root.join("doc_a.md").exists());

    // Test ods rm --dry-run
    let out_rm = Command::new(ods_bin())
        .args([
            "rm",
            root.to_str().unwrap(),
            "doc_a.md",
            "--dry-run",
            "--format",
            "json",
        ])
        .output()
        .unwrap();

    let stdout_rm = String::from_utf8_lossy(&out_rm.stdout);
    let stderr_rm = String::from_utf8_lossy(&out_rm.stderr);
    assert!(
        out_rm.status.success(),
        "ods rm dry run failed:\nstdout: {stdout_rm}\nstderr: {stderr_rm}"
    );
    assert!(stdout_rm.contains("\"dry_run\":true"));
    assert!(root.join("doc_a.md").exists());
}

/// Directive CLI errors: short summary + Next line from central catalog.
#[test]
fn directive_error_messages_unknown_command_and_not_workspace() {
    let dir = tempdir().unwrap();

    // Unknown command → usage + Next (+ optional did-you-mean)
    let out = Command::new(ods_bin())
        .current_dir(dir.path())
        .args(["lintt"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("unknown command"), "{err}");
    assert!(err.contains("Next:"), "{err}");
    assert!(
        err.contains("did you mean `lint`?") || err.contains("ods help"),
        "{err}"
    );

    // Lint outside workspace → error + Next (ods init)
    let out2 = Command::new(ods_bin())
        .current_dir(dir.path())
        .args(["lint"])
        .output()
        .unwrap();
    assert!(!out2.status.success());
    let err2 = String::from_utf8_lossy(&out2.stderr);
    assert!(
        err2.contains("not an ODS workspace") || err2.contains("error:"),
        "{err2}"
    );
    assert!(err2.contains("Next:"), "{err2}");
    assert!(err2.contains("ods init"), "{err2}");

    // Forbidden --ods flag
    let out3 = Command::new(ods_bin())
        .current_dir(dir.path())
        .args(["lint", "--ods"])
        .output()
        .unwrap();
    assert!(!out3.status.success());
    let err3 = String::from_utf8_lossy(&out3.stderr);
    assert!(err3.contains("--ods"), "{err3}");
    assert!(err3.contains("Next:"), "{err3}");
}
