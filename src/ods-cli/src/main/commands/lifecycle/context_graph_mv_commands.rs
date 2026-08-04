fn run_context_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "ods context <id-or-path> [flags]\n\n\
             Resolve a bounded AI reading list (target + depends + context.load).\n\
             Does not walk `related` unless --include-related. Code edges off unless --include-code.\n\
             With --okf on a hybrid workspace, merges OKF markdown-link neighborhood after ODS graph.\n\n\
             Flags:\n\
               --root <dir>           Workspace root (default: cwd)\n\
               --include-private      Include share: private documents\n\
               --include-code         Expand code: edges into the reading list\n\
               --include-related      Also walk soft related: edges\n\
               --explain              Show why each path was included\n\
               --max-tokens <N>       Cap estimated tokens (bytes/4 heuristic)\n\
               --print                Print file contents under the budget (prompt pack)\n\
               --format text|json     Output format (default text)\n\
               --okf                  Include OKF context (pure OKF or hybrid merge)\n"
        );
        return Ok(ExitCode::from(0));
    }
    // Context is special: the primary positional is a *document id*, not a workspace root.
    let (_ignored_root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;

    let positionals = positional_args(args, 2);
    let root_flag = parse_flag_val(args, "--root").map(PathBuf::from);
    let (root_dir, query) = match (root_flag, positionals.as_slice()) {
        (_, []) => return Err(usage_msg(ods_core::missing_context_id())),
        (Some(rf), [id]) => (rf, id.clone()),
        (Some(rf), [_, id]) => (rf, id.clone()),
        (Some(rf), rest) => (rf, rest.last().cloned().unwrap()),
        (None, [maybe_root, id]) if PathBuf::from(maybe_root).is_dir() => {
            (PathBuf::from(maybe_root), id.clone())
        }
        (None, [only]) if PathBuf::from(only).is_dir() => {
            return Err(usage_msg(ods_core::missing_context_id()));
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
    let root_specs = ods_core::load_root_specs_config(&root);
    let engines =
        ods_core::resolve_engines_with_config(extra, detected, Some(&root_specs), true)
            .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_context_command(args);
    }
    if !engines.ods {
        return Err(fail_msg(ods_core::context_requires_ods_or_okf()));
    }

    let include_private = args.iter().any(|arg| arg == "--include-private");
    let include_code = args.iter().any(|arg| arg == "--include-code");
    let include_related = args.iter().any(|arg| arg == "--include-related");
    let explain = args.iter().any(|arg| arg == "--explain");
    let print_pack = args.iter().any(|arg| arg == "--print");
    let max_tokens = parse_flag_val(args, "--max-tokens")
        .map(|v| {
            v.parse::<usize>().map_err(|_| {
                usage_msg(ods_core::missing_flag_value(
                    "--max-tokens",
                    "`ods context id --max-tokens 4000`",
                ))
            })
        })
        .transpose()?;

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;
    let mut result = ods_core::resolve_context_with_options(
        &workspace,
        &query,
        &ods_core::ContextOptions {
            include_private,
            include_code,
            include_related,
            max_tokens,
        },
    );

    // Hybrid: merge OKF markdown-link neighborhood when OKF engine is active.
    if engines.okf {
        if let Ok(bundle) = ods_core::load_okf_bundle(&root) {
            let okf_paths = ods_core::okf_context(&bundle, &query);
            for p in okf_paths {
                if !result.paths.iter().any(|existing| existing == &p) {
                    let tokens = ods_core::estimate_path_tokens(&p);
                    if let Some(budget) = max_tokens {
                        if !result.paths.is_empty()
                            && result.token_estimate.saturating_add(tokens) > budget
                        {
                            result.truncated = true;
                            continue;
                        }
                    }
                    result.token_estimate = result.token_estimate.saturating_add(tokens);
                    result.paths.push(p);
                    result.reasons.push("okf link neighborhood".into());
                }
            }
        }
    }

    if result.paths.is_empty() {
        return Err(fail_msg(ods_core::document_not_found_context(&query)));
    }

    if !result.skipped_private.is_empty() && matches!(format, OutputFormat::Text) {
        eprintln!(
            "warning: skipped {} private document(s) (pass --include-private to include)",
            result.skipped_private.len()
        );
    }
    if result.truncated && matches!(format, OutputFormat::Text) {
        eprintln!(
            "warning: context truncated at ~{} tokens (pass a higher --max-tokens)",
            max_tokens.unwrap_or(0)
        );
    }

    if print_pack {
        let pack = ods_core::render_context_pack(&result.paths, max_tokens);
        print!("{pack}");
        return Ok(ExitCode::from(0));
    }

    match format {
        OutputFormat::Text => {
            for (i, path) in result.paths.iter().enumerate() {
                if explain {
                    let reason = result.reasons.get(i).map(String::as_str).unwrap_or("?");
                    println!("{}  # {reason}", path.display());
                } else {
                    println!("{}", path.display());
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = result
                .paths
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if explain {
                        let reason = result.reasons.get(i).map(String::as_str).unwrap_or("?");
                        format!(
                            r#"{{"path":{},"reason":{}}}"#,
                            json_escape(&p.display().to_string()),
                            json_escape(reason)
                        )
                    } else {
                        json_escape(&p.display().to_string())
                    }
                })
                .collect();
            let skipped: Vec<_> = result
                .skipped_private
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            println!(
                r#"{{"id":{},"root":{},"paths":[{}],"token_estimate":{},"truncated":{},"skipped_private":[{}],"explain":{}}}"#,
                json_escape(&query),
                json_escape(&root.display().to_string()),
                items.join(","),
                result.token_estimate,
                result.truncated,
                skipped.join(","),
                explain
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn run_graph_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;
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
            return Err(usage_msg(ods_core::missing_required_arg("from/to", "ods mv --root <dir> <from> <to>")));
        }
    } else if positionals.len() >= 3 && PathBuf::from(&positionals[0]).is_dir() {
        (PathBuf::from(&positionals[0]), positionals[1].clone(), positionals[2].clone())
    } else if positionals.len() == 2 {
        (env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), positionals[0].clone(), positionals[1].clone())
    } else {
        return Err(usage_msg(ods_core::missing_required_arg("from/to", "ods mv [root] <from> <to> [--dry-run]")));
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
        .map_err(|err| fail_io("mv/graph", err))?;
    print_path_change_report(&root, &from, &to, &report, format, "moved");
    Ok(ExitCode::from(if report.errors.is_empty() { 0 } else { 1 }))
}
