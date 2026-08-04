

pub(super) fn lint_root_spec(
    root_index: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    match frontmatter.ods.as_deref() {
        Some(version) if version == crate::model::current_ods_spec_version() => {}
        Some(version) => diagnostics.push(Diagnostic {
            path: root_index.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_root_version_mismatch(
                version,
                crate::model::current_ods_spec_version(),
            ),
        }),
        None => diagnostics.push(Diagnostic {
            path: root_index.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_root_missing_ods_version(
                crate::model::current_ods_spec_version(),
            ),
        }),
    }

    diagnostics
}

pub(super) fn lint_packs(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if document.path == workspace.root.join("index.ods.md") {
        for pack in &frontmatter.packs {
            let path = normalize_join(&workspace.root, Path::new(pack));
            if !path.exists() {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: crate::error::lint_missing_pack_path(pack),
                });
            }
        }
    }
    diagnostics
}

pub(super) fn lint_references(
    workspace: &Workspace,
    document: &Document,
    ids: &BTreeMap<String, Vec<&Document>>,
    frontmatter: &crate::model::Frontmatter,
    canonical_refs: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for reference in frontmatter.depends.iter().chain(frontmatter.related.iter()) {
        if crate::refs::document_ref_to_id(workspace, document, reference).is_none()
            && !ids.contains_key(reference)
        {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_dangling_reference(reference),
            });
        } else if canonical_refs
            && !crate::refs::is_markdown_ref(reference)
            && let Some(canonical) =
                crate::refs::canonical_document_ref_for_reference(workspace, document, reference)
            && canonical != *reference
        {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Warning,
                message: crate::error::lint_non_canonical_ref(reference, &canonical),
            });
        }
    }

    if let Some(context) = &frontmatter.context {
        for load in &context.load {
            if crate::refs::document_ref_to_path(workspace, document, load).is_some() {
                if canonical_refs
                    && !crate::refs::is_markdown_ref(load)
                    && let Some(canonical) =
                        crate::refs::canonical_document_ref_for_reference(workspace, document, load)
                    && canonical != *load
                {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Warning,
                        message: crate::error::lint_non_canonical_context_ref(load, &canonical),
                    });
                }
            } else if is_resource_like(load) {
                let path = normalize_join(&document.directory, Path::new(load));
                if !path.exists() {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Error,
                        message: crate::error::lint_missing_context_resource(load),
                    });
                }
            } else if !ids.contains_key(&load.to_lowercase()) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: crate::error::lint_dangling_context_reference(load),
                });
            }
        }

        for ignore in &context.ignore {
            let ignored = normalize_join(&document.directory, Path::new(ignore));
            if !ignored.exists() && !ids.contains_key(&ignore.to_lowercase()) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Warning,
                    message: crate::error::lint_context_ignore_not_found(ignore),
                });
            }
        }
    }

    diagnostics
}

pub(super) fn lint_ods_scope(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    if (frontmatter.ods.is_none() && frontmatter.ods.is_none())
        || document.path == workspace.root.join("index.ods.md")
    {
        return Vec::new();
    }
    if document.path.file_name().is_some_and(|name| name == "index.ods.md") {
        return Vec::new();
    }

    vec![Diagnostic {
        path: document.path.clone(),
        severity: Severity::Error,
        message: crate::error::lint_root_ods_scope_only(),
    }]
}

pub(super) fn lint_alias_scope(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    if frontmatter.aliases.is_empty() {
        return Vec::new();
    }

    if document.path == workspace.root.join("index.ods.md") {
        return Vec::new();
    }

    vec![Diagnostic {
        path: document.path.clone(),
        severity: Severity::Warning,
        message: crate::error::lint_aliases_root_only(),
    }]
}

#[cfg(test)]
mod test_canonical_rules {
    use super::*;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn test_canonical_rules_helpers() {
        let td = tempdir().unwrap();
        let root = td.path();
        std::fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.999\n---\n\n# Root\n",
        )
        .unwrap();

        let ws = load_workspace(root).unwrap();
        let doc = ws.documents.first().unwrap();
        if let crate::model::FrontmatterState::Parsed(fm) = &doc.frontmatter {
            let diags = lint_root_spec(doc, fm);
            assert!(!diags.is_empty());

            let alias_diags = lint_alias_scope(&ws, doc, fm);
            assert!(alias_diags.is_empty());
        }
    }
}
