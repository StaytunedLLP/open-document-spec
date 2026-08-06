use ods_test_support::temp_workspace;
use std::fs;
use std::process::Command;

fn ods_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn test_ods_read_command_full_surface() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();

    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let doc_content = r#"---
profile: architecture
status: stable
tags: [core, engine]
---

# High Level Architecture

Overview of system design.

## Core Modules

Engine details.

### Storage

Database and filesystem layout.

## API Surface

REST and CLI entry points.
"#;
    fs::write(dir.join("arch.md"), doc_content).unwrap();

    // 1. Full read
    let out = Command::new(ods_bin())
        .args(["read", root, "arch.md"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("High Level Architecture"), "{stdout}");
    assert!(stdout.contains("Core Modules"), "{stdout}");

    // 2. Section read: --section "Core Modules"
    let out = Command::new(ods_bin())
        .args(["read", root, "arch.md", "--section", "Core Modules"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Core Modules"), "{stdout}");
    assert!(stdout.contains("Storage"), "{stdout}");
    assert!(!stdout.contains("API Surface"), "{stdout}");

    // 3. Summary mode: --summary
    let out = Command::new(ods_bin())
        .args(["read", root, "arch.md", "--summary"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Outline Summary"), "{stdout}");
    assert!(stdout.contains("Core Modules"), "{stdout}");

    // 4. Token budget limit: --max-tokens 10
    let out = Command::new(ods_bin())
        .args(["read", root, "arch.md", "--max-tokens", "10"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("truncated") || stderr.contains("truncated"),
        "{stdout}\n{stderr}"
    );

    // 5. JSON format: --format json
    let out = Command::new(ods_bin())
        .args(["read", root, "arch.md", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"outline\""), "{stdout}");
    assert!(stdout.contains("\"token_estimate\""), "{stdout}");

    // 6. Path traversal rejection
    let out = Command::new(ods_bin())
        .args(["read", root, "../../etc/passwd"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{:?}", out);
}
