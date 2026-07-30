//! Deterministic workspace graph export for AI / review (`ods export`).

use crate::fs::load_workspace;
use crate::index::generate_indexes;
use crate::model::{Frontmatter, FrontmatterState, Workspace};
use crate::parse::document_id;
use std::io;
use std::path::{Path, PathBuf};

fn parsed_fm(document: &crate::model::Document) -> Option<&Frontmatter> {
    match &document.frontmatter {
        FrontmatterState::Parsed(fm) => Some(fm),
        _ => None,
    }
}

/// Write a Markdown graph of the workspace. Returns the absolute path written.
///
/// When `out` lands under the workspace root, indexes are regenerated so
/// `ods doctor` does not report stale indexes after a normal export.
///
/// Documents marked `share: private` or `share: org` are excluded from the
/// rendered graph unless `include_private` is set, matching the same
/// visibility contract enforced by `ods context` (see `crate::context`).
pub fn export_workspace_graph(
    root: impl AsRef<Path>,
    out: impl AsRef<Path>,
    include_private: bool,
) -> io::Result<PathBuf> {
    let root = root.as_ref();
    let workspace = load_workspace(root)?;
    let md = render_graph_markdown(&workspace, include_private);
    let out = out.as_ref();
    if let Some(parent) = out.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(out, md)?;

    let out_abs = if out.is_absolute() {
        out.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| root.to_path_buf())
            .join(out)
    };
    let root_abs = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let under_workspace = out_abs
        .canonicalize()
        .unwrap_or(out_abs.clone())
        .starts_with(&root_abs);
    if under_workspace {
        // Reload so the new file is included, then refresh indexes.
        let workspace = load_workspace(root)?;
        let _ = generate_indexes(&workspace)?;
    }

    Ok(out.to_path_buf())
}

fn is_shared_out(document: &crate::model::Document) -> bool {
    !matches!(
        parsed_fm(document).and_then(|fm| fm.share.as_deref()),
        Some("private") | Some("org")
    )
}

