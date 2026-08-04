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

fn run_profile_show_command(args: &[String]) -> Result<ExitCode, CliError> {
    let profile_name = args
        .get(3)
        .filter(|a| !a.starts_with('-'))
        .ok_or_else(|| {
            usage_msg(ods_core::missing_required_arg(
                "name",
                "ods profile show <name>",
            ))
        })?;
    let root = args
        .get(4)
        .filter(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = resolve_root_path(root);

    let workspace = load_workspace(&root).ok();
    let roots = workspace
        .as_ref()
        .map(|ws| {
            let idx = ws
                .document_by_path(&root.join("index.ods.md"))
                .or_else(|| ws.document_by_path(&root.join("index.md")));
            profile_catalog_roots(&root, idx)
        })
        .unwrap_or_else(|| profile_catalog_roots(&root, None));
    let catalog = load_profile_catalog(&root, &roots).map_err(|err| fail_io("profile", err))?;

    let def = catalog.definitions.get(profile_name.as_str()).ok_or_else(|| {
        fail_msg(ods_core::UserMsg::new(
            "unknown_profile",
            ods_core::ErrorStage::Resolve,
            format!("unknown profile: {profile_name}"),
        )
        .next("ods profiles  # list available profiles")
        .hint("ods profile init <name>  # scaffold + register a custom profile"))
    })?;

    let layer = if def.source.to_string_lossy().starts_with("<builtin:") {
        "standard"
    } else {
        "custom"
    };
    let sections: Vec<String> = def
        .sections
        .iter()
        .map(|g| g.join(" | "))
        .collect();
    println!("profile: {profile_name}");
    println!("  layer: {layer}");
    println!("  source: {}", def.source.display());
    if def.expected_keys.is_empty() {
        println!("  expected_keys: (none)");
    } else {
        println!("  expected_keys: {}", def.expected_keys.join(", "));
    }
    if sections.is_empty() {
        println!("  sections: (none)");
    } else {
        println!("  sections:");
        for s in sections {
            println!("    - {s}");
        }
    }
    Ok(ExitCode::from(0))
}

fn run_profile_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    // argv: ods profile init <name>  → name at index 3
    let profile_name = args
        .get(3)
        .filter(|a| !a.starts_with('-'))
        .or_else(|| {
            args.get(2)
                .filter(|a| a.as_str() != "init" && !a.starts_with('-'))
        })
        .ok_or_else(|| {
            usage_msg(ods_core::missing_required_arg(
                "name",
                "ods profile init <name> [--no-register]",
            ))
        })?;

    let no_register = args.iter().any(|a| a == "--no-register");
    let register = !no_register;

    // Optional root path after name: ods profile init rfc /path
    // Prefer first non-flag positional after name that looks like a path (not a flag).
    let root = args
        .iter()
        .skip(4)
        .find(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = resolve_root_path(root);
    let profiles_dir = root.join(".ods").join("profiles");
    fs::create_dir_all(&profiles_dir).map_err(|err| fail_io("profile", err))?;

    let file_path = profiles_dir.join(format!("{profile_name}.md"));
    let rel_register = format!(".ods/profiles/{profile_name}.md");
    let created = if file_path.exists() {
        println!(
            "profile definition already exists at {}",
            file_path.display()
        );
        false
    } else {
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
        println!(
            "scaffolded custom profile definition at {}",
            file_path.display()
        );
        true
    };

    if register {
        match register_custom_profile_in_root(&root, &rel_register) {
            Ok(RegisterResult::Registered(path)) => {
                println!("registered in {} under custom-profiles:", path.display());
                println!("  - {rel_register}");
            }
            Ok(RegisterResult::AlreadyRegistered(path)) => {
                println!(
                    "already registered in {} under custom-profiles:",
                    path.display()
                );
                println!("  - {rel_register}");
            }
            Ok(RegisterResult::NoRootIndex) => {
                println!(
                    "warning: no root index.ods.md / index.md — profile not registered"
                );
                println!("Next: ods init  then re-run: ods profile init {profile_name}");
            }
            Err(e) => return Err(e),
        }
    } else {
        println!(
            "skipped registration (--no-register). Add to root custom-profiles:\n  - {rel_register}"
        );
    }

    if created || register {
        println!("use in a document:");
        println!("  ods:");
        println!("    profile: {profile_name}");
        println!("    status: draft");
        println!("Next: ods lint");
    }

    Ok(ExitCode::from(0))
}

enum RegisterResult {
    Registered(PathBuf),
    AlreadyRegistered(PathBuf),
    NoRootIndex,
}

fn register_custom_profile_in_root(
    root: &Path,
    rel_entry: &str,
) -> Result<RegisterResult, CliError> {
    let index_path = if root.join("index.ods.md").is_file() {
        root.join("index.ods.md")
    } else if root.join("index.md").is_file() {
        root.join("index.md")
    } else {
        return Ok(RegisterResult::NoRootIndex);
    };

    let text = fs::read_to_string(&index_path).map_err(|e| fail_io("profile", e))?;
    if profile_entry_already_listed(&text, rel_entry) {
        return Ok(RegisterResult::AlreadyRegistered(index_path));
    }

    let updated = insert_custom_profile_into_root_index(&text, rel_entry);
    fs::write(&index_path, updated).map_err(|e| fail_io("profile", e))?;
    Ok(RegisterResult::Registered(index_path))
}

fn profile_entry_already_listed(text: &str, rel_entry: &str) -> bool {
    text.contains(&format!("- {rel_entry}"))
        || text.contains(&format!("- \"{rel_entry}\""))
        || text.contains(&format!("- '{rel_entry}'"))
}

fn insert_custom_profile_into_root_index(text: &str, rel_entry: &str) -> String {
    if text.contains("custom-profiles:") {
        return text.replace(
            "custom-profiles:",
            &format!("custom-profiles:\n  - {rel_entry}"),
        );
    }
    // Legacy key still accepted by parser.
    if text.contains("profiles:") {
        // Prefer adding canonical key rather than extending ambiguous legacy key when both absent of custom.
        // If only legacy `profiles:` exists, append under it for compatibility.
        return text.replace("profiles:", &format!("profiles:\n  - {rel_entry}"));
    }
    if text.starts_with("---\n") || text.starts_with("---\r\n") {
        return text.replacen(
            "---",
            &format!("---\ncustom-profiles:\n  - {rel_entry}"),
            1,
        );
    }
    format!("---\ncustom-profiles:\n  - {rel_entry}\n---\n\n{text}")
}

fn run_aliases_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    match sub {
        "--help" | "-h" => {
            println!(
                "ods aliases [list] [path]\n\
                 ods alias add <Canonical> <Synonym>\n\n\
                 Section-heading aliases for profile section matching (root index only).\n\
                 Standard profiles also ship builtin alternatives (e.g. Goal | Objective)."
            );
            Ok(ExitCode::from(0))
        }
        "add" => run_alias_add_command(args),
        "list" => run_aliases_list_command(args, 3),
        _ => run_aliases_list_command(args, 2),
    }
}

