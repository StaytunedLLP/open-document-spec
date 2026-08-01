fn run_tag_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args
        .get(2)
        .map(String::as_str)
        .ok_or_else(|| usage("usage: odc tag rename <old> <new> [--write] [path]"))?;
    match sub {
        "rename" => {
            let write = args.iter().any(|a| a == "--write");
            let mut format = OutputFormat::Text;
            // Parse flags without treating tag names as workspace path.
            let mut i = 3;
            let mut bare = Vec::new();
            while i < args.len() {
                match args[i].as_str() {
                    "--write" | "--all" => i += 1,
                    "--format" => {
                        let value = args
                            .get(i + 1)
                            .ok_or_else(|| usage("missing value for --format"))?;
                        format = match value.as_str() {
                            "text" => OutputFormat::Text,
                            "json" => OutputFormat::Json,
                            other => {
                                return Err(usage(format!(
                                    "invalid --format {other} (use text or json)"
                                )));
                            }
                        };
                        i += 2;
                    }
                    "--level" => i += 2,
                    flag if flag.starts_with('-') => {
                        return Err(usage(format!("unknown flag: {flag}")));
                    }
                    other => {
                        bare.push(other.to_string());
                        i += 1;
                    }
                }
            }
            let (root, from, to) = match bare.as_slice() {
                [from, to] => (
                    resolve_root_path(
                        env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
                    ),
                    from.clone(),
                    to.clone(),
                ),
                [maybe_root, from, to] if PathBuf::from(maybe_root).is_dir() => (
                    resolve_root_path(PathBuf::from(maybe_root)),
                    from.clone(),
                    to.clone(),
                ),
                _ => {
                    return Err(usage(
                        "usage: odc tag rename [path] <old> <new> [--write]",
                    ));
                }
            };
            let workspace =
                load_workspace(&root).map_err(|err| failure(err.to_string()))?;
            let report = rename_tag_in_workspace(&workspace, &from, &to, write)
                .map_err(|err| failure(err.to_string()))?;
            match format {
                OutputFormat::Text => {
                    let mode = if report.dry_run { "dry-run" } else { "wrote" };
                    println!(
                        "tag rename {} → {} ({mode}; {} doc(s), {} file(s))",
                        report.from,
                        report.to,
                        report.matched_docs,
                        report.rewritten_files.len()
                    );
                    for path in &report.rewritten_files {
                        if let Ok(rel) = path.strip_prefix(&root) {
                            println!("  {}", rel.display());
                        } else {
                            println!("  {}", path.display());
                        }
                    }
                    if report.dry_run && !report.rewritten_files.is_empty() {
                        println!("re-run with --write to apply");
                    }
                }
                OutputFormat::Json => {
                    let files: Vec<_> = report
                        .rewritten_files
                        .iter()
                        .map(|p| json_escape(&p.display().to_string()))
                        .collect();
                    println!(
                        r#"{{"from":{},"to":{},"dry_run":{},"matched_docs":{},"files":[{}]}}"#,
                        json_escape(&report.from),
                        json_escape(&report.to),
                        if report.dry_run { "true" } else { "false" },
                        report.matched_docs,
                        files.join(",")
                    );
                }
            }
            Ok(ExitCode::from(0))
        }
        other => Err(usage(format!(
            "unknown tag subcommand: {other} (try: rename)"
        ))),
    }
}

#[cfg(test)]
mod test_tag_command {
    use super::*;

    #[test]
    fn test_run_tag_command_errors() {
        let err1 = run_tag_command(&["ods".into(), "tag".into()]);
        assert!(err1.is_err());

        let err2 = run_tag_command(&["ods".into(), "tag".into(), "invalid".into()]);
        assert!(err2.is_err());

        let err3 = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            "--unknown".into(),
        ]);
        assert!(err3.is_err());

        let err4 = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            "--format".into(),
            "invalid".into(),
        ]);
        assert!(err4.is_err());

        let td = tempfile::tempdir().unwrap();
        std::fs::write(td.path().join("index.md"), "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n").unwrap();
        std::fs::write(td.path().join("doc.md"), "---\nprofile: note\ntags:\n  - oldtag\n---\n\n# D\n").unwrap();

        let res_txt = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            td.path().to_string_lossy().to_string(),
            "oldtag".into(),
            "newtag".into(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res_txt.is_ok());

        let res_json = run_tag_command(&[
            "ods".into(),
            "tag".into(),
            "rename".into(),
            td.path().to_string_lossy().to_string(),
            "oldtag".into(),
            "newtag".into(),
            "--write".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_json.is_ok());
    }
}

