fn run_adopt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_adopt_command(args);
    }
    if !engines.ods {
        return Err(failure(
            "adopt requires an ODS workspace (or pass `--okf` for OKF adopt)",
        ));
    }
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
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    if extra.okf {
        return run_okf_init_command(args);
    }
    if extra.skills {
        return run_skills_init_command(args);
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
            println!("next: ods lint   # or: ods watch");
        }
        OutputFormat::Json | OutputFormat::Sarif => {
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

fn run_skills_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let name = args
        .windows(2)
        .find(|w| w[0] == "--name")
        .map(|w| w[1].clone());
    let report = ods_core::init_skill_package(
        &root,
        ods_core::SkillsInitOptions { name },
    )
    .map_err(|e| failure(e.to_string()))?;
    match format {
        OutputFormat::Text => {
            println!("initialized Agent Skills package at {}", report.root.display());
            for p in &report.created {
                println!("  created {}", p.display());
            }
            for p in &report.skipped {
                println!("  skipped {}", p.display());
            }
            println!("next: ods lint --skills");
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"root":{},"created":{},"skipped":{}}}"#,
                json_escape(&report.root.display().to_string()),
                report.created.len(),
                report.skipped.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}
