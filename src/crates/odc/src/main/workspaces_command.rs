// Global machine configuration (TOML) and `odc workspaces` command.
//
// Registry: ~/.odc/odcconfig.toml (legacy: ~/.ods/odsconfig.toml, ~/.ods/config.toml)
// Parsed manually to avoid adding external serde/toml crate dependencies.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackEntry {
    pub workspace: String,
    pub name: String,
    pub path: String,
    pub source: String,
    pub auto_update: String, // "hourly", "daily", "weekly", "never"
    pub last_updated: String,
}

/// Path to the global machine configuration file.
/// Prefer `~/.odc/odcconfig.toml`; fall back to legacy `~/.ods/odsconfig.toml`.
pub fn registry_path() -> PathBuf {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let modern = PathBuf::from(&home).join(".odc/odcconfig.toml");
    if modern.exists() {
        return modern;
    }
    let legacy_primary = PathBuf::from(&home).join(".ods/odsconfig.toml");
    if legacy_primary.exists() {
        return legacy_primary;
    }
    let legacy_fallback = PathBuf::from(&home).join(".ods/config.toml");
    if legacy_fallback.exists() {
        return legacy_fallback;
    }
    // Default write path going forward
    modern
}

/// Load registered workspace paths from machine config (or legacy workspaces.toml).
pub fn load_registry_paths() -> Vec<String> {
    let path = registry_path();
    if let Ok(content) = fs::read_to_string(&path) {
        return parse_workspace_paths(&content);
    }

    // Check legacy ~/.ods/workspaces.toml for automatic migration
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let legacy_path = PathBuf::from(home).join(".ods/workspaces.toml");
    if let Ok(content) = fs::read_to_string(&legacy_path) {
        let paths = parse_workspace_paths(&content);
        if !paths.is_empty() {
            let _ = save_registry_paths(&paths);
        }
        return paths;
    }

    Vec::new()
}

/// Load registered pack entries from machine config.
pub fn load_registered_packs() -> Vec<PackEntry> {
    let path = registry_path();
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };
    parse_pack_entries(&content)
}

