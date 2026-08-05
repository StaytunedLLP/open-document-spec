
/// Apply ref rewrites when files were already moved on disk (LSP watch path).
pub fn rewrite_refs_after_moves(
    root: impl AsRef<Path>,
    moves: &[(PathBuf, PathBuf)],
) -> io::Result<PathChangeReport> {
    let root = root.as_ref();
    let changes: Vec<PathChange> = moves
        .iter()
        .map(|(from, to)| {
            if to.is_dir() || (!to.exists() && from.extension().is_none()) {
                if to.is_dir() || from.extension().is_none() {
                    PathChange::DirMoved {
                        from: from.clone(),
                        to: to.clone(),
                        disk_already_moved: true,
                    }
                } else {
                    PathChange::FileMoved {
                        from: from.clone(),
                        to: to.clone(),
                        disk_already_moved: true,
                    }
                }
            } else {
                PathChange::FileMoved {
                    from: from.clone(),
                    to: to.clone(),
                    disk_already_moved: true,
                }
            }
        })
        .collect();
    apply_path_changes(root, &changes)
}

/// No-op: nested indexes removed. Kept for API callers that used to regenerate indexes.
pub fn reindex_workspace(root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    // Touch load so callers still validate the path is an ODS workspace.
    let _ = load_workspace(root.as_ref())?;
    Ok(Vec::new())
}

/// Collapse excess blank lines between closing frontmatter `---` and body to exactly one.
/// Idempotent. Leaves files without frontmatter unchanged (aside from no-op).
pub fn normalize_frontmatter_body_spacing(text: &str) -> String {
    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let Some(frontmatter) = frontmatter else {
        return text.to_string();
    };

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let body = strip_leading_blank_lines(body);
    if body.is_empty() {
        format!("---{ending}{frontmatter}{ending}---{ending}")
    } else {
        // Exactly one blank line after closing ---.
        format!("---{ending}{frontmatter}{ending}---{ending}{ending}{body}")
    }
}

/// Normalize spacing for every markdown document under `root` that has frontmatter.
/// Returns paths that were rewritten.
pub fn normalize_workspace_frontmatter_spacing(root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let workspace = load_workspace(root.as_ref())?;
    normalize_workspace_frontmatter_spacing_with_workspace(&workspace)
}

/// Same as [`normalize_workspace_frontmatter_spacing`], but takes an
/// already-loaded `Workspace` instead of reloading — safe to reuse across a
/// sequence of `fmt`-style operations since this only rewrites blank-line
/// spacing, never the semantic frontmatter fields a caller's workspace was
/// parsed from.
pub fn normalize_workspace_frontmatter_spacing_with_workspace(
    workspace: &crate::model::Workspace,
) -> io::Result<Vec<PathBuf>> {
    let mut changed = Vec::new();
    for document in &workspace.documents {
        let Ok(text) = fs::read_to_string(&document.path) else {
            continue;
        };
        let next = normalize_frontmatter_body_spacing(&text);
        if next != text {
            fs::write(&document.path, &next)?;
            changed.push(document.path.clone());
        }
    }
    Ok(changed)
}

/// Rewrite document references in frontmatter to editor-jumpable `.md` paths.
///
/// Only block-list entries in `depends`, `related`, and `context.load` are
/// touched. Resource paths, code paths, ids, prefixes, tags, and body prose are
/// left unchanged.
pub fn canonicalize_workspace_document_refs(root: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let workspace = load_workspace(root.as_ref())?;
    canonicalize_workspace_document_refs_with_workspace(&workspace)
}

