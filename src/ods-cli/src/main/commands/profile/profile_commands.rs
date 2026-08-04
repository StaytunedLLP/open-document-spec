fn run_profile_list_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let workspace = load_workspace(&root).ok();
    let roots = workspace
        .as_ref()
        .map(|ws| profile_catalog_roots(&root, ws.document_by_path(&root.join("index.ods.md"))))
        .unwrap_or_else(|| profile_catalog_roots(&root, None));
    let catalog = load_profile_catalog(&root, &roots).map_err(|err| fail_io("profile", err))?;

    match format {
        OutputFormat::Text => {
            println!("profiles:");
            for (name, def) in &catalog.definitions {
                let kind = if def.source.to_string_lossy().starts_with("<builtin:") {
                    "[default ODS]"
                } else {
                    "[project]"
                };
                let section_summary: Vec<String> = def
                    .sections
                    .iter()
                    .filter_map(|g| g.first().cloned())
                    .collect();
                let source_path = def.source.to_string_lossy().replace('\\', "/");
                println!("{name}: {kind} ({}) — {source_path}", section_summary.join(", "));
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let mut list = Vec::new();
            for (name, def) in &catalog.definitions {
                let layer = if def.source.to_string_lossy().starts_with("<builtin:") {
                    "standard"
                } else {
                    "custom"
                };
                list.push(format!(
                    r#"{{"name":{},"layer":{},"source":{},"expected_keys":{:?}}}"#,
                    json_escape(name),
                    json_escape(layer),
                    json_escape(&def.source.to_string_lossy()),
                    def.expected_keys
                ));
            }
            println!("[{}]", list.join(","));
        }
    }

    Ok(ExitCode::from(0))
}

fn run_profile_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    // argv: ods profile init <name>  → name at index 3
    // (dispatch already matched subcommand "init" at index 2)
    let profile_name = args
        .get(3)
        .filter(|a| !a.starts_with('-'))
        .or_else(|| {
            // tolerate accidental `ods profile <name>` when routed here
            args.get(2)
                .filter(|a| a.as_str() != "init" && !a.starts_with('-'))
        })
        .ok_or_else(|| {
            usage_msg(ods_core::missing_required_arg("name", "ods profile init <name>"))
        })?;

    // Optional root path after name: ods profile init rfc /path
    let root = args
        .get(4)
        .filter(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let profiles_dir = root.join(".ods").join("profiles");
    fs::create_dir_all(&profiles_dir).map_err(|err| fail_io("profile", err))?;

    let file_path = profiles_dir.join(format!("{profile_name}.md"));
    if file_path.exists() {
        println!("profile definition already exists at {}", file_path.display());
        return Ok(ExitCode::from(0));
    }

    let template = format!(
        "---
name: {profile_name}
description: \"Custom profile definition for {profile_name}\"
expected_keys:
  - owner
ods:
  profile: custom-profile
  status: stable
---

# {profile_name} Profile

## Overview

## Specification

### Details

## Verification & Testing
"
    );

    fs::write(&file_path, template).map_err(|err| fail_io("profile", err))?;
    println!("scaffolded custom profile definition at {}", file_path.display());
    println!("remember to register it in root index.ods.md under custom-profiles:\n  - .ods/profiles/{profile_name}.md");

    Ok(ExitCode::from(0))
}
