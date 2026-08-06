fn sync_git_renames(root: &Path) -> Result<ods_core::PathChangeReport, CliError> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let root = root.as_path();
    let Some(renames) = git_detect_renames(root)? else {
        return Err(fail_msg(
            ods_core::UserMsg::new(
                "git_unavailable",
                ods_core::ErrorStage::Service,
                "git is not available or this folder is not a git repo",
            )
            .next("install git, or run renames with `ods mv` / `ods watch` instead of `ods sync`"),
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
    ods_core::apply_path_changes(root, &changes).map_err(|err| fail_io("apply path changes", err))
}

#[cfg(test)]
mod tests_path_change_reporter {
    use super::*;

    #[test]
    fn test_sync_git_renames_non_git_repo() {
        let td = tempfile::tempdir().unwrap();
        let res = sync_git_renames(td.path());
        // Since tempdir is not a git repo, git_detect_renames returns None or error
        assert!(res.is_err());
    }
}

