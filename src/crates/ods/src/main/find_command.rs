fn run_find_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let mut tags = Vec::new();
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--tag" => {
                let v = args
                    .get(i + 1)
                    .ok_or_else(|| usage("missing value for --tag"))?;
                tags.push(v.clone());
                i += 2;
            }
            "--level" | "--format" => {
                // consumed by parse_common_flags via path walk; skip value
                i += 2;
            }
            "--all" | "--write" | "--check" | "--force" => i += 1,
            other if other.starts_with('-') => {
                return Err(usage(format!("unknown find flag: {other}")));
            }
            _ => i += 1, // path positional
        }
    }
    if tags.is_empty() {
        return Err(usage(
            "usage: ods find [path] --tag <name> [--tag <name> ...]  (OR match)",
        ));
    }
    let workspace = load_workspace(&root).map_err(|err| failure(err.to_string()))?;
    let ids = docs_with_any_tag(&workspace, &tags);
    match format {
        OutputFormat::Text => {
            for id in &ids {
                println!("{id}");
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = ids.iter().map(|id| json_escape(id)).collect();
            println!(
                r#"{{"tags":[{}],"ids":[{}],"count":{}}}"#,
                tags.iter()
                    .map(|t| json_escape(t))
                    .collect::<Vec<_>>()
                    .join(","),
                items.join(","),
                ids.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_find_command {
    use super::*;

    #[test]
    fn test_run_find_command_errors() {
        let err1 = run_find_command(&["ods".into(), "find".into(), "--unknown".into()]);
        assert!(err1.is_err());

        let err2 = run_find_command(&["ods".into(), "find".into(), "--tag".into()]);
        assert!(err2.is_err());

        let sample = std::path::Path::new("ods-test/ecommerce");
        if sample.exists() {
            let res = run_find_command(&[
                "ods".into(),
                "find".into(),
                sample.to_str().unwrap().into(),
                "--tag".into(),
                "auth".into(),
                "--format".into(),
                "json".into(),
            ]);
            assert!(res.is_ok());
        }
    }
}

