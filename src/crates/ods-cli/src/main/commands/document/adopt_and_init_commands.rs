fn run_adopt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let write = args.iter().any(|a| a == "--write");
    let workspace = load_workspace(&root).map_err(|err| failure(err.to_string()))?;
    let report = adopt_workspace(&workspace, AdoptOptions { write })
        .map_err(|err| failure(err.to_string()))?;
    // Re-load after writes for accurate lint
    let workspace = if write {
        load_workspace(&root).map_err(|err| failure(err.to_string()))?
    } else {
        workspace
    };
    let diagnostics = lint_workspace_with_level(&workspace, level);
    print_diagnostics(&diagnostics, format);
    println!("profiles: {}", known_profiles(&workspace).join(", "));
    print_aliases(&workspace);
    print_alias_suggestions(&workspace);
    if write {
        println!("adopt wrote {} document(s)", report.written.len());
        for path in &report.written {
            println!("  wrote {}", path.display());
        }
    } else {
        println!(
            "adopt dry-run: {} document(s) would receive frontmatter (pass --write)",
            report.would_write.len()
        );
        for path in report.would_write.iter().take(20) {
            println!("  would write {}", path.display());
        }
        if report.would_write.len() > 20 {
            println!("  ... and {} more", report.would_write.len() - 20);
        }
    }
    Ok(exit_code(&diagnostics))
}

fn run_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--okf") {
        return run_okf_init_command(args);
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let adopt = args.iter().any(|a| a == "--adopt");
    let report = init_workspace(&root, InitOptions { adopt })
        .map_err(|err| failure(err.to_string()))?;
    match format {
        OutputFormat::Text => {
            if report.already_initialized && !report.initialized {
                println!("ODS already initialized at {}", report.root.display());
            } else if report.initialized {
                println!("initialized ODS at {}", report.root.display());
            } else {
                println!("ODS workspace {}", report.root.display());
            }
            if !report.adopted.is_empty() {
                println!("adopted {} document(s)", report.adopted.len());
            }
            println!("indexes: {} file(s)", report.indexes.len());
            println!("next: odc lint   # or: odc watch");
        }
        OutputFormat::Json => {
            println!(
                r#"{{"root":{},"initialized":{},"already_initialized":{},"adopted":{},"indexes":{}}}"#,
                json_escape(&report.root.display().to_string()),
                report.initialized,
                report.already_initialized,
                report.adopted.len(),
                report.indexes.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}
