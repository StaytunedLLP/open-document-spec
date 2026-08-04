use crate::model::{FrontmatterState, Workspace};
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

/// Options for bounded context resolution.
#[derive(Clone, Debug, Default)]
pub struct ContextOptions {
    /// Include documents with `share: private`.
    pub include_private: bool,
    /// Walk `code:` edges (default false for AI context — source dumps burn tokens).
    pub include_code: bool,
    /// Soft token budget (bytes/4 heuristic). `None` = unlimited.
    pub max_tokens: Option<usize>,
}

/// Result of context resolution with diagnostics for agents/humans.
#[derive(Clone, Debug, Default)]
pub struct ContextResult {
    pub paths: Vec<PathBuf>,
    /// Document paths skipped because `share: private` (or org when filtered).
    pub skipped_private: Vec<PathBuf>,
    /// Estimated tokens of included paths (file size / 4).
    pub token_estimate: usize,
    /// True when `max_tokens` stopped expansion early.
    pub truncated: bool,
}

/// Resolve a bounded reading list (legacy API).
///
/// Code edges are **included** here for backward compatibility with existing
/// tests and graph tooling. Prefer [`resolve_context_with_options`] for AI use
/// (`include_code: false` by default).
pub fn resolve_context(workspace: &Workspace, query: &str, include_private: bool) -> Vec<PathBuf> {
    resolve_context_with_options(
        workspace,
        query,
        &ContextOptions {
            include_private,
            include_code: true,
            max_tokens: None,
        },
    )
    .paths
}

