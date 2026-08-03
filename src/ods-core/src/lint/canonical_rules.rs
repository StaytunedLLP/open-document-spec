

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
            message: format!(
                "root ods spec version mismatch: {version} (expected {})",
                crate::model::current_ods_spec_version()
            ),
        }),
        None => diagnostics.push(Diagnostic {
            path: root_index.path.clone(),
            severity: Severity::Error,
            message: format!(
                "root index.ods.md missing ods: {}",
                crate::model::current_ods_spec_version()
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
                    message: format!("missing pack path: {pack}"),
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
                message: format!("dangling reference: {reference}"),
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
                message: format!(
                    "non-canonical document reference: {reference} (prefer {canonical})"
                ),
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
                        message: format!(
                            "non-canonical context document reference: {load} (prefer {canonical})"
                        ),
                    });
                }
            } else if is_resource_like(load) {
                let path = normalize_join(&document.directory, Path::new(load));
                if !path.exists() {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Error,
                        message: format!("missing context resource: {load}"),
                    });
                }
            } else if !ids.contains_key(&load.to_lowercase()) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: format!("dangling context reference: {load}"),
                });
            }
        }

        for ignore in &context.ignore {
            let ignored = normalize_join(&document.directory, Path::new(ignore));
            if !ignored.exists() && !ids.contains_key(&ignore.to_lowercase()) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Warning,
                    message: format!("context ignore target not found: {ignore}"),
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
        message: "ods and ods should be declared only in root index.ods.md".to_string(),
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
        message: "workspace aliases should be declared in the root index.ods.md".to_string(),
    }]
}
