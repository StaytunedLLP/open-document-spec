/// Compute document content after path changes **without writing** those documents.
///
/// Performs filesystem renames when `disk_already_moved` is false (same as
/// [`apply_path_changes`]), then returns `(report, path → new full text)` for every
/// markdown file whose content would change. Index regeneration is left to the
/// caller ([`apply_path_changes`] writes edits then calls [`generate_indexes`]).
///
/// Useful for LSP `workspace/willRenameFiles`: run against a temp copy, map the
/// returned paths back to the live pre-rename URIs, and return a `WorkspaceEdit`.
pub fn compute_path_change_edits(
    root: impl AsRef<Path>,
    changes: &[PathChange],
) -> io::Result<(PathChangeReport, Vec<(PathBuf, String)>)> {
    let original_root = root.as_ref().to_path_buf();
    let root = root
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root.as_ref().to_path_buf());
    let root = root.as_path();
    let mut report = PathChangeReport::default();
    if changes.is_empty() {
        return Ok((report, Vec::new()));
    }

    // Expand directory moves into file pairs (relative subpaths).
    let mut file_moves: Vec<(PathBuf, PathBuf, bool)> = Vec::new();
    let mut prefix_rewrites: Vec<(String, String)> = Vec::new();
    // old absolute path → new absolute path (md + any explicit file moves)
    let mut abs_moves: Vec<(PathBuf, PathBuf)> = Vec::new();

    for change in changes {
        match change {
            PathChange::FileMoved {
                from,
                to,
                disk_already_moved,
            } => {
                let from = normalize_change_path(&original_root, root, from);
                let to = normalize_change_path(&original_root, root, to);
                let norm_from = crate::fs::normalize_path(&from);
                let norm_to = crate::fs::normalize_path(&to);
                if !norm_from.starts_with(root) || !norm_to.starts_with(root) {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Path traversal attempt blocked: paths must remain within the workspace root",
                    ));
                }
                abs_moves.push((from.clone(), to.clone()));
                file_moves.push((from, to, *disk_already_moved));
            }
            PathChange::DirMoved {
                from,
                to,
                disk_already_moved,
            } => {
                let from = normalize_change_path(&original_root, root, from);
                let to = normalize_change_path(&original_root, root, to);
                let norm_from = crate::fs::normalize_path(&from);
                let norm_to = crate::fs::normalize_path(&to);
                if !norm_from.starts_with(root) || !norm_to.starts_with(root) {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied,
                        "Path traversal attempt blocked: paths must remain within the workspace root",
                    ));
                }
                let old_pref = rel_slash(root, &from);
                let new_pref = rel_slash(root, &to);
                if !old_pref.is_empty() {
                    prefix_rewrites.push((old_pref.clone(), new_pref.clone()));
                }
                abs_moves.push((from.clone(), to.clone()));

                if *disk_already_moved {
                    let mut md_files = Vec::new();
                    if let Err(err) = collect_md_files(&to, &mut md_files) {
                        report
                            .errors
                            .push(format!("list markdown under {}: {err}", to.display()));
                    }
                    for new_path in md_files {
                        let rel = new_path
                            .strip_prefix(&to)
                            .unwrap_or(Path::new(""))
                            .to_path_buf();
                        let old_path = from.join(&rel);
                        file_moves.push((old_path, new_path, true));
                    }
                } else {
                    let mut md_files = Vec::new();
                    if let Err(err) = collect_md_files(&from, &mut md_files) {
                        report
                            .errors
                            .push(format!("list markdown under {}: {err}", from.display()));
                    }
                    if let Some(parent) = to.parent()
                        && let Err(err) = fs::create_dir_all(parent)
                    {
                        report
                            .errors
                            .push(format!("create parent {}: {err}", parent.display()));
                        continue;
                    }
                    match fs::rename(&from, &to) {
                        Ok(()) => {
                            report.moves.push((from.clone(), to.clone()));
                            for old_path in md_files {
                                let rel = old_path
                                    .strip_prefix(&from)
                                    .unwrap_or(Path::new(""))
                                    .to_path_buf();
                                let new_path = to.join(&rel);
                                file_moves.push((old_path, new_path, true));
                            }
                        }
                        Err(err) => {
                            report.errors.push(format!(
                                "rename {} → {}: {err}",
                                from.display(),
                                to.display()
                            ));
                        }
                    }
                }
            }
        }
    }

    // Perform pending file renames on disk (or record moves already done by OS/editor).
    for (from, to, already) in &file_moves {
        if *already {
            // Disk already has `to`; still count as a move for reports / watch logs.
            report.moves.push((from.clone(), to.clone()));
            continue;
        }
        if !from.exists() {
            report
                .warnings
                .push(format!("skip missing source {}", from.display()));
            continue;
        }
        if let Some(parent) = to.parent()
            && let Err(err) = fs::create_dir_all(parent)
        {
            report
                .errors
                .push(format!("create parent {}: {err}", parent.display()));
            continue;
        }
        match fs::rename(from, to) {
            Ok(()) => report.moves.push((from.clone(), to.clone())),
            Err(err) => report.errors.push(format!(
                "rename {} → {}: {err}",
                from.display(),
                to.display()
            )),
        }
    }

    // Build id/path substitution pairs (longest prefixes first for dirs).
    prefix_rewrites.sort_by_key(|b| std::cmp::Reverse(b.0.len()));
    let mut id_pairs: Vec<(String, String)> = Vec::new();
    let mut path_pairs: Vec<(String, String)> = Vec::new();

    for (from, to, _) in &file_moves {
        let old_rel = rel_slash(root, from);
        let new_rel = rel_slash(root, to);
        // Only path-id rewrite for markdown documents.
        if from.extension().is_some_and(|e| e == "md") || to.extension().is_some_and(|e| e == "md")
        {
            let old_id = path_to_default_id(&old_rel);
            let new_id = path_to_default_id(&new_rel);
            if old_id != new_id {
                id_pairs.push((old_id, new_id));
            }
        }
        if old_rel != new_rel {
            path_pairs.push((old_rel.clone(), new_rel.clone()));
        }
    }

    for (old_p, new_p) in &prefix_rewrites {
        id_pairs.push((old_p.to_lowercase(), new_p.to_lowercase()));
        path_pairs.push((old_p.clone(), new_p.clone()));
    }

    // Longest id/path first so nested prefixes replace correctly.
    id_pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    id_pairs.dedup();
    path_pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    path_pairs.dedup();

    // Map new doc path → old parent dir (for recomputing relative resource paths).
    let mut old_dir_for_new: std::collections::HashMap<PathBuf, PathBuf> =
        std::collections::HashMap::new();
    for (from, to, _) in &file_moves {
        if let (Some(old_dir), Some(_)) = (from.parent(), to.parent()) {
            old_dir_for_new.insert(to.clone(), old_dir.to_path_buf());
        }
    }

    // Compute rewrites for every markdown document (respect ignore via load).
    let workspace = match load_workspace(root) {
        Ok(ws) => ws,
        Err(err) => {
            report.errors.push(format!("load workspace: {err}"));
            return Ok((report, Vec::new()));
        }
    };
    let mut edits = Vec::new();
    let mut rewritten = BTreeSet::new();

    for document in &workspace.documents {
        let path = &document.path;
        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(err) => {
                report
                    .warnings
                    .push(format!("skip unreadable {}: {err}", path.display()));
                continue;
            }
        };
        let doc_dir = path.parent().unwrap_or(root);
        let resolve_dir = old_dir_for_new
            .get(path)
            .map(|p| p.as_path())
            .unwrap_or(doc_dir);
        let mut next = text.clone();
        for (old_id, new_id) in &id_pairs {
            let old_target = format!("{old_id}.md");
            let new_target = format!("{new_id}.md");
            next = rewrite_references_in_text(&next, old_id, new_id, &old_target, &new_target);
        }
        for (old_path, new_path) in &path_pairs {
            next = rewrite_path_prefix_in_text(&next, old_path, new_path);
        }
        if !path_pairs.is_empty() {
            next = rewrite_frontmatter_document_ref_paths_in_text(&next, doc_dir, root, &path_pairs);
            next = rewrite_relative_links_in_text(&next, doc_dir, root, &path_pairs);
            next = rewrite_resource_paths_in_text(
                &next,
                doc_dir,
                resolve_dir,
                root,
                &path_pairs,
                &abs_moves,
            );
        }
        // Always normalize frontmatter/body spacing when path ops touch the workspace.
        next = normalize_frontmatter_body_spacing(&next);
        if next != text {
            rewritten.insert(path.clone());
            edits.push((path.clone(), next));
        }
    }

    // Force path-derived `id:` on each moved markdown destination (content parser
    // view of identity must match the file path after rename).
    for (from, to, _) in &file_moves {
        if to.extension().and_then(|e| e.to_str()) != Some("md")
            && from.extension().and_then(|e| e.to_str()) != Some("md")
        {
            continue;
        }
        if to.file_name().and_then(|n| n.to_str()) == Some("index.md") {
            continue;
        }
        let new_id = path_to_default_id(&rel_slash(root, to));
        let idx = edits.iter().position(|(p, _)| p == to);
        let (base_text, from_disk) = if let Some(i) = idx {
            (edits[i].1.clone(), false)
        } else if to.is_file() {
            match fs::read_to_string(to) {
                Ok(t) => (t, true),
                Err(_) => continue,
            }
        } else {
            continue;
        };
        let forced = force_frontmatter_id(&base_text, &new_id);
        let forced = normalize_frontmatter_body_spacing(&forced);
        if forced != base_text || from_disk {
            if let Some(i) = idx {
                edits[i].1 = forced;
            } else {
                rewritten.insert(to.clone());
                edits.push((to.clone(), forced));
            }
            rewritten.insert(to.clone());
        }
    }

    report.rewritten_files = rewritten.into_iter().collect();
    Ok((report, edits))
}
