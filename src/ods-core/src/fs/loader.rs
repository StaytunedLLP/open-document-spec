use crate::model::{Document, LoadOptions, Workspace};
use crate::parse::{document_id, split_frontmatter};
use crate::pipeline::{discover_markdown_paths, parse_path, parse_paths_parallel};
use crate::profiles::{load_profile_catalog, profile_catalog_roots};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Directory / file base names that tooling never treats as documentation content.
pub(crate) const DEFAULT_IGNORE_NAMES: &[&str] = &[
    "target",
    "node_modules",
    "dist",
    "build",
    ".artifacts",
    ".git",
    ".hg",
    ".svn",
    ".jj",
    "__pycache__",
    ".venv",
    "venv",
    "vendor",
];

/// Load options optimized for graph ops (lint, doctor, index, context): no body retention.
pub fn load_options_graph() -> LoadOptions {
    LoadOptions {
        include_body: false,
        respect_gitignore: true,
    }
}

pub fn load_workspace(root: impl AsRef<Path>) -> io::Result<Workspace> {
    load_workspace_with_options(root, LoadOptions::default())
}

/// Functional pipeline: discover → parallel parse → rebuild_indexes.
pub fn load_workspace_with_options(
    root: impl AsRef<Path>,
    options: LoadOptions,
) -> io::Result<Workspace> {
    let root = root
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root.as_ref().to_path_buf());
    let gitignore = if options.respect_gitignore {
        load_gitignore_patterns(&root)
    } else {
        Vec::new()
    };

    let root_index_path = root.join("index.ods.md");
    let root_index = if root_index_path.exists() {
        Some(parse_path(
            &root,
            root_index_path.clone(),
            options.include_body,
        )?)
    } else {
        None
    };

    let profile_roots = profile_catalog_roots(&root, root_index.as_ref());
    let profile_catalog = load_profile_catalog(&root, &profile_roots)?;
    let mut workspace_ignore = workspace_ignore_from_root(root_index.as_ref());
    workspace_ignore.extend(load_odsignore_patterns(&root));

    let mut paths = discover_markdown_paths(
        &root,
        &profile_roots,
        &gitignore,
        &workspace_ignore,
    )?;
    let root_index_path = root_index.as_ref().map(|doc| doc.path.clone());
    paths.retain(|path| Some(path) != root_index_path.as_ref());

    let mut documents = Vec::with_capacity(paths.len() + usize::from(root_index.is_some()));
    if let Some(root_index) = root_index {
        documents.push(root_index);
    }
    let mut rest = parse_paths_parallel(&root, &paths, options.include_body)?;
    documents.append(&mut rest);

    let mut workspace = Workspace {
        root,
        documents,
        profiles: profile_catalog,
        profile_roots,
        by_id: HashMap::new(),
        by_path: HashMap::new(),
        children: HashMap::new(),
        resource_paths: HashSet::new(),
        code_paths: HashSet::new(),
        ignore: workspace_ignore,
        tag_index: std::collections::BTreeMap::new(),
        profile_catalog_paths: HashSet::new(),
        doc_dirs: HashSet::new(),
    };
    rebuild_indexes(&mut workspace);
    Ok(workspace)
}

/// Load ignore patterns from a `.odsignore` file if present at `root`.
pub fn load_odsignore_patterns(root: &Path) -> Vec<String> {
    let odsignore_path = root.join(".odsignore");
    let Ok(content) = fs::read_to_string(odsignore_path) else {
        return Vec::new();
    };
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Locate the ODS workspace root for a file path.
///
/// Prefers the **nearest** ancestor whose `index.ods.md` declares `ods:` (workspace marker).
///
/// Relative paths are resolved against the process cwd **before** walking parents so
/// component pop never yields an empty `PathBuf` (which would match cwd files via
/// `"".join("index.ods.md")` and return a broken empty root).
pub fn find_workspace_root(path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    let abs = absolute_probe_path(path)?;
    let start = if abs.is_dir() {
        abs
    } else {
        abs.parent()
            .filter(|p| !p.as_os_str().is_empty())?
            .to_path_buf()
    };

    let mut current = start;
    let mut nearest_index = None::<PathBuf>;
    let mut nearest_ods = None::<PathBuf>;

    loop {
        // Empty paths must never be treated as roots (relative-walk bug).
        if current.as_os_str().is_empty() {
            break;
        }

        let index_ods_path = current.join("index.ods.md");
        let index_md_path = current.join("index.md");

        let index_path = if index_ods_path.is_file() {
            Some(index_ods_path)
        } else if index_md_path.is_file() {
            Some(index_md_path)
        } else {
            None
        };

        if let Some(idx_path) = index_path {
            if nearest_index.is_none() {
                nearest_index = Some(current.clone());
            }
            if nearest_ods.is_none() && index_has_ods_field(&idx_path) {
                nearest_ods = Some(current.clone());
                break;
            }
        }

        if current.join(".git").exists() {
            break;
        }

        if !current.pop() || current.as_os_str().is_empty() {
            break;
        }
    }

    let found = nearest_ods.or(nearest_index)?;
    if found.as_os_str().is_empty() {
        return None;
    }
    Some(found.canonicalize().unwrap_or(found))
}

/// Make a probe path absolute without requiring it to exist on disk.
///
/// Used so ancestor walks for document ids like `specs/ods/core` start from a real
/// directory chain under cwd, not a relative path that collapses to `""`.
fn absolute_probe_path(path: &Path) -> Option<PathBuf> {
    if path.as_os_str().is_empty() {
        return std::env::current_dir().ok();
    }
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    Some(
        joined
            .canonicalize()
            .unwrap_or_else(|_| crate::fs::normalize_path(&joined)),
    )
}

/// True if the given `index.md` or `index.ods.md` path declares an `ods:` frontmatter field.
pub fn index_has_ods_field(index_path: &Path) -> bool {
    let Ok(text) = fs::read_to_string(index_path) else {
        return false;
    };
    let (frontmatter, _) = split_frontmatter(&text);
    let Some(block) = frontmatter else {
        return false;
    };
    block.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.starts_with("ods:")
            || trimmed
                .split_once(':')
                .is_some_and(|(k, _)| k.trim() == "ods")
    })
}

/// Insert or replace a document and rebuild indexes (incremental LSP path).
pub fn upsert_document(workspace: &mut Workspace, document: Document) {
    if let Some(idx) = workspace.by_path.get(&document.path).copied() {
        workspace.documents[idx] = document;
    } else if let Some(idx) = workspace
        .documents
        .iter()
        .position(|doc| doc.path == document.path)
    {
        workspace.documents[idx] = document;
    } else {
        workspace.documents.push(document);
    }
    rebuild_indexes(workspace);
}

/// Remove a document by path and rebuild indexes.
pub fn remove_document(workspace: &mut Workspace, path: &Path) -> bool {
    let before = workspace.documents.len();
    workspace.documents.retain(|doc| doc.path != path);
    if workspace.documents.len() != before {
        rebuild_indexes(workspace);
        true
    } else {
        false
    }
}
