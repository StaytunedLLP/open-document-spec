fn run_find_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "ods find [path] [--tag <name> ...] [<query>]\n\n\
             Find documents by tag and/or id/path/stem query.\n\n\
             Examples:\n\
               ods find --tag caching\n\
               ods find specs/ods/core\n\
               ods find core --tag note\n\
               ods find --format json specs/ods/keys\n"
        );
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let mut tags = Vec::new();
    let mut query: Option<String> = None;
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
            "--level" | "--format" | "--mode" | "--root" => i += 2,
            "--all" | "--write" | "--check" | "--force" | "--help" | "-h" => i += 1,
            other if other.starts_with('-') => {
                return Err(usage(format!("unknown find flag: {other}")));
            }
            other => {
                let p = PathBuf::from(other);
                // Skip workspace path positionals already consumed as root.
                let is_root_positional = p.is_dir()
                    && (resolve_root_path(p.clone()) == root
                        || p.canonicalize().ok().as_ref() == Some(&root));
                if !is_root_positional && query.is_none() {
                    query = Some(other.to_string());
                }
                i += 1;
            }
        }
    }

    if tags.is_empty() && query.is_none() {
        return Err(usage(
            "usage: ods find [path] [--tag <name> ...] [<id-or-path-query>]\n\
             Provide at least one --tag or a free-text query (id/path/stem).",
        ));
    }

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;

    let mut ids: Vec<String> = if tags.is_empty() {
        let mut all: Vec<String> = workspace.by_id.keys().cloned().collect();
        all.sort();
        all
    } else {
        docs_with_any_tag(&workspace, &tags)
    };

    if let Some(q) = query.as_deref() {
        let q_lc = q.trim().to_lowercase();
        let q_path = Path::new(q);
        ids.retain(|id| {
            if id == &q_lc || id.ends_with(&q_lc) || id.contains(&q_lc) {
                return true;
            }
            if let Some(doc) = workspace.document_by_id(id) {
                let lossy = doc.path.to_string_lossy().replace('\\', "/").to_lowercase();
                if lossy.ends_with(&q_lc)
                    || lossy.ends_with(&format!("{q_lc}.md"))
                    || doc.path.ends_with(q_path)
                    || doc
                        .path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .is_some_and(|s| s.eq_ignore_ascii_case(&q_lc))
                {
                    return true;
                }
            }
            false
        });
        // Unique context start fallback when string filter finds nothing.
        if ids.is_empty() {
            if let Some(start) = ods_core::resolve_context_start(&workspace, q) {
                if let Some(doc) = workspace.document_by_path(&start) {
                    let fm = match &doc.frontmatter {
                        FrontmatterState::Parsed(fm) => Some(fm),
                        _ => None,
                    };
                    ids.push(ods_core::document_id(&workspace.root, &doc.path, fm));
                }
            }
        }
    }

    match format {
        OutputFormat::Text => {
            for id in &ids {
                println!("{id}");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = ids.iter().map(|id| json_escape(id)).collect();
            let tag_items: Vec<_> = tags.iter().map(|t| json_escape(t)).collect();
            println!(
                r#"{{"tags":[{}],"query":{},"ids":[{}],"count":{}}}"#,
                tag_items.join(","),
                json_escape(query.as_deref().unwrap_or("")),
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

        let sample = std::path::Path::new("src/fixtures/ecommerce");
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
