fn run_lint_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let canonical_refs = args.iter().any(|arg| arg == "--canonical-refs");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    let diagnostics = if canonical_refs {
        lint_workspace_with_ref_style(&workspace, level, true)
    } else {
        lint_workspace_with_level(&workspace, level)
    };
    print_diagnostics(&diagnostics, format);
    write_or_clear_ods_error_report(&root, &diagnostics, format)?;
    if diagnostics.is_empty() && matches!(format, OutputFormat::Text) {
        println!(
            "Everything is fine — graph and links are consistent. No update required."
        );
    }
    Ok(exit_code(&diagnostics))
}

fn run_index_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let check = args.iter().any(|a| a == "--check");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    if check {
        let current =
            indexes_are_current(&workspace).map_err(|err| failure(err.to_string()))?;
        match format {
            OutputFormat::Text => {
                if current {
                    println!("indexes up to date");
                } else {
                    eprintln!("indexes out of date; run `ods index`");
                }
            }
            OutputFormat::Json => {
                println!(
                    r#"{{"current":{},"root":{}}}"#,
                    if current { "true" } else { "false" },
                    json_escape(&root.display().to_string())
                );
            }
        }
        Ok(ExitCode::from(if current { 0 } else { 1 }))
    } else {
        let paths = generate_indexes(&workspace).map_err(|err| failure(err.to_string()))?;
        match format {
            OutputFormat::Text => {
                for path in &paths {
                    println!("{}", path.display());
                }
            }
            OutputFormat::Json => {
                let items: Vec<_> = paths
                    .iter()
                    .map(|p| json_escape(&p.display().to_string()))
                    .collect();
                println!(
                    r#"{{"written":[{}],"count":{}}}"#,
                    items.join(","),
                    paths.len()
                );
            }
        }
        Ok(ExitCode::from(0))
    }
}

fn run_profiles_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    print_profiles(&workspace, format);
    Ok(ExitCode::from(0))
}

fn run_tags_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let include_all = args.iter().any(|a| a == "--all");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    print_tags(&workspace, include_all, format);
    Ok(ExitCode::from(0))
}
