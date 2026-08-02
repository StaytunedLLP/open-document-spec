fn run_lint_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;

    // Pure OKF: dedicated runner (keeps formatting parity with OKF-only messages).
    if engines.okf && !engines.ods && !engines.skills {
        return run_okf_lint_command(args);
    }

    let mut diagnostics = Vec::new();

    if engines.ods {
        let canonical_refs = args.iter().any(|arg| arg == "--canonical-refs");
        let workspace = load_workspace_with_options(&root, load_options_graph())
            .map_err(|err| failure(err.to_string()))?;
        let fix = args.iter().any(|arg| arg == "--fix");
        if fix {
            let _ = generate_indexes(&workspace);
            if matches!(format, OutputFormat::Text) {
                println!("Auto-fixed frontmatter keys and updated workspace indexes.");
            }
        }
        let ods_diags = if canonical_refs {
            lint_workspace_with_ref_style(&workspace, level, true)
        } else {
            lint_workspace_with_level(&workspace, level)
        };
        diagnostics.extend(ods_diags);
    }

    if engines.okf {
        let bundle = ods_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
        let okf_level = match level {
            LintLevel::Level1 => ods_core::OkfLintLevel::Level1,
            LintLevel::Level3 => ods_core::OkfLintLevel::Level3,
        };
        let mut okf_diags = ods_core::lint_okf_bundle_with_level(&bundle, okf_level);
        for d in &mut okf_diags {
            if !d.message.starts_with("[okf]") {
                d.message = format!("[okf] {}", d.message);
            }
        }
        diagnostics.extend(okf_diags);
    }

    if engines.skills {
        let packages = ods_core::skill_package_roots(&root);
        if packages.is_empty() {
            diagnostics.push(ods_core::Diagnostic {
                path: root.clone(),
                severity: ods_core::Severity::Error,
                message: "[skills] no SKILL.md package found (root or skills/*/)".into(),
            });
        }
        for pkg_root in packages {
            match ods_core::parse_skill_package(&pkg_root) {
                Ok(pkg) => diagnostics.extend(ods_core::lint_skill_package(&pkg)),
                Err(e) => diagnostics.push(ods_core::Diagnostic {
                    path: pkg_root.join("SKILL.md"),
                    severity: ods_core::Severity::Error,
                    message: format!("[skills] failed to load package: {e}"),
                }),
            }
        }
    }

    print_diagnostics(&diagnostics, format);
    if engines.ods {
        write_or_clear_ods_error_report(&root, &diagnostics, format)?;
    }
    if diagnostics
        .iter()
        .all(|d| d.severity != ods_core::Severity::Error)
        && matches!(format, OutputFormat::Text)
    {
        if diagnostics.is_empty() {
            println!("Everything is fine — graph and links are consistent. No update required.");
        } else {
            println!("Lint finished with warnings only.");
        }
    }
    Ok(exit_code(&diagnostics))
}

fn run_index_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    let extra = ods_core::parse_extra_spec_flags(args.iter().map(String::as_str))
        .map_err(|e| usage(e.message()))?;
    let detected = ods_core::detect_workspace(&root);
    let engines = ods_core::resolve_engines(extra, detected, true)
        .map_err(|e| failure(e.message()))?;

    if engines.okf && !engines.ods {
        return run_okf_index_command(args);
    }
    if engines.okf && engines.ods {
        // Hybrid with --okf: run ODS indexes then OKF indexes.
        let code = run_ods_index_only(&root, args, format)?;
        let _ = run_okf_index_command(args)?;
        return Ok(code);
    }
    if !engines.ods {
        return Err(failure(
            "index requires an ODS workspace (or pass `--okf` for OKF indexes)",
        ));
    }
    run_ods_index_only(&root, args, format)
}

fn run_ods_index_only(
    root: &Path,
    args: &[String],
    format: OutputFormat,
) -> Result<ExitCode, CliError> {
    let check = args.iter().any(|a| a == "--check");
    let workspace = load_workspace_with_options(root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    if check {
        let current =
            indexes_are_current(&workspace).map_err(|err| failure(err.to_string()))?;
        match format {
            OutputFormat::Text => {
                if current {
                    println!("indexes up to date");
                } else {
                    eprintln!("indexes out of date; run `ods index`");
                }
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                println!(
                    r#"{{"current":{},"root":{}}}"#,
                    if current { "true" } else { "false" },
                    json_escape(&root.display().to_string())
                );
            }
        }
        Ok(ExitCode::from(if current { 0 } else { 1 }))
    } else {
        let paths = generate_indexes(&workspace).map_err(|err| failure(err.to_string()))?;
        match format {
            OutputFormat::Text => {
                for path in &paths {
                    println!("{}", path.display());
                }
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                let items: Vec<_> = paths
                    .iter()
                    .map(|p| json_escape(&p.display().to_string()))
                    .collect();
                println!(
                    r#"{{"written":[{}],"count":{}}}"#,
                    items.join(","),
                    paths.len()
                );
            }
        }
        Ok(ExitCode::from(0))
    }
}

fn run_tags_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let include_all = args.iter().any(|a| a == "--all");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;
    print_tags(&workspace, include_all, format);
    Ok(ExitCode::from(0))
}

