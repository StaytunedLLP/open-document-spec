

fn run_okf_watch_command(args: &[String], headless: bool) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let okf_level = ods_core::OkfLintLevel::Level3;
    let shutdown = install_shutdown_flag();
    let poll = Duration::from_secs(2);

    let tick = |root: &Path| -> Result<(), CliError> {
        let bundle = ods_core::load_okf_bundle(root).map_err(|e| fail_io("okf watch", e))?;
        let diagnostics = ods_core::lint_okf_bundle_with_level(&bundle, okf_level);
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
        eprintln!("ods serve --okf: polling {} every {}s", root.display(), poll.as_secs());
    } else {
        eprintln!(
            "ods watch --okf: re-linting {} on change (poll {}s; Ctrl+C to stop)",
            root.display(),
            poll.as_secs()
        );
    }

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
