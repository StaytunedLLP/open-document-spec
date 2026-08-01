/// Detect engines from CWD/path markers and run ODS and/or OKF handlers.
fn dispatch_auto_detect(args: &[String]) -> Result<ExitCode, CliError> {
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    let probe = positional_args(args, 2)
        .first()
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = resolve_root_path(probe);

    let has_ods = ods_core::ods_enabled(&root);
    let has_okf = ods_core::okf_enabled(&root);

    if matches!(cmd, "init" | "enable") {
        if args.iter().any(|a| a == "--okf") {
            let mut okf_args = vec![args[0].clone(), "okf".into(), "init".into()];
            okf_args.extend(args.iter().skip(2).filter(|a| a.as_str() != "--okf").cloned());
            return dispatch_okf_command(&okf_args);
        }
        return dispatch_ods_command(args);
    }

    // Commands implemented on both engines (OKF namespace has a handler).
    let dual_engine = matches!(
        cmd,
        "lint" | "index" | "doctor" | "audit" | "adopt" | "fmt" | "export" | "context" | "watch"
            | "serve"
    );
    let ods_only_extra = matches!(
        cmd,
        "profiles" | "tags" | "find" | "tag" | "graph" | "mv" | "new" | "rm" | "remove"
            | "archive" | "disable" | "revert" | "sync" | "start" | "stop" | "share" | "bench"
            | "sandbox" | "logs" | "coverage"
    );

    // Hybrid R1: dual-run only lint/doctor/audit; other dual-engine bare cmds need namespace.
    if has_ods && has_okf {
        if matches!(cmd, "lint" | "doctor" | "audit") {
            let ods_code = dispatch_ods_command(args)?;
            let mut okf_args = vec![args[0].clone(), "okf".into()];
            okf_args.extend(args.iter().skip(1).cloned());
            let okf_code = dispatch_okf_command(&okf_args)?;
            if ods_code != ExitCode::SUCCESS {
                return Ok(ods_code);
            }
            return Ok(okf_code);
        }
        if dual_engine {
            return Err(failure(format!(
                "hybrid workspace (ODS + OKF markers) at {}\n\n\
                 Bare `odc {cmd}` is ambiguous here. Use explicit:\n\
                 • `odc ods {cmd}` — ODS engine\n\
                 • `odc okf {cmd}` — OKF engine\n\n\
                 Note: bare dual-run applies only to lint / doctor / audit.",
                root.display(),
                cmd = cmd
            )));
        }
        // ODS-only commands still run against the ODS side of a hybrid tree.
        if ods_only_extra {
            return dispatch_ods_command(args);
        }
    }

    if has_okf && !has_ods && dual_engine {
        let mut okf_args = vec![args[0].clone(), "okf".into()];
        okf_args.extend(args.iter().skip(1).cloned());
        return dispatch_okf_command(&okf_args);
    }

    if has_ods {
        return dispatch_ods_command(args);
    }

    if has_okf && dual_engine {
        let mut okf_args = vec![args[0].clone(), "okf".into()];
        okf_args.extend(args.iter().skip(1).cloned());
        return dispatch_okf_command(&okf_args);
    }

    if ods_only_extra || dual_engine {
        return Err(failure(format!(
            "not an ODS or OKF workspace: {}\n\n\
             • ODS: root index.md with `ods:` (+ `odc:` CLI pin) — run `odc init`\n\
             • OKF: root index.md with `okf_version:` — run `odc init --okf`\n\
             • Or use explicit: `odc ods {cmd}` / `odc okf {cmd}`",
            root.display(),
            cmd = cmd
        )));
    }

    dispatch_ods_command(args)
}

fn dispatch_platform_command(args: &[String]) -> Result<ExitCode, CliError> {
    let command = args.get(1).map(String::as_str).unwrap_or("");
    match command {
        "--version" | "-V" | "version" => {
            let name = if allows_bare_ods_commands(args) {
                "ods"
            } else {
                "odc"
            };
            println!("{name} {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::from(0))
        }
        "--help" | "-h" | "help" => {
            print_help();
            Ok(ExitCode::from(0))
        }
        "setup" => run_setup_command(args),
        "update" => run_update_command(args),
        "upgrade" => run_upgrade_command(args),
        "workspaces" => run_workspaces_command(args),
        "skill" => run_skill_command(args),
        "pack" => run_pack_command(args),
        other => Err(usage(format!("unknown platform command: {other}"))),
    }
}

fn dispatch_ods_command(args: &[String]) -> Result<ExitCode, CliError> {
    let command = args.get(1).map(String::as_str).unwrap_or("");
    match command {
        "--version" | "-V" | "version" => {
            println!("odc ods {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::from(0))
        }
        "--help" | "-h" | "help" => {
            print_ods_help();
            Ok(ExitCode::from(0))
        }
        "lint" => run_lint_command(args),
        "index" => run_index_command(args),
        "profile" | "profiles" => {
            let sub = args.get(2).map(String::as_str).unwrap_or("");
            if sub == "init" {
                run_profile_init_command(args)
            } else {
                run_profile_list_command(args)
            }
        },
        "tags" => run_tags_command(args),
        "find" => run_find_command(args),
        "tag" => run_tag_command(args),
        "context" => run_context_command(args),
        "graph" => run_graph_command(args),
        "mv" => run_mv_command(args),
        "fmt" => run_fmt_command(args),
        "adopt" => run_adopt_command(args),
        "new" => run_new_command(args),
        "rm" | "remove" => run_rm_command(args),
        "archive" => run_archive_command(args),
        "init" | "enable" => run_init_command(args),
        "disable" | "revert" => run_disable_command(args),
        "doctor" => run_doctor_command(args),
        "sync" => run_sync_command(args),
        "watch" => run_watch_command(args),
        "logs" => run_logs_command(args),
        "serve" => run_serve_command(args),
        "export" => run_export_command(args),
        "start" => run_start_command(args),
        "stop" => run_stop_command(args),
        "share" => run_share_command(args),
        "bench" | "sandbox" => run_bench_command(args),
        "audit" => run_ods_audit_command(args),
        "coverage" => run_coverage_command(args),
        "update" => run_update_command(args),
        "upgrade" => run_upgrade_command(args),
        other => Err(usage(format!("unknown ods command: {other}"))),
    }
}
