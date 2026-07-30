/// Rewrite frontmatter `path:` values (resources) after moves.
///
/// `resolve_dir` is the directory used to interpret existing relative paths
/// (the **pre-move** document directory when the doc itself moved).
/// `doc_dir` is the current on-disk directory (post-move) used to emit new relatives.
fn rewrite_resource_paths_in_text(
    text: &str,
    doc_dir: &Path,
    resolve_dir: &Path,
    root: &Path,
    path_pairs: &[(String, String)],
    abs_moves: &[(PathBuf, PathBuf)],
) -> String {
    let mut lines = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim_start();
        // Accept both `path: …` and list form `- path: …`
        let (prefix, rest) = if let Some(rest) = trimmed.strip_prefix("- path:") {
            ("- path:", rest)
        } else if let Some(rest) = trimmed.strip_prefix("path:") {
            ("path:", rest)
        } else {
            lines.push(line.to_string());
            continue;
        };
        let val = rest.trim();
        if val.is_empty() || val.contains("://") {
            lines.push(line.to_string());
            continue;
        }
        let resolved = normalize_join(resolve_dir, val);
        let mapped = map_abs_through_moves(&resolved, abs_moves, path_pairs, root);
        let new_rel = relative_path(doc_dir, &mapped);
        if new_rel == val {
            lines.push(line.to_string());
            continue;
        }
        let indent = &line[..line.len() - trimmed.len()];
        lines.push(format!("{indent}{prefix} {new_rel}"));
    }
    let mut joined = lines.join("\n");
    if text.ends_with('\n') && !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

fn rewrite_frontmatter_document_ref_paths_in_text(
    text: &str,
    doc_dir: &Path,
    root: &Path,
    path_pairs: &[(String, String)],
) -> String {
    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let Some(frontmatter) = frontmatter else {
        return text.to_string();
    };

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines = Vec::new();
    let mut simple_list = false;
    let mut in_context = false;
    let mut in_context_load = false;

    for line in frontmatter.lines() {
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let trimmed = line.trim_start();
        if indent == 0 {
            simple_list = trimmed.starts_with("depends:") || trimmed.starts_with("related:");
            in_context = trimmed.starts_with("context:");
            in_context_load = false;
            lines.push(line.to_string());
            continue;
        }
        if in_context && indent == 2 {
            in_context_load = trimmed.starts_with("load:");
            lines.push(line.to_string());
            continue;
        }

        let in_ref_list =
            (simple_list && indent >= 2) || (in_context && in_context_load && indent >= 4);
        if in_ref_list
            && let Some(item) = trimmed.strip_prefix("- ")
        {
            let value = item.trim();
            let rewritten_value = rewrite_one_link_target(value, doc_dir, root, path_pairs);
            if rewritten_value != value {
                let prefix = &line[..line.len() - trimmed.len()];
                lines.push(format!("{prefix}- {rewritten_value}"));
                continue;
            }
        }
        lines.push(line.to_string());
    }

    let frontmatter = lines.join("\n");
    if body.is_empty() {
        format!("---{ending}{frontmatter}{ending}---{ending}")
    } else {
        format!("---{ending}{frontmatter}{ending}---{ending}{body}")
    }
}

fn map_abs_through_moves(
    abs: &Path,
    abs_moves: &[(PathBuf, PathBuf)],
    path_pairs: &[(String, String)],
    root: &Path,
) -> PathBuf {
    // Exact absolute move
    for (from, to) in abs_moves {
        if abs == from {
            return to.clone();
        }
        if abs.starts_with(from)
            && let Ok(rel) = abs.strip_prefix(from)
        {
            return to.join(rel);
        }
    }
    // Workspace-relative path pairs (longest first — caller sorts)
    let rel = rel_slash(root, abs);
    for (old_p, new_p) in path_pairs {
        if rel == *old_p {
            return root.join(new_p);
        }
        if let Some(suffix) = rel.strip_prefix(&format!("{old_p}/")) {
            return root.join(new_p).join(suffix);
        }
    }
    abs.to_path_buf()
}

