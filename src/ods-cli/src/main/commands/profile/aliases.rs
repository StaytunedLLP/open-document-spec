fn run_aliases_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args.get(2).map(String::as_str).unwrap_or("list");
    match sub {
        "--help" | "-h" => {
            println!(
                "ods aliases [list] [path]\n\
                 ods alias add <Canonical> <Synonym>\n\n\
                 Section-heading aliases for profile section matching.\n\
                 Prefer root ods.toml [aliases]; legacy root index frontmatter still accepted.\n\
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

fn insert_alias_into_ods_toml(text: &str, canonical: &str, synonym: &str) -> String {
    if text.contains("[aliases]") {
        let target = format!("{canonical} = [");
        if text.contains(&target) {
            if let Some(idx) = text.find(&target) {
                let rest = &text[idx..];
                if let Some(end) = rest.find(']') {
                    let abs_end = idx + end;
                    return format!("{}\"{synonym}\", {}", &text[..abs_end], &text[abs_end..]);
                }
            }
        }
        let insert = format!("{canonical} = [\"{synonym}\"]\n");
        return text.replace("[aliases]", &format!("[aliases]\n{insert}"));
    }
    format!("{}\n\n[aliases]\n{canonical} = [\"{synonym}\"]\n", text.trim_end())
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
    let index_path = if root.join("ods.toml").is_file() {
        Some(root.join("ods.toml"))
    } else if root.join("index.ods.md").is_file() {
        Some(root.join("index.ods.md"))
    } else if root.join("index.md").is_file() {
        Some(root.join("index.md"))
    } else {
        return Err(fail_msg(ods_core::root_index_missing()));
    };

    let p = index_path.unwrap();
    let text = fs::read_to_string(&p).map_err(|e| fail_io("alias", e))?;
    let updated = if p.extension().is_some_and(|ext| ext == "toml") {
        insert_alias_into_ods_toml(&text, &canonical, &synonym)
    } else {
        insert_section_alias_into_root_index(&text, &canonical, &synonym)
    };
    fs::write(&p, updated).map_err(|e| fail_io("alias", e))?;
    println!("registered section alias: {canonical} -> {synonym}");
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
