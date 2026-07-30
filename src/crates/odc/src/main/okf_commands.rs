fn print_okf_help() {
    println!(
        "odc okf <command> [path] [flags]

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
  audit --write-report     Write .odc/odc-errors.md
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

See also: odc ods <cmd> for ODS workspaces; odc update for binary updates.
"
    );
}

fn require_okf_bundle(root: &Path) -> Result<(), CliError> {
    if odc_core::okf_enabled(root) {
        return Ok(());
    }
    Err(failure(format!(
        "not an OKF bundle: {}\n\n\
         No root index.md with okf_version found.\n\
         Run `odc okf init` here to create an OKF v0.2 bundle.",
        root.display()
    )))
}

fn run_okf_init_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut opts = odc_core::OkfInitOptions::default();
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
        odc_core::init_okf_bundle(&root, opts).map_err(|e| failure(e.to_string()))?;
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
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let okf_level = match level {
        LintLevel::Level1 => odc_core::OkfLintLevel::Level1,
        LintLevel::Level3 => odc_core::OkfLintLevel::Level3,
    };
    let diagnostics = odc_core::lint_okf_bundle_with_level(&bundle, okf_level);
    print_diagnostics(&diagnostics, format);
    if diagnostics.is_empty() && matches!(format, OutputFormat::Text) {
        println!("Everything is fine — OKF bundle is consistent.");
    }
    Ok(exit_code(&diagnostics))
}

