use crate::model::{Document, LoadOptions, Workspace};
use crate::parse::{document_id, parse_document_text, split_frontmatter};
use crate::profiles::{load_profile_catalog, profile_catalog_roots};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Directory / file base names that tooling never treats as documentation content.
const DEFAULT_IGNORE_NAMES: &[&str] = &[
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

pub fn load_workspace(root: impl AsRef<Path>) -> io::Result<Workspace> {
    load_workspace_with_options(root, LoadOptions::default())
}

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

    let root_index_path = root.join("index.md");
    let root_index = if root_index_path.exists() {
        let text = fs::read_to_string(&root_index_path)?;
        Some(parse_document_text(
            &root,
            root_index_path.clone(),
            &text,
            options.include_body,
        ))
    } else {
        None
    };

    let profile_roots = profile_catalog_roots(&root, root_index.as_ref());
    let profile_catalog = load_profile_catalog(&root, &profile_roots)?;
    let workspace_ignore = workspace_ignore_from_root(root_index.as_ref());

    let mut paths = Vec::new();
    collect_markdown_paths(
        &root,
        &root,
        &mut paths,
        &profile_roots,
        &gitignore,
        &workspace_ignore,
    )?;
    paths.sort();
    let root_index_path = root_index.as_ref().map(|doc| doc.path.clone());
    paths.retain(|path| Some(path) != root_index_path.as_ref());

    let mut documents = Vec::new();
    if let Some(root_index) = root_index {
        documents.push(root_index);
    }
    for path in paths {
        let text = fs::read_to_string(&path)?;
        documents.push(parse_document_text(
            &root,
            path,
            &text,
            options.include_body,
        ));
    }

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

/// Locate the ODS workspace root for a file path.
///
/// Prefers the **nearest** ancestor whose `index.md` declares `ods:` (workspace marker).
/// Falls back to the nearest `index.md`, then the file's parent directory.
pub fn find_workspace_root(path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    let start = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent()?.to_path_buf()
    };

    let mut current = start;
    let mut nearest_index = None::<PathBuf>;
    let mut nearest_ods = None::<PathBuf>;

    loop {
        let index_path = current.join("index.md");
        if index_path.is_file() {
            if nearest_index.is_none() {
                nearest_index = Some(current.clone());
            }
            if nearest_ods.is_none() && index_has_ods_field(&index_path) {
                nearest_ods = Some(current.clone());
                // Nearest ods: wins — stop climbing for a closer match already found.
                break;
            }
        }

        if current.join(".git").exists() {
            break;
        }

        if !current.pop() {
            break;
        }
    }

    nearest_ods.or(nearest_index)
}

/// True if the given `index.md` path declares an `ods:` frontmatter field.
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
