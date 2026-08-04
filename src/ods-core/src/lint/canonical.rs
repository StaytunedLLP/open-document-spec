
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

            if let Some(created) = &frontmatter.created {
                if !is_valid_date_str(created) {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Warning,
                        message: format!("invalid created date format: '{created}' (expected YYYY-MM-DD or ISO-8601)"),
                    });
                }
            }

            if let Some(updated) = &frontmatter.updated {
                if !is_valid_date_str(updated) {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Warning,
                        message: format!("invalid updated date format: '{updated}' (expected YYYY-MM-DD or ISO-8601)"),
                    });
                }
            }

            let profile = frontmatter.profile.as_deref().unwrap_or("note");
            if let Some(def) = workspace.profiles.definitions.get(profile) {
                for key in &def.expected_keys {
                    let key_present = (frontmatter.owner.is_some() && key == "owner")
                        || (frontmatter.description.is_some() && key == "description")
                        || document.body.contains(&format!("{key}:"));
                    if !key_present {
                        diagnostics.push(Diagnostic {
                            path: document.path.clone(),
                            severity: Severity::Warning,
                            message: format!("missing expected key '{key}' for profile '{profile}'"),
                        });
                    }
                }
            } else {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Warning,
                    message: format!("unknown profile: {profile}"),
                });
            }

            diagnostics.extend(lint_alias_scope(workspace, document, frontmatter));
            diagnostics.extend(lint_ods_scope(workspace, document, frontmatter));
            diagnostics.extend(crate::tags::lint_document_tags(document, workspace));

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
    let root_index_path = workspace.root.join("index.ods.md");
    let Some(root_index) = workspace
        .documents
        .iter()
        .find(|document| document.path == root_index_path)
    else {
        return vec![Diagnostic {
            path: root_index_path,
            severity: Severity::Error,
            message: format!(
                "missing root index.ods.md with ods: {}",
                crate::model::current_ods_spec_version()
            ),
        }];
    };

    match &root_index.frontmatter {
        FrontmatterState::Parsed(frontmatter) => lint_root_spec(root_index, frontmatter),
        _ => Vec::new(),
    }
}

include!("canonical_rules.rs");

fn lint_profile_sections(
    workspace: &Workspace,
    document: &Document,
    profile: &str,
) -> Vec<Diagnostic> {
    let aliases = workspace_aliases(workspace);
    lint_profile_sections_with_aliases(document, workspace, profile, &aliases)
}

fn is_valid_date_str(s: &str) -> bool {
    let s = s.trim();
    if s.len() < 8 {
        return false;
    }
    let date_part = s.split('T').next().unwrap_or(s).split(' ').next().unwrap_or(s);
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() == 3 {
        parts[0].len() == 4
            && parts[0].parse::<u32>().is_ok()
            && parts[1].parse::<u32>().is_ok()
            && parts[2].parse::<u32>().is_ok()
    } else {
        false
    }
}
