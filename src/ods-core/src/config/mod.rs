//! Workspace configuration (`ods.toml` at repo root).

mod ods_toml;

pub use ods_toml::{
    ServiceConfig, WorkspaceConfig, load_workspace_config, migrate_root_index_to_toml,
    ods_toml_enabled, ods_toml_path, parse_ods_toml, render_ods_toml, write_ods_toml,
};