/// Resolve context with token budget, private-skip tracking, and optional code edges.
pub fn resolve_context_with_options(
    workspace: &Workspace,
    query: &str,
    options: &ContextOptions,
) -> ContextResult {
    let Some(start) = resolve_context_start(workspace, query) else {
        return ContextResult::default();
    };

    let mut queue = VecDeque::from([(start.clone(), 0usize)]);
    let mut visited = BTreeSet::<PathBuf>::new();
    let mut output = Vec::<PathBuf>::new();
    let mut skipped_private = Vec::<PathBuf>::new();
    let mut token_estimate = 0usize;
    let mut truncated = false;
    let max_depth = context_depth(workspace, &start).unwrap_or(2);
    let ignore_rules = context_ignore_rules(workspace, &start);

    while let Some((path, depth)) = queue.pop_front() {
        if is_ignored(&path, &workspace.root, &ignore_rules) {
            continue;
        }
        let is_private = workspace
            .document_by_path(&path)
            .and_then(|doc| frontmatter(doc))
            .is_some_and(|fm| fm.share.as_deref() == Some("private"));
        if !options.include_private && is_private {
            if visited.insert(path.clone()) {
                skipped_private.push(path);
            }
            continue;
        }
        if !visited.insert(path.clone()) {
            continue;
        }

        let file_tokens = estimate_path_tokens(&path);
        if let Some(budget) = options.max_tokens {
            if !output.is_empty() && token_estimate.saturating_add(file_tokens) > budget {
                truncated = true;
                continue;
            }
        }
        token_estimate = token_estimate.saturating_add(file_tokens);
        output.push(path.clone());

        if depth >= max_depth {
            continue;
        }

        let Some(document) = workspace.document_by_path(&path) else {
            continue;
        };
        let Some(frontmatter) = frontmatter(document) else {
            continue;
        };

        let mut next = frontmatter
            .depends
            .iter()
            .chain(frontmatter.context.iter().flat_map(|ctx| ctx.load.iter()))
            .filter_map(|reference| {
                if let Some(document_path) =
                    crate::refs::document_ref_to_path(workspace, document, reference)
                {
                    Some(document_path)
                } else if crate::refs::is_file_like_ref(reference) {
                    let resource_path =
                        crate::fs::normalize_join(&document.directory, Path::new(reference));
                    if resource_path.exists() {
                        Some(resource_path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .filter(|candidate| !is_ignored(candidate, &workspace.root, &ignore_rules))
            .collect::<Vec<_>>();
        next.sort();

        if options.include_code {
            let mut code_next = frontmatter
                .code
                .iter()
                .filter_map(|code| {
                    let code_path = crate::fs::normalize_join(&document.directory, &code.path);
                    if code_path.exists() && !is_ignored(&code_path, &workspace.root, &ignore_rules)
                    {
                        Some(code_path)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            code_next.sort();
            next.extend(code_next);
        }

        queue.extend(next.into_iter().map(|path| (path, depth + 1)));
    }

    ContextResult {
        paths: output,
        skipped_private,
        token_estimate,
        truncated,
    }
}

/// Rough token estimate: file bytes / 4 (same heuristic as bench).
pub fn estimate_path_tokens(path: &Path) -> usize {
    std::fs::metadata(path)
        .map(|m| (m.len() as usize) / 4)
        .unwrap_or(0)
}

/// Concatenate file bodies under a token budget for agent prompt packs.
pub fn render_context_pack(paths: &[PathBuf], max_tokens: Option<usize>) -> String {
    let mut out = String::new();
    let mut used = 0usize;
    for path in paths {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let header = format!("\n---\n# file: {}\n\n", path.display());
        let chunk_tokens = (header.len() + text.len()) / 4;
        if let Some(budget) = max_tokens {
            if used > 0 && used.saturating_add(chunk_tokens) > budget {
                out.push_str(&format!(
                    "\n---\n# truncated: remaining files omitted (budget ~{budget} tokens)\n"
                ));
                break;
            }
        }
        used = used.saturating_add(chunk_tokens);
        out.push_str(&header);
        out.push_str(&text);
        if !text.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

fn context_ignore_rules(workspace: &Workspace, path: &Path) -> Vec<String> {
    let Some(document) = workspace.document_by_path(path) else {
        return Vec::new();
    };
    let Some(frontmatter) = frontmatter(document) else {
        return Vec::new();
    };
    frontmatter
        .context
        .as_ref()
        .map(|ctx| ctx.ignore.clone())
        .unwrap_or_default()
}

fn is_ignored(path: &Path, root: &Path, rules: &[String]) -> bool {
    if rules.is_empty() {
        return false;
    }

    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let relative = relative.trim_start_matches("./");

    rules.iter().any(|rule| {
        let rule = rule.trim().trim_end_matches('/');
        if rule.is_empty() {
            return false;
        }
        relative == rule
            || relative.starts_with(&format!("{rule}/"))
            || relative.contains(&format!("/{rule}/"))
            || relative.ends_with(&format!("/{rule}"))
    })
}

fn frontmatter(document: &crate::model::Document) -> Option<&crate::model::Frontmatter> {
    match &document.frontmatter {
        FrontmatterState::Parsed(frontmatter) => Some(frontmatter),
        _ => None,
    }
}

fn context_depth(workspace: &Workspace, path: &Path) -> Option<usize> {
    let document = workspace.document_by_path(path)?;
    let frontmatter = frontmatter(document)?;
    frontmatter.context.as_ref()?.max_depth
}

/// Resolve a context query to a starting document path.
///
/// Accepts document ids (`specs/ods/core`), paths with or without `.md`, bare stems
/// (`core` when unique), and absolute paths under the workspace root.
pub fn resolve_context_start(workspace: &Workspace, query: &str) -> Option<PathBuf> {
    let raw = query.trim();
    if raw.is_empty() {
        return None;
    }
    let query_lc = raw.to_lowercase();
    let query_path = Path::new(raw);
    let id_query = query_lc
        .strip_suffix(".md")
        .unwrap_or(query_lc.as_str())
        .trim_end_matches('/')
        .to_string();

    // Exact id (path-shaped ids are stored lowercase without extension).
    if let Some(doc) = workspace.document_by_id(&id_query) {
        return Some(doc.path.clone());
    }

    // Absolute or workspace-relative filesystem path (with or without .md).
    let mut path_candidates = vec![query_path.to_path_buf(), workspace.root.join(query_path)];
    if query_path.extension().is_none() {
        let mut with_md = query_path.to_path_buf();
        with_md.set_extension("md");
        path_candidates.push(workspace.root.join(&with_md));
        path_candidates.push(with_md);
    }
    for candidate in path_candidates {
        let normalized = crate::fs::normalize_path(&candidate);
        if let Some(doc) = workspace.document_by_path(&normalized) {
            return Some(doc.path.clone());
        }
        if let Ok(canon) = normalized.canonicalize() {
            if let Some(doc) = workspace.document_by_path(&canon) {
                return Some(doc.path.clone());
            }
        }
    }

    // Absolute path under workspace → id form.
    if query_path.is_absolute() {
        if let Ok(rel) = query_path.strip_prefix(&workspace.root) {
            let rel_id = rel
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase();
            if let Some(doc) = workspace.document_by_id(&rel_id) {
                return Some(doc.path.clone());
            }
        }
        if let Ok(canon) = query_path.canonicalize() {
            if let Ok(rel) = canon.strip_prefix(&workspace.root) {
                let rel_id = rel
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_lowercase();
                if let Some(doc) = workspace.document_by_id(&rel_id) {
                    return Some(doc.path.clone());
                }
            }
        }
    }

    // Path suffix match (…/specs/ods/core.md) then unique file-stem match.
    if let Some(doc) = workspace.documents.iter().find(|doc| {
        doc.path.ends_with(query_path)
            || doc
                .path
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase()
                .ends_with(&query_lc)
            || doc
                .path
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase()
                .ends_with(&format!("{id_query}.md"))
    }) {
        return Some(doc.path.clone());
    }

    let stem_hits: Vec<_> = workspace
        .documents
        .iter()
        .filter(|doc| {
            doc.path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(&id_query))
        })
        .map(|doc| doc.path.clone())
        .collect();
    if stem_hits.len() == 1 {
        return Some(stem_hits.into_iter().next().unwrap());
    }

    None
}