fn run_aliases_list_command(args: &[String], flag_start: usize) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, flag_start)?;
    let workspace = load_workspace(&root).map_err(|e| fail_load(&root, e))?;
    let aliases = workspace_aliases(&workspace);

    match format {
        OutputFormat::Text => {
            println!("section aliases (workspace root):");
            if aliases.is_empty() {
                println!("  (none declared — standard profile pipe-alternatives still apply)");
                println!("hint: ods alias add Goal Objective");
            } else {
                for (canonical, values) in &aliases {
                    let mut v: Vec<_> = values.iter().cloned().collect();
                    v.sort();
                    println!("  {canonical}: {}", v.join(", "));
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let mut items = Vec::new();
            for (canonical, values) in &aliases {
                let mut v: Vec<_> = values.iter().cloned().collect();
                v.sort();
                let vals: Vec<_> = v.iter().map(|s| json_escape(s)).collect();
                items.push(format!(
                    r#"{{"canonical":{},"aliases":[{}]}}"#,
                    json_escape(canonical),
                    vals.join(",")
                ));
            }
            println!("[{}]", items.join(","));
        }
    }
    Ok(ExitCode::from(0))
}

fn run_alias_add_command(args: &[String]) -> Result<ExitCode, CliError> {
    // ods alias add <Canonical> <Synonym> [root]
    let positionals: Vec<&String> = args
        .iter()
        .skip(3)
        .filter(|a| !a.starts_with('-'))
        .collect();
    let (canonical, synonym, root) = match positionals.as_slice() {
        [c, s] => (
            (*c).clone(),
            (*s).clone(),
            env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        ),
        [c, s, r] => ((*c).clone(), (*s).clone(), PathBuf::from(*r)),
        _ => {
            return Err(usage_msg(ods_core::missing_required_arg(
                "Canonical Synonym",
                "ods alias add <Canonical> <Synonym>",
            )));
        }
    };
    let root = resolve_root_path(root);
    let index_path = if root.join("index.ods.md").is_file() {
        root.join("index.ods.md")
    } else if root.join("index.md").is_file() {
        root.join("index.md")
    } else {
        return Err(fail_msg(ods_core::root_index_missing()));
    };

    let text = fs::read_to_string(&index_path).map_err(|e| fail_io("alias", e))?;
    let updated = insert_section_alias_into_root_index(&text, &canonical, &synonym);
    fs::write(&index_path, updated).map_err(|e| fail_io("alias", e))?;
    println!(
        "added section alias {canonical} ← {synonym} in {}",
        index_path.display()
    );
    Ok(ExitCode::from(0))
}

/// Insert or extend root `aliases:` map entry for section matching.
fn insert_section_alias_into_root_index(text: &str, canonical: &str, synonym: &str) -> String {
    // If synonym already present under any form, leave as-is (idempotent).
    if text.contains(&format!("- {synonym}"))
        && text.contains("aliases:")
        && text.contains(canonical)
    {
        // Best-effort: still rewrite carefully below if structure allows.
    }

    if !text.contains("aliases:") {
        if text.starts_with("---\n") || text.starts_with("---\r\n") {
            return text.replacen(
                "---",
                &format!("---\naliases:\n  {canonical}:\n    - {synonym}"),
                1,
            );
        }
        return format!("---\naliases:\n  {canonical}:\n    - {synonym}\n---\n\n{text}");
    }

    // aliases: exists — try to find `  Canonical:` block and append synonym.
    let mut lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
    let mut in_aliases = false;
    let mut aliases_indent = 0usize;
    let mut canonical_line: Option<usize> = None;
    let mut insert_at: Option<usize> = None;
    let mut synonym_present = false;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();
        if trimmed == "---" && i > 0 {
            break;
        }
        if !in_aliases {
            if trimmed == "aliases:" {
                in_aliases = true;
                aliases_indent = indent;
            }
            continue;
        }
        if indent <= aliases_indent && !trimmed.is_empty() && !trimmed.starts_with('#') {
            // left aliases map
            break;
        }
        if indent == aliases_indent + 2 && trimmed.starts_with(&format!("{canonical}:")) {
            canonical_line = Some(i);
            continue;
        }
        if let Some(ci) = canonical_line {
            if i > ci {
                if indent >= aliases_indent + 4
                    && (trimmed == format!("- {synonym}")
                        || trimmed == format!("- \"{synonym}\"")
                        || trimmed == format!("- '{synonym}'"))
                {
                    synonym_present = true;
                }
                if indent <= aliases_indent + 2 && i > ci && !trimmed.starts_with('-') {
                    insert_at = Some(i);
                    break;
                }
                insert_at = Some(i + 1);
            }
        }
    }

    if synonym_present {
        return text.to_string();
    }

    if let Some(ci) = canonical_line {
        let at = insert_at.unwrap_or(ci + 1);
        lines.insert(at, format!("    - {synonym}"));
        return lines.join("\n")
            + if text.ends_with('\n') { "\n" } else { "" };
    }

    // aliases: present but canonical missing — insert after aliases:
    if let Some(ai) = lines.iter().position(|l| l.trim() == "aliases:") {
        lines.insert(ai + 1, format!("  {canonical}:"));
        lines.insert(ai + 2, format!("    - {synonym}"));
        return lines.join("\n")
            + if text.ends_with('\n') { "\n" } else { "" };
    }

    text.to_string()
}

