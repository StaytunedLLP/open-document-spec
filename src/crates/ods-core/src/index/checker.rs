pub fn render_index(workspace: &Workspace, directory: &Path, existing: Option<&str>) -> String {
    let (title, profile, ods, ods_cli, profiles, packs, ignore) = existing
        .and_then(extract_title_and_meta)
        .unwrap_or_else(|| default_index_header(directory, workspace));
    let entries = directory_children(workspace, directory);

    // Workspace-marker keys belong on a workspace root index. Preserve them on nested
    // indexes when already present (nested demo workspaces). Only inject defaults at
    // the loaded workspace root.
    let is_root = is_workspace_root(directory, workspace);
    let ods = if is_root {
        ods.or_else(|| Some(crate::model::current_ods_spec_version().to_string()))
    } else {
        ods
    };
    let ods_cli = if is_root {
        ods_cli.or_else(|| Some(crate::model::current_ods_cli_requirement()))
    } else {
        ods_cli
    };
    let ignore = if is_root && ignore.is_empty() {
        workspace.ignore.clone()
    } else {
        ignore
    };

    let (header_prose, footer_prose) = existing
        .map(|content| extract_prose(content, &entries))
        .unwrap_or_else(|| (String::new(), String::new()));

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("profile: {profile}\n"));
    if let Some(ods) = ods {
        out.push_str(&format!("ods: {ods}\n"));
    }
    if let Some(ods_cli) = ods_cli {
        out.push_str(&format!("ods-cli: \"{ods_cli}\"\n"));
    }
    if !profiles.is_empty() {
        out.push_str("profiles:\n");
        for catalog in &profiles {
            out.push_str(&format!("  - {catalog}\n"));
        }
    }
    let packs = if is_root && packs.is_empty() {
        workspace
            .document_by_path(&workspace.root.join("index.md"))
            .and_then(|doc| match &doc.frontmatter {
                crate::model::FrontmatterState::Parsed(fm) => Some(fm.packs.clone()),
                _ => None,
            })
            .unwrap_or(packs)
    } else {
        packs
    };
    if !packs.is_empty() {
        out.push_str("packs:\n");
        for pack in &packs {
            out.push_str(&format!("  - {pack}\n"));
        }
    }
    if !ignore.is_empty() {
        out.push_str("ignore:\n");
        for prefix in &ignore {
            out.push_str(&format!("  - {prefix}\n"));
        }
    }
    out.push_str("---\n\n");
    out.push_str(&format!("# {title}\n\n"));

    if !header_prose.is_empty() {
        out.push_str(&header_prose);
        out.push_str("\n\n");
    }

    for entry in &entries {
        let full_path = directory.join(&entry.target);
        let description =
            workspace
                .document_by_path(&full_path)
                .and_then(|doc| match &doc.frontmatter {
                    crate::model::FrontmatterState::Parsed(fm) => fm.description.clone(),
                    _ => None,
                });
        if let Some(desc) = description {
            out.push_str(&format!(
                "- [{}]({}) - {}\n",
                entry.label, entry.target, desc
            ));
        } else {
            out.push_str(&format!("- [{}]({})\n", entry.label, entry.target));
        }
    }

    if !footer_prose.is_empty() {
        out.push_str("\n");
        out.push_str(&footer_prose);
        out.push_str("\n");
    }

    out
}

fn extract_prose(existing: &str, entries: &[IndexEntry]) -> (String, String) {
    let mut header_lines = Vec::new();
    let mut footer_lines = Vec::new();
    let mut title_found = false;
    let mut first_link_idx = None;
    let mut last_link_idx = None;
    let mut in_frontmatter = false;

    let lines: Vec<&str> = existing.lines().collect();

    let mut is_link_line = vec![false; lines.len()];
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            continue;
        }

        if !title_found {
            if trimmed.starts_with("# ") {
                title_found = true;
            }
            continue;
        }

        if trimmed.starts_with("- [") {
            for entry in entries {
                let target_pattern = format!("]({})", entry.target);
                if trimmed.contains(&target_pattern) {
                    is_link_line[idx] = true;
                    if first_link_idx.is_none() {
                        first_link_idx = Some(idx);
                    }
                    last_link_idx = Some(idx);
                    break;
                }
            }
        }
    }

    if let (Some(first), Some(last)) = (first_link_idx, last_link_idx) {
        let mut title_idx = None;
        in_frontmatter = false;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "---" {
                in_frontmatter = !in_frontmatter;
                continue;
            }
            if in_frontmatter {
                continue;
            }
            if trimmed.starts_with("# ") {
                title_idx = Some(idx);
                break;
            }
        }

        if let Some(t_idx) = title_idx {
            for i in (t_idx + 1)..first {
                header_lines.push(lines[i]);
            }
        }

        for i in (last + 1)..lines.len() {
            footer_lines.push(lines[i]);
        }
    } else {
        let mut title_idx = None;
        in_frontmatter = false;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "---" {
                in_frontmatter = !in_frontmatter;
                continue;
            }
            if in_frontmatter {
                continue;
            }
            if trimmed.starts_with("# ") {
                title_idx = Some(idx);
                break;
            }
        }
        if let Some(t_idx) = title_idx {
            for i in (t_idx + 1)..lines.len() {
                header_lines.push(lines[i]);
            }
        }
    }

    fn clean_prose(lines: Vec<&str>) -> String {
        let mut start = 0;
        while start < lines.len() && lines[start].trim().is_empty() {
            start += 1;
        }
        let mut end = lines.len();
        while end > start && lines[end - 1].trim().is_empty() {
            end -= 1;
        }
        if start < end {
            lines[start..end].join("\n")
        } else {
            String::new()
        }
    }

    (clean_prose(header_lines), clean_prose(footer_lines))
}

