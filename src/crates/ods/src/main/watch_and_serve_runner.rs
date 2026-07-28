/// Install a `SIGTERM`/`SIGINT`/Ctrl-C handler that flips a shared flag instead
/// of leaving the process to be hard-killed. Lets `ods serve`/`watch` exit via
/// a normal return from `main` (flushing coverage/profiling data, closing
/// files cleanly) instead of only ever dying to an external `SIGKILL`. Safe to
/// call once per process; a failed registration (e.g. handler already set) is
/// non-fatal — the process just falls back to old hard-kill-only behavior.
fn install_shutdown_flag() -> Arc<AtomicBool> {
    let shutdown = Arc::new(AtomicBool::new(false));
    let handler_flag = Arc::clone(&shutdown);
    let _ = ctrlc::set_handler(move || {
        handler_flag.store(true, Ordering::SeqCst);
    });
    shutdown
}

/// Sleep up to `total`, waking early (without sleeping the full remainder) once
/// `shutdown` flips, by polling in short increments.
fn sleep_checking_shutdown(total: Duration, shutdown: &AtomicBool) {
    const STEP: Duration = Duration::from_millis(200);
    let mut waited = Duration::ZERO;
    while waited < total {
        if shutdown.load(Ordering::SeqCst) {
            return;
        }
        let this_step = STEP.min(total - waited);
        std::thread::sleep(this_step);
        waited += this_step;
    }
}

