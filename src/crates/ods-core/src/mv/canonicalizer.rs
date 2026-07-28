fn common_parent(paths: &[PathBuf]) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }
    let mut prefix = paths[0].parent()?.to_path_buf();
    for p in &paths[1..] {
        let parent = p.parent()?;
        while !parent.starts_with(&prefix) {
            prefix = prefix.parent()?.to_path_buf();
        }
    }
    Some(prefix)
}
