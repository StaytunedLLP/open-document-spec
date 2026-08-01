fn run_lint_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|arg| arg == "--okf") {
        return run_okf_lint_command(args);
    }
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
            OutputFormat::Json | OutputFormat::Sarif => {
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
            OutputFormat::Json | OutputFormat::Sarif => {
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

fn run_tags_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let include_all = args.iter().any(|a| a == "--all");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    print_tags(&workspace, include_all, format);
    Ok(ExitCode::from(0))
}

fn run_coverage_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let write_report = args.iter().any(|a| a == "--write-report");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;

    let total = workspace.documents.len();
    let mut compliant = 0usize;
    let mut non_compliant = 0usize;

    for doc in &workspace.documents {
        let is_parsed = matches!(doc.frontmatter, ods_core::FrontmatterState::Parsed(_));
        let diags = ods_core::lint_document_in_workspace(&workspace, &doc.path, level);
        if is_parsed && diags.is_empty() {
            compliant += 1;
        } else {
            non_compliant += 1;
        }
    }

    let pct = if total == 0 {
        100.0
    } else {
        (compliant as f64 / total as f64) * 100.0
    };

    match format {
        OutputFormat::Text => {
            println!("Documentation Health: {:.1}% Compliant ({}/{} files)", pct, compliant, total);
            println!("  ✔ Compliant:     {} documents", compliant);
            println!("  ✖ Non-Compliant:  {} documents", non_compliant);
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"health_pct":{:.1},"compliant":{},"non_compliant":{},"total":{}}}"#,
                pct, compliant, non_compliant, total
            );
        }
    }

    if write_report {
        let report_content = format!(
            "# Documentation Health & Coverage Report\n\n- Score: {:.1}% Compliant\n- Compliant Documents: {}\n- Non-Compliant Documents: {}\n- Total Documents: {}\n\nNote: this is separate from lint/audit diagnostics (`.ods/ods-errors.md`).\n",
            pct, compliant, non_compliant, total
        );
        let odc_dir = root.join(".ods");
        let _ = std::fs::create_dir_all(&odc_dir);
        let report_path = odc_dir.join("coverage.md");
        std::fs::write(&report_path, report_content)
            .map_err(|e| failure(format!("write {}: {e}", report_path.display())))?;
        if matches!(format, OutputFormat::Text) {
            println!("wrote {}", report_path.display());
        }
    }

    Ok(ExitCode::from(0))
}
