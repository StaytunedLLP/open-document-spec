/// Rebuild path/id/children/resource maps after documents change (e.g. LSP overlay).
pub fn rebuild_indexes(workspace: &mut Workspace) {
    // Prefer config ignore (ods.toml); keep any runtime extensions already on workspace.
    let ignore = if workspace.ignore.is_empty() {
        workspace.config.ignore.clone()
    } else {
        workspace.ignore.clone()
    };

    workspace.by_id.clear();
    workspace.by_path.clear();
    workspace.children.clear();
    workspace.resource_paths.clear();
    workspace.code_paths.clear();
    workspace.tag_index.clear();
    workspace.profile_catalog_paths.clear();
    workspace.doc_dirs.clear();
    workspace.ignore = ignore;

    for (idx, document) in workspace.documents.iter().enumerate() {
        workspace.by_path.insert(document.path.clone(), idx);
        let frontmatter = match &document.frontmatter {
            crate::model::FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        };
        let id = document_id(&workspace.root, &document.path, frontmatter);
        workspace.by_id.entry(id.clone()).or_insert(idx);

        if let Some(fm) = frontmatter {
            for resource in &fm.resources {
                let abs = normalize_join(&document.directory, &resource.path);
                workspace.resource_paths.insert(abs);
            }
            for code in &fm.code {
                let abs = normalize_join(&document.directory, &code.path);
                workspace.code_paths.insert(abs);
            }
            for tag in &fm.tags {
                workspace
                    .tag_index
                    .entry(tag.clone())
                    .or_default()
                    .push(id.clone());
            }
        }
    }

    // Profile catalog dirs from ods.toml custom_profiles.
    for catalog in &workspace.config.custom_profiles {
        workspace
            .profile_catalog_paths
            .insert(workspace.root.join(catalog));
    }

    for docs in workspace.tag_index.values_mut() {
        docs.sort();
        docs.dedup();
    }

    // Build children maps from filesystem + document/resource knowledge.
    let mut dirs = HashSet::new();
    dirs.insert(workspace.root.clone());
    for document in &workspace.documents {
        if path_matches_workspace_ignore(&workspace.root, &document.path, &workspace.ignore) {
            continue;
        }
        let mut current = document.directory.clone();
        loop {
            if !path_matches_workspace_ignore(&workspace.root, &current, &workspace.ignore)
                || current == workspace.root
            {
                dirs.insert(current.clone());
            }
            if current == workspace.root {
                break;
            }
            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => break,
            }
        }
    }
    workspace.doc_dirs = dirs.clone();

    for directory in dirs {
        let children = directory_children_for(workspace, &directory);
        workspace.children.insert(directory, children);
    }
}


/// True when `path` lies under a workspace-relative ignore prefix from root `ignore:`.
pub fn path_matches_workspace_ignore(root: &Path, path: &Path, ignore: &[String]) -> bool {
    if ignore.is_empty() {
        return false;
    }

    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let relative = relative.trim_start_matches("./");
    if relative.is_empty() {
        return false;
    }

    ignore.iter().any(|rule| {
        let rule = rule.trim().trim_end_matches('/');
        if rule.is_empty() {
            return false;
        }
        relative == rule || relative.starts_with(&format!("{rule}/"))
    })
}

/// True when `path` is a profile catalog root (or inside one) declared on any index.
///
/// Checks `workspace.profile_roots` and `workspace.profile_catalog_paths`, both
/// bounded by the (typically tiny) number of catalog declarations in the
/// workspace — not by document count. Both are populated by `rebuild_indexes`;
/// callers with a fresh `Workspace` from `load_workspace` always have them set.
pub fn is_excluded_profile_catalog(workspace: &Workspace, path: &Path) -> bool {
    if workspace
        .profile_roots
        .iter()
        .any(|root| path == *root || path.strip_prefix(root).is_ok())
    {
        return true;
    }
    workspace
        .profile_catalog_paths
        .iter()
        .any(|catalog_root| path == catalog_root || path.strip_prefix(catalog_root).is_ok())
}

fn directory_children_for(workspace: &Workspace, directory: &Path) -> Vec<String> {
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
            if is_excluded_profile_catalog(workspace, &path) {
                return None;
            }

            let file_type = entry.file_type().ok()?;
            if file_type.is_dir() {
                let is_doc_dir = workspace.doc_dirs.contains(&path);
                if is_doc_dir {
                    Some(format!("{name}/index.ods.md"))
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
        .collect()
}

pub fn normalize_join(base: &Path, relative: &Path) -> PathBuf {
    let joined = base.join(relative);
    normalize_path(&joined)
}

/// Lexically normalize `..` and `.` without touching the filesystem.
pub fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn paths_equal_normalized(a: &Path, b: &Path) -> bool {
    normalize_path(a) == normalize_path(b)
}

#[cfg(test)]
mod test_scanner {
    use super::*;

    #[test]
    fn scanner_helper_edge_cases() {
        assert_eq!(normalize_path(Path::new("a/./b")), PathBuf::from("a/b"));
        assert!(!path_matches_workspace_ignore(Path::new("/root"), Path::new("/root/a.md"), &["".into(), "  ".into()]));
        assert!(path_matches_workspace_ignore(Path::new("/root"), Path::new("/root/build/a.md"), &["build/".into()]));
        assert!(!path_matches_workspace_ignore(Path::new("/root"), Path::new("/root"), &["build".into()]));

        let mut ws = Workspace::empty(PathBuf::from("/root"));
        ws.ignore = vec!["ignored".into()];
        ws.profile_roots.push(PathBuf::from("/root/specs"));
        ws.profile_catalog_paths.insert(PathBuf::from("/root/okf"));

        assert!(is_excluded_profile_catalog(&ws, Path::new("/root/specs/ods")));
        assert!(is_excluded_profile_catalog(&ws, Path::new("/root/okf/sub")));
        assert!(!is_excluded_profile_catalog(&ws, Path::new("/root/docs")));

        let mut d = crate::parse::parse_document_text(&ws.root, PathBuf::from("/root/ignored/doc.md"), "---\nprofile: note\n---\n\n# Doc\n", true);
        d.directory = PathBuf::from("/root/ignored");
        ws.documents.push(d);
        rebuild_indexes(&mut ws);

        let children = directory_children_for(&ws, Path::new("/nonexistent_dir_12345"));
        assert!(children.is_empty());

        let joined = normalize_join(Path::new("/tmp/a/b"), Path::new("../c/d.png"));
        assert_eq!(joined, PathBuf::from("/tmp/a/c/d.png"));

        assert!(paths_equal_normalized(Path::new("/tmp/a/./b"), Path::new("/tmp/a/b")));
    }
}
