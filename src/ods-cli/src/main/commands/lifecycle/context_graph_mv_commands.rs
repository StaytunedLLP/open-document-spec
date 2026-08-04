fn run_context_command(args: &[String]) -> Result<ExitCode, CliError> {
    // Context is special: the primary positional is a *document id*, not a workspace
    // root. Using parse_common_flags alone treats `ods context specs/ods/core` as
    // root=specs/ods/core and historically collapsed find_workspace_root to "".
    let (_ignored_root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;

    let positionals = positional_args(args, 2);
    let root_flag = parse_flag_val(args, "--root").map(PathBuf::from);
    let (root_dir, query) = match (root_flag, positionals.as_slice()) {
        (_, []) => return Err(usage("missing document id (usage: ods context <id-or-path>)")),
        (Some(rf), [id]) => (rf, id.clone()),
        (Some(rf), [_, id]) => (rf, id.clone()),
        (Some(rf), rest) => (rf, rest.last().cloned().unwrap()),
        // `ods context <existing-dir> <id>` — explicit workspace + id
        (None, [maybe_root, id]) if PathBuf::from(maybe_root).is_dir() => {
            (PathBuf::from(maybe_root), id.clone())
        }
        // `ods context <id>` — workspace is cwd (document id is NOT a root path)
        (None, [only]) if PathBuf::from(only).is_dir() => {
            return Err(usage(
                "missing document id (usage: ods context <id-or-path> or ods context <workspace-dir> <id>)",
            ));
        }
        (None, [only]) => (
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            only.clone(),
        ),
        (None, rest) => (
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            rest.last().cloned().unwrap(),
        ),
    };

    let root = resolve_root_path(root_dir);
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_context_command(args);
    }
    if !engines.ods {
        return Err(failure(
            "context requires an ODS workspace (or pass `--okf` for OKF context)",
        ));
    }

    let include_private = args.iter().any(|arg| arg == "--include-private");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    let paths = resolve_context(&workspace, &query, include_private);
    if paths.is_empty() {
        return Err(failure(format!(
            "document not found for context query `{query}` in workspace {} \
             (try a path-shaped id like `specs/ods/core`, a relative `.md` path, or `ods find`)",
            root.display()
        )));
    }
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
                r#"{{"id":{},"root":{},"paths":[{}]}}"#,
                json_escape(&query),
                json_escape(&root.display().to_string()),
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
    let (_, _level, format) = parse_common_flags(args, 2)?;
    let positionals = positional_args(args, 2);
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let root_flag = parse_flag_val(args, "--root").map(PathBuf::from);
    let (root_dir, from, to) = if let Some(rf) = root_flag {
        if positionals.len() >= 2 {
            (rf, positionals[0].clone(), positionals[1].clone())
        } else {
            return Err(usage("usage: ods mv --root <dir> <from> <to>"));
        }
    } else if positionals.len() >= 3 && PathBuf::from(&positionals[0]).is_dir() {
        (PathBuf::from(&positionals[0]), positionals[1].clone(), positionals[2].clone())
    } else if positionals.len() == 2 {
        (env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), positionals[0].clone(), positionals[1].clone())
    } else {
        return Err(usage("usage: ods mv [root] <from> <to> [--dry-run]"));
    };

    let root = resolve_root_path(root_dir);
    require_ods_workspace(&root)?;

    if dry_run {
        match format {
            OutputFormat::Text => {
                println!("(dry-run) would move document {} to {} and rewrite references across workspace {}", from, to, root.display());
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                println!(
                    r#"{{"dry_run":true,"from":{},"to":{},"root":{}}}"#,
                    json_escape(&from),
                    json_escape(&to),
                    json_escape(&root.display().to_string())
                );
            }
        }
        return Ok(ExitCode::from(0));
    }

    let report = move_document_and_rewrite_refs_report(&root, &from, &to)
        .map_err(|err| failure(err.to_string()))?;
    print_path_change_report(&root, &from, &to, &report, format, "moved");
    Ok(ExitCode::from(if report.errors.is_empty() { 0 } else { 1 }))
}
