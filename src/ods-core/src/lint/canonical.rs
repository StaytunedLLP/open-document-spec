
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
                            message: crate::error::lint_depends_cycle(&cycle),
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
                message: crate::error::lint_frontmatter_parse(message),
            });
            return diagnostics;
        }
        FrontmatterState::Absent => {}
        FrontmatterState::Parsed(frontmatter) => {
            // Schema-driven enum + placement checks (status, share, tags under ods:).
            for issue in crate::spec::validate_ods_frontmatter(frontmatter) {
                diagnostics.push(issue.to_diagnostic(document.path.clone()));
            }

            if let Some(created) = &frontmatter.created {
                if !is_valid_date_str(created) {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Warning,
                        message: crate::error::lint_invalid_date("created", created),
                    });
                }
            }

            if let Some(updated) = &frontmatter.updated {
                if !is_valid_date_str(updated) {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Warning,
                        message: crate::error::lint_invalid_date("updated", updated),
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
                            message: crate::error::lint_missing_expected_key(key, profile),
                        });
                    }
                }
            } else {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Warning,
                    message: crate::error::lint_unknown_profile(profile),
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
            message: crate::error::lint_missing_root_index(
                crate::model::current_ods_spec_version(),
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

#[cfg(test)]
mod test_canonical_dates_and_cycles {
    use super::*;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn test_invalid_date_lint() {
        assert!(!is_valid_date_str("bad"));
        assert!(is_valid_date_str("2024-01-01"));

        let td = tempdir().unwrap();
        let root = td.path();
        std::fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();
        std::fs::write(
            root.join("bad_date.md"),
            "---\nprofile: note\ncreated: bad-date\nupdated: not-a-date\n---\n\n# Doc\n",
        )
        .unwrap();

        let ws = load_workspace(root).unwrap();
        let diags = crate::lint_workspace(&ws);
        assert!(diags.iter().any(|d| d.message.contains("created") || d.message.contains("updated")));
    }

    #[test]
    fn test_dangling_refs_and_packs_lint() {
        let td = tempdir().unwrap();
        let root = td.path();
        std::fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\npacks:\n  - missing-pack\n---\n\n# Root\n",
        )
        .unwrap();
        std::fs::write(
            root.join("sub.md"),
            "---\nprofile: note\nods: 0.1\ndepends:\n  - missing.md\nrelated:\n  - missing2.md\nresources:\n  - path: missing_resource.png\ncode:\n  - path: src/main.rs:L10\n    role: implementation\ncontext:\n  load:\n    - missing_res.png\n    - missing_doc.md\n  ignore:\n    - missing_ignore.png\n---\n\n# Sub\n",
        )
        .unwrap();

        let ws = load_workspace(root).unwrap();
        let diags = crate::lint_workspace(&ws);
        assert!(diags.iter().any(|d| d.message.contains("dangling") || d.message.contains("pack") || d.message.contains("ODS version")));
    }
}
