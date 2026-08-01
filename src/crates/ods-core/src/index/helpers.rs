use crate::model::Workspace;
use std::path::{Path, PathBuf};

pub(super) fn is_workspace_root(directory: &Path, workspace: &Workspace) -> bool {
    let canonical_dir = directory
        .canonicalize()
        .unwrap_or_else(|_| directory.to_path_buf());
    let canonical_root = workspace
        .root
        .canonicalize()
        .unwrap_or_else(|_| workspace.root.clone());
    canonical_dir == canonical_root
}

#[allow(clippy::type_complexity)]
pub(super) fn default_index_header(
    directory: &Path,
    workspace: &Workspace,
) -> (String, String, Option<String>, Vec<String>, Vec<String>, Vec<String>) {
    let name = directory
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Workspace");
    let is_root = is_workspace_root(directory, workspace);
    let title = if is_root {
        format!("{name} Workspace")
    } else {
        format!("{name}/")
    };
    (
        title,
        "index".to_string(),
        None,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}
