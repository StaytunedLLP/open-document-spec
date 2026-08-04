fn lint_profile_sections_with_aliases(
    document: &Document,
    workspace: &Workspace,
    profile: &str,
    workspace_aliases: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Diagnostic> {
    let expected = profile_sections(workspace, profile);
    if expected.is_empty() {
        return Vec::new();
    }

    let headings = document
        .headings
        .iter()
        .map(|heading| normalize_heading(heading))
        .collect::<BTreeSet<_>>();

    expected
        .iter()
        .filter(|group| {
            let mut accepted = group
                .iter()
                .map(|heading| normalize_heading(heading))
                .collect::<BTreeSet<_>>();

            if let Some(canonical) = group.first()
                && let Some(values) = workspace_aliases.get(canonical)
            {
                accepted.extend(values.iter().map(|alias| normalize_heading(alias)));
            }

            !headings.iter().any(|heading| accepted.contains(heading))
        })
        .map(|group| Diagnostic {
            path: document.path.clone(),
            severity: Severity::Warning,
            message: crate::error::lint_missing_expected_section(&group[0]),
        })
        .collect()
}

fn lint_resources(document: &Document, frontmatter: &crate::model::Frontmatter) -> Vec<Diagnostic> {
    frontmatter
        .resources
        .iter()
        .filter_map(|resource| {
            let path = normalize_join(&document.directory, &resource.path);
            (!path.exists()).then(|| Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_missing_resource(resource.path.display()),
            })
        })
        .collect()
}

fn lint_code_refs(document: &Document, frontmatter: &crate::model::Frontmatter) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for code in &frontmatter.code {
        let path_str = code.path.to_string_lossy();
        if path_str.contains(":L") || path_str.contains(":line") {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_code_path_line_suffix(code.path.display()),
            });
            continue;
        }
        let path = normalize_join(&document.directory, &code.path);
        if !path.exists() {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_missing_code_path(code.path.display()),
            });
        }
    }
    diagnostics
}

fn lint_index(workspace: &Workspace, document: &Document) -> Vec<Diagnostic> {
    if document.path.file_name().and_then(|name| name.to_str()) != Some("index.md") {
        return Vec::new();
    }

    let expected = workspace
        .children
        .get(&document.directory)
        .cloned()
        .unwrap_or_else(|| directory_children(workspace, &document.directory));
    // Only top-level list links count as index children (not prose/table links).
    let actual = extract_index_list_links(&document.body);

    let missing = expected
        .iter()
        .filter(|item| !actual.contains(*item))
        .cloned()
        .collect::<Vec<_>>();
    let extra = actual
        .iter()
        .filter(|item| !expected.contains(*item))
        .cloned()
        .collect::<Vec<_>>();

    let mut diagnostics = Vec::new();

    if !missing.is_empty() {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_index_stale_missing(&missing.join(", ")),
        });
    }

    if !extra.is_empty() {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_index_stale_extra(&extra.join(", ")),
        });
    }

    diagnostics
}

fn directory_children(workspace: &Workspace, directory: &Path) -> Vec<String> {
    let mut entries = match fs::read_dir(directory) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>().unwrap_or_default(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by_key(|entry| entry.file_name());

    entries
        .into_iter()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "index.md" || name.starts_with('.') {
                return None;
            }

            let path = entry.path();
            if crate::fs::path_matches_workspace_ignore(&workspace.root, &path, &workspace.ignore) {
                return None;
            }
            if crate::fs::is_excluded_profile_catalog(workspace, &path) {
                return None;
            }

            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                let is_doc_dir = workspace.doc_dirs.contains(&path);
                if is_doc_dir {
                    Some(format!("{name}/index.md"))
                } else {
                    None
                }
            } else if file_type.is_file() {
                let is_resource = workspace.resource_paths.contains(&path)
                    || workspace
                        .resource_paths
                        .iter()
                        .any(|res| paths_equal_normalized(res, &path));
                if name.ends_with(".md") || is_resource {
                    Some(name)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// Links from top-level markdown list items only (`- [label](target)`).
fn extract_index_list_links(body: &str) -> BTreeSet<String> {
    let mut links = BTreeSet::new();
    let mut in_code_block = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        let list_line = line.trim_start();
        if !list_line.starts_with("- ") && !list_line.starts_with("* ") {
            continue;
        }
        if let Some(target) = split_markdown_link_target(line) {
            links.insert(target);
        }
    }
    links
}

fn extract_markdown_links(body: &str) -> BTreeSet<String> {
    let mut links = BTreeSet::new();
    let mut in_code_block = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block
            && let Some(target) = split_markdown_link_target(line)
        {
            links.insert(target);
        }
    }
    links
}

fn lint_body_links(document: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let links = extract_markdown_links(&document.body);

    for link in links {
        if link.starts_with("http://")
            || link.starts_with("https://")
            || link.starts_with("mailto:")
            || link.starts_with("ws://")
            || link.starts_with("wss://")
        {
            continue;
        }

        if link.starts_with('#') {
            continue;
        }

        let path_part = link.split('#').next().unwrap_or(&link);
        if path_part.is_empty() {
            continue;
        }

        let decoded_path = path_part.replace("%20", " ");

        let target_path = normalize_join(&document.directory, Path::new(&decoded_path));
        if !target_path.exists() {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_dangling_body_link(&link),
            });
        }
    }

    diagnostics
}

fn normalize_heading(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}
