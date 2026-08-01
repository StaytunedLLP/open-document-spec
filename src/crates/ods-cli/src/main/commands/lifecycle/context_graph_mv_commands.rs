fn run_context_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let positionals = positional_args(args, 2);
    let query = match positionals.as_slice() {
        [] => return Err(usage("missing document id")),
        [only] if PathBuf::from(only).is_dir() => {
            return Err(usage("missing document id"));
        }
        [only] => only.clone(),
        [maybe_root, id] if PathBuf::from(maybe_root).is_dir() => id.clone(),
        rest => rest.last().cloned().unwrap(),
    };

    let include_private = args.iter().any(|arg| arg == "--include-private");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    let paths = resolve_context(&workspace, &query, include_private);
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
                r#"{{"id":{},"paths":[{}]}}"#,
                json_escape(&query),
                items.join(",")
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn run_graph_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    let lines = graph_lines(&workspace);
    match format {
        OutputFormat::Text => {
            for line in &lines {
                println!("{line}");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = lines.iter().map(|l| json_escape(l)).collect();
            println!("[{}]", items.join(","));
        }
    }
    Ok(ExitCode::from(0))
}

fn run_mv_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let positionals = positional_args(args, 2);
    // root may be first positional if provided
    let (from, to) = if positionals.len() >= 3 {
        (positionals[1].clone(), positionals[2].clone())
    } else if positionals.len() == 2 {
        (positionals[0].clone(), positionals[1].clone())
    } else {
        return Err(usage("usage: ods mv [root] <from> <to>"));
    };
    let report = move_document_and_rewrite_refs_report(&root, &from, &to)
        .map_err(|err| failure(err.to_string()))?;
    print_path_change_report(&root, &from, &to, &report, format, "moved");
    Ok(ExitCode::from(if report.errors.is_empty() { 0 } else { 1 }))
}