fn parse_pack_entries(content: &str) -> Vec<PackEntry> {
    let mut entries = Vec::new();
    let mut current_workspace = String::new();
    let mut current_name = String::new();
    let mut current_path = String::new();
    let mut current_source = String::new();
    let mut current_auto_update = String::from("daily");
    let mut current_last_updated = String::new();
    let mut in_pack_block = false;

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == "[[pack]]" {
            if in_pack_block && !current_name.is_empty() {
                entries.push(PackEntry {
                    workspace: current_workspace.clone(),
                    name: current_name.clone(),
                    path: current_path.clone(),
                    source: current_source.clone(),
                    auto_update: current_auto_update.clone(),
                    last_updated: current_last_updated.clone(),
                });
            }
            in_pack_block = true;
            current_workspace.clear();
            current_name.clear();
            current_path.clear();
            current_source.clear();
            current_auto_update = String::from("daily");
            current_last_updated.clear();
            continue;
        }

        if in_pack_block && trimmed.starts_with('[') && trimmed.ends_with(']') {
            if !current_name.is_empty() {
                entries.push(PackEntry {
                    workspace: current_workspace.clone(),
                    name: current_name.clone(),
                    path: current_path.clone(),
                    source: current_source.clone(),
                    auto_update: current_auto_update.clone(),
                    last_updated: current_last_updated.clone(),
                });
            }
            in_pack_block = false;
        }

        if in_pack_block
            && let Some((key, val)) = trimmed.split_once('=')
        {
            let key = key.trim();
            let val = unquote_str(val.trim());
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

    if in_pack_block && !current_name.is_empty() {
        entries.push(PackEntry {
            workspace: current_workspace,
            name: current_name,
            path: current_path,
            source: current_source,
            auto_update: current_auto_update,
            last_updated: current_last_updated,
        });
    }

    entries
}

pub(crate) fn save_pack_entry(entry: PackEntry) -> Result<(), CliError> {
    let mut packs = load_registered_packs();
    packs.retain(|p| !(p.workspace == entry.workspace && p.name == entry.name));
    packs.push(entry);
    save_config_with_packs(&load_registry_paths(), &packs)
}

/// Helper to check if an asset is due for auto-update based on frequency.
pub fn should_auto_update(last_updated: &str, frequency: &str) -> bool {
    if frequency == "never" || frequency == "off" {
        return false;
    }

    if last_updated.is_empty() {
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

/// Parse workspace paths from TOML content supporting both `[workspaces] paths = [...]` and legacy `[[workspace]] path = "..."`.
fn parse_workspace_paths(content: &str) -> Vec<String> {
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

        // Support [workspaces] paths = ["/a", "/b"]
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

        // Support legacy [[workspace]] path = "/some/path"
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

/// Write the registry file (prefer ~/.odc/odcconfig.toml).
pub(crate) fn save_registry_paths(paths: &[String]) -> Result<(), CliError> {
    let packs = load_registered_packs();
    save_config_with_packs(paths, &packs)
}

fn save_config_with_packs(paths: &[String], packs: &[PackEntry]) -> Result<(), CliError> {
    // Always write modern path when creating/updating (migrate off legacy).
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
        .map_err(|e| failure(format!("failed to write machine config: {e}")))?;
    Ok(())
}

/// Check if a path is inside a registered workspace from the global registry.
fn is_registered_workspace(root: &Path) -> bool {
    let paths = load_registry_paths();
    let abs_root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    for ws in &paths {
        let ws_path = PathBuf::from(ws);
        let abs_ws = fs::canonicalize(&ws_path).unwrap_or(ws_path);
        if abs_root.starts_with(&abs_ws) {
            return true;
        }
    }
    false
}

/// Guard: ensures the given root is a valid ODS workspace (has marker or is registered).
fn require_ods_workspace(root: &Path) -> Result<(), CliError> {
    if odc_core::ods_enabled(root) {
        return Ok(());
    }

    if is_registered_workspace(root) {
        return Ok(());
    }

    Err(failure(format!(
        "not an ODS workspace: {}\n\n\
         No root index.md with 'ods:' marker found, and this path is not\n\
         registered in the global machine config (~/.odc/odcconfig.toml).\n\n\
         To fix:\n\
         • Run 'odc init' here to make this folder ODS-compliant, or\n\
         • Run 'odc workspaces add' to track it globally without modifying files.",
        root.display()
    )))
}

fn run_workspaces_command(args: &[String]) -> Result<ExitCode, CliError> {
    let subcommand = args.get(2).map(String::as_str).unwrap_or("list");

    match subcommand {
        "--help" | "-h" | "help" => {
            println!(
                "odc workspaces <subcommand>\n\n\
                 Manage globally tracked ODS workspaces.\n\
                 Config file: ~/.odc/odcconfig.toml (legacy ~/.ods/odsconfig.toml is read)\n\n\
                 Subcommands:\n\
                 \x20 add [path]     Register a folder as an ODS workspace (default: current dir)\n\
                 \x20 remove [path]  Unregister a folder (default: current dir)\n\
                 \x20 list           List all registered workspaces\n\
                 \x20 path           Print the machine config file path"
            );
            Ok(ExitCode::from(0))
        }
        "add" => {
            let raw_path = args
                .get(3)
                .map(PathBuf::from)
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let abs_path = fs::canonicalize(&raw_path)
                .map_err(|e| failure(format!("invalid path '{}': {e}", raw_path.display())))?;
            let path_str = abs_path.to_string_lossy().into_owned();

            let mut paths = load_registry_paths();
            if paths.contains(&path_str) {
                println!("{} is already tracked", abs_path.display());
            } else {
                paths.push(path_str);
                save_registry_paths(&paths)?;
                println!("added {} to tracked ODS workspaces", abs_path.display());
                println!(
                    "config: {}",
                    registry_path().display()
                );
            }
            Ok(ExitCode::from(0))
        }
        "remove" => {
            let raw_path = args
                .get(3)
                .map(PathBuf::from)
                .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let abs_path = fs::canonicalize(&raw_path)
                .map_err(|e| failure(format!("invalid path '{}': {e}", raw_path.display())))?;
            let path_str = abs_path.to_string_lossy().into_owned();

            let mut paths = load_registry_paths();
            if let Some(pos) = paths.iter().position(|w| w == &path_str) {
                paths.remove(pos);
                save_registry_paths(&paths)?;
                println!("removed {} from tracked ODS workspaces", abs_path.display());
            } else {
                println!("{} is not currently tracked", abs_path.display());
            }
            Ok(ExitCode::from(0))
        }
        "list" => {
            let paths = load_registry_paths();
            let packs = load_registered_packs();
            if paths.is_empty() && packs.is_empty() {
                println!("no tracked ODS workspaces or packs");
                println!(
                    "run 'ods workspaces add [path]' to register a workspace"
                );
            } else {
                println!("tracked ODS workspaces ({}):", paths.len());
                for ws in &paths {
                    let marker = if odc_core::ods_enabled(Path::new(ws)) {
                        "✓"
                    } else {
                        "○"
                    };
                    println!("  {marker} {ws}");
                }
                if !packs.is_empty() {
                    println!("\ntracked ODS packs ({}):", packs.len());
                    for p in &packs {
                        println!("  • {} (source: {}, auto_update: {})", p.name, p.source, p.auto_update);
                    }
                }
                println!();
                println!("✓ = has root index.md with ods: marker");
                println!("○ = registered but no local ods: marker");
            }
            Ok(ExitCode::from(0))
        }
        "path" => {
            println!("{}", registry_path().display());
            Ok(ExitCode::from(0))
        }
        other => Err(usage(format!(
            "unknown workspaces subcommand: {other} (use add, remove, list, or path)"
        ))),
    }
}

#[cfg(test)]
mod test_workspaces_command {
    use super::*;

    #[test]
    fn test_run_workspaces_command_subcommands() {
        assert!(run_workspaces_command(&["ods".into(), "workspaces".into(), "help".into()]).is_ok());
        assert!(run_workspaces_command(&["ods".into(), "workspaces".into(), "path".into()]).is_ok());
        assert!(run_workspaces_command(&["ods".into(), "workspaces".into(), "list".into()]).is_ok());

        let err = run_workspaces_command(&["ods".into(), "workspaces".into(), "unknown".into()]);
        assert!(err.is_err());

        let td = tempfile::tempdir().unwrap();
        let sample = td.path().join("ws");
        std::fs::create_dir_all(&sample).unwrap();
        std::fs::write(sample.join("index.md"), "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n").unwrap();

        let res_list_txt = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "list".into(),
        ]);
        assert!(res_list_txt.is_ok());

        let res_list_json = run_workspaces_command(&[
            "ods".into(),
            "workspaces".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res_list_json.is_ok());
    }
}

