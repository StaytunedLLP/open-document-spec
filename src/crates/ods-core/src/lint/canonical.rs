
fn lint_cycles(workspace: &Workspace, ids: &BTreeMap<String, Vec<&Document>>) -> Vec<Diagnostic> {
    let graph = dependency_graph(workspace, ids);
    let mut diagnostics = Vec::new();
    let mut visiting = HashSet::<String>::new();
    let mut visited = HashSet::<String>::new();
    let mut stack = Vec::<String>::new();

    for node in graph.keys() {
        dfs_cycles(
            node,
            &graph,
            &mut visiting,
            &mut visited,
            &mut stack,
            &mut diagnostics,
            ids,
        );
    }

    diagnostics
}

fn dfs_cycles(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    visiting: &mut HashSet<String>,
    visited: &mut HashSet<String>,
    stack: &mut Vec<String>,
    diagnostics: &mut Vec<Diagnostic>,
    ids: &BTreeMap<String, Vec<&Document>>,
) {
    if visited.contains(node) {
        return;
    }

    if !visiting.insert(node.to_string()) {
        return;
    }

    stack.push(node.to_string());

    if let Some(children) = graph.get(node) {
        for child in children {
            if visiting.contains(child) {
                if let Some(first) = stack.iter().position(|item| item == child) {
                    let cycle = stack[first..]
                        .iter()
                        .cloned()
                        .chain(std::iter::once(child.clone()))
                        .collect::<Vec<_>>()
                        .join(" -> ");
                    if let Some(doc) = ids.get(node).and_then(|docs| docs.first()) {
                        diagnostics.push(Diagnostic {
                            path: doc.path.clone(),
                            severity: Severity::Error,
                            message: format!("depends cycle detected: {cycle}"),
                        });
                    }
                }
            } else {
                dfs_cycles(child, graph, visiting, visited, stack, diagnostics, ids);
            }
        }
    }

    stack.pop();
    visiting.remove(node);
    visited.insert(node.to_string());
}

fn dependency_graph(
    workspace: &Workspace,
    ids: &BTreeMap<String, Vec<&Document>>,
) -> BTreeMap<String, Vec<String>> {
    let mut graph = BTreeMap::new();

    for (id, docs) in ids {
        let Some(doc) = docs.first() else { continue };
        let Some(frontmatter) = frontmatter(doc) else {
            continue;
        };
        let mut edges = Vec::new();
        edges.extend(
            frontmatter
                .depends
                .iter()
                .filter_map(|reference| crate::refs::document_ref_to_id(workspace, doc, reference)),
        );
        graph.insert(id.clone(), edges);
    }

    graph
}

fn lint_document(
    workspace: &Workspace,
    document: &Document,
    ids: &BTreeMap<String, Vec<&Document>>,
    level: LintLevel,
    canonical_refs: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    match &document.frontmatter {
        FrontmatterState::Invalid(message) => {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: format!("frontmatter parse error: {message}"),
            });
            return diagnostics;
        }
        FrontmatterState::Absent => {}
        FrontmatterState::Parsed(frontmatter) => {
            if let Some(status) = &frontmatter.status
                && !matches!(
                    status.as_str(),
                    "draft" | "stable" | "deprecated" | "archived"
                )
            {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: format!("invalid status: {status}"),
                });
            }

            if let Some(share) = &frontmatter.share
                && !matches!(
                    share.as_str(),
                    "public" | "org" | "private"
                )
            {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: format!("invalid share value: {share}"),
                });
            }

            let profile = frontmatter.profile.as_deref().unwrap_or("note");
            if !workspace.profiles.definitions.contains_key(profile) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Warning,
                    message: format!("unknown profile: {profile}"),
                });
            }

            diagnostics.extend(lint_alias_scope(workspace, document, frontmatter));
            diagnostics.extend(lint_ods_scope(workspace, document, frontmatter));
            diagnostics.extend(crate::tags::lint_document_tags(document));

            if matches!(level, LintLevel::Level3) {
                diagnostics.extend(lint_references(
                    workspace,
                    document,
                    ids,
                    frontmatter,
                    canonical_refs,
                ));
                diagnostics.extend(lint_profile_sections(workspace, document, profile));
                diagnostics.extend(lint_resources(document, frontmatter));
                diagnostics.extend(lint_code_refs(document, frontmatter));
                diagnostics.extend(lint_index(workspace, document));
                diagnostics.extend(lint_packs(workspace, document, frontmatter));
                if !document.body.is_empty() {
                    diagnostics.extend(lint_body_links(document));
                }
            }
        }
    }

    diagnostics
}

