

/// Compute a stable hash identifier for a repository root path.
pub fn compute_repo_hash(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let path_str = canonical.to_string_lossy();
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in path_str.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Resolve directory path for machine-level snapshots: ~/.odc/backups/<repo_hash>/
/// (falls back to legacy ~/.ods/backups if that tree already has snapshots for this repo).
pub fn get_backup_dir(root: &Path) -> io::Result<PathBuf> {
    let repo_hash = compute_repo_hash(root);
    let home_base = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile)
    } else {
        let dir = std::env::temp_dir().join("odc_backups").join(&repo_hash);
        fs::create_dir_all(&dir)?;
        return Ok(dir);
    };
    let modern = home_base.join(".odc").join("backups").join(&repo_hash);
    let legacy = home_base.join(".ods").join("backups").join(&repo_hash);
    let dir = if modern.exists() || !legacy.exists() {
        modern
    } else {
        legacy
    };
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
