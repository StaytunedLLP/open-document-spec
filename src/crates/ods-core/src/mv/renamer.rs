/// Reconcile path-shaped frontmatter `id:` with each file's path-derived id.
///
/// When a document's `id:` looks path-shaped (contains `/`) but no file currently
/// owns that path-id, rewrite the id to the path-derived value and update inbound
/// depends/related/body links. Stable handles without `/` are left alone.
pub fn heal_orphan_path_ids(root: impl AsRef<Path>) -> io::Result<PathChangeReport> {
    let root = root
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root.as_ref().to_path_buf());
    let root = root.as_path();
    let mut report = PathChangeReport::default();
    let workspace = load_workspace(root)?;

    // Path-derived ids currently claimed by a real file.
    let mut owned_path_ids: BTreeSet<String> = BTreeSet::new();
    for document in &workspace.documents {
        if document.path.file_name().and_then(|n| n.to_str()) == Some("index.md") {
            continue;
        }
        owned_path_ids.insert(path_to_default_id(&rel_slash(root, &document.path)));
    }

    // Collect (doc_path, old_id, new_id) heals.
    let mut heals: Vec<(PathBuf, String, String)> = Vec::new();
    for document in &workspace.documents {
        if document.path.file_name().and_then(|n| n.to_str()) == Some("index.md") {
            continue;
        }
        let fm = match &document.frontmatter {
            crate::model::FrontmatterState::Parsed(fm) => fm,
            _ => continue,
        };
        let Some(raw_id) = fm.id.as_ref() else {
            continue;
        };
        let old_id = raw_id.replace('\\', "/").to_lowercase();
        // Path-shaped only (directory/id form). Bare tokens stay stable handles.
        if !old_id.contains('/') {
            continue;
        }
        let new_id = path_to_default_id(&rel_slash(root, &document.path));
        if old_id == new_id {
            continue;
        }
        // Only heal when the stated id is not owned by any other path (orphan after rename).
        if owned_path_ids.contains(&old_id) {
            continue;
        }
        heals.push((document.path.clone(), old_id, new_id));
    }

    if heals.is_empty() {
        let indexes = generate_indexes(&workspace)?;
        report.indexes = indexes;
        return Ok(report);
    }

    // Union of id rewrites for every document body/frontmatter.
    let mut id_pairs: Vec<(String, String)> = heals
        .iter()
        .map(|(_, o, n)| (o.clone(), n.clone()))
        .collect();
    id_pairs.sort_by(|a, b| b.0.len().cmp(&a.0.len()).then_with(|| a.0.cmp(&b.0)));
    id_pairs.dedup();

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
        let mut next = text.clone();
        for (old_id, new_id) in &id_pairs {
            let old_target = format!("{old_id}.md");
            let new_target = format!("{new_id}.md");
            next = rewrite_references_in_text(&next, old_id, new_id, &old_target, &new_target);
        }
        // Force the healed document's own id line even if rewrite missed casing/spacing.
        if let Some((_, _, new_id)) = heals.iter().find(|(p, _, _)| p == path) {
            next = force_frontmatter_id(&next, new_id);
        }
        next = normalize_frontmatter_body_spacing(&next);
        if next != text {
            match fs::write(path, &next) {
                Ok(()) => report.rewritten_files.push(path.clone()),
                Err(err) => report
                    .errors
                    .push(format!("write {}: {err}", path.display())),
            }
        }
    }

    report.rewritten_files.sort();
    report.rewritten_files.dedup();
    match load_workspace(root) {
        Ok(ws) => match generate_indexes(&ws) {
            Ok(indexes) => report.indexes = indexes,
            Err(err) => report.errors.push(format!("regenerate indexes: {err}")),
        },
        Err(err) => report.errors.push(format!("reload after heal: {err}")),
    }
    Ok(report)
}

/// Set or replace the frontmatter `id:` line.
fn force_frontmatter_id(text: &str, new_id: &str) -> String {
    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let Some(fm) = frontmatter else {
        return text.to_string();
    };
    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = fm.lines().map(|l| l.to_string()).collect();
    let mut found = false;
    for line in &mut lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("id:") {
            let indent = &line[..line.len() - trimmed.len()];
            *line = format!("{indent}id: {new_id}");
            found = true;
            break;
        }
    }
    if !found {
        // Insert after profile if present, else at top of frontmatter.
        let mut insert_at = 0;
        for (i, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("profile:") {
                insert_at = i + 1;
                break;
            }
        }
        lines.insert(insert_at, format!("id: {new_id}"));
    }
    let rewritten_fm = lines.join("\n");
    let body = strip_leading_blank_lines(body);
    if body.is_empty() {
        format!("---{ending}{rewritten_fm}{ending}---{ending}")
    } else {
        format!("---{ending}{rewritten_fm}{ending}---{ending}{ending}{body}")
    }
}
