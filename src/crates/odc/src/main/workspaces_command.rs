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



include!("workspaces_config.rs");

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

include!("workspaces_tests.rs");

