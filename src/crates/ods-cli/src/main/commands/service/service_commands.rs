fn run_doctor_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let report = doctor_workspace(&root)?;
    match format {
        OutputFormat::Text => println!("{}", report.text),
        OutputFormat::Json => println!("{}", report.json),
    }
    Ok(ExitCode::from(if report.has_error { 1 } else { 0 }))
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
    require_ods_workspace(&root)?;
    watch_workspace(&root, level, format, false)?;
    Ok(ExitCode::from(0))
}

fn run_serve_command(args: &[String]) -> Result<ExitCode, CliError> {
    // Headless loop for OS service (no interactive green spam).
    let options = serve_options_from_args(args)?;
    require_ods_workspace(&options.root)?;
    serve_workspace(options)?;
    Ok(ExitCode::from(0))
}

fn run_export_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, out) = parse_export_args(args)?;
    require_ods_workspace(&root)?;
    let include_private = args.iter().any(|a| a == "--include-private");
    let path = export_workspace_graph(&root, &out, include_private)
        .map_err(|e| failure(e.to_string()))?;
    println!("wrote {}", path.display());
    if !include_private {
        println!("(documents marked share: private or share: org were omitted; pass --include-private to include them)");
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
