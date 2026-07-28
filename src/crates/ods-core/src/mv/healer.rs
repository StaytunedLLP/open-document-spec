fn rewrite_line(
    line: &str,
    old_id: &str,
    new_id: &str,
    old_target: &str,
    new_target: &str,
) -> String {
    if old_id.is_empty() && old_target.is_empty() {
        return line.to_string();
    }
    let trimmed = line.trim_start();
    let mut rewritten = line.to_string();

    if !old_id.is_empty() {
        if trimmed.starts_with("- ") {
            let val = trimmed.trim_start_matches("- ").trim();
            if val == old_id || val.starts_with(&format!("{old_id}/")) {
                rewritten = rewritten.replacen(old_id, new_id, 1);
            }
        }

        // Path-derived identity lines only — exact match for full id.
        if trimmed.starts_with("id:")
            && trimmed
                .split_once(':')
                .is_some_and(|(_, value)| value.trim() == old_id)
        {
            rewritten = rewritten.replacen(old_id, new_id, 1);
        }

        rewritten = rewritten.replace(&format!("]({old_id})"), &format!("]({new_id})"));
        rewritten = rewritten.replace(&format!("]({old_id}.md)"), &format!("]({new_id}.md)"));
        rewritten = rewritten.replace(&format!("]({old_id}/"), &format!("]({new_id}/"));
    }

    if !old_target.is_empty() {
        if trimmed.starts_with("- ") {
            let val = trimmed.trim_start_matches("- ").trim();
            if val == old_target || val.starts_with(&format!("{old_target}/")) {
                rewritten = rewritten.replacen(old_target, new_target, 1);
            }
        }
        rewritten = rewritten.replace(&format!("]({old_target})"), &format!("]({new_target})"));
    }
    rewritten
}

fn path_to_default_id(rel: &str) -> String {
    let rel = rel.replace('\\', "/");
    let without = rel.strip_suffix(".md").unwrap_or(&rel);
    without.to_lowercase()
}

fn rel_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_string()
}

fn absolutize(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn normalize_change_path(original_root: &Path, canonical_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        if let Ok(relative) = path.strip_prefix(original_root) {
            canonical_root.join(relative)
        } else if let Ok(canonical) = path.canonicalize() {
            canonical
        } else if let (Some(parent), Some(name)) = (path.parent(), path.file_name()) {
            parent
                .canonicalize()
                .map(|parent| parent.join(name))
                .unwrap_or_else(|_| path.to_path_buf())
        } else {
            path.to_path_buf()
        }
    } else {
        canonical_root.join(path)
    }
}

fn collect_md_files(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with('.') {
            continue;
        }
        if path.is_dir() {
            collect_md_files(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}

/// Classify filesystem watch events into path changes (for LSP).
///
/// `events`: list of (path, type) where type 1=Created 2=Changed 3=Deleted.
///
/// `final_pass`: when `false` (events may still be trickling in across
/// separate notifications), only a *complete* directory move — every deleted
/// markdown file paired with a matching created one — is reported. The
/// looser per-file fallback heuristics below are held back until `final_pass`
/// is `true` (the debounce window elapsed / caller is settling), so that a
/// same-named sibling arriving early during a folder rename isn't peeled off
/// and committed as a lone `FileMoved` before the rest of the directory's
/// events show up — which would otherwise leave the remaining files'
/// references (depends/related ids, relative links) unrewritten, since the
/// caller drops whatever it doesn't consume here.
pub fn classify_watch_events(
    root: &Path,
    events: &[(PathBuf, u8)],
    final_pass: bool,
) -> Vec<PathChange> {
    let mut deleted: Vec<PathBuf> = Vec::new();
    let mut created: Vec<PathBuf> = Vec::new();

    for (path, kind) in events {
        match kind {
            3 => deleted.push(path.clone()),
            1 => created.push(path.clone()),
            _ => {}
        }
    }

    let mut changes = Vec::new();
    let mut used_deleted = BTreeSet::new();
    let mut used_created = BTreeSet::new();

    // Directory moves: group by parent-of-file relative structure.
    if let Some((from_dir, to_dir, pairs)) = match_dir_move(root, &deleted, &created) {
        for (d, c) in &pairs {
            used_deleted.insert(d.clone());
            used_created.insert(c.clone());
        }
        changes.push(PathChange::DirMoved {
            from: from_dir,
            to: to_dir,
            disk_already_moved: true,
        });
    }

    if !final_pass {
        let _ = root;
        return changes;
    }

    // File-level pairing: same file name different parent, or single remaining pair.
    for d in &deleted {
        if used_deleted.contains(d) {
            continue;
        }
        if d.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let d_name = d.file_name();
        let mut best: Option<PathBuf> = None;
        for c in &created {
            if used_created.contains(c) {
                continue;
            }
            if c.file_name() == d_name {
                best = Some(c.clone());
                break;
            }
        }
        if best.is_none() {
            let remaining_c: Vec<_> = created
                .iter()
                .filter(|c| !used_created.contains(*c) && c.extension().is_some_and(|e| e == "md"))
                .cloned()
                .collect();
            let remaining_d: Vec<_> = deleted
                .iter()
                .filter(|x| !used_deleted.contains(*x) && x.extension().is_some_and(|e| e == "md"))
                .cloned()
                .collect();
            if remaining_d.len() == 1 && remaining_c.len() == 1 && remaining_d[0] == *d {
                best = Some(remaining_c[0].clone());
            }
        }
        if let Some(c) = best {
            used_deleted.insert(d.clone());
            used_created.insert(c.clone());
            changes.push(PathChange::FileMoved {
                from: d.clone(),
                to: c,
                disk_already_moved: true,
            });
        }
    }

    let _ = root;
    changes
}

#[allow(clippy::type_complexity)]
fn match_dir_move(
    root: &Path,
    deleted: &[PathBuf],
    created: &[PathBuf],
) -> Option<(PathBuf, PathBuf, Vec<(PathBuf, PathBuf)>)> {
    let del_md: Vec<_> = deleted
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .cloned()
        .collect();
    let cre_md: Vec<_> = created
        .iter()
        .filter(|p| p.extension().is_some_and(|e| e == "md"))
        .cloned()
        .collect();
    if del_md.is_empty() || del_md.len() != cre_md.len() {
        return None;
    }

    let del_parent = common_parent(&del_md)?;
    let cre_parent = common_parent(&cre_md)?;
    if del_parent == cre_parent {
        return None;
    }

    // Single-file: only treat as directory move when the source parent is gone
    // (folder rename). If the parent still exists, this is a file move into another folder.
    if del_md.len() == 1 {
        let still_has_md = del_parent.is_dir()
            && fs::read_dir(&del_parent)
                .ok()
                .map(|rd| {
                    rd.filter_map(|e| e.ok()).any(|e| {
                        e.path()
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                    })
                })
                .unwrap_or(false);
        if still_has_md || del_parent.exists() && del_parent.is_dir() {
            // Parent directory still present → not a full dir rename.
            // Exception: empty dir left behind without md still means file move.
            if del_parent.exists() {
                return None;
            }
        }
    }

    let mut pairs = Vec::new();
    for d in &del_md {
        let rel = d.strip_prefix(&del_parent).ok()?;
        let c = cre_parent.join(rel);
        if !cre_md.iter().any(|x| x == &c) {
            return None;
        }
        pairs.push((d.clone(), c));
    }

    let _ = root;
    Some((del_parent, cre_parent, pairs))
}
