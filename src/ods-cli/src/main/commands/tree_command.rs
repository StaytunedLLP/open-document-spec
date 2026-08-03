fn run_tree_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;

    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;

    match format {
        OutputFormat::Text => {
            println!("ODS Workspace Tree: {}", root.display());
            println!("└── index.ods.md (root index)");
            let root_index_path = root.join("index.ods.md");

            let docs_by_dir = workspace
                .documents
                .iter()
                .filter(|doc| doc.path != root_index_path)
                .fold(
                    std::collections::BTreeMap::<PathBuf, Vec<PathBuf>>::new(),
                    |mut map, doc| {
                        let relative = doc.path.strip_prefix(&root).unwrap_or(&doc.path);
                        let parent = relative.parent().map(|p| p.to_path_buf()).unwrap_or_default();
                        map.entry(parent).or_default().push(doc.path.clone());
                        map
                    },
                );

            for (dir, docs) in &docs_by_dir {
                if dir.as_os_str().is_empty() {
                    for (i, doc_path) in docs.iter().enumerate() {
                        let is_last = i == docs.len() - 1 && docs_by_dir.len() == 1;
                        let prefix = if is_last { "└── " } else { "├── " };
                        let rel = doc_path.strip_prefix(&root).unwrap_or(doc_path);
                        println!("{}{}", prefix, rel.display());
                    }
                } else {
                    println!("├── {}/", dir.display());
                    for (i, doc_path) in docs.iter().enumerate() {
                        let is_last = i == docs.len() - 1;
                        let prefix = if is_last { "│   └── " } else { "│   ├── " };
                        let file_name = doc_path.file_name().unwrap_or(doc_path.as_os_str());
                        println!("{}{}", prefix, Path::new(file_name).display());
                    }
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let docs: Vec<String> = workspace
                .documents
                .iter()
                .map(|d| format!(r#""{}""#, d.path.strip_prefix(&root).unwrap_or(&d.path).display()))
                .collect();
            println!(r#"{{"root":"{}","tree":[{}]}}"#, root.display(), docs.join(","));
        }
    }

    Ok(ExitCode::from(0))
}