pub(super) fn rewrite_one_link_target(
    target: &str,
    doc_dir: &Path,
    root: &Path,
    path_pairs: &[(String, String)],
) -> String {
    let target = target.trim();
    if target.is_empty() {
        return target.to_string();
    }
    // Skip URLs and anchors-only / absolute site paths that are not relative files.
    if target.contains("://") || target.starts_with('#') || target.starts_with('/') {
        return target.to_string();
    }

    let (path_part, fragment) = match target.split_once('#') {
        Some((p, f)) => (p, Some(f)),
        None => (target, None),
    };
    if path_part.is_empty() {
        return target.to_string();
    }

    // Only rewrite relative-looking paths (./ ../ or plain relative with no scheme).
    let resolved = normalize_join(doc_dir, path_part);
    let Ok(rel_to_root) = resolved.strip_prefix(root) else {
        // Outside workspace — leave alone.
        return target.to_string();
    };
    let resolved_rel = rel_to_root.to_string_lossy().replace('\\', "/");

    // Match longest path pair (file exact or directory prefix).
    let mut best: Option<(String, String)> = None;
    for (old_p, new_p) in path_pairs {
        let old_id = path_to_default_id(old_p);
        let resolved_id = path_to_default_id(&resolved_rel);
        let matches = resolved_rel == *old_p
            || resolved_id == old_id
            || resolved_rel.starts_with(&format!("{old_p}/"))
            || resolved_id.starts_with(&format!("{old_id}/"));
        if !matches {
            continue;
        }
        if best.as_ref().is_none_or(|(b, _)| old_p.len() > b.len()) {
            best = Some((old_p.clone(), new_p.clone()));
        }
    }

    let Some((old_p, new_p)) = best else {
        return target.to_string();
    };

    let old_id = path_to_default_id(&old_p);
    let new_id = path_to_default_id(&new_p);
    let resolved_id = path_to_default_id(&resolved_rel);

    let new_resolved_rel = if resolved_rel == old_p || resolved_id == old_id {
        // File-level: prefer keeping .md if original target had it.
        if path_part.ends_with(".md") {
            if new_p.ends_with(".md") {
                new_p.clone()
            } else {
                format!("{new_p}.md")
            }
        } else if new_p.ends_with(".md") {
            path_to_default_id(&new_p)
        } else {
            new_p.clone()
        }
    } else if let Some(suffix) = resolved_rel.strip_prefix(&format!("{old_p}/")) {
        format!("{new_p}/{suffix}")
    } else if let Some(suffix) = resolved_id.strip_prefix(&format!("{old_id}/")) {
        let base = format!("{new_id}/{suffix}");
        if path_part.ends_with(".md") && !base.ends_with(".md") {
            format!("{base}.md")
        } else {
            base
        }
    } else {
        return target.to_string();
    };

    let new_abs = root.join(Path::new(&new_resolved_rel));
    let new_rel = relative_path(doc_dir, &new_abs);
    let mut result = new_rel;
    if let Some(frag) = fragment {
        result.push('#');
        result.push_str(frag);
    }
    result
}

fn normalize_join(base: &Path, rel: &str) -> PathBuf {
    let mut out = base.to_path_buf();
    for comp in Path::new(rel).components() {
        match comp {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = out.pop();
            }
            Component::Normal(s) => out.push(s),
            Component::RootDir | Component::Prefix(_) => {
                out = PathBuf::from(rel);
                break;
            }
        }
    }
    out
}

fn relative_path(from_dir: &Path, to: &Path) -> String {
    let from_components: Vec<_> = from_dir.components().collect();
    let to_components: Vec<_> = to.components().collect();
    let mut i = 0;
    while i < from_components.len()
        && i < to_components.len()
        && from_components[i] == to_components[i]
    {
        i += 1;
    }
    let mut parts: Vec<String> = Vec::new();
    for _ in i..from_components.len() {
        parts.push("..".to_string());
    }
    for c in &to_components[i..] {
        if let Component::Normal(s) = c {
            parts.push(s.to_string_lossy().to_string());
        }
    }
    if parts.is_empty() {
        to.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    } else {
        parts.join("/")
    }
}

fn rewrite_document_body(
    body: &str,
    old_id: &str,
    new_id: &str,
    old_target: &str,
    new_target: &str,
) -> String {
    let mut rewritten = String::new();
    let mut body_lines = body.split('\n');
    if let Some(first) = body_lines.next() {
        rewritten.push_str(&rewrite_line(first, old_id, new_id, old_target, new_target));
    }
    for line in body_lines {
        rewritten.push('\n');
        rewritten.push_str(&rewrite_line(line, old_id, new_id, old_target, new_target));
    }
    rewritten
}
