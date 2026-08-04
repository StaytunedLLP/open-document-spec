fn git_detect_renames(root: &Path) -> Result<Option<Vec<(PathBuf, PathBuf)>>, CliError> {
    let probe = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map_err(|err| fail_msg(ods_core::io_failed("git", err)))?;
    if !probe.status.success() {
        return Ok(None);
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["status", "--porcelain=v1", "-z", "--untracked-files=no"])
        .output()
        .map_err(|err| fail_msg(ods_core::io_failed("git status", err)))?;
    if !output.status.success() {
        return Err(fail_msg(ods_core::io_failed("git status", output.status)));
    }
    // Porcelain -z rename: first NUL field is "R[score] newpath" (or "R  newpath"),
    // second field is the original path (verified against git status --porcelain -z).
    let mut renames = Vec::new();
    let entries: Vec<_> = output
        .stdout
        .split(|b| *b == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| String::from_utf8_lossy(entry).into_owned())
        .collect();
    let mut i = 0;
    while i < entries.len() {
        let text = &entries[i];
        let status = text.chars().next().unwrap_or(' ');
        if (status == 'R' || status == 'C') && i + 1 < entries.len() {
            let new_path = text
                .split_once(' ')
                .map(|(_, path)| path.trim_start())
                .filter(|p| !p.is_empty())
                .unwrap_or("");
            if !new_path.is_empty() {
                let old_path = &entries[i + 1];
                renames.push((root.join(old_path), root.join(new_path)));
                i += 2;
                continue;
            }
        }
        i += 1;
    }
    Ok(Some(renames))
}
