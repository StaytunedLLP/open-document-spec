fn print_okf_help() {
    println!(
        "ods okf <command> [path] [flags]

Native Google OKF v0.2 commands (knowledge bundles).

Commands:
  init [path]              Scaffold OKF bundle (okf_version: \"0.2\" + sample concept)
  init --attested [path]   Also write Attested Computation stub
  init --log [path]        Also write log.md
  lint [path]              Validate OKF concepts (type required; full shape at level 3)
  index [path]             Generate progressive-disclosure index.md files
  index --check [path]     Exit 1 if indexes are stale
  context [path] <id>      Reading list: concept + markdown link targets
  export [path]            Write okf-graph.md (--out PATH)
  fmt [path]               Normalize frontmatter trailing whitespace
  doctor [path]            Bundle health: version, stale counts, trust tiers
  audit [path]             Classify concepts (plain/invalid/partial/compliant)
  audit --write-report     Write .ods/ods-errors.md
  adopt [path]             Report plain .md files (dry-run)
  adopt --write [path]     Draft minimal type/title frontmatter for plain files
  watch [path]             Re-lint OKF bundle on file changes (foreground)
  serve [path]             Headless re-lint loop (poll; for OS service use)

Flags:
  --level 1|3              Lint level (default 3)
  --format text|json       Output format
  --out PATH               With export: output path (default okf-graph.md)
  --write-report           With audit: write report file
  --report-path <file>     Custom audit report path
  --fail-on plain|invalid|any  CI gate for audit

See also: ods ods <cmd> for ODS workspaces; ods update for binary updates.
"
    );
}

fn require_okf_bundle(root: &Path) -> Result<(), CliError> {
    if ods_core::okf_enabled(root) {
        return Ok(());
    }
    Err(failure(format!(
        "not an OKF bundle: {}\n\n\
         No root index.md with okf_version found.\n\
         Run `ods okf init` here to create an OKF v0.2 bundle.",
        root.display()
    )))
}

fn run_okf_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut opts = ods_core::OkfInitOptions::default();
    let mut filtered: Vec<String> = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--attested" => opts.write_attested_stub = true,
            "--log" => opts.write_log = true,
            other => filtered.push(other.to_string()),
        }
        i += 1;
    }
    let (root, _level, format) = parse_common_flags(&filtered, 2)?;
    let report =
        ods_core::init_okf_bundle(&root, opts).map_err(|e| failure(e.to_string()))?;
    match format {
        OutputFormat::Text => {
            println!("initialized OKF bundle at {}", report.root.display());
            for p in &report.created {
                println!("  created {}", p.display());
            }
            for p in &report.skipped {
                println!("  skipped {}", p.display());
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"root":{},"created":{},"skipped":{}}}"#,
                json_escape(&report.root.display().to_string()),
                report.created.len(),
                report.skipped.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_lint_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let okf_level = match level {
        LintLevel::Level1 => ods_core::OkfLintLevel::Level1,
        LintLevel::Level3 => ods_core::OkfLintLevel::Level3,
    };
    let diagnostics = ods_core::lint_okf_bundle_with_level(&bundle, okf_level);
    print_diagnostics(&diagnostics, format);
    if diagnostics.is_empty() && matches!(format, OutputFormat::Text) {
        println!("Everything is fine — OKF bundle is consistent.");
    }
    Ok(exit_code(&diagnostics))
}

fn run_okf_doctor_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let diags = ods_core::lint_okf_bundle(&bundle);
    let audit = ods_core::audit_okf_bundle(&bundle);
    let mut human = 0usize;
    let mut machine = 0usize;
    let mut unverified = 0usize;
    let mut concepts = 0usize;
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        concepts += 1;
        if let ods_core::OkfFrontmatterState::Parsed(ref fm) = doc.frontmatter {
            match ods_core::derive_trust_tier(&fm.verified) {
                ods_core::OkfTrustTier::HumanReviewed => human += 1,
                ods_core::OkfTrustTier::MachineConfirmed => machine += 1,
                ods_core::OkfTrustTier::Unverified => unverified += 1,
            }
        } else {
            unverified += 1;
        }
    }
    let stale = diags
        .iter()
        .filter(|d| d.message.contains("concept is stale"))
        .count();
    let lint_errors = diags
        .iter()
        .filter(|d| matches!(d.severity, Severity::Error))
        .count();
    let has_error = lint_errors > 0;
    match format {
        OutputFormat::Text => {
            println!("OKF doctor — {}", root.display());
            println!(
                "  okf_version: {}",
                bundle.okf_version.as_deref().unwrap_or("(missing)")
            );
            println!("  concepts: {concepts}");
            println!(
                "  audit: compliant={} plain={} invalid={} partial={}",
                audit.compliant, audit.plain, audit.invalid, audit.partial
            );
            println!(
                "  trust: human-reviewed={human} machine-confirmed={machine} unverified={unverified}"
            );
            println!("  stale_warnings: {stale}");
            println!("  lint_errors: {lint_errors}");
            println!(
                "  status: {}",
                if has_error {
                    "issues found (run ods okf lint)"
                } else {
                    "ok"
                }
            );
        }
        OutputFormat::Json => {
            println!(
                r#"{{"okf_version":{},"concepts":{},"stale":{},"human_reviewed":{},"machine_confirmed":{},"unverified":{},"has_error":{}}}"#,
                json_escape(bundle.okf_version.as_deref().unwrap_or("")),
                concepts,
                stale,
                human,
                machine,
                unverified,
                has_error
            );
        }
    }
    Ok(ExitCode::from(if has_error { 1 } else { 0 }))
}

