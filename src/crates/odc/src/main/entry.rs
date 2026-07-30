use odc_core::{
    AdoptOptions, Diagnostic, DisableOptions, FrontmatterState, InitOptions, LintLevel, Severity,
    WatchTree, Workspace, adopt_workspace, apply_document_removes,
    apply_document_upserts, apply_path_changes,
    canonicalize_workspace_document_refs_with_workspace, disable_workspace, docs_with_any_tag,
    export_workspace_graph, generate_indexes, heal_orphan_path_ids, indexes_are_current,
    init_workspace, known_profiles, lint_workspace_with_level, lint_workspace_with_ref_style,
    load_options_graph, load_workspace, load_workspace_with_options,
    migrate_workspace_frontmatter_with_workspace, move_document_and_rewrite_refs_report,
    normalize_workspace_frontmatter_spacing_with_workspace, observe_renames, paired_from_paths,
    parse_paths_parallel, rename_tag_in_workspace, resolve_context,
    scan_markdown_tree_with_code_paths, tag_usage_with_builtins, workspace_alias_suggestions,
    workspace_aliases,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::Arc;
use std::time::Duration;
use update::{
    UpdateOptions, UpdateOutcome, maybe_auto_update, maybe_auto_update_on_watch, run_update,
};

fn main() -> ExitCode {
    match run(env::args().collect()) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.message());
            ExitCode::from(err.code())
        }
    }
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Failure(String),
}

impl CliError {
    fn message(&self) -> &str {
        match self {
            CliError::Usage(message) | CliError::Failure(message) => message,
        }
    }

    fn code(&self) -> u8 {
        match self {
            CliError::Usage(_) => 2,
            CliError::Failure(_) => 1,
        }
    }
}

fn usage<T: Into<String>>(message: T) -> CliError {
    CliError::Usage(message.into())
}

fn failure<T: Into<String>>(message: T) -> CliError {
    CliError::Failure(message.into())
}

/// Binary basename (e.g. `ods`, `odc`) — used for argv0 compat routing.
fn invoked_name(args: &[String]) -> String {
    Path::new(args.first().map(String::as_str).unwrap_or("odc"))
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("odc")
        .to_string()
}

/// When the process is invoked as `ods` (legacy binary / symlink), bare document
/// commands always use the ODS engine. When invoked as `odc`, bare document
/// commands auto-detect ODS vs OKF (see `dispatch_auto_detect`).
fn allows_bare_ods_commands(args: &[String]) -> bool {
    let name = invoked_name(args);
    name == "ods" || name.ends_with("-ods")
}

fn strip_namespace(args: &[String]) -> Vec<String> {
    // [bin, ods|okf|agents, cmd, ...] -> [bin, cmd, ...]
    let mut out = Vec::with_capacity(args.len().saturating_sub(1));
    if let Some(bin) = args.first() {
        out.push(bin.clone());
    }
    out.extend(args.iter().skip(2).cloned());
    out
}

fn is_platform_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "--version"
            | "-V"
            | "version"
            | "--help"
            | "-h"
            | "help"
            | "update"
            | "upgrade"
            | "setup"
            | "workspaces"
            | "skill"
            | "pack"
    )
}

fn is_ods_document_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "lint"
            | "index"
            | "profiles"
            | "tags"
            | "find"
            | "tag"
            | "context"
            | "graph"
            | "mv"
            | "fmt"
            | "adopt"
            | "new"
            | "rm"
            | "remove"
            | "archive"
            | "init"
            | "enable"
            | "disable"
            | "revert"
            | "doctor"
            | "sync"
            | "logs"
            | "watch"
            | "serve"
            | "export"
            | "start"
            | "stop"
            | "share"
            | "bench"
            | "sandbox"
            | "audit"
    )
}