#[cfg(test)]
mod test_profile_commands {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_profile_list_and_show() {
        let res = run_profile_list_command(&["ods".into(), "profile".into(), "list".into()]);
        assert!(res.is_ok());

        let res = run_profile_list_command(&[
            "ods".into(),
            "profile".into(),
            "list".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_profile_show_command(&["ods".into(), "profile".into(), "show".into(), "note".into()]);
        assert!(res.is_ok());

        let res = run_profile_show_command(&[
            "ods".into(),
            "profile".into(),
            "show".into(),
            "note".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let err = run_profile_show_command(&["ods".into(), "profile".into(), "show".into()]).unwrap_err();
        assert!(err.message().contains("name"));

        let err = run_profile_show_command(&["ods".into(), "profile".into(), "show".into(), "nonexistent_xyz".into()]).unwrap_err();
        assert!(err.message().contains("unknown profile"));
    }

    #[test]
    fn test_profile_init() {
        let td = tempdir().unwrap();
        let root = td.path();

        let err = run_profile_init_command(&["ods".into(), "profile".into(), "init".into()]).unwrap_err();
        assert!(err.message().contains("name"));

        let res = run_profile_init_command(&[
            "ods".into(),
            "profile".into(),
            "init".into(),
            "custom-spec".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());

        let profile_file = root.join(".ods").join("profiles").join("custom-spec.md");
        assert!(profile_file.exists());

        // duplicate init
        let res = run_profile_init_command(&[
            "ods".into(),
            "profile".into(),
            "init".into(),
            "custom-spec".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());
    }

    #[test]
    fn test_alias_command_and_insert_alias() {
        let td = tempdir().unwrap();
        let root = td.path();

        // help/usage
        let res = run_aliases_command(&["ods".into(), "alias".into(), "--help".into()]);
        assert!(res.is_ok());

        // list
        let res = run_aliases_command(&["ods".into(), "alias".into(), "list".into()]);
        assert!(res.is_ok());

        let err = run_aliases_command(&["ods".into(), "alias".into(), "add".into(), "Overview".into()]).unwrap_err();
        assert!(err.message().contains("Synonym"));

        // create index.ods.md and add alias
        let index_path = root.join("index.ods.md");
        fs::write(&index_path, "---\nprofile: index\nods: 0.1\n---\n\n# Root\n").unwrap();

        let res = run_aliases_command(&[
            "ods".into(),
            "alias".into(),
            "add".into(),
            "Overview".into(),
            "Summary".into(),
            root.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("Overview"));
        assert!(content.contains("Summary"));
    }

    #[test]
    fn test_insert_section_alias_into_root_index_pure_helper() {
        let text_no_aliases = "---\nprofile: index\n---\n";
        let out = insert_section_alias_into_root_index(text_no_aliases, "Overview", "Summary");
        assert!(out.contains("aliases:"));
        assert!(out.contains("Overview:"));
        assert!(out.contains("Summary"));

        let text_with_aliases = "---\naliases:\n  Overview:\n    - Summary\n---\n";
        let out = insert_section_alias_into_root_index(text_with_aliases, "Overview", "Summary");
        assert_eq!(out, text_with_aliases);

        let text_with_different_canonical = "---\naliases:\n  Architecture:\n    - Design\n---\n";
        let out = insert_section_alias_into_root_index(text_with_different_canonical, "Overview", "Summary");
        assert!(out.contains("Overview:"));
        assert!(out.contains("Summary"));
    }
}