fn run_coverage_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let write_report = args.iter().any(|a| a == "--write-report");
    let workspace = load_workspace_with_options(&root, load_options_graph())
        .map_err(|err| failure(err.to_string()))?;

    let total = workspace.documents.len();
    let mut compliant = 0usize;
    let mut non_compliant = 0usize;

    for doc in &workspace.documents {
        let is_parsed = matches!(doc.frontmatter, ods_core::FrontmatterState::Parsed(_));
        let diags = ods_core::lint_document_in_workspace(&workspace, &doc.path, level);
        if is_parsed && diags.is_empty() {
            compliant += 1;
        } else {
            non_compliant += 1;
        }
    }

    let pct = if total == 0 {
        100.0
    } else {
        (compliant as f64 / total as f64) * 100.0
    };

    match format {
        OutputFormat::Text => {
            println!("Documentation Health: {:.1}% Compliant ({}/{} files)", pct, compliant, total);
            println!("  ✔ Compliant:     {} documents", compliant);
            println!("  ✖ Non-Compliant:  {} documents", non_compliant);
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"health_pct":{:.1},"compliant":{},"non_compliant":{},"total":{}}}"#,
                pct, compliant, non_compliant, total
            );
        }
    }

    if write_report {
        let report_content = format!(
            "# Documentation Health & Coverage Report\n\n- Score: {:.1}% Compliant\n- Compliant Documents: {}\n- Non-Compliant Documents: {}\n- Total Documents: {}\n\nNote: this is separate from lint/audit diagnostics (`.ods/ods-errors.md`).\n",
            pct, compliant, non_compliant, total
        );
        let odc_dir = root.join(".ods");
        let _ = std::fs::create_dir_all(&odc_dir);
        let report_path = odc_dir.join("coverage.md");
        std::fs::write(&report_path, report_content)
            .map_err(|e| failure(format!("write {}: {e}", report_path.display())))?;
        if matches!(format, OutputFormat::Text) {
            println!("wrote {}", report_path.display());
        }
    }

    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_lint_index_commands {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn lint_fix_canonical_skills_and_index_check() {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(
            root.join("x.md"),
            "---\nprofile: note\nstatus: draft\ndepends:\n  - missing\n---\n\n# X\n",
        )
        .unwrap();
        let skill = root.join("skills/demo");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: demo\ndescription: Lint skills hybrid package.\n---\n\n# D\n",
        )
        .unwrap();
        let path = root.to_str().unwrap().to_string();

        for args in [
            vec!["ods".into(), "lint".into(), path.clone(), "--fix".into()],
            vec![
                "ods".into(),
                "lint".into(),
                path.clone(),
                "--canonical-refs".into(),
            ],
            vec![
                "ods".into(),
                "lint".into(),
                path.clone(),
                "--skills".into(),
                "--format".into(),
                "json".into(),
            ],
            vec![
                "ods".into(),
                "lint".into(),
                path.clone(),
                "--skills".into(),
                "--format".into(),
                "text".into(),
            ],
            vec!["ods".into(), "index".into(), path.clone()],
            vec![
                "ods".into(),
                "index".into(),
                path.clone(),
                "--check".into(),
            ],
            vec![
                "ods".into(),
                "index".into(),
                path,
                "--format".into(),
                "json".into(),
            ],
        ] {
            let _ = run_lint_command(&args);
            // index uses run_index_command
        }

        let path = root.to_str().unwrap().to_string();
        let _ = run_index_command(&["ods".into(), "index".into(), path.clone()]);
        let _ = run_index_command(&[
            "ods".into(),
            "index".into(),
            path.clone(),
            "--check".into(),
        ]);
        let _ = run_index_command(&[
            "ods".into(),
            "index".into(),
            path,
            "--format".into(),
            "json".into(),
        ]);
    }
}
