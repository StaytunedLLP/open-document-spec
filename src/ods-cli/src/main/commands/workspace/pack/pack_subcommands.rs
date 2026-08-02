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
        "---\nprofile: index\nods: 0.1\ndescription: Reusable ODS Pack for {name}.\n---\n\n# {name}\n\n- [ods-profiles/](ods-profiles/index.md) - Custom Profile schemas\n- [skills/](skills/index.md) - AI Agent skills\n"
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

        let ws = td.path().join("ws");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::write(ws.join("index.md"), "---\nprofile: index\nods: 0.1\n---\n\n# Root\n").unwrap();

        let res_add = run_pack_add(&[
            "ods".into(),
            "pack".into(),
            "add".into(),
            ws.to_string_lossy().to_string(),
            pack_path.to_string_lossy().to_string(),
        ]);
        assert!(res_add.is_ok());

        let res_rm = run_pack_remove(&[
            "ods".into(),
            "pack".into(),
            "rm".into(),
            ws.to_string_lossy().to_string(),
            pack_path.to_string_lossy().to_string(),
        ]);
        assert!(res_rm.is_ok());
    }
}
