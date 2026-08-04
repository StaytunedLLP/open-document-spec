use ods_core::{
    AdoptOptions, Diagnostic, DisableOptions, FrontmatterState, InitOptions, LintLevel, Severity,
    WatchTree, Workspace, adopt_workspace, apply_document_removes,
    apply_document_upserts, apply_path_changes,
    canonicalize_workspace_document_refs_with_workspace, disable_workspace, docs_with_any_tag,
    export_workspace_graph, generate_indexes, heal_orphan_path_ids, indexes_are_current,
    init_workspace, known_profiles, lint_workspace_with_level, lint_workspace_with_ref_style,
    load_options_graph, load_profile_catalog, load_workspace, load_workspace_with_options,
    profile_catalog_roots,
    migrate_workspace_frontmatter_with_workspace, move_document_and_rewrite_refs_report,
    normalize_workspace_frontmatter_spacing_with_workspace, observe_renames, paired_from_paths,
    parse_paths_parallel, rename_tag_in_workspace, scan_markdown_tree_with_code_paths,
    tag_usage_with_builtins, workspace_alias_suggestions, workspace_aliases,
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
            | "stats"
            | "completion"
            | "schema"
            | "tree"
            | "diff"
            | "clean"
            | "lsp"
    )
}

fn is_ods_document_command(cmd: &str) -> bool {
    matches!(
        cmd,
        "lint"
            | "index"
            | "profiles"
            | "profile"
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
            | "stats"
            | "completion"
            | "schema"
            | "tree"
            | "diff"
            | "clean"
            | "lsp"
            | "undo"
    )
}

fn run(args: Vec<String>) -> Result<ExitCode, CliError> {
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
            | "agents"
    ) {
        maybe_auto_update();
    }

    match first {
        "okf" => {
            // Namespace removed: OKF is flag-only (`ods <cmd> --okf`).
            return Err(usage(
                "unknown command: okf\n\n\
                 OKF is enabled with the `--okf` flag on shared commands (no namespace).\n\
                 Examples:\n\
                   ods init --okf\n\
                   ods lint --okf\n\
                   ods doctor --okf\n\
                   ods index --okf\n\
                 See: ods help",
            ));
        }
        "agents" => {
            return dispatch_agents_command(&args);
        }
        _ => {}
    }

    if is_platform_command(first) {
        return dispatch_platform_command(&args);
    }

    if is_ods_document_command(first) {
        return dispatch_ods_command(&args);
    }

    Err(usage(format!("unknown command: {first}")))
}

include!("entry_dispatch.rs");
