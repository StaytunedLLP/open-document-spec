pub fn render_index(workspace: &Workspace, directory: &Path, existing: Option<&str>) -> String {
    let (title, profile, ods, profiles, packs, ignore) = existing
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
    let profiles = if is_root && profiles.is_empty() {
        workspace
            .document_by_path(&workspace.root.join("index.ods.md"))
            .and_then(|doc| match &doc.frontmatter {
                crate::model::FrontmatterState::Parsed(fm) => Some(fm.profiles.clone()),
                _ => None,
            })
            .unwrap_or(profiles)
    } else {
        profiles
    };
    if !profiles.is_empty() {
        out.push_str("custom-profiles:\n");
        for catalog in &profiles {
            out.push_str(&format!("  - {catalog}\n"));
        }
    }
    let packs = if is_root && packs.is_empty() {
        workspace
            .document_by_path(&workspace.root.join("index.ods.md"))
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
        out.push('\n');
        out.push_str(&footer_prose);
        out.push('\n');
    }

    out
}







#[derive(Debug, Clone)]
pub(super) struct IndexEntry {
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
            if name == "index.md" || name == "index.ods.md" || should_ignore_name(entry.file_name().as_os_str()) {
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
                        target: format!("{name}/index.ods.md"),
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
    if let Some(dir) = target.strip_suffix("/index.md").or_else(|| target.strip_suffix("/index.ods.md")) {
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
) -> Option<(String, String, Option<String>, Vec<String>, Vec<String>, Vec<String>)> {
    let mut title = None::<String>;
    let mut profile = None::<String>;
    let mut ods = None::<String>;
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
                    "profiles" | "custom-profiles" | "packs" | "ignore" => {}
                    _ => {}
                }
            } else if trimmed.starts_with("- ") {
                let item = trimmed.trim_start_matches("- ").trim().to_string();
                match current_key.as_deref() {
                    Some("profiles") | Some("custom-profiles") => profiles.push(item),
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
        profiles,
        packs,
        ignore,
    ))
}

#[cfg(test)]
mod test_checker {
    use super::*;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn test_checker_helpers() {
        let text = "---\nprofile: index\nods: 0.1\ncustom-profiles:\n  - custom.yaml\npacks:\n  - my-pack\nignore:\n  - build/\n---\n\n# Custom Title\n";
        let meta = extract_title_and_meta(text).unwrap();
        assert_eq!(meta.0, "Custom Title");
        assert_eq!(meta.1, "index");
        assert_eq!(meta.2, Some("0.1".into()));
        assert_eq!(meta.3, vec!["custom.yaml"]);
        assert_eq!(meta.4, vec!["my-pack"]);
        assert_eq!(meta.5, vec!["build"]);

        let td = tempdir().unwrap();
        let root = td.path();
        std::fs::write(root.join("index.md"), text).unwrap();
        let ws = load_workspace(root).unwrap();
        let rendered = render_index(&ws, root, Some(text));
        assert!(rendered.contains("# Custom Title"));
    }
}