/// Same as [`canonicalize_workspace_document_refs`], but takes an
/// already-loaded `Workspace` instead of reloading — safe to reuse after
/// [`normalize_workspace_frontmatter_spacing_with_workspace`] ran against the
/// same workspace, since that only rewrites blank-line spacing and never the
/// id/path/depends/related fields this function resolves refs against; each
/// document's text is still re-read fresh from disk below.
pub fn canonicalize_workspace_document_refs_with_workspace(
    workspace: &crate::model::Workspace,
) -> io::Result<Vec<PathBuf>> {
    let mut changed = Vec::new();

    for document in &workspace.documents {
        let text = match fs::read_to_string(&document.path) {
            Ok(text) => text,
            Err(_) => continue,
        };
        let next = canonicalize_document_refs_in_text(workspace, document, &text);
        if next != text {
            fs::write(&document.path, next)?;
            changed.push(document.path.clone());
        }
    }

    Ok(changed)
}

fn canonicalize_document_refs_in_text(
    workspace: &crate::model::Workspace,
    document: &crate::model::Document,
    text: &str,
) -> String {
    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let Some(frontmatter) = frontmatter else {
        return text.to_string();
    };

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines = Vec::new();
    let mut simple_list: Option<&str> = None;
    let mut in_context = false;
    let mut in_context_load = false;

    for line in frontmatter.lines() {
        let indent = line.chars().take_while(|ch| *ch == ' ').count();
        let trimmed = line.trim_start();

        if indent == 0 {
            simple_list = None;
            in_context_load = false;
            in_context = trimmed.starts_with("context:");
            if trimmed.starts_with("depends:") {
                simple_list = Some("depends");
            } else if trimmed.starts_with("related:") {
                simple_list = Some("related");
            }
            lines.push(line.to_string());
            continue;
        }

        if in_context && indent == 2 {
            in_context_load = trimmed.starts_with("load:");
            lines.push(line.to_string());
            continue;
        }

        let in_document_ref_list = (simple_list.is_some() && indent >= 2)
            || (in_context && in_context_load && indent >= 4);
        if in_document_ref_list
            && let Some(item) = trimmed.strip_prefix("- ")
            && let Some(canonical) =
                crate::refs::canonical_document_ref_for_reference(workspace, document, item.trim())
        {
            let prefix = &line[..line.len() - trimmed.len()];
            lines.push(format!("{prefix}- {canonical}"));
            continue;
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

pub fn rewrite_references_in_text(
    text: &str,
    old_id: &str,
    new_id: &str,
    old_target: &str,
    new_target: &str,
) -> String {
    if old_id.is_empty() && old_target.is_empty() {
        return normalize_frontmatter_body_spacing(text);
    }
    if old_id == new_id && old_target == new_target {
        return normalize_frontmatter_body_spacing(text);
    }

    let (frontmatter, body) = crate::parse::split_frontmatter(text);
    let Some(frontmatter) = frontmatter else {
        let rewritten = rewrite_document_body(text, old_id, new_id, old_target, new_target);
        return rewritten;
    };

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };

    let mut rewritten_frontmatter = String::new();
    let mut fm_lines = frontmatter.split('\n');
    if let Some(first) = fm_lines.next() {
        rewritten_frontmatter
            .push_str(&rewrite_line(first, old_id, new_id, old_target, new_target));
    }
    for line in fm_lines {
        rewritten_frontmatter.push('\n');
        rewritten_frontmatter.push_str(&rewrite_line(line, old_id, new_id, old_target, new_target));
    }

    let rewritten_body = rewrite_document_body(body, old_id, new_id, old_target, new_target);
    let body = strip_leading_blank_lines(&rewritten_body);
    if body.is_empty() {
        format!("---{ending}{rewritten_frontmatter}{ending}---{ending}")
    } else {
        format!("---{ending}{rewritten_frontmatter}{ending}---{ending}{ending}{body}")
    }
}

fn strip_leading_blank_lines(body: &str) -> String {
    let mut s = body;
    // Drop leading blank lines only (preserve intentional indentation on first content line).
    loop {
        if let Some(rest) = s.strip_prefix("\r\n") {
            s = rest;
            continue;
        }
        if let Some(rest) = s.strip_prefix('\n') {
            s = rest;
            continue;
        }
        if let Some(rest) = s.strip_prefix('\r') {
            s = rest;
            continue;
        }
        break;
    }
    s.to_string()
}

include!("classifier_rewriter.rs");