fn run_okf_audit_command(args: &[String]) -> Result<ExitCode, CliError> {
    let write_report = args.iter().any(|a| a == "--write-report");
    let report_path_opt = parse_report_path(args);
    let fail_on = parse_fail_on(args);
    let filtered = filter_audit_flags(args);
    let (root, _level, format) = parse_common_flags(&filtered, 2)?;
    require_okf_bundle(&root)?;
    let report_path = report_path_opt.unwrap_or_else(|| root.join(".ods/ods-errors.md"));
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let report = ods_core::audit_okf_bundle(&bundle);

    match format {
        OutputFormat::Text => {
            println!(
                "OKF audit: total={} compliant={} plain={} invalid={} partial={} skipped={}",
                report.total_md,
                report.compliant,
                report.plain,
                report.invalid,
                report.partial,
                report.skipped
            );
            for item in &report.items {
                if matches!(
                    item.class,
                    ods_core::OkfAuditClass::Plain
                        | ods_core::OkfAuditClass::Invalid
                        | ods_core::OkfAuditClass::Partial
                ) {
                    let rel = item.path.strip_prefix(&root).unwrap_or(&item.path);
                    println!("  [{:?}] {} — {}", item.class, rel.display(), item.note);
                }
            }
        }
        OutputFormat::Json => {
            println!(
                r#"{{"total_md":{},"compliant":{},"plain":{},"invalid":{},"partial":{},"skipped":{}}}"#,
                report.total_md,
                report.compliant,
                report.plain,
                report.invalid,
                report.partial,
                report.skipped
            );
        }
    }

    if write_report {
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).map_err(|e| failure(e.to_string()))?;
        }
        let md = ods_core::render_okf_audit_markdown(&root, &report);
        fs::write(&report_path, md).map_err(|e| failure(e.to_string()))?;
        if matches!(format, OutputFormat::Text) {
            println!("wrote {}", report_path.display());
        }
    }

    let fail = match fail_on {
        None => false,
        Some("plain") => report.plain > 0,
        Some("invalid") => report.invalid > 0,
        Some("any") => report.plain + report.invalid + report.partial > 0,
        Some(other) => {
            return Err(usage(format!(
                "invalid --fail-on {other} (use plain|invalid|any)"
            )));
        }
    };
    Ok(ExitCode::from(if fail { 1 } else { 0 }))
}

include!("okf_commands_extra.rs");

fn dispatch_okf_command(full_args: &[String]) -> Result<ExitCode, CliError> {
    let cmd = full_args.get(2).map(String::as_str).unwrap_or("help");
    let mut args = vec![full_args[0].clone()];
    args.extend(full_args.iter().skip(2).cloned());

    match cmd {
        "init" => run_okf_init_command(&args),
        "lint" => run_okf_lint_command(&args),
        "index" => run_okf_index_command(&args),
        "context" => run_okf_context_command(&args),
        "export" => run_okf_export_command(&args),
        "fmt" => run_okf_fmt_command(&args),
        "doctor" => run_okf_doctor_command(&args),
        "audit" => run_okf_audit_command(&args),
        "adopt" => run_okf_adopt_command(&args),
        "watch" => run_okf_watch_command(&args, false),
        "serve" => run_okf_watch_command(&args, true),
        "help" | "--help" | "-h" => {
            print_okf_help();
            Ok(ExitCode::from(0))
        }
        other => Err(usage(format!(
            "unknown okf command: {other}\nRun `ods okf help` for available commands."
        ))),
    }
}


