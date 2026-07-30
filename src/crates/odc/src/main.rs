// OpenDocify primary binary entrypoint
#![forbid(unsafe_code)]

mod service;
mod update;

include!("main/entry.rs");
include!("main/okf_commands.rs");
include!("main/upgrade_command.rs");
include!("main/lint_and_index_commands.rs");
include!("main/find_command.rs");
include!("main/tag_command.rs");
include!("main/context_graph_mv_commands.rs");
include!("main/fmt_command.rs");
include!("main/adopt_and_init_commands.rs");
include!("main/lifecycle_commands.rs");
include!("main/disable_command.rs");
include!("main/service_commands.rs");
include!("main/cli_arg_parser.rs");
include!("main/setup_command.rs");
include!("main/skill_command.rs");
include!("main/diagnostics_formatter.rs");
include!("main/doctor_reporter.rs");
include!("main/git_sync.rs");
include!("main/path_change_reporter.rs");
include!("main/update_command.rs");
include!("main/watch_and_serve_runner.rs");
include!("main/workspace_light_loader.rs");
include!("main/process_memory.rs");
include!("main/graph_formatter.rs");
include!("main/exit_code_helper.rs");
include!("main/alias_printer.rs");
include!("main/workspaces_command.rs");
include!("main/pack_command.rs");
include!("main/share_command.rs");
include!("main/bench_command.rs");