fn run_okf_doctor_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let diags = odc_core::lint_okf_bundle(&bundle);
    let audit = odc_core::audit_okf_bundle(&bundle);
    let mut human = 0usize;
    let mut machine = 0usize;
    let mut unverified = 0usize;
    let mut concepts = 0usize;
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        concepts += 1;
        if let odc_core::OkfFrontmatterState::Parsed(ref fm) = doc.frontmatter {
            match odc_core::derive_trust_tier(&fm.verified) {
                odc_core::OkfTrustTier::HumanReviewed => human += 1,
                odc_core::OkfTrustTier::MachineConfirmed => machine += 1,
                odc_core::OkfTrustTier::Unverified => unverified += 1,
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
                    "issues found (run odc okf lint)"
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
    let report_path = report_path_opt.unwrap_or_else(|| root.join(".odc/odc-errors.md"));
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let report = odc_core::audit_okf_bundle(&bundle);

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
                    odc_core::OkfAuditClass::Plain
                        | odc_core::OkfAuditClass::Invalid
                        | odc_core::OkfAuditClass::Partial
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
        let md = odc_core::render_okf_audit_markdown(&root, &report);
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

fn run_okf_adopt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let write = args.iter().any(|a| a == "--write");
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let mut changed = 0usize;
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        if !matches!(doc.frontmatter, odc_core::OkfFrontmatterState::Absent) {
            continue;
        }
        let rel = doc.path.strip_prefix(&root).unwrap_or(&doc.path);
        if write {
            let stem = doc
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("concept");
            let body = fs::read_to_string(&doc.path).map_err(|e| failure(e.to_string()))?;
            let drafted = format!("---\ntype: Reference\ntitle: {stem}\nstatus: draft\n---\n\n{body}");
            fs::write(&doc.path, drafted).map_err(|e| failure(e.to_string()))?;
            changed += 1;
            if matches!(format, OutputFormat::Text) {
                println!("adopted {}", rel.display());
            }
        } else if matches!(format, OutputFormat::Text) {
            println!("would adopt {}", rel.display());
            changed += 1;
        }
    }
    if matches!(format, OutputFormat::Text) {
        if write {
            println!("adopted {changed} file(s)");
        } else {
            println!("{changed} plain file(s) (pass --write to draft frontmatter)");
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_index_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let check = args.iter().any(|a| a == "--check");
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    if check {
        let current =
            odc_core::okf_indexes_are_current(&bundle).map_err(|e| failure(e.to_string()))?;
        match format {
            OutputFormat::Text => {
                if current {
                    println!("okf indexes up to date");
                } else {
                    eprintln!("okf indexes out of date; run `odc okf index`");
                }
            }
            OutputFormat::Json => {
                println!(r#"{{"current":{}}}"#, if current { "true" } else { "false" });
            }
        }
        return Ok(ExitCode::from(if current { 0 } else { 1 }));
    }
    let paths =
        odc_core::generate_okf_indexes(&bundle).map_err(|e| failure(e.to_string()))?;
    match format {
        OutputFormat::Text => {
            for p in &paths {
                println!("{}", p.display());
            }
        }
        OutputFormat::Json => {
            println!(r#"{{"written":{},"count":{}}}"#, paths.len(), paths.len());
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_context_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    // Prefer last non-flag arg that is not the workspace root path.
    let root_s = root.to_string_lossy();
    let id = args
        .iter()
        .skip(2)
        .rfind(|a| !a.starts_with('-') && a.as_str() != root_s.as_ref())
        .cloned()
        .ok_or_else(|| usage("okf context requires a concept id or path"))?;
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let list = odc_core::okf_context(&bundle, &id);
    if list.is_empty() {
        return Err(failure(format!("concept not found: {id}")));
    }
    match format {
        OutputFormat::Text => {
            for p in &list {
                let rel = p.strip_prefix(&root).unwrap_or(p);
                println!("{}", rel.display());
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = list
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            println!(r#"{{"context":[{}]}}"#, items.join(","));
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_export_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut out = None;
    let mut path = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                out = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            other if other.starts_with("--out=") => {
                out = Some(PathBuf::from(&other["--out=".len()..]));
                i += 1;
            }
            other if !other.starts_with('-') => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let root = resolve_root_path(
        path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    );
    require_okf_bundle(&root)?;
    let out = out.unwrap_or_else(|| root.join("okf-graph.md"));
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let written =
        odc_core::export_okf_graph(&bundle, &out).map_err(|e| failure(e.to_string()))?;
    println!("wrote {}", written.display());
    Ok(ExitCode::from(0))
}

fn run_okf_fmt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let bundle = odc_core::load_okf_bundle(&root).map_err(|e| failure(e.to_string()))?;
    let changed = odc_core::fmt_okf_bundle(&bundle).map_err(|e| failure(e.to_string()))?;
    match format {
        OutputFormat::Text => {
            if changed.is_empty() {
                println!("okf frontmatter already clean");
            } else {
                println!("formatted {} file(s)", changed.len());
                for p in &changed {
                    let rel = p.strip_prefix(&root).unwrap_or(p);
                    println!("  {}", rel.display());
                }
            }
        }
        OutputFormat::Json => {
            println!(r#"{{"changed":{},"count":{}}}"#, changed.len(), changed.len());
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_watch_command(args: &[String], headless: bool) -> Result<ExitCode, CliError> {
    let (root, level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let okf_level = match level {
        LintLevel::Level1 => odc_core::OkfLintLevel::Level1,
        LintLevel::Level3 => odc_core::OkfLintLevel::Level3,
    };
    let shutdown = install_shutdown_flag();
    let poll = Duration::from_secs(2);

    let tick = |root: &Path| -> Result<(), CliError> {
        let bundle = odc_core::load_okf_bundle(root).map_err(|e| failure(e.to_string()))?;
        let diagnostics = odc_core::lint_okf_bundle_with_level(&bundle, okf_level);
        if !headless || !diagnostics.is_empty() {
            print_diagnostics(&diagnostics, format);
        }
        if diagnostics.is_empty() && !headless {
            println!("okf: clean — {}", root.display());
        }
        Ok(())
    };

    tick(&root)?;
    if headless {
        eprintln!("odc okf serve: polling {} every {}s", root.display(), poll.as_secs());
    } else {
        eprintln!(
            "odc okf watch: re-linting {} on change (poll {}s; Ctrl+C to stop)",
            root.display(),
            poll.as_secs()
        );
    }

    // Prefer notify when available; fall back to poll if watcher fails.
    use notify_debouncer_mini::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
    let (tx, rx) = channel();
    let mut debouncer = match new_debouncer(Duration::from_millis(500), move |res: DebounceEventResult| {
        let _ = tx.send(res);
    }) {
        Ok(d) => Some(d),
        Err(err) => {
            eprintln!("okf watch: notify unavailable ({err}); using poll only");
            None
        }
    };
    if let Some(ref mut d) = debouncer {
        if let Err(err) = d.watcher().watch(&root, RecursiveMode::Recursive) {
            eprintln!("okf watch: watch failed ({err}); using poll only");
            debouncer = None;
        }
    }

    loop {
        if shutdown.load(Ordering::SeqCst) {
            break;
        }
        if debouncer.is_some() {
            match rx.recv_timeout(Duration::from_millis(250)) {
                Ok(Ok(_events)) => {
                    if let Err(err) = tick(&root) {
                        eprintln!("{}", err.message());
                    }
                }
                Ok(Err(err)) => eprintln!("okf watch error: {err:?}"),
                Err(RecvTimeoutError::Timeout) => {
                    // idle
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        } else {
            sleep_checking_shutdown(poll, &shutdown);
            if shutdown.load(Ordering::SeqCst) {
                break;
            }
            if let Err(err) = tick(&root) {
                eprintln!("{}", err.message());
            }
        }
    }
    Ok(ExitCode::from(0))
}

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
            "unknown okf command: {other}\nRun `odc okf help` for available commands."
        ))),
    }
}

fn filter_audit_flags(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write-report" => {}
            "--report-path" => {
                i += 1; // skip value
            }
            "--fail-on" => {
                i += 1;
            }
            other if other.starts_with("--report-path=") => {}
            other => out.push(other.to_string()),
        }
        i += 1;
    }
    out
}

fn parse_report_path(args: &[String]) -> Option<PathBuf> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--report-path" {
            return args.get(i + 1).map(PathBuf::from);
        }
        if let Some(rest) = args[i].strip_prefix("--report-path=") {
            return Some(PathBuf::from(rest));
        }
        i += 1;
    }
    None
}

fn parse_fail_on(args: &[String]) -> Option<&'static str> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--fail-on" {
            return match args.get(i + 1).map(String::as_str) {
                Some("plain") => Some("plain"),
                Some("invalid") => Some("invalid"),
                Some("any") => Some("any"),
                Some(_) => Some("?"),
                None => Some("?"),
            };
        }
        i += 1;
    }
    None
}
