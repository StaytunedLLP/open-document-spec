use super::*;

/// Diff two scans and emit path changes for unique content-hash renames/moves.
pub fn observe_renames(previous: &TreeSnapshot, current: &TreeSnapshot) -> Vec<PathChange> {
    let mut removed: Vec<(PathBuf, u64)> = Vec::new();
    let mut added: Vec<(PathBuf, u64)> = Vec::new();

    for (path, hash) in &previous.files {
        if !current.files.contains_key(path) {
            removed.push((path.clone(), *hash));
        }
    }
    for (path, hash) in &current.files {
        if !previous.files.contains_key(path) {
            added.push((path.clone(), *hash));
        }
    }

    if removed.is_empty() || added.is_empty() {
        return Vec::new();
    }

    let mut removed_by_hash: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (path, hash) in &removed {
        removed_by_hash.entry(*hash).or_default().push(path.clone());
    }
    let mut added_by_hash: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for (path, hash) in &added {
        added_by_hash.entry(*hash).or_default().push(path.clone());
    }

    let mut pairs: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut used_added: BTreeSet<PathBuf> = BTreeSet::new();

    for (hash, froms) in &removed_by_hash {
        let Some(tos) = added_by_hash.get(hash) else {
            continue;
        };
        if froms.len() != 1 || tos.len() != 1 {
            continue;
        }
        let from = &froms[0];
        let to = &tos[0];
        if used_added.contains(to) {
            continue;
        }
        used_added.insert(to.clone());
        pairs.push((from.clone(), to.clone()));
    }

    if let Some(dir_change) = try_collapse_dir_move(&pairs) {
        return vec![dir_change];
    }

    pairs
        .into_iter()
        .map(|(from, to)| PathChange::FileMoved {
            from,
            to,
            disk_already_moved: true,
        })
        .collect()
}

fn try_collapse_dir_move(pairs: &[(PathBuf, PathBuf)]) -> Option<PathChange> {
    if pairs.len() < 2 {
        return None;
    }

    let mut old_dir: Option<PathBuf> = None;
    let mut new_dir: Option<PathBuf> = None;

    for (from, to) in pairs {
        let from_parent = from.parent()?.to_path_buf();
        let to_parent = to.parent()?.to_path_buf();
        let from_name = from.file_name()?;
        let to_name = to.file_name()?;
        if from_name != to_name {
            return None;
        }
        match (&old_dir, &new_dir) {
            (None, None) => {
                old_dir = Some(from_parent);
                new_dir = Some(to_parent);
            }
            (Some(o), Some(n)) => {
                if o != &from_parent || n != &to_parent {
                    return None;
                }
            }
            _ => return None,
        }
    }

    let from = old_dir?;
    let to = new_dir?;
    if from == to {
        return None;
    }
    Some(PathChange::DirMoved {
        from,
        to,
        disk_already_moved: true,
    })
}
