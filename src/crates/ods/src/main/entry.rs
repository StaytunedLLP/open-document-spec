use ods_core::{
    AdoptOptions, Diagnostic, DisableOptions, FrontmatterState, InitOptions, LintLevel, LoadOptions, Severity,
    WatchTree, adopt_workspace, apply_path_changes,
    canonicalize_workspace_document_refs_with_workspace, disable_workspace, docs_with_any_tag,
    export_workspace_graph, generate_indexes, heal_orphan_path_ids, indexes_are_current,
    init_workspace, known_profiles, lint_workspace_with_level, lint_workspace_with_ref_style,
    load_workspace, load_workspace_with_options, migrate_workspace_frontmatter_with_workspace,
    move_document_and_rewrite_refs_report,
    normalize_workspace_frontmatter_spacing_with_workspace, observe_renames, paired_from_paths,
    rename_tag_in_workspace, resolve_context, scan_markdown_tree,
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
fn run(args: Vec<String>) -> Result<ExitCode, CliError> {
    if args.iter().skip(1).any(|arg| arg == "--help" || arg == "-h") {
        let cmd = args.get(1).map(String::as_str);
        if !matches!(cmd, Some("setup") | Some("workspaces") | Some("skill")) {
            print_help();
            return Ok(ExitCode::from(0));
        }
    }

    let Some(command) = args.get(1).map(String::as_str) else {
        print_help();
        return Ok(ExitCode::from(0));
    };

    // Seamless updates: periodic check on normal commands (opt-out: ODS_AUTO_UPDATE=0).
    // Skip version/help/update (fast/predictable). `watch` uses maybe_auto_update_on_watch.
    if !matches!(
        command,
        "--version" | "-V" | "version" | "--help" | "-h" | "help" | "update" | "watch" | "setup"
    ) {
        maybe_auto_update();
    }

    match command {
        "--version" | "-V" | "version" => {
            println!("ods {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::from(0))
        }
        "--help" | "-h" | "help" => {
            print_help();
            Ok(ExitCode::from(0))
        }
        "setup" => run_setup_command(&args),
        "update" => run_update_command(&args),
        "lint" => run_lint_command(&args),
        "index" => run_index_command(&args),
        "profiles" => run_profiles_command(&args),
        "tags" => run_tags_command(&args),
        "find" => run_find_command(&args),
        "tag" => run_tag_command(&args),
        "context" => run_context_command(&args),
        "graph" => run_graph_command(&args),
        "mv" => run_mv_command(&args),
        "fmt" => run_fmt_command(&args),
        "adopt" => run_adopt_command(&args),
        "new" => run_new_command(&args),
        "rm" | "remove" => run_rm_command(&args),
        "archive" => run_archive_command(&args),
        "init" | "enable" => run_init_command(&args),
        "disable" | "revert" => run_disable_command(&args),
        "doctor" => run_doctor_command(&args),
        "sync" => run_sync_command(&args),
        "logs" | "watch" => run_logs_command(&args),
        "serve" => run_serve_command(&args),
        "export" => run_export_command(&args),
        "start" => run_start_command(&args),
        "stop" => run_stop_command(&args),
        "workspaces" => run_workspaces_command(&args),
        "pack" => run_pack_command(&args),
        "share" => run_share_command(&args),
        "skill" => run_skill_command(&args),
        "bench" | "sandbox" => run_bench_command(&args),
        other => Err(usage(format!("unknown command: {other}"))),
    }
}
