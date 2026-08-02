fn run_doctor_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;

    let mut has_error = false;

    if engines.ods {
        let report = doctor_workspace(&root)?;
        match format {
            OutputFormat::Text => println!("{}", report.text),
            OutputFormat::Json | OutputFormat::Sarif => println!("{}", report.json),
        }
        has_error |= report.has_error;
    }

    if engines.okf {
        // OKF doctor (flag path: `ods doctor --okf`).
        return run_okf_doctor_command(args);
    }

    if engines.skills && !engines.ods {
        println!("skills: package detected; full doctor lands with skills engine (use `ods lint --skills`).");
    }

    Ok(ExitCode::from(if has_error { 1 } else { 0 }))
}

fn run_sync_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let report = sync_git_renames(&root)?;
    print_path_change_report(&root, "git", "sync", &report, format, "synced");
    Ok(ExitCode::from(if report.errors.is_empty() { 0 } else { 1 }))
}

fn run_watch_command(args: &[String]) -> Result<ExitCode, CliError> {
    maybe_auto_update_on_watch();
    let (root, level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_watch_command(args, false);
    }
    if !engines.ods {
        return Err(failure(
            "watch requires an ODS workspace (or pass `--okf` for OKF watch)",
        ));
    }
    watch_workspace(&root, level, format, false)?;
    Ok(ExitCode::from(0))
}

fn run_serve_command(args: &[String]) -> Result<ExitCode, CliError> {
    // Headless loop for OS service (no interactive green spam).
    let options = serve_options_from_args(args)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&options.root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;
    if engines.okf && !engines.ods {
        return run_okf_watch_command(args, true);
    }
    if !engines.ods {
        return Err(failure(
            "serve requires an ODS workspace (or pass `--okf` for OKF serve)",
        ));
    }
    serve_workspace(options)?;
    Ok(ExitCode::from(0))
}

fn run_export_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, out, format, spec) = parse_export_args(args)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let want_okf = extra.okf || spec.starts_with("okf");

    if want_okf {
        if !detected.okf {
            return Err(failure(
                "not an OKF bundle: no root index.md with okf_version.\nRun `ods init --okf`.",
            ));
        }
        // JSON with --spec okf still uses ODS workspace renderer when ODS is present.
        if matches!(format, OutputFormat::Json) && detected.ods {
            // fall through to ODS JSON export with okf spec tag
        } else {
            return run_okf_export_command(args);
        }
    }

    if !detected.ods {
        if detected.okf {
            return run_okf_export_command(args);
        }
        return Err(failure(
            "export requires an ODS workspace (or pass `--okf` for OKF export)",
        ));
    }
    let include_private = args.iter().any(|a| a == "--include-private");

    match format {
        OutputFormat::Json => {
            let workspace = ods_core::load_workspace(&root).map_err(|e| failure(e.to_string()))?;
            let json_str = ods_core::render_graph_json(&workspace, include_private, &spec);
            println!("{json_str}");
        }
        OutputFormat::Text | OutputFormat::Sarif => {
            let path = export_workspace_graph(&root, &out, include_private)
                .map_err(|e| failure(e.to_string()))?;
            println!("wrote {}", path.display());
            if !include_private {
                println!("(documents marked share: private or share: org were omitted; pass --include-private to include them)");
            }
        }
    }
    Ok(ExitCode::from(0))
}

fn run_start_command(args: &[String]) -> Result<ExitCode, CliError> {
    let status_only = args.iter().any(|a| a == "--status");
    let (root, _level, _format) = parse_common_flags(args, 2)?;
    let root = resolve_root_path(root);
    require_ods_workspace(&root)?;
    if status_only {
        let st = service::service_status(&root);
        println!(
            "installed={} running={} ({})",
            st.installed, st.running, st.detail
        );
        return Ok(ExitCode::from(if st.running { 0 } else { 1 }));
    }
    let msg = service::start_service(&root).map_err(|e| failure(e.to_string()))?;
    println!("{msg}");
    Ok(ExitCode::from(0))
}

fn run_stop_command(args: &[String]) -> Result<ExitCode, CliError> {
    let unregister = args.iter().any(|a| a == "--unregister");
    let (root, _level, _format) = parse_common_flags(args, 2)?;
    let root = resolve_root_path(root);
    require_ods_workspace(&root)?;
    let msg =
        service::stop_service(&root, unregister).map_err(|e| failure(e.to_string()))?;
    println!("{msg}");
    Ok(ExitCode::from(0))
}
