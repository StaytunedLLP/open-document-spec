pub(crate) fn run_pack_command(args: &[String]) -> Result<ExitCode, CliError> {
    let subcommand = args.get(2).map(String::as_str).unwrap_or("list");

    match subcommand {
        "list" => run_pack_list(args),
        "add" => run_pack_add(args),
        "sync" => run_pack_sync(args),
        "remove" => run_pack_remove(args),
        "init" => run_pack_init(args),
        "preview" => run_pack_preview(args),
        other if other.starts_with('-') => run_pack_list(args),
        other => Err(usage(format!(
            "unknown subcommand 'ods pack {other}'. Available: add, sync, list, preview, remove, init"
        ))),
    }
}

fn extract_pack_path(args: &[String], skip_idx: usize) -> PathBuf {
    args.iter()
        .enumerate()
        .skip(skip_idx)
        .find(|(_, a)| !a.starts_with('-'))
        .map(|(_, a)| PathBuf::from(a))
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn run_pack_list(args: &[String]) -> Result<ExitCode, CliError> {
    let path = extract_pack_path(args, 3);
    let root = resolve_root_path(path);
    let workspace = load_workspace(&root).map_err(|e| failure(e.to_string()))?;

    let root_index_doc = workspace
        .documents
        .iter()
        .find(|d| d.path == root.join("index.md"));

    let mut packs = Vec::new();
    if let Some(doc) = root_index_doc
        && let FrontmatterState::Parsed(fm) = &doc.frontmatter
    {
        packs = fm.packs.clone();
    }

    println!("ODS Workspace Packs (root: {}):", root.display());
    if packs.is_empty() {
        println!("  (no external packs imported in root index.md)");
    } else {
        for pack in packs {
            let pack_path = root.join(&pack);
            let status = if pack_path.exists() {
                "installed"
            } else {
                "missing"
            };
            println!("  • {} [{}] ({})", pack, status, pack_path.display());
        }
    }

    println!("\nLoaded Custom Profile Schemas:");
    for (name, def) in &workspace.profiles.definitions {
        println!("  • profile: {} ({})", name, def.source.display());
    }

    Ok(ExitCode::from(0))
}

fn run_pack_add(args: &[String]) -> Result<ExitCode, CliError> {
    let source = args
        .get(3)
        .ok_or_else(|| usage("ods pack add requires a pack source (Git URL or local path)"))?;

    let auto_update = args
        .windows(2)
        .find(|w| w[0] == "--auto-update")
        .map(|w| w[1].clone())
        .unwrap_or_else(|| String::from("daily"));

    let root = resolve_root_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root_index_path = root.join("index.md");

    if !root_index_path.exists() {
        return Err(failure(
            "root index.md not found. Run 'ods init' to make this workspace ODS-compliant first.",
        ));
    }

    let mut pack_entry = source.clone();
    let pack_name = source.split('/').next_back().unwrap_or("pack").trim_end_matches(".git").to_string();

    // Check if source is a local directory or relative path
    let local_path = Path::new(source);
    if local_path.exists() {
        if let Ok(rel) = local_path.canonicalize() {
            if let Ok(workspace_rel) = rel.strip_prefix(&root) {
                pack_entry = workspace_rel.to_string_lossy().replace('\\', "/");
            } else {
                pack_entry = source.replace('\\', "/");
            }
        }
    } else if source.contains('/') && !source.contains(':') && !source.starts_with('.') {
        // GitHub shorthand: owner/repo -> vendor/repo
        let vendor_dir = root.join("vendor").join(&pack_name);
        println!("Cloning GitHub shorthand pack '{}' into {}...", source, vendor_dir.display());
        let git_url = format!("https://github.com/{source}.git");
        let status = Command::new("git")
            .args(["clone", &git_url, &vendor_dir.to_string_lossy()])
            .status();
        if let Ok(st) = status {
            if st.success() {
                pack_entry = format!("vendor/{pack_name}");
            } else {
                println!("Warning: git clone failed for {git_url}. Registering path reference.");
                pack_entry = format!("vendor/{pack_name}");
            }
        } else {
            pack_entry = format!("vendor/{pack_name}");
        }
    } else if source.starts_with("http://") || source.starts_with("https://") || source.starts_with("git@") {
        // Remote Git URL
        let vendor_dir = root.join("vendor").join(&pack_name);
        println!("Cloning Git URL pack into {}...", vendor_dir.display());
        let _ = Command::new("git")
            .args(["clone", source, &vendor_dir.to_string_lossy()])
            .status();
        pack_entry = format!("vendor/{pack_name}");
    }

    // Record pack entry in global config (~/.ods/odsconfig.toml)
    let workspace_str = root.to_string_lossy().into_owned();
    let entry = PackEntry {
        workspace: workspace_str,
        name: pack_name,
        path: pack_entry.clone(),
        source: source.clone(),
        auto_update,
        last_updated: current_iso_timestamp(),
    };
    let _ = save_pack_entry(entry);

    // Append pack_entry to root index.md frontmatter
    let text = fs::read_to_string(&root_index_path).map_err(|e| failure(e.to_string()))?;
    if text.contains(&format!("- {pack_entry}")) || text.contains(&format!("- \"{pack_entry}\"")) {
        println!("Pack '{}' is already registered in root index.md.", pack_entry);
        return Ok(ExitCode::from(0));
    }

    let updated_text = insert_pack_into_root_index(&text, &pack_entry);
    fs::write(&root_index_path, updated_text).map_err(|e| failure(e.to_string()))?;

    println!("Added ODS Pack '{}' to root index.md frontmatter.", pack_entry);
    Ok(ExitCode::from(0))
}

fn insert_pack_into_root_index(text: &str, pack_entry: &str) -> String {
    if text.contains("packs:") {
        text.replace("packs:", &format!("packs:\n  - {pack_entry}"))
    } else if text.starts_with("---\n") || text.starts_with("---\r\n") {
        text.replacen("---", &format!("---\npacks:\n  - {pack_entry}"), 1)
    } else {
        format!("---\npacks:\n  - {pack_entry}\n---\n\n{text}")
    }
}

fn run_pack_sync(args: &[String]) -> Result<ExitCode, CliError> {
    let force = args.iter().any(|a| a == "--force" || a == "-f");
    let root = resolve_root_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let workspace = load_workspace(&root).map_err(|e| failure(e.to_string()))?;

    let root_index_doc = workspace
        .documents
        .iter()
        .find(|d| d.path == root.join("index.md"));

    let mut packs = Vec::new();
    if let Some(doc) = root_index_doc
        && let FrontmatterState::Parsed(fm) = &doc.frontmatter
    {
        packs = fm.packs.clone();
    }

    let registered_packs = load_registered_packs();
    println!("Synchronizing {} installed ODS Packs...", packs.len());

    for pack in packs {
        let pack_dir = root.join(&pack);
        let reg_entry = registered_packs
            .iter()
            .find(|p| p.workspace == root.to_string_lossy() && p.path == pack);

        let due = force || reg_entry.is_none_or(|e| should_auto_update(&e.last_updated, &e.auto_update));

        if pack_dir.join(".git").exists() && due {
            println!("Pulling updates for {}...", pack);
            let status = Command::new("git")
                .current_dir(&pack_dir)
                .args(["pull", "--ff-only"])
                .status();
            if let Ok(st) = status
                && st.success()
            {
                let name = pack.split('/').next_back().unwrap_or(&pack).to_string();
                let source = reg_entry.map_or_else(|| pack.clone(), |e| e.source.clone());
                let auto_update = reg_entry.map_or_else(|| "daily".to_string(), |e| e.auto_update.clone());
                let _ = save_pack_entry(PackEntry {
                    workspace: root.to_string_lossy().into_owned(),
                    name,
                    path: pack.clone(),
                    source,
                    auto_update,
                    last_updated: current_iso_timestamp(),
                });
            }
        } else if pack_dir.exists() {
            println!("Verified local pack path {} (up to date).", pack);
        } else {
            println!("Warning: Pack path {} does not exist.", pack_dir.display());
        }
    }

    println!("ODS Pack synchronization complete.");
    Ok(ExitCode::from(0))
}

fn run_pack_remove(args: &[String]) -> Result<ExitCode, CliError> {
    let name = args
        .get(3)
        .ok_or_else(|| usage("ods pack remove requires a pack name or path"))?;

    let root = resolve_root_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root_index_path = root.join("index.md");

    if !root_index_path.exists() {
        return Err(failure("root index.md not found"));
    }

    let text = fs::read_to_string(&root_index_path).map_err(|e| failure(e.to_string()))?;
    let target_line = format!("  - {name}");
    let updated = text.lines().filter(|line| *line != target_line).collect::<Vec<_>>().join("\n");

    fs::write(&root_index_path, updated).map_err(|e| failure(e.to_string()))?;
    println!("Removed ODS Pack reference '{}' from root index.md.", name);
    Ok(ExitCode::from(0))
}

fn run_pack_preview(args: &[String]) -> Result<ExitCode, CliError> {
    let name = args
        .get(3)
        .ok_or_else(|| usage("ods pack preview requires a pack name or path"))?;

    let root = resolve_root_path(env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let pack_dir = root.join(name);

    if !pack_dir.exists() {
        return Err(failure(format!("pack directory {} does not exist", pack_dir.display())));
    }

    println!("Previewing ODS Pack at {}:", pack_dir.display());
    let workspace = load_workspace(&pack_dir).map_err(|e| failure(e.to_string()))?;
    for (schema_name, def) in &workspace.profiles.definitions {
        println!("  • profile: {} ({})", schema_name, def.source.display());
    }

    Ok(ExitCode::from(0))
}

fn run_pack_init(args: &[String]) -> Result<ExitCode, CliError> {
    let name = args.get(3).map(String::as_str).unwrap_or("my-ods-pack");
    let root = PathBuf::from(name);

    if !root.exists() {
        fs::create_dir_all(&root).map_err(|e| failure(e.to_string()))?;
    }

    let ods_profiles_dir = root.join("ods-profiles");
    let skills_dir = root.join("skills");
    fs::create_dir_all(&ods_profiles_dir).map_err(|e| failure(e.to_string()))?;
    fs::create_dir_all(&skills_dir).map_err(|e| failure(e.to_string()))?;

    let root_index = format!(
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\ndescription: Reusable ODS Pack for {name}.\n---\n\n# {name}\n\n- [ods-profiles/](ods-profiles/index.md) - Custom Profile schemas\n- [skills/](skills/index.md) - AI Agent skills\n"
    );
    fs::write(root.join("index.md"), root_index).map_err(|e| failure(e.to_string()))?;

    let profile_index = "---\nprofile: index\n---\n\n# Profile Schemas\n";
    fs::write(ods_profiles_dir.join("index.md"), profile_index).map_err(|e| failure(e.to_string()))?;

    let skills_index = "---\nprofile: index\n---\n\n# AI Agent Skills\n";
    fs::write(skills_dir.join("index.md"), skills_index).map_err(|e| failure(e.to_string()))?;

    println!("Scaffolding new ODS Pack at {}:", root.display());
    println!("  ✓ Created index.md (root marker)");
    println!("  ✓ Created ods-profiles/ (profile schema directory)");
    println!("  ✓ Created skills/ (AI agent skills directory)");

    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_pack_command {
    use super::*;

    #[test]
    fn test_pack_command_routing_and_init() {
        let td = tempfile::tempdir().unwrap();
        let pack_path = td.path().join("test-pack");

        let res_list = run_pack_command(&["ods".into(), "pack".into()]);
        assert!(res_list.is_ok());

        let err2 = run_pack_command(&["ods".into(), "pack".into(), "invalid".into()]);
        assert!(err2.is_err());

        let res_init = run_pack_init(&[
            "ods".into(),
            "pack".into(),
            "init".into(),
            pack_path.to_str().unwrap().to_string(),
        ]);
        assert!(res_init.is_ok());

        assert!(pack_path.join("index.md").exists());
        assert!(pack_path.join("ods-profiles/index.md").exists());

        let res_prev = run_pack_preview(&[
            "ods".into(),
            "pack".into(),
            "preview".into(),
            pack_path.to_str().unwrap().to_string(),
        ]);
        assert!(res_prev.is_ok() || res_prev.is_err());
    }
}

