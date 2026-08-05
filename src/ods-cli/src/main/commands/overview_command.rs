fn run_overview_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "ods overview [path] [--format text|json]\n\n\
             Generate a compact workspace snapshot for AI cold-start orientation.\n\
             Alias: ods summary\n\n\
             Outputs total documents, profile/status breakdown, top tags, custom keys, and graph stats.\n\
             For lint health %, use `ods stats` instead.\n"
        );
        return Ok(ExitCode::from(0));
    }

    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| fail_load(&root, err))?;

    let total_docs = workspace.documents.len();
    let mut profiles_count = std::collections::BTreeMap::<String, usize>::new();
    let mut status_count = std::collections::BTreeMap::<String, usize>::new();
    let mut custom_keys_seen = std::collections::BTreeSet::<String>::new();
    let mut total_depends = 0usize;
    let mut total_related = 0usize;
    let mut unparsed = 0usize;

    for doc in &workspace.documents {
        match &doc.frontmatter {
            FrontmatterState::Parsed(fm) => {
                let prof = fm.profile.as_deref().unwrap_or("note");
                *profiles_count.entry(prof.to_string()).or_insert(0) += 1;

                let st = fm.status.as_deref().unwrap_or("unspecified");
                *status_count.entry(st.to_string()).or_insert(0) += 1;

                for ck in fm.custom_keys.keys() {
                    custom_keys_seen.insert(ck.clone());
                }

                total_depends += fm.depends.len();
                total_related += fm.related.len();
            }
            _ => {
                unparsed += 1;
            }
        }
    }

    let top_tags = ods_core::tag_usage(&workspace);
    let custom_keys_list: Vec<String> = custom_keys_seen.into_iter().collect();

    match format {
        OutputFormat::Text => {
            println!("ODS Workspace Overview: {}", root.display());
            println!("==================================================");
            println!("  Total Documents:       {}", total_docs);
            println!(
                "  Graph Edges:           {} depends, {} related",
                total_depends, total_related
            );
            println!("  Unique Tags:           {}", top_tags.len());
            println!("  Custom Schema Keys:    {}", custom_keys_list.len());
            if unparsed > 0 {
                println!("  Unparsed/Plain Docs:   {}", unparsed);
            }

            println!("\nProfiles:");
            for (p, c) in &profiles_count {
                println!("  - {:<18} {}", p, c);
            }

            println!("\nStatus:");
            for (s, c) in &status_count {
                println!("  - {:<18} {}", s, c);
            }

            if !top_tags.is_empty() {
                println!("\nTop Tags:");
                for (t, c) in top_tags.iter().take(8) {
                    println!("  - #{:<17} {}", t, c);
                }
            }

            if !custom_keys_list.is_empty() {
                println!("\nObserved Custom Keys:");
                println!("  - {}", custom_keys_list.join(", "));
            }

            println!("\nSuggested next: ods find --key status=draft  |  ods tag list  |  ods schema keys");
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let tags_obj: serde_json::Map<String, serde_json::Value> = top_tags
                .iter()
                .take(10)
                .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                .collect();
            let profiles_obj: serde_json::Map<String, serde_json::Value> = profiles_count
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                .collect();
            let status_obj: serde_json::Map<String, serde_json::Value> = status_count
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::json!(v)))
                .collect();
            let payload = serde_json::json!({
                "root": root.display().to_string(),
                "total_documents": total_docs,
                "depends_count": total_depends,
                "related_count": total_related,
                "unparsed": unparsed,
                "profiles": profiles_obj,
                "status": status_obj,
                "tags": tags_obj,
                "custom_keys": custom_keys_list,
            });
            println!("{}", serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into()));
        }
    }

    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_overview_command {
    use super::*;

    #[test]
    fn test_overview_command_help() {
        let res = run_overview_command(&["ods".into(), "overview".into(), "--help".into()]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_overview_command_text_and_json() {
        // Prefer cargo-workspace-relative fixture, fall back to monorepo-root path.
        let sample = ["fixtures/ecommerce", "src/fixtures/ecommerce"]
            .into_iter()
            .map(std::path::Path::new)
            .find(|p| p.exists());
        if let Some(sample) = sample {
            let res_txt = run_overview_command(&[
                "ods".into(),
                "overview".into(),
                sample.to_str().unwrap().into(),
                "--format".into(),
                "text".into(),
            ]);
            assert!(res_txt.is_ok());

            let res_json = run_overview_command(&[
                "ods".into(),
                "overview".into(),
                sample.to_str().unwrap().into(),
                "--format".into(),
                "json".into(),
            ]);
            assert!(res_json.is_ok());

            let res_alias = run_overview_command(&[
                "ods".into(),
                "summary".into(),
                sample.to_str().unwrap().into(),
            ]);
            assert!(res_alias.is_ok());
        }
    }

    #[test]
    fn test_overview_temp_workspace_with_custom_keys() {
        let td = tempfile::tempdir().unwrap();
        std::fs::write(
            td.path().join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("a.md"),
            "---\nprofile: note\nstatus: draft\nteam: infra\ntags:\n  - t1\ndepends:\n  - b.md\n---\n\n# A\n",
        )
        .unwrap();
        std::fs::write(
            td.path().join("b.md"),
            "---\nprofile: note\nstatus: stable\n---\n\n# B\n",
        )
        .unwrap();
        std::fs::write(td.path().join("plain.md"), "# Plain\n").unwrap();
        let root = td.path().to_string_lossy().to_string();
        let res = run_overview_command(&[
            "ods".into(),
            "overview".into(),
            root.clone(),
            "--format".into(),
            "text".into(),
        ]);
        assert!(res.is_ok());
        let res_json = run_overview_command(&[
            "ods".into(),
            "overview".into(),
            root,
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_json.is_ok());
    }
}
