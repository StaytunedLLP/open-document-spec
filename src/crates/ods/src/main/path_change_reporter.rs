fn sync_git_renames(root: &Path) -> Result<ods_core::PathChangeReport, CliError> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let Some(renames) = git_detect_renames(root)? else {
        return Err(failure(
            "git is not available or workspace is not a git repo",
        ));
    };
    if renames.is_empty() {
        return Ok(ods_core::PathChangeReport::default());
    }
    let changes = renames
        .into_iter()
        .map(|(from, to)| {
            if to.is_dir() || from.extension().is_none() {
                ods_core::PathChange::DirMoved {
                    from,
                    to,
                    disk_already_moved: true,
                }
            } else {
                ods_core::PathChange::FileMoved {
                    from,
                    to,
                    disk_already_moved: true,
                }
            }
        })
        .collect::<Vec<_>>();
    ods_core::apply_path_changes(root, &changes).map_err(|err| failure(err.to_string()))
}
