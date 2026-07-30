use odc_core::{
    NewDocumentOptions, RemoveDocumentOptions, atomic_delete_document, document_id,
    scaffold_new_document,
};

fn run_new_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.len() < 3 {
        return Err(usage("ods new <path> [--profile <p>] [--title \"<t>\"]"));
    }

    let mut target_path = None;
    let mut profile = None;
    let mut title = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--profile" | "-p" => {
                let v = args.get(i + 1).ok_or_else(|| usage("--profile requires a profile name"))?;
                profile = Some(v.clone());
                i += 2;
            }
            "--title" | "-t" => {
                let v = args.get(i + 1).ok_or_else(|| usage("--title requires a title string"))?;
                title = Some(v.clone());
                i += 2;
            }
            other if !other.starts_with('-') => {
                if target_path.is_none() {
                    target_path = Some(PathBuf::from(other));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    let Some(path) = target_path else {
        return Err(usage("ods new requires a file path (e.g. ods new docs/guides/oauth.md)"));
    };

    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let report = scaffold_new_document(&root, &path, NewDocumentOptions { profile, title })
        .map_err(|e| failure(format!("failed to scaffold document: {e}")))?;

    println!(
        "created document {}\n  id: {}\n  profile: {}\n  indexes updated: {}",
        report.created_file.display(),
        report.doc_id,
        report.profile,
        report.updated_indexes.len()
    );

    Ok(ExitCode::from(0))
}

fn run_rm_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.len() < 3 {
        return Err(usage("ods rm <path-or-id>"));
    }

    let target = PathBuf::from(&args[2]);
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    let report = atomic_delete_document(&root, &target, RemoveDocumentOptions { scrub_dependencies: true })
        .map_err(|e| failure(format!("failed to delete document: {e}")))?;

    println!(
        "deleted document {}\n  id: {}\n  cleaned graph references: {}\n  indexes updated: {}",
        report.deleted_file.display(),
        report.doc_id,
        report.cleaned_references_count,
        report.updated_indexes.len()
    );

    Ok(ExitCode::from(0))
}

fn run_archive_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.len() < 3 {
        return Err(usage("ods archive <path-or-id>"));
    }

    let target = PathBuf::from(&args[2]);
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace = load_workspace(&root).map_err(|e| failure(format!("load workspace error: {e}")))?;

    let target_abs = if target.is_absolute() {
        target.clone()
    } else {
        root.join(&target)
    };
    let target_canon = target_abs.canonicalize().ok();
    let target_stem = target.file_stem().map(|s| s.to_string_lossy().to_lowercase());

    let target_id_str = target.to_string_lossy().to_lowercase();

    let doc = workspace
        .documents
        .iter()
        .find(|d| {
            let did = document_id(
                &root,
                &d.path,
                match &d.frontmatter {
                    FrontmatterState::Parsed(fm) => Some(fm),
                    _ => None,
                },
            );
            d.path == target_abs
                || target_canon.as_ref() == Some(&d.path)
                || (target_canon.is_some() && d.path.canonicalize().ok() == target_canon)
                || did == target_id_str
                || target_stem.as_deref() == Some(did.as_str())
        })
        .ok_or_else(|| failure(format!("document not found: {}", target.display())))?;

    let doc_id = document_id(&root, &doc.path, match &doc.frontmatter {
        FrontmatterState::Parsed(fm) => Some(fm),
        _ => None,
    });

    let text = fs::read_to_string(&doc.path).map_err(|e| failure(format!("read error: {e}")))?;
    let (fm_opt, body) = odc_core::split_frontmatter(&text);

    let new_text = if let Some(fm) = fm_opt {
        let mut lines: Vec<String> = fm.lines().map(|s| s.to_string()).collect();
        let mut found_status = false;
        for line in &mut lines {
            if line.trim().starts_with("status:") {
                *line = "status: archived".to_string();
                found_status = true;
                break;
            }
        }
        if !found_status {
            lines.push("status: archived".to_string());
        }
        format!("---\n{}\n---\n\n{}", lines.join("\n"), body.trim_start())
    } else {
        format!("---\nstatus: archived\n---\n\n{}", text.trim_start())
    };

    fs::write(&doc.path, new_text).map_err(|e| failure(format!("write error: {e}")))?;

    println!("archived document {}\n  id: {}\n  status: archived", doc.path.display(), doc_id);
    Ok(ExitCode::from(0))
}

fn run_logs_command(args: &[String]) -> Result<ExitCode, CliError> {
    let follow = args.iter().any(|a| a == "-f" || a == "--follow");
    println!("streaming ods serve logs (follow: {follow})...");
    run_watch_command(args)
}
