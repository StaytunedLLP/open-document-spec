// Tags: normalize, built-in suggestions, project index, query, rename.
//
// Project tags are observed from document frontmatter. Default ODS tags are a
// soft built-in suggestion set for completions/docs only (never required).

use crate::model::{Diagnostic, Document, FrontmatterState, Severity, Workspace};
use crate::parse::{document_id, split_frontmatter};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::PathBuf;
/// Soft built-in suggestion palette. Never required; free-form tags remain valid.
pub fn builtin_tags() -> &'static [&'static str] {
    &[
        "oncall",
        "security",
        "compliance",
        "billing",
        "customer-care",
        "internal",
        "public",
    ]
}

/// Normalize a tag for comparison and storage: trim, lowercase.
/// Returns `None` if empty after trim.
pub fn normalize_tag(raw: &str) -> Option<String> {
    let t = raw.trim().to_lowercase();
    if t.is_empty() { None } else { Some(t) }
}

/// Normalize a list: trim, lowercase, drop empties, dedupe (first wins).
pub fn normalize_tag_list(raw: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for item in raw {
        if let Some(n) = normalize_tag(item.as_ref())
            && seen.insert(n.clone())
        {
            out.push(n);
        }
    }
    out
}

/// Build tag → document ids from a workspace's documents.
pub fn build_tag_index(workspace: &Workspace) -> BTreeMap<String, Vec<String>> {
    let mut index = BTreeMap::<String, Vec<String>>::new();
    for document in &workspace.documents {
        let FrontmatterState::Parsed(fm) = &document.frontmatter else {
            continue;
        };
        if fm.tags.is_empty() {
            continue;
        }
        let id = document_id(&workspace.root, &document.path, Some(fm));
        for tag in &fm.tags {
            index.entry(tag.clone()).or_default().push(id.clone());
        }
    }
    for docs in index.values_mut() {
        docs.sort();
        docs.dedup();
    }
    index
}

/// Sorted unique project tags (observed).
pub fn observed_tags(workspace: &Workspace) -> Vec<String> {
    workspace.tag_index.keys().cloned().collect()
}

/// Tags for completions: observed (by frequency desc) then unused builtins.
pub fn completion_tags(workspace: &Workspace) -> Vec<String> {
    let mut scored: Vec<(String, usize)> = workspace
        .tag_index
        .iter()
        .map(|(tag, docs)| (tag.clone(), docs.len()))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let mut out: Vec<String> = scored.into_iter().map(|(t, _)| t).collect();
    let seen: BTreeSet<String> = out.iter().cloned().collect();
    for tag in builtin_tags() {
        if !seen.contains(*tag) {
            out.push((*tag).to_string());
        }
    }
    out
}

/// Document ids that carry `tag` (normalized match).
pub fn docs_with_tag(workspace: &Workspace, tag: &str) -> Vec<String> {
    let Some(key) = normalize_tag(tag) else {
        return Vec::new();
    };
    workspace.tag_index.get(&key).cloned().unwrap_or_default()
}

/// Document ids matching any of the tags (OR).
pub fn docs_with_any_tag(workspace: &Workspace, tags: &[String]) -> Vec<String> {
    let mut set = BTreeSet::new();
    for tag in tags {
        for id in docs_with_tag(workspace, tag) {
            set.insert(id);
        }
    }
    set.into_iter().collect()
}

/// Document ids matching ALL of the tags (AND).
pub fn docs_with_all_tags(workspace: &Workspace, tags: &[String]) -> Vec<String> {
    if tags.is_empty() {
        return Vec::new();
    }
    let mut iter = tags.iter();
    let first = iter.next().unwrap();
    let mut set: BTreeSet<String> = docs_with_tag(workspace, first).into_iter().collect();
    for tag in iter {
        let tag_docs: BTreeSet<String> = docs_with_tag(workspace, tag).into_iter().collect();
        set.retain(|id| tag_docs.contains(id));
    }
    set.into_iter().collect()
}

/// (tag, count) for observed tags, sorted by count desc then name.
pub fn tag_usage(workspace: &Workspace) -> Vec<(String, usize)> {
    let mut rows: Vec<(String, usize)> = workspace
        .tag_index
        .iter()
        .map(|(t, docs)| (t.clone(), docs.len()))
        .collect();
    rows.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    rows
}