/// Render graph Markdown (stable order for tests).
///
/// Documents marked `share: private` or `share: org` are omitted unless
/// `include_private` is `true`.
pub fn render_graph_markdown(workspace: &Workspace, include_private: bool) -> String {
    let mut docs: Vec<_> = workspace
        .documents
        .iter()
        .filter(|doc| include_private || is_shared_out(doc))
        .collect();
    docs.sort_by(|a, b| a.path.cmp(&b.path));

    let mut out = String::new();
    out.push_str("# ODS workspace graph\n\n");
    out.push_str(&format!("- **Root:** `{}`\n", workspace.root.display()));
    out.push_str(&format!("- **Documents:** {}\n\n", docs.len()));

    out.push_str("## Documents\n\n");
    for document in &docs {
        let fm = parsed_fm(document);
        let id = document_id(&workspace.root, &document.path, fm);
        let rel = document
            .path
            .strip_prefix(&workspace.root)
            .unwrap_or(&document.path);
        let profile = fm.and_then(|f| f.profile.as_deref()).unwrap_or("(none)");
        let status = fm.and_then(|f| f.status.as_deref()).unwrap_or("(none)");
        let description = fm.and_then(|f| f.description.as_deref()).unwrap_or("");
        out.push_str(&format!("### `{id}`\n\n"));
        out.push_str(&format!("- **path:** `{}`\n", rel.display()));
        out.push_str(&format!("- **profile:** `{profile}`\n"));
        out.push_str(&format!("- **status:** `{status}`\n"));
        if !description.is_empty() {
            out.push_str(&format!("- **description:** {description}\n"));
        }
        if let Some(fm) = fm {
            if !fm.depends.is_empty() {
                out.push_str("- **depends:**\n");
                for d in &fm.depends {
                    let rendered =
                        crate::refs::canonical_document_ref_for_reference(workspace, document, d)
                            .unwrap_or_else(|| d.clone());
                    out.push_str(&format!("  - `{rendered}`\n"));
                }
            }
            if !fm.related.is_empty() {
                out.push_str("- **related:**\n");
                for r in &fm.related {
                    let rendered =
                        crate::refs::canonical_document_ref_for_reference(workspace, document, r)
                            .unwrap_or_else(|| r.clone());
                    out.push_str(&format!("  - `{rendered}`\n"));
                }
            }
            if !fm.resources.is_empty() {
                out.push_str("- **resources:**\n");
                for res in &fm.resources {
                    out.push_str(&format!("  - `{}`\n", res.path.display()));
                }
            }
            if !fm.code.is_empty() {
                out.push_str("- **code:**\n");
                for code in &fm.code {
                    out.push_str(&format!(
                        "  - `{}` ({})",
                        code.path.display(),
                        code.role.as_str()
                    ));
                    if let Some(symbol) = &code.symbol {
                        out.push_str(&format!(" `#{symbol}`"));
                    }
                    out.push('\n');
                }
            }
        }
        out.push('\n');
    }

    out.push_str("## Edges (depends)\n\n");
    let mut edges = Vec::new();
    for document in &docs {
        let Some(fm) = parsed_fm(document) else {
            continue;
        };
        let from = document_id(&workspace.root, &document.path, Some(fm));
        for to in &fm.depends {
            let to = crate::refs::document_ref_to_id(workspace, document, to)
                .unwrap_or_else(|| to.clone());
            edges.push((from.clone(), to, "depends"));
        }
        for to in &fm.related {
            let to = crate::refs::document_ref_to_id(workspace, document, to)
                .unwrap_or_else(|| to.clone());
            edges.push((from.clone(), to, "related"));
        }
    }
    edges.sort();
    if edges.is_empty() {
        out.push_str("_No depends/related edges._\n");
    } else {
        out.push_str("| from | relation | to |\n| --- | --- | --- |\n");
        for (from, to, rel) in edges {
            out.push_str(&format!("| `{from}` | {rel} | `{to}` |\n"));
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn export_includes_depends_edge() {
        let dir = std::env::temp_dir().join(format!(
            "ods-export-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(
            dir.join("a.md"),
            "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
        )
        .unwrap();
        fs::write(
            dir.join("b.md"),
            "---\nprofile: note\nstatus: draft\nid: b\ndepends:\n  - a\n---\n\n# B\n",
        )
        .unwrap();
        let ws = load_workspace(&dir).unwrap();
        let md = render_graph_markdown(&ws, false);
        assert!(md.contains("`b`"), "{md}");
        assert!(md.contains("| `b` | depends | `a` |"), "{md}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_excludes_private_and_org_docs_by_default() {
        let dir = std::env::temp_dir().join(format!(
            "ods-export-share-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        )
        .unwrap();
        fs::write(
            dir.join("public.md"),
            "---\nprofile: note\nstatus: draft\nid: public\n---\n\n# Public\n",
        )
        .unwrap();
        fs::write(
            dir.join("secret.md"),
            "---\nprofile: note\nstatus: draft\nid: secret\nshare: private\n---\n\n# Secret\n",
        )
        .unwrap();
        fs::write(
            dir.join("internal.md"),
            "---\nprofile: note\nstatus: draft\nid: internal\nshare: org\n---\n\n# Internal\n",
        )
        .unwrap();

        let ws = load_workspace(&dir).unwrap();

        let default_md = render_graph_markdown(&ws, false);
        assert!(default_md.contains("`public`"), "{default_md}");
        assert!(!default_md.contains("`secret`"), "{default_md}");
        assert!(!default_md.contains("`internal`"), "{default_md}");

        let full_md = render_graph_markdown(&ws, true);
        assert!(full_md.contains("`public`"), "{full_md}");
        assert!(full_md.contains("`secret`"), "{full_md}");
        assert!(full_md.contains("`internal`"), "{full_md}");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_description_resources_code_and_relative_out() {
        let td = tempfile::tempdir().unwrap();
        let dir = td.path();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\ndescription: Root index\nresources:\n  - path: res.txt\ncode:\n  - path: src/main.rs\n    role: entrypoint\n    symbol: main\n---\n\n# R\n",
        ).unwrap();
        fs::write(
            dir.join("plain.md"),
            "# Plain Doc No FM\n",
        ).unwrap();

        let ws = load_workspace(dir).unwrap();
        let md = render_graph_markdown(&ws, true);
        assert!(md.contains("description:** Root index"));
        assert!(md.contains("resources:**"));
        assert!(md.contains("code:**"));
        assert!(md.contains("#main"));

        let out_rel = dir.join("out_graph.md");
        let res = export_workspace_graph(dir, &out_rel, true);
        assert!(res.is_ok());
        assert!(out_rel.exists());
    }
}