fn lint_root_ods_metadata(workspace: &Workspace) -> Vec<Diagnostic> {
    let root_index_path = workspace.root.join("index.md");
    let Some(root_index) = workspace
        .documents
        .iter()
        .find(|document| document.path == root_index_path)
    else {
        return vec![Diagnostic {
            path: root_index_path,
            severity: Severity::Error,
            message: format!(
                "missing root index.md with ods: {} and ods-cli: \"{}\"",
                crate::model::current_ods_spec_version(),
                crate::model::current_ods_cli_requirement()
            ),
        }];
    };

    let Some(frontmatter) = frontmatter(root_index) else {
        return vec![Diagnostic {
            path: root_index.path.clone(),
            severity: Severity::Error,
            message: format!(
                "root index.md missing ods: {} and ods-cli: \"{}\"",
                crate::model::current_ods_spec_version(),
                crate::model::current_ods_cli_requirement()
            ),
        }];
    };

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
                "root index.md missing ods: {}",
                crate::model::current_ods_spec_version()
            ),
        }),
    }

    match frontmatter.ods_cli.as_deref() {
        Some(requirement) => match crate::model::ods_cli_requirement_satisfied(requirement) {
            Ok(true) => {}
            Ok(false) => diagnostics.push(Diagnostic {
                path: root_index.path.clone(),
                severity: Severity::Error,
                message: format!(
                    "root ods-cli requirement not satisfied: {requirement} (installed {})",
                    crate::model::current_ods_version()
                ),
            }),
            Err(err) => diagnostics.push(Diagnostic {
                path: root_index.path.clone(),
                severity: Severity::Error,
                message: err,
            }),
        },
        None => diagnostics.push(Diagnostic {
            path: root_index.path.clone(),
            severity: Severity::Error,
            message: format!(
                "root index.md missing ods-cli: \"{}\"",
                crate::model::current_ods_cli_requirement()
            ),
        }),
    }

    diagnostics
}

fn lint_packs(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if document.path == workspace.root.join("index.md") {
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

fn lint_references(
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

fn lint_ods_scope(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    if (frontmatter.ods.is_none() && frontmatter.ods_cli.is_none())
        || document.path == workspace.root.join("index.md")
    {
        return Vec::new();
    }
    if document.path.file_name().and_then(|name| name.to_str()) == Some("index.md") {
        return Vec::new();
    }

    vec![Diagnostic {
        path: document.path.clone(),
        severity: Severity::Error,
        message: "ods and ods-cli should be declared only in root index.md".to_string(),
    }]
}

fn lint_alias_scope(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    if frontmatter.aliases.is_empty() {
        return Vec::new();
    }

    if document.path == workspace.root.join("index.md") {
        return Vec::new();
    }

    vec![Diagnostic {
        path: document.path.clone(),
        severity: Severity::Warning,
        message: "workspace aliases should be declared in the root index.md".to_string(),
    }]
}

fn lint_profile_sections(
    workspace: &Workspace,
    document: &Document,
    profile: &str,
) -> Vec<Diagnostic> {
    let aliases = workspace_aliases(workspace);
    lint_profile_sections_with_aliases(document, workspace, profile, &aliases)
}