fn absolutize_for_compare(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn is_workspace_root(directory: &Path, workspace: &Workspace) -> bool {
    paths_equal_normalized(directory, &workspace.root)
        || paths_equal_normalized(
            &absolutize_for_compare(directory),
            &absolutize_for_compare(&workspace.root),
        )
}

#[derive(Debug, Clone)]
struct IndexEntry {
    label: String,
    target: String,
}

fn directory_children(workspace: &Workspace, directory: &Path) -> Vec<IndexEntry> {
    if let Some(cached) = workspace.children.get(directory) {
        return cached
            .iter()
            .map(|target| IndexEntry {
                label: index_label(target),
                target: target.clone(),
            })
            .collect();
    }

    // Fallback if indexes were not rebuilt.
    let mut entries = match fs::read_dir(directory) {
        Ok(entries) => entries.collect::<Result<Vec<_>, _>>().unwrap_or_default(),
        Err(_) => return Vec::new(),
    };
    entries.sort_by_key(|entry| entry.file_name());

    entries
        .into_iter()
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "index.md" || should_ignore_name(entry.file_name().as_os_str()) {
                return None;
            }

            let path = entry.path();
            if path_matches_workspace_ignore(&workspace.root, &path, &workspace.ignore) {
                return None;
            }
            if crate::fs::is_excluded_profile_catalog(workspace, &path) {
                return None;
            }

            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                let is_doc_dir = workspace.doc_dirs.contains(&path);
                if is_doc_dir {
                    Some(IndexEntry {
                        label: format!("{name}/"),
                        target: format!("{name}/index.md"),
                    })
                } else {
                    None
                }
            } else if file_type.is_file() {
                if name.ends_with(".md") || is_referenced_resource(workspace, &path) {
                    Some(IndexEntry {
                        label: name.clone(),
                        target: name,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn index_label(target: &str) -> String {
    if let Some(dir) = target.strip_suffix("/index.md") {
        format!("{dir}/")
    } else {
        target.to_string()
    }
}

fn is_referenced_resource(workspace: &Workspace, path: &Path) -> bool {
    if workspace.resource_paths.contains(path) {
        return true;
    }
    workspace
        .resource_paths
        .iter()
        .any(|res| paths_equal_normalized(res, path))
        || workspace.documents.iter().any(|doc| {
            if let crate::model::FrontmatterState::Parsed(fm) = &doc.frontmatter {
                fm.resources.iter().any(|res| {
                    let res_path = normalize_join(&doc.directory, &res.path);
                    paths_equal_normalized(&res_path, path)
                })
            } else {
                false
            }
        })
}

#[allow(clippy::type_complexity)]
fn extract_title_and_meta(
    existing: &str,
) -> Option<(String, String, Option<String>, Option<String>, Vec<String>, Vec<String>, Vec<String>)> {
    let mut title = None::<String>;
    let mut profile = None::<String>;
    let mut ods = None::<String>;
    let mut ods_cli = None::<String>;
    let mut profiles = Vec::<String>::new();
    let mut packs = Vec::<String>::new();
    let mut ignore = Vec::<String>::new();
    let mut in_frontmatter = false;
    let mut current_key = None::<String>;

    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            current_key = None;
            continue;
        }

        if in_frontmatter {
            if let Some((key, value)) = trimmed.split_once(':') {
                current_key = Some(key.trim().to_string());
                match key.trim() {
                    "profile" => profile = Some(value.trim().to_string()),
                    "ods" => ods = Some(value.trim().to_string()),
                    "ods-cli" => ods_cli = Some(unquote_index_value(value.trim())),
                    "profiles" | "packs" | "ignore" => {}
                    _ => {}
                }
            } else if trimmed.starts_with("- ") {
                let item = trimmed.trim_start_matches("- ").trim().to_string();
                match current_key.as_deref() {
                    Some("profiles") => profiles.push(item),
                    Some("packs") => packs.push(item),
                    Some("ignore") => {
                        let item = item.trim_end_matches('/').to_string();
                        if !item.is_empty() {
                            ignore.push(item);
                        }
                    }
                    _ => {}
                }
            }
            continue;
        }

        if let Some(stripped) = trimmed.strip_prefix("# ") {
            title = Some(stripped.trim().to_string());
            break;
        }
    }

    Some((
        title.unwrap_or_else(|| "Index".to_string()),
        profile.unwrap_or_else(|| "index".to_string()),
        ods,
        ods_cli,
        profiles,
        packs,
        ignore,
    ))
}

#[allow(clippy::type_complexity)]
fn default_index_header(
    directory: &Path,
    workspace: &Workspace,
) -> (String, String, Option<String>, Option<String>, Vec<String>, Vec<String>, Vec<String>) {
    let name = directory
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Workspace");
    let is_root = is_workspace_root(directory, workspace);
    let title = if is_root {
        format!("{name} Workspace")
    } else {
        format!("{name}/")
    };
    (
        title,
        "index".to_string(),
        None,
        None,
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
}

fn unquote_index_value(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            value
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
        .unwrap_or(value)
        .to_string()
}
