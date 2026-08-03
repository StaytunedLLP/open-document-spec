fn run_undo_command(args: &[String]) -> Result<ExitCode, CliError> {
    let target = args.get(2).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let root = find_marked_ods_workspace_root(&target).unwrap_or(target);

    let report = ods_core::undo_latest_snapshot(&root).map_err(|err| failure(err.to_string()))?;
    println!("✓ Undid changes using snapshot {}", report.snapshot_id);
    println!("  Restored {} document frontmatter(s)", report.total_restored);
    if report.total_indexes_restored > 0 {
        println!("  Restored {} index file(s)", report.total_indexes_restored);
    }
    if report.total_profiles_restored > 0 {
        println!("  Restored {} profile definition(s)", report.total_profiles_restored);
    }
    Ok(ExitCode::from(0))
}