fn run(args: Vec<String>) -> Result<ExitCode, CliError> {
    // Global help: `odc --help` / `odc help` (not nested skill/setup help)
    if args.get(1).map(String::as_str) == Some("help")
        || args.get(1).map(String::as_str) == Some("--help")
        || args.get(1).map(String::as_str) == Some("-h")
    {
        print_help();
        return Ok(ExitCode::from(0));
    }

    let Some(first) = args.get(1).map(String::as_str) else {
        print_help();
        return Ok(ExitCode::from(0));
    };

    // Seamless updates: skip on version/help/update/watch/setup
    if !matches!(
        first,
        "--version"
            | "-V"
            | "version"
            | "--help"
            | "-h"
            | "help"
            | "update"
            | "upgrade"
            | "watch"
            | "setup"
            | "ods"
            | "okf"
            | "agents"
    ) {
        maybe_auto_update();
    } else if matches!(first, "ods" | "agents") {
        let sub = args.get(2).map(String::as_str).unwrap_or("");
        if !matches!(
            sub,
            "--version" | "-V" | "version" | "--help" | "-h" | "help" | "update" | "watch" | "setup"
                | ""
        ) {
            maybe_auto_update();
        }
    }
    // `odc okf …` skips auto-update (fast, offline-friendly knowledge lint path)

    // Spec namespaces (optional force; bare commands auto-detect)
    match first {
        "ods" => {
            if args.get(2).map(String::as_str) == Some("--help")
                || args.get(2).map(String::as_str) == Some("-h")
                || args.get(2).map(String::as_str) == Some("help")
            {
                print_ods_help();
                return Ok(ExitCode::from(0));
            }
            let rewritten = strip_namespace(&args);
            return dispatch_ods_command(&rewritten);
        }
        "okf" => {
            if args.get(2).map(String::as_str) == Some("--help")
                || args.get(2).map(String::as_str) == Some("-h")
                || args.get(2).map(String::as_str) == Some("help")
                || args.get(2).is_none()
            {
                print_okf_help();
                return Ok(ExitCode::from(0));
            }
            return dispatch_okf_command(&args);
        }
        "agents" => {
            return dispatch_agents_command(&args);
        }
        _ => {}
    }

    // Platform commands (global)
    if is_platform_command(first) {
        return dispatch_platform_command(&args);
    }

    // Bare document command: auto-detect ODS vs OKF from workspace root markers.
    // Explicit `odc ods` / `odc okf` remain available. Binary name `ods` always
    // routes to ODS (legacy argv0).
    if is_ods_document_command(first) {
        if allows_bare_ods_commands(&args) {
            return dispatch_ods_command(&args);
        }
        return dispatch_auto_detect(&args);
    }

    Err(usage(format!("unknown command: {first}")))
}

/// Detect engines from CWD/path markers and run ODS and/or OKF handlers.
fn dispatch_auto_detect(args: &[String]) -> Result<ExitCode, CliError> {
    let cmd = args.get(1).map(String::as_str).unwrap_or("");
    // Resolve probe root from common path args (best-effort; handlers re-parse).
    let probe = args
        .iter()
        .skip(2)
        .find(|a| !a.starts_with('-'))
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let root = resolve_root_path(probe);

    let has_ods = odc_core::ods_enabled(&root);
    let has_okf = odc_core::okf_enabled(&root);

    // init without markers: default ODS (okf via --okf flag on init handler)
    if matches!(cmd, "init" | "enable") {
        if args.iter().any(|a| a == "--okf") {
            let mut okf_args = vec![args[0].clone(), "okf".into(), "init".into()];
            okf_args.extend(args.iter().skip(2).filter(|a| a.as_str() != "--okf").cloned());
            return dispatch_okf_command(&okf_args);
        }
        return dispatch_ods_command(args);
    }

    // Commands that only make sense for one engine
    let okf_only = matches!(cmd, "lint" | "index" | "doctor" | "audit" | "adopt" | "fmt" | "export" | "context" | "watch" | "serve");
    let ods_only_extra = matches!(
        cmd,
        "profiles" | "tags" | "find" | "tag" | "graph" | "mv" | "new" | "rm" | "remove"
            | "archive" | "disable" | "revert" | "sync" | "start" | "stop" | "share" | "bench"
            | "sandbox" | "logs"
    );

    if has_ods && has_okf && okf_only && matches!(cmd, "lint" | "doctor" | "audit") {
        // Hybrid: run both for lint/doctor/audit; prefer non-zero if either fails
        let ods_code = dispatch_ods_command(args)?;
        let mut okf_args = vec![args[0].clone(), "okf".into()];
        okf_args.extend(args.iter().skip(1).cloned());
        let okf_code = dispatch_okf_command(&okf_args)?;
        if ods_code != ExitCode::SUCCESS {
            return Ok(ods_code);
        }
        return Ok(okf_code);
    }

    if has_okf && !has_ods && okf_only {
        let mut okf_args = vec![args[0].clone(), "okf".into()];
        okf_args.extend(args.iter().skip(1).cloned());
        return dispatch_okf_command(&okf_args);
    }

    if has_ods {
        return dispatch_ods_command(args);
    }

    if has_okf && okf_only {
        let mut okf_args = vec![args[0].clone(), "okf".into()];
        okf_args.extend(args.iter().skip(1).cloned());
        return dispatch_okf_command(&okf_args);
    }

    if ods_only_extra || okf_only {
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
    // Nested help for skill/setup still works via those handlers
    if args.iter().skip(1).any(|arg| arg == "--help" || arg == "-h")
        && !matches!(command, "setup" | "workspaces" | "skill")
    {
        // keep existing behavior for `ods lint --help` → full help
    }
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
        "profiles" => run_profiles_command(args),
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
        "logs" | "watch" => run_logs_command(args),
        "serve" => run_serve_command(args),
        "export" => run_export_command(args),
        "start" => run_start_command(args),
        "stop" => run_stop_command(args),
        "share" => run_share_command(args),
        "bench" | "sandbox" => run_bench_command(args),
        "audit" => run_ods_audit_command(args),
        other => Err(usage(format!("unknown ods command: {other}"))),
    }
}
