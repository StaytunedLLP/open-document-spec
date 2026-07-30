pub(crate) fn pack_update_due(last_updated: &str, frequency: &str) -> bool {
    if frequency == "never" || last_updated.is_empty() {
        return true;
    }
    let Ok(updated_sec) = parse_iso_timestamp(last_updated) else {
        return true;
    };
    let now_sec = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now_sec < updated_sec {
        return false;
    }
    let elapsed = now_sec - updated_sec;
    match frequency {
        "hourly" => elapsed >= 3600,
        "daily" => elapsed >= 86400,
        "weekly" => elapsed >= 604800,
        _ => elapsed >= 86400,
    }
}

pub fn current_iso_timestamp() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{now}")
}

fn parse_iso_timestamp(text: &str) -> Result<u64, ()> {
    text.trim().parse::<u64>().map_err(|_| ())
}



pub(crate) fn get_config_path() -> Result<PathBuf, CliError> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| failure("could not determine home directory"))?;

    let modern = PathBuf::from(&home).join(".odc/odcconfig.toml");
    let legacy = PathBuf::from(&home).join(".ods/odsconfig.toml");

    if modern.exists() || !legacy.exists() {
        Ok(modern)
    } else {
        Ok(legacy)
    }
}

pub(crate) fn load_registered_paths() -> Vec<String> {
    let Ok(reg_path) = get_config_path() else {
        return Vec::new();
    };
    if !reg_path.exists() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&reg_path) else {
        return Vec::new();
    };
    parse_config_paths(&content)
}

pub fn parse_workspace_paths(content: &str) -> Vec<String> {
    parse_config_paths(content)
}

pub(crate) fn parse_config_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_workspaces_section = false;
    let mut collecting_paths_array = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[workspaces]" {
            in_workspaces_section = true;
            collecting_paths_array = false;
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed != "[[workspace]]" {
            in_workspaces_section = false;
            collecting_paths_array = false;
        }

        if (in_workspaces_section || collecting_paths_array)
            && (trimmed.starts_with("paths = [") || collecting_paths_array || trimmed.starts_with("paths="))
        {
            collecting_paths_array = !trimmed.contains(']');
            for part in trimmed.split(',') {
                let cleaned = part
                    .trim()
                    .trim_start_matches("paths = [")
                    .trim_start_matches("paths=[")
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim();
                let unquoted = unquote_str(cleaned);
                if !unquoted.is_empty() && unquoted != "paths" && !unquoted.starts_with('#') {
                    paths.push(unquoted);
                }
            }
        }

        if let Some(rest) = trimmed.strip_prefix("path") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let unquoted = unquote_str(rest.trim());
                if !unquoted.is_empty() {
                    paths.push(unquoted);
                }
            }
        }
    }

    paths.sort();
    paths.dedup();
    paths
}

fn unquote_str(text: &str) -> String {
    let trimmed = text.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2)
        || (trimmed.starts_with('\'') && trimmed.ends_with('\'') && trimmed.len() >= 2)
    {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

pub(crate) fn save_registry_paths(paths: &[String]) -> Result<(), CliError> {
    let packs = load_registered_packs();
    save_config_with_packs(paths, &packs)
}

pub(crate) fn save_config_with_packs(paths: &[String], packs: &[PackEntry]) -> Result<(), CliError> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let reg_path = PathBuf::from(&home).join(".odc/odcconfig.toml");
    if let Some(parent) = reg_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| failure(format!("failed to create registry directory: {e}")))?;
    }
    let mut content = String::from(
        "# OpenDocify global machine configuration (~/.odc/odcconfig.toml)\n\
         # Managed by `odc workspaces` and `odc pack`. You can also edit this file directly.\n\n\
         [workspaces]\n\
         paths = [\n",
    );
    for path in paths {
        content.push_str(&format!("  \"{path}\",\n"));
    }
    content.push_str("]\n\n");

    for pack in packs {
        content.push_str("[[pack]]\n");
        content.push_str(&format!("workspace = \"{}\"\n", pack.workspace));
        content.push_str(&format!("name = \"{}\"\n", pack.name));
        content.push_str(&format!("path = \"{}\"\n", pack.path));
        content.push_str(&format!("source = \"{}\"\n", pack.source));
        content.push_str(&format!("auto_update = \"{}\"\n", pack.auto_update));
        content.push_str(&format!("last_updated = \"{}\"\n\n", pack.last_updated));
    }

    fs::write(&reg_path, content)
        .map_err(|e| failure(format!("failed to save config file: {e}")))
}

pub(crate) fn load_registered_packs() -> Vec<PackEntry> {
    let Ok(reg_path) = get_config_path() else {
        return Vec::new();
    };
    if !reg_path.exists() {
        return Vec::new();
    }
    let Ok(content) = fs::read_to_string(&reg_path) else {
        return Vec::new();
    };
    parse_config_packs(&content)
}

pub(crate) fn parse_config_packs(content: &str) -> Vec<PackEntry> {
    let mut packs = Vec::new();
    let mut current_workspace = String::new();
    let mut current_name = String::new();
    let mut current_path = String::new();
    let mut current_source = String::new();
    let mut current_auto_update = String::from("true");
    let mut current_last_updated = String::new();
    let mut in_pack = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "[[pack]]" {
            if in_pack && !current_name.is_empty() {
                packs.push(PackEntry {
                    workspace: current_workspace.clone(),
                    name: current_name.clone(),
                    path: current_path.clone(),
                    source: current_source.clone(),
                    auto_update: current_auto_update.clone(),
                    last_updated: current_last_updated.clone(),
                });
            }
            in_pack = true;
            current_workspace.clear();
            current_name.clear();
            current_path.clear();
            current_source.clear();
            current_auto_update = "true".into();
            current_last_updated.clear();
            continue;
        }

        if in_pack {
            if let Some((k, v)) = trimmed.split_once('=') {
                let key = k.trim();
                let val = unquote_str(v.trim());
                match key {
                    "workspace" => current_workspace = val,
                    "name" => current_name = val,
                    "path" => current_path = val,
                    "source" => current_source = val,
                    "auto_update" => current_auto_update = val,
                    "last_updated" => current_last_updated = val,
                    _ => {}
                }
            }
        }
    }

    if in_pack && !current_name.is_empty() {
        packs.push(PackEntry {
            workspace: current_workspace,
            name: current_name,
            path: current_path,
            source: current_source,
            auto_update: current_auto_update,
            last_updated: current_last_updated,
        });
    }

    packs
}
