fn print_help() {
    println!(
        "ods <command> [path] [flags]

Commands:
  init [path]              Make folder/repo ODS-compliant (add root index.md + ods: spec, generate indexes)
  disable [path]           Opt-out dry-run: strip ODS metadata (alias: revert)
  disable --write [path]   Apply disable / revert to plain Markdown
  lint [path]              Validate workspace (green message when clean)
  index [path]             Generate index.md files
  index --check [path]     Exit 1 if indexes are stale
  profiles [path]          List loaded profiles
  tags [path]              List project tags (observed) with use counts
  tags --all [path]        Include unused default ODS tags
  find [path] --tag <t>    List document ids with tag (repeat --tag = OR)
  tag rename <old> <new>   Rewrite a tag across frontmatter (dry-run; --write)
  setup [path]             Set up machine service for workspace + check updates and workspace health
  context [path] <id>      Resolve reading list for a document
  graph [path]             Print depends/related edges
  export [path]            Write graph.md for AI (optional --out PATH, --include-private)
  share [path] --out DIR   Publish a share-filtered copy of a workspace/subtree (git it yourself; --include-org, --include-private)
  new <path>               Scaffold new document with inferred profile and valid frontmatter
  rm <path-or-id>          Atomically delete document and scrub graph references workspace-wide
  archive <path-or-id>     Set document status to archived (frontmatter only; does not move the file)
  mv [path] <from> <to>    Move file/folder and rewrite refs + indexes
  fmt [path]               Normalize frontmatter/body blank lines
  fmt --refs md-paths      Also rewrite Document refs to .md paths
  fmt --migrate            Also migrate legacy/out-of-order ods: keys into canonical nested block
  doctor [path]            Report workspace health and version skew
  sync [path]              Reconcile git-tracked renames and rewrite refs
  logs [path] [-f]         Currently an alias for `watch` (foreground re-lint loop, not a log tail)
  watch [path]             Foreground live rename map + re-lint
  serve --root <path>      Headless watch loop (used by OS service)
  serve --mode poll        Low-memory polling loop (auto|watch|poll)
  start [path]             Register+start user service (background watch)
  start --status [path]    Service install/running status
  stop [path]              Stop user service
  stop --unregister [path] Stop and remove service registration
  update                   Update ods from GitHub Releases and restart background service
  update --check           Exit 0 if current, 1 if newer release available
  update --force           Reinstall latest even if versions match
  update --version <tag>   Install exact release tag (e.g. v0.1.5)
  adopt [path]             Report adoption status (dry-run)
  adopt --write [path]     Draft minimal frontmatter for plain Markdown
  workspaces list          List globally tracked ODS workspaces
  workspaces add [path]    Register a folder as an ODS workspace
  workspaces remove [path] Unregister a folder
  workspaces path          Print the registry file path (~/.ods/odsconfig.toml)
  bench stats [path]       Display token & cost efficiency ROI report
  bench strip [path]       Backup frontmatter to JSON snapshot and strip (dry-run; --write)
  bench restore [path]     Restore frontmatter from snapshot (--snapshot <id>)
  bench run [path]         Print a simulated LLM task cost/token comparison (--prompt \"<task>\"; does not call any LLM API)
  skill install            Install ODS skill or rules configuration into an AI agent's directory (--agent <name> [--scope <project|user>])

Flags:
  --version, -V            Print version and exit
  --level 1|3              Lint level (default 3)
  --format text|json       Output format for supported commands (default text)
  --write                  With adopt / tag rename / disable: apply changes
  --adopt                  With init: also draft frontmatter on plain files
  --keep-frontmatter       With disable: only drop ods: / root policy keys
  --remove-indexes         With disable: delete non-root index.md files
  --all                    With tags: include unused default ODS tags
  --tag <name>             With find: filter by tag (repeatable, OR)
  --check                  With index / update: check only
  --canonical-refs         With lint: warn on extensionless Document refs
  --refs md-paths          With fmt: rewrite Document refs to .md paths
  --migrate                With fmt: migrate flat/legacy ods-engine keys into canonical nested ods: block
  --force                  With update: reinstall even if current
  --version <tag>          With update: install exact release tag (e.g. v0.1.5)
  --mode auto|watch|poll   With serve: choose watcher strategy
  --memory-report          With serve: print RSS/document snapshot diagnostics
  --poll-secs <n>          With serve --mode poll: polling interval

Environment:
  ODS_AUTO_UPDATE=0           Disable auto-update on commands and watch (default: on)
  ODS_LOW_MEMORY=1            Make serve --mode auto choose poll mode
  ODS_SERVE_MODE=auto|watch|poll  Default serve mode
  ODS_POLL_SECS=<n>           Default serve poll interval
  GH_TOKEN / GITHUB_TOKEN  Required for release download while the repo is private
"
    );
}
fn run_setup_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut path = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                println!(
                    "ods setup [path]\n\nChecks release freshness, detects an ODS workspace, starts the user service when possible, and runs doctor."
                );
                return Ok(ExitCode::from(0));
            }
            flag if flag.starts_with('-') => {
                return Err(usage(format!("unknown setup flag: {flag}")));
            }
            other => {
                if path.is_none() {
                    path = Some(PathBuf::from(other));
                }
            }
        }
        i += 1;
    }

    if setup_update_check_enabled() {
        println!("setup: checking for updates");
        match run_update(UpdateOptions {
            check_only: true,
            force: false,
            version: None,
        }) {
            Ok(UpdateOutcome::UpToDate { current, remote }) => {
                println!("setup: ods {current} is up to date (latest {remote})");
            }
            Ok(UpdateOutcome::Available { current, remote }) => {
                println!("setup: update available: {current} -> {remote}");
                println!("run: ods update");
                return Ok(ExitCode::from(1));
            }
            Ok(UpdateOutcome::Updated { .. }) => {}
            Err(err) => {
                println!("setup: update check skipped ({err})");
            }
        }
    } else {
        println!("setup: update check skipped (ODS_AUTO_UPDATE=0)");
    }

    let probe = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = match find_marked_ods_workspace_root(&probe) {
        Some(root) => root,
        None => {
            let target = if probe.is_dir() {
                probe.clone()
            } else {
                probe
                    .parent()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."))
            };
            println!("setup: no ODS workspace found at or above {}", probe.display());
            println!("setup: run 'ods init {}' to make this folder ODS-compliant", target.display());
            return Ok(ExitCode::from(0));
        }
    };

    let init = init_workspace(&root, ods_core::InitOptions { adopt: false })
        .map_err(|err| failure(err.to_string()))?;
    if init.initialized {
        println!(
            "setup: root index ensured with ods: {} and ods-cli: \"{}\"",
            ods_core::current_ods_spec_version(),
            ods_core::current_ods_cli_requirement()
        );
    }

    println!("setup: workspace {}", root.display());
    let status = service::service_status(&root);
    println!(
        "setup: service installed={} running={} ({})",
        status.installed, status.running, status.detail
    );

    if !status.running {
        if env::var("ODS_SETUP_NO_START")
            .map(|v| {
                let v = v.trim().to_ascii_lowercase();
                v == "1" || v == "true" || v == "yes" || v == "on"
            })
            .unwrap_or(false)
        {
            println!("setup: service start skipped by ODS_SETUP_NO_START");
        } else {
            let msg = service::start_service(&root).map_err(|e| failure(e.to_string()))?;
            println!("setup: {msg}");
        }
    }

    println!("setup: doctor");
    let report = doctor_workspace(&root)?;
    println!("{}", report.text);
    Ok(ExitCode::from(if report.has_error { 1 } else { 0 }))
}

fn setup_update_check_enabled() -> bool {
    match env::var("ODS_AUTO_UPDATE") {
        Ok(v) => {
            let v = v.trim().to_ascii_lowercase();
            !(v == "0" || v == "false" || v == "no" || v == "off")
        }
        Err(_) => true,
    }
}

fn find_marked_ods_workspace_root(path: &Path) -> Option<PathBuf> {
    let mut current = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    loop {
        let index = current.join("index.md");
        if index.is_file() && ods_core::index_has_ods_field(&index) {
            return Some(current);
        }
        if current.join(".git").exists() {
            return None;
        }
        if !current.pop() {
            return None;
        }
    }
}
