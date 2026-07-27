fn run_share_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, scope, out, include_org, include_private) = parse_share_args(args)?;
    require_ods_workspace(&root)?;
    let workspace = load_workspace(&root).map_err(|e| failure(e.to_string()))?;
    let report = ods_core::publish_workspace(
        &workspace,
        &scope,
        &out,
        ods_core::ShareOptions {
            include_org,
            include_private,
        },
    )
    .map_err(|e| failure(e.to_string()))?;

    println!(
        "wrote {} document(s) to {}",
        report.written.len(),
        out.display()
    );
    if !report.excluded.is_empty() {
        println!(
            "({} document(s) excluded by share visibility; pass --include-org/--include-private to include more)",
            report.excluded.len()
        );
    }
    println!("(this only writes files; run git init/add/commit/push yourself to publish {})", out.display());
    Ok(ExitCode::from(0))
}
