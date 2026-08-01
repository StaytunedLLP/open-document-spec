/// Forward workspace/machine cutover helper (not dual-compat).
/// Dry-run by default; `--write` applies safe machine steps + optional FM migrate.
fn run_upgrade_command(args: &[String]) -> Result<ExitCode, CliError> {
    let write = args.iter().any(|a| a == "--write");
    let check = args.iter().any(|a| a == "--check");
    let migrate_fm = args.iter().any(|a| a == "--migrate-fm");
    let (root, _level, format) = parse_common_flags(args, 2)?;

    let mut actions: Vec<String> = Vec::new();
    let mut pending = 0usize;

    // Detect ODS / OKF roots
    let ods = ods_core::ods_enabled(&root);
    let okf = ods_core::okf_enabled(&root);
    if ods {
        actions.push(format!("ODS workspace detected at {}", root.display()));
    }
    if okf {
        actions.push(format!("OKF bundle detected at {}", root.display()));
    }
    if !ods && !okf {
        actions.push(format!(
            "no ODS/OKF root markers under {} (run ods ods init or ods okf init)",
            root.display()
        ));
        pending += 1;
    }

    // Config dir forward hint ~/.ods
    let home = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE"));
    if let Some(home) = home {
        let legacy = PathBuf::from(&home).join(".ods");
        let modern = PathBuf::from(&home).join(".ods");
        if legacy.exists() && !modern.exists() {
            actions.push(format!(
                "machine: legacy config {} present; prefer {}",
                legacy.display(),
                modern.display()
            ));
            pending += 1;
            if write {
                // Best-effort copy registry if present
                let _ = fs::create_dir_all(&modern);
                for name in ["odsconfig.toml", "workspaces.toml"] {
                    let src = legacy.join(name);
                    if src.exists() {
                        let dst_name = if name == "odsconfig.toml" {
                            "odcconfig.toml"
                        } else {
                            name
                        };
                        let dst = modern.join(dst_name);
                        if !dst.exists() {
                            let _ = fs::copy(&src, &dst);
                            actions.push(format!("  copied {} -> {}", src.display(), dst.display()));
                        }
                    }
                }
            }
        } else if modern.exists() {
            actions.push(format!("machine: config dir {} ok", modern.display()));
        }
    }

    if ods {
        actions.push(
            "manual: review root index.md ods: / ods: pins if needed (~3 known repos)"
                .into(),
        );
        actions.push("next: ods ods audit --write-report".into());
    }
    if okf {
        actions.push("next: ods okf lint && ods okf audit --write-report".into());
    }



    if migrate_fm && ods {
        if write {
            let workspace =
                load_workspace(&root).map_err(|err| failure(err.to_string()))?;
            let changed = migrate_workspace_frontmatter_with_workspace(&workspace)
                .map_err(|err| failure(err.to_string()))?;
            actions.push(format!(
                "migrated canonical ods: layout in {} file(s)",
                changed.len()
            ));
        } else {
            actions.push(
                "would run fmt --migrate for canonical nested ods: keys (pass --write)".into(),
            );
            pending += 1;
        }
    }

    match format {
        OutputFormat::Text => {
            println!(
                "ods upgrade {} — {}",
                if write { "--write" } else { "dry-run" },
                root.display()
            );
            for a in &actions {
                println!("  • {a}");
            }
            if !write && pending > 0 {
                println!("pending actions: {pending} (re-run with --write to apply safe steps)");
            } else if write {
                println!("upgrade pass complete");
            } else {
                println!("nothing required");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"write":{},"pending":{},"ods":{},"okf":{}}}"#,
                write, pending, ods, okf
            );
        }
    }

    if check && pending > 0 {
        return Ok(ExitCode::from(1));
    }
    Ok(ExitCode::from(0))
}

fn run_ods_audit_command(args: &[String]) -> Result<ExitCode, CliError> {
    let write_report = args.iter().any(|a| a == "--write-report");
    let mut report_path_opt = None;
    let mut fail_on = None;
    let mut filtered = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write-report" => {}
            "--report-path" => {
                report_path_opt = args.get(i + 1).map(PathBuf::from);
                i += 1;
            }
            "--fail-on" => {
                fail_on = args.get(i + 1).map(|s| s.as_str());
                i += 1;
            }
            other => filtered.push(other.to_string()),
        }
        i += 1;
    }
    let (root, _level, format) = parse_common_flags(&filtered, 2)?;
    require_ods_workspace(&root)?;
    let report_path = report_path_opt.unwrap_or_else(|| root.join(".ods/ods-errors.md"));

    let workspace = load_workspace(&root).map_err(|err| failure(err.to_string()))?;
    let mut plain = 0usize;
    let mut invalid = 0usize;
    let mut partial = 0usize;
    let mut compliant = 0usize;
    let mut lines: Vec<String> = Vec::new();

    for doc in &workspace.documents {
        let rel = doc
            .path
            .strip_prefix(&root)
            .unwrap_or(&doc.path)
            .display()
            .to_string();
        match &doc.frontmatter {
            FrontmatterState::Absent => {
                plain += 1;
                lines.push(format!("- `{rel}` — no frontmatter"));
            }
            FrontmatterState::Invalid(err) => {
                invalid += 1;
                lines.push(format!("- `{rel}` — {err}"));
            }
            FrontmatterState::Parsed(fm) => {
                // root index with ods: counts as compliant shape for audit inventory
                let has_profile = fm.profile.as_deref().map(|p| !p.is_empty()).unwrap_or(false);
                if doc.path == root.join("index.md") {
                    compliant += 1;
                } else if !has_profile {
                    partial += 1;
                    lines.push(format!("- `{rel}` — missing profile"));
                } else {
                    compliant += 1;
                }
            }
        }
    }
    let total = plain + invalid + partial + compliant;

    match format {
        OutputFormat::Text => {
            println!(
                "ODS audit: total={total} compliant={compliant} plain={plain} invalid={invalid} partial={partial}"
            );
            for l in &lines {
                // only non-compliant already in lines
                println!("  {l}");
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"total_md":{total},"compliant":{compliant},"plain":{plain},"invalid":{invalid},"partial":{partial}}}"#
            );
        }
    }

    if write_report {
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).map_err(|e| failure(e.to_string()))?;
        }
        let mut md = String::new();
        md.push_str("---\ngenerated_by: ods ods audit\n");
        md.push_str(&format!("workspace: {}\n", root.display()));
        md.push_str(&format!(
            "summary:\n  total_md: {total}\n  compliant: {compliant}\n  plain: {plain}\n  invalid: {invalid}\n  partial: {partial}\n---\n\n"
        ));
        md.push_str("# ODS ODS Audit Report\n\n## Non-compliant\n\n");
        if lines.is_empty() {
            md.push_str("_None._\n");
        } else {
            for l in &lines {
                md.push_str(l);
                md.push('\n');
            }
        }
        md.push_str("\n## Suggested next commands\n\n```bash\nodc ods adopt --write\nodc ods fmt --migrate\nodc ods lint\n```\n");
        fs::write(&report_path, md).map_err(|e| failure(e.to_string()))?;
        if matches!(format, OutputFormat::Text) {
            println!("wrote {}", report_path.display());
        }
    }

    let fail = match fail_on {
        None => false,
        Some("plain") => plain > 0,
        Some("invalid") => invalid > 0,
        Some("any") => plain + invalid + partial > 0,
        Some(other) => {
            return Err(usage(format!(
                "invalid --fail-on {other} (use plain|invalid|any)"
            )));
        }
    };
    Ok(ExitCode::from(if fail { 1 } else { 0 }))
}

include!("agents_command.rs");
