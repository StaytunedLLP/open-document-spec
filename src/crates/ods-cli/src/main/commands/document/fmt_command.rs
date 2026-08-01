fn run_fmt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let refs_mode = parse_refs_mode(args)?;
    let migrate = wants_migrate(args);
    let workspace = load_workspace(&root).map_err(|err| failure(err.to_string()))?;

    let mut actions: Vec<&str> = vec!["frontmatter spacing"];
    let mut changed = normalize_workspace_frontmatter_spacing_with_workspace(&workspace)
        .map_err(|err| failure(err.to_string()))?;

    if refs_mode == Some("md-paths") {
        actions.push("document refs");
        for path in canonicalize_workspace_document_refs_with_workspace(&workspace)
            .map_err(|err| failure(err.to_string()))?
        {
            if !changed.iter().any(|existing| existing == &path) {
                changed.push(path);
            }
        }
    }

    if migrate {
        actions.push("ods: key layout");
        for path in migrate_workspace_frontmatter_with_workspace(&workspace)
            .map_err(|err| failure(err.to_string()))?
        {
            if !changed.iter().any(|existing| existing == &path) {
                changed.push(path);
            }
        }
    }

    changed.sort();
    changed.dedup();

    match format {
        OutputFormat::Text => {
            if changed.is_empty() {
                println!("{} already clean", actions.join("/"));
            } else {
                println!(
                    "formatted {} in {} file(s)",
                    actions.join("/"),
                    changed.len()
                );
                for path in &changed {
                    if let Ok(rel) = path.strip_prefix(&root) {
                        println!("  {}", rel.display());
                    } else {
                        println!("  {}", path.display());
                    }
                }
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = changed
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            println!(
                r#"{{"changed":[{}],"count":{}}}"#,
                items.join(","),
                changed.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn parse_refs_mode(args: &[String]) -> Result<Option<&'static str>, CliError> {
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--refs" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| usage("fmt --refs requires md-paths"))?;
                return match value.as_str() {
                    "md-paths" => Ok(Some("md-paths")),
                    other => Err(usage(format!(
                        "invalid fmt --refs {other} (use md-paths)"
                    ))),
                };
            }
            _ => i += 1,
        }
    }
    Ok(None)
}

/// `--migrate`: also rewrite legacy flat/out-of-order `ods:` engine keys into
/// the canonical nested block. Opt-in — unlike spacing/refs normalization,
/// this relocates whole key blocks and is a bigger change to review.
fn wants_migrate(args: &[String]) -> bool {
    args[2..].iter().any(|arg| arg == "--migrate")
}
