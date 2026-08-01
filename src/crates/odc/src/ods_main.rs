// ODS legacy binary entrypoint (argv0 `ods` → ODS engine only)
// Layout under main/: cli/ (dispatch) · commands/ (user commands) · support/ (helpers)
// Keep include list identical to main.rs so both bins share the same surface.
#![forbid(unsafe_code)]

mod service;
mod update;

// --- cli: entry + argv + exit ---
include!("main/cli/entry.rs");
include!("main/cli/cli_arg_parser.rs");
include!("main/cli/exit_code_helper.rs");

// --- commands (user-facing) ---
include!("main/commands/okf/okf_commands.rs");
include!("main/commands/upgrade_command.rs");
include!("main/commands/lint_and_index_commands.rs");
include!("main/commands/find_command.rs");
include!("main/commands/tag_command.rs");
include!("main/commands/context_graph_mv_commands.rs");
include!("main/commands/fmt_command.rs");
include!("main/commands/adopt_and_init_commands.rs");
include!("main/commands/lifecycle_commands.rs");
include!("main/commands/disable_command.rs");
include!("main/commands/service_commands.rs");
include!("main/commands/setup_command.rs");
include!("main/commands/skill_command.rs");
include!("main/commands/update_command.rs");
include!("main/commands/watch_and_serve_runner.rs");
include!("main/commands/workspaces/workspaces_command.rs");
include!("main/commands/pack/pack_command.rs");
include!("main/commands/share_command.rs");
include!("main/commands/bench_command.rs");

// --- support (formatters, loaders, helpers) ---
include!("main/support/diagnostics_formatter.rs");
include!("main/support/doctor_reporter.rs");
include!("main/support/git_sync.rs");
include!("main/support/path_change_reporter.rs");
include!("main/support/workspace_light_loader.rs");
include!("main/support/process_memory.rs");
include!("main/support/graph_formatter.rs");
include!("main/support/alias_printer.rs");
