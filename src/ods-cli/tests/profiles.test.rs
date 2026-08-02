use ods_test_support::temp_workspace;
use std::env;
use std::fs;
use std::process::{Command, Stdio};

#[test]
fn profiles_command_reports_merged_catalog_sources() {
    let root = temp_workspace();
    fs::create_dir_all(root.join("ods-profiles")).expect("catalog dir");
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\ncustom-profiles:\n  - ods-profiles/custom.md\n---\n\n# Root\n",
    )
    .expect("root index");
    fs::write(
        root.join("ods-profiles").join("custom.md"),
        "# Custom\n\n## Overview\n",
    )
    .expect("custom profile");

    let bin = env!("CARGO_BIN_EXE_ods");
    let output = Command::new(bin)
        .current_dir(&root)
        .arg("profiles")
        .arg(&root)
        .stdin(Stdio::null())
        .output()
        .expect("run ods profiles");

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(stdout.contains("profiles:"), "{stdout}");
    assert!(stdout.contains("custom:"), "{stdout}");
    assert!(
        stdout.replace('\\', "/").contains("ods-profiles/custom.md"),
        "{stdout}"
    );
    assert!(
        stdout.contains("[project]"),
        "project layer label missing: {stdout}"
    );
    assert!(
        stdout.contains("checklist:") && stdout.contains("[default ODS]"),
        "checklist default ODS missing: {stdout}"
    );
}
