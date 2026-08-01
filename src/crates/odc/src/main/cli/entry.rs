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
    let arg = args.first().map(String::as_str).unwrap_or("odc");
    let name = arg
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(arg)
        .to_ascii_lowercase();
    name.strip_suffix(".exe").unwrap_or(&name).to_string()
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
            | "coverage"
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

include!("entry_dispatch.rs");

#[cfg(test)]
mod entry_tests {
    use super::*;

    #[test]
    fn test_invoked_name_and_allows_bare_ods_commands() {
        assert_eq!(invoked_name(&["ods".into()]), "ods");
        assert_eq!(invoked_name(&["C:\\path\\to\\ods.exe".into()]), "ods");
        assert_eq!(invoked_name(&["C:\\path\\to\\ODS.EXE".into()]), "ods");
        assert_eq!(invoked_name(&["/usr/bin/ods".into()]), "ods");
        assert_eq!(invoked_name(&["odc".into()]), "odc");
        assert_eq!(invoked_name(&["C:\\path\\to\\odc.exe".into()]), "odc");

        assert!(allows_bare_ods_commands(&["C:\\path\\to\\ods.exe".into(), "lint".into()]));
        assert!(allows_bare_ods_commands(&["ods".into(), "doctor".into()]));
        assert!(!allows_bare_ods_commands(&["odc.exe".into(), "lint".into()]));
    }
}