/// Observed usage plus unused builtins when `include_unused_builtins`.
pub fn tag_usage_with_builtins(
    workspace: &Workspace,
    include_unused_builtins: bool,
) -> Vec<(String, usize, bool)> {
    let mut rows: Vec<(String, usize, bool)> = tag_usage(workspace)
        .into_iter()
        .map(|(t, c)| (t, c, false))
        .collect();
    if include_unused_builtins {
        let seen: BTreeSet<String> = rows.iter().map(|(t, _, _)| t.clone()).collect();
        for tag in builtin_tags() {
            if !seen.contains(*tag) {
                rows.push(((*tag).to_string(), 0, true));
            }
        }
    }
    rows
}

/// Hygiene diagnostics for one document's tags (Level 1+).
///
/// Tags are **root-level only**. Nested `tags` under `ods:` sets
/// [`Frontmatter::tags_misplaced`] and yields a placement warning; run
/// `ods fmt --migrate` to hoist them to top-level for SSG/tool interop.
///
/// Profile-name collisions use the workspace catalog (standard ∪ custom).
pub fn lint_document_tags(document: &Document, workspace: &Workspace) -> Vec<Diagnostic> {
    let FrontmatterState::Parsed(fm) = &document.frontmatter else {
        return Vec::new();
    };
    let mut diagnostics = Vec::new();

    if fm.tags_misplaced {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Warning,
            message: crate::error::lint_tags_misplaced(),
        });
    }

    // Tags are already normalized on parse; detect issues from stored values.
    let mut seen = BTreeSet::new();
    for tag in &fm.tags {
        if !seen.insert(tag.clone()) {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Warning,
                message: crate::error::lint_duplicate_tag(tag),
            });
        }
        if tag.contains(' ') {
            let suggested = tag.replace(' ', "-");
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Warning,
                message: crate::error::lint_tag_has_spaces(tag, &suggested),
            });
        }
        if is_reserved_status_token(tag) {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Warning,
                message: crate::error::lint_tag_collides_status(tag),
            });
        }
        if is_profile_name(tag, workspace) {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Warning,
                message: crate::error::lint_tag_collides_profile(tag),
            });
        }
    }
    diagnostics
}

fn is_reserved_status_token(tag: &str) -> bool {
    matches!(tag, "draft" | "stable" | "deprecated" | "archived")
}

fn is_standard_profile_name(tag: &str) -> bool {
    matches!(
        tag,
        "note"
            | "feature"
            | "guide"
            | "api"
            | "architecture"
            | "decision"
            | "sop"
            | "policy"
            | "meeting"
            | "faq"
            | "checklist"
            | "index"
    )
}

/// True if `tag` matches a standard profile name or a workspace-registered custom profile.
fn is_profile_name(tag: &str, workspace: &Workspace) -> bool {
    if is_standard_profile_name(tag) {
        return true;
    }
    workspace.profiles.definitions.contains_key(tag)
}

#[cfg(test)]
mod test_tags_catalog {
    use super::*;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn test_tags_catalog_functions() {
        assert!(!builtin_tags().is_empty());

        assert_eq!(normalize_tag("  Security "), Some("security".into()));
        assert_eq!(normalize_tag("   "), None);

        let list = normalize_tag_list(vec!["   ", "OnCall", "oncall", "Security"]);
        assert_eq!(list, vec!["oncall", "security"]);

        let td = tempdir().unwrap();
        let root = td.path();
        std::fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();
        std::fs::write(
            root.join("doc1.md"),
            "---\nprofile: note\nstatus: draft\ntags:\n  - oncall\n  - oncall\n  - tag with space\n  - draft\n  - note\n---\n\n# Doc1\n",
        )
        .unwrap();

        let workspace = load_workspace(root).unwrap();

        let obs = observed_tags(&workspace);
        assert!(obs.contains(&"oncall".to_string()));

        let comp = completion_tags(&workspace);
        assert!(comp.contains(&"security".to_string()));

        let with_tag = docs_with_tag(&workspace, "oncall");
        assert!(!with_tag.is_empty());
        assert!(docs_with_tag(&workspace, "   ").is_empty());

        let any_tag = docs_with_any_tag(&workspace, &["oncall".into(), "nonexistent".into()]);
        assert!(!any_tag.is_empty());

        let usage = tag_usage(&workspace);
        assert!(!usage.is_empty());

        let usage_builtins = tag_usage_with_builtins(&workspace, true);
        assert!(usage_builtins.iter().any(|(_, _, unused)| *unused));

        if let Some(doc1) = workspace.document_by_path(&root.join("doc1.md")) {
            let diags = lint_document_tags(doc1, &workspace);
            assert!(!diags.is_empty());
        }
    }
}