fn watch_workspace(
    root: &Path,
    level: LintLevel,
    format: OutputFormat,
    headless: bool,
) -> Result<(), CliError> {
    use notify_debouncer_mini::{DebounceEventResult, new_debouncer, notify::RecursiveMode};
    use std::cell::RefCell;
    use std::rc::Rc;

    let shutdown = install_shutdown_flag();

    // Retain unpaired removals across debounce batches so OS renames still map.
    let tree = Rc::new(RefCell::new(WatchTree::from_scan(
        scan_markdown_tree(root, &[]).map_err(|err| failure(err.to_string()))?,
    )));

    run_watch_tick(root, &tree, level, format, headless)?;

    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(
        Duration::from_millis(500),
        move |res: DebounceEventResult| {
            let _ = tx.send(res);
        },
    )
    .map_err(|err| failure(format!("watch init failed: {err}")))?;

    debouncer
        .watcher()
        .watch(root, RecursiveMode::Recursive)
        .map_err(|err| failure(format!("watch {}: {err}", root.display())))?;

    if !headless {
        eprintln!(
            "watching {} — renames map automatically (Ctrl+C to stop)",
            root.display()
        );
    } else {
        eprintln!("ods serve: watching {}", root.display());
    }
    loop {
        match rx.recv_timeout(Duration::from_millis(250)) {
            Ok(Ok(_events)) => {
                if let Err(err) = run_watch_tick(root, &tree, level, format, headless) {
                    eprintln!("{}", err.message());
                }
            }
            Ok(Err(err)) => eprintln!("watch error: {err:?}"),
            Err(RecvTimeoutError::Timeout) => {
                if shutdown.load(Ordering::SeqCst) {
                    break;
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    eprintln!("ods serve: shutting down {}", root.display());
    Ok(())
}

fn serve_workspace(options: ServeOptions) -> Result<(), CliError> {
    match resolved_serve_mode(options.mode) {
        ServeMode::Watch => {
            if options.memory_report {
                print_memory_report("watch", &options.root, 0);
            }
            watch_workspace(&options.root, LintLevel::Level3, OutputFormat::Text, true)
        }
        ServeMode::Poll => poll_workspace(options),
        ServeMode::Auto => unreachable!("auto mode is resolved before serve"),
    }
}

fn poll_workspace(options: ServeOptions) -> Result<(), CliError> {
    use std::cell::RefCell;
    use std::rc::Rc;

    let shutdown = install_shutdown_flag();
    let tree = Rc::new(RefCell::new(WatchTree::from_scan(
        scan_markdown_tree(&options.root, &[]).map_err(|err| failure(err.to_string()))?,
    )));
    eprintln!("ods serve: polling {}", options.root.display());
    while !shutdown.load(Ordering::SeqCst) {
        run_watch_tick(
            &options.root,
            &tree,
            LintLevel::Level3,
            OutputFormat::Text,
            true,
        )?;
        if options.memory_report {
            let retained = tree.borrow().snapshot.files.len();
            print_memory_report("poll", &options.root, retained);
        }
        sleep_checking_shutdown(Duration::from_secs(options.poll_secs), &shutdown);
    }
    eprintln!("ods serve: shutting down {}", options.root.display());
    Ok(())
}

fn run_watch_tick(
    root: &Path,
    tree: &std::rc::Rc<std::cell::RefCell<WatchTree>>,
    level: LintLevel,
    format: OutputFormat,
    headless: bool,
) -> Result<(), CliError> {
    let metadata = if headless {
        load_light(root).map_err(failure)?
    } else {
        load_workspace(root).map_err(|err| failure(err.to_string()))?
    };
    let current = scan_markdown_tree_with_code_paths(root, &metadata.ignore, &metadata.code_paths)
        .map_err(|err| failure(err.to_string()))?;
    let changes = {
        let watch = tree.borrow();
        observe_renames(&watch.effective_previous(), &current)
    };
    if !changes.is_empty() {
        let report = apply_path_changes(root, &changes).map_err(|err| failure(err.to_string()))?;
        if matches!(format, OutputFormat::Text) && !headless {
            eprintln!("path map: {}", report.summary());
            for (from, to) in &report.moves {
                let from = from.strip_prefix(root).unwrap_or(from);
                let to = to.strip_prefix(root).unwrap_or(to);
                eprintln!("  move {} → {}", from.display(), to.display());
            }
            for w in &report.warnings {
                eprintln!("warning: {w}");
            }
        }
    } else {
        let heal = heal_orphan_path_ids(root).map_err(|err| failure(err.to_string()))?;
        if !heal.rewritten_files.is_empty() && matches!(format, OutputFormat::Text) && !headless {
            eprintln!("path id heal: {}", heal.summary());
        }
    }
    let _ = generate_indexes(&metadata).map_err(|err| failure(err.to_string()))?;
    drop(metadata);

    let workspace = load_workspace(root).map_err(|err| failure(err.to_string()))?;
    let diagnostics = lint_workspace_with_level(&workspace, level);
    if !headless {
        print_diagnostics(&diagnostics, format);
        write_or_clear_ods_error_report(root, &diagnostics, format)?;
        if diagnostics.is_empty() && matches!(format, OutputFormat::Text) {
            println!("Everything is fine — graph and links are consistent. No update required.");
        }
    } else {
        write_or_clear_ods_error_report(root, &diagnostics, OutputFormat::Text)?;
        if !diagnostics.is_empty() {
            eprintln!(
                "ods serve: {} diagnostic(s) in {}",
                diagnostics.len(),
                root.display()
            );
        }
    }
    let ignore = workspace.ignore.clone();
    let code_paths = workspace.code_paths.clone();
    drop(workspace);
    let after = scan_markdown_tree_with_code_paths(root, &ignore, &code_paths)
        .map_err(|err| failure(err.to_string()))?;
    let paired = paired_from_paths(&changes);
    tree.borrow_mut().commit_scan(after, &paired);
    Ok(())
}

fn print_memory_report(mode: &str, root: &Path, retained_snapshot_files: usize) {
    let rss = current_rss_kb()
        .map(|kb| kb.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let documents = load_light(root)
        .map(|workspace| workspace.documents.len())
        .unwrap_or(0);
    eprintln!(
        "ods serve: mode={mode} documents={documents} retained_snapshot_files={retained_snapshot_files} rss_kb={rss}"
    );
}
