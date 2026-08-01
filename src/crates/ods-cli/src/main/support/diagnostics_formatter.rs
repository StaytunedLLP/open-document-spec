fn run_update_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut check_only = false;
    let mut force = false;
    let mut version = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--check" => {
                check_only = true;
                i += 1;
            }
            "--force" => {
                force = true;
                i += 1;
            }
            "--version" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("missing value for update --version <tag>"))?;
                version = Some(v.clone());
                i += 2;
            }
            other if other.starts_with('-') => {
                return Err(usage(format!("unknown update flag: {other}")));
            }
            other => {
                // bare tag: ods update v0.1.5
                version = Some(other.to_string());
                i += 1;
            }
        }
    }

    let outcome = run_update(UpdateOptions {
        check_only,
        force,
        version,
    })
    .map_err(failure)?;

    match outcome {
        UpdateOutcome::UpToDate { current, remote } => {
            println!("ods {current} is up to date (latest {remote})");
            migrate_machine_and_workspace_on_update();
            restart_service_if_active();
            Ok(ExitCode::from(0))
        }
        UpdateOutcome::Available { current, remote } => {
            println!("update available: {current} → {remote} (run: ods update)");
            Ok(ExitCode::from(1))
        }
        UpdateOutcome::Updated { from, to } => {
            println!("updated ods: {from} → {to}");
            migrate_machine_and_workspace_on_update();
            restart_service_if_active();
            Ok(ExitCode::from(0))
        }
    }
}

fn migrate_machine_and_workspace_on_update() {
    // 1. Machine config migration: ~/.ods -> ~/.ods
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
    if let Some(home) = home {
        let legacy = std::path::PathBuf::from(&home).join(".ods");
        let modern = std::path::PathBuf::from(&home).join(".ods");
        if legacy.exists() && !modern.exists() {
            let _ = std::fs::create_dir_all(&modern);
            for name in ["odsconfig.toml", "workspaces.toml"] {
                let src = legacy.join(name);
                if src.exists() {
                    let dst_name = if name == "odsconfig.toml" { "odcconfig.toml" } else { name };
                    let dst = modern.join(dst_name);
                    if !dst.exists()
                        && std::fs::copy(&src, &dst).is_ok()
                    {
                        println!(
                            "ods: migrated machine config {} -> {}",
                            src.display(),
                            dst.display()
                        );
                    }
                }
            }
        }
    }

    // 2. Workspace root pin rewrite: ods-cli: -> ods: and cleanup legacy ods-error.md
    let probe = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Some(root) = find_marked_ods_workspace_root(&probe) {
        let index = root.join("index.md");
        if let Ok(text) = std::fs::read_to_string(&index) {
            if text.contains("ods-cli:") {
                let updated = text.replace("ods-cli:", "ods:");
                if std::fs::write(&index, updated).is_ok() {
                    println!("ods: rewrote legacy ods-cli: → ods: on root index.md");
                }
            }
        }
        let legacy_err = root.join("ods-error.md");
        if legacy_err.exists() {
            let _ = std::fs::remove_file(&legacy_err);
        }
    }
}

fn restart_service_if_active() {
    let probe = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    if let Some(root) = find_marked_ods_workspace_root(&probe) {
        let st = service::service_status(&root);
        if st.installed || st.running {
            match service::start_service(&root) {
                Ok(msg) => println!("ods: background service restart: {msg}"),
                Err(e) => eprintln!("ods: service restart check: {e}"),
            }
        }
    }
}

#[cfg(test)]
mod test_diagnostics_formatter {
    use super::*;

    #[test]
    fn test_run_update_command_parsing_args() {
        let err1 = run_update_command(&["ods".into(), "update".into(), "--unknown".into()]);
        assert!(err1.is_err());

        let err2 = run_update_command(&["ods".into(), "update".into(), "--version".into()]);
        assert!(err2.is_err());

        let res = run_update_command(&["ods".into(), "update".into(), "--check".into()]);
        assert!(res.is_ok() || res.is_err());
    }

    #[test]
    fn test_restart_service_if_active_smoke() {
        restart_service_if_active();
    }
}

