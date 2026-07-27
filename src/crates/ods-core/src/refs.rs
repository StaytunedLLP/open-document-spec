use crate::fs::{normalize_join, normalize_path};
use crate::model::{Document, FrontmatterState, Workspace};
use crate::parse::document_id;
use std::path::{Component, Path, PathBuf};

pub fn is_markdown_ref(reference: &str) -> bool {
    Path::new(reference)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

pub fn is_file_like_ref(reference: &str) -> bool {
    Path::new(reference).extension().is_some() || reference.contains('.')
}

pub fn document_ref_to_path(
    workspace: &Workspace,
    document: &Document,
    reference: &str,
) -> Option<PathBuf> {
    let reference = normalize_ref(reference);
    if reference.is_empty() {
        return None;
    }

    if is_markdown_ref(&reference) {
        for candidate in [
            normalize_join(&document.directory, Path::new(&reference)),
            normalize_join(&workspace.root, Path::new(&reference)),
        ] {
            if let Some(target) = workspace.document_by_path(&candidate) {
                return Some(target.path.clone());
            }
        }

        let without_md = reference
            .strip_suffix(".md")
            .or_else(|| reference.strip_suffix(".MD"))
            .unwrap_or(&reference)
            .to_lowercase();
        return workspace
            .document_by_id(&without_md)
            .map(|doc| doc.path.clone());
    }

    if let Some(document) = workspace.document_by_id(&reference.to_lowercase()) {
        return Some(document.path.clone());
    }

    let markdown_reference = format!("{reference}.md");
    for candidate in [
        normalize_join(&document.directory, Path::new(&markdown_reference)),
        normalize_join(&workspace.root, Path::new(&markdown_reference)),
    ] {
        if let Some(target) = workspace.document_by_path(&candidate) {
            return Some(target.path.clone());
        }
    }

    None
}

pub fn document_ref_to_id(
    workspace: &Workspace,
    document: &Document,
    reference: &str,
) -> Option<String> {
    let path = document_ref_to_path(workspace, document, reference)?;
    let target = workspace.document_by_path(&path)?;
    Some(id_for(workspace, target))
}

pub fn canonical_document_ref(
    _workspace: &Workspace,
    from_document: &Document,
    target_document: &Document,
) -> String {
    relative_path(&from_document.directory, &target_document.path)
}

pub fn canonical_document_ref_for_reference(
    workspace: &Workspace,
    document: &Document,
    reference: &str,
) -> Option<String> {
    let path = document_ref_to_path(workspace, document, reference)?;
    let target = workspace.document_by_path(&path)?;
    Some(canonical_document_ref(workspace, document, target))
}

pub fn id_for(workspace: &Workspace, document: &Document) -> String {
    let frontmatter = match &document.frontmatter {
        FrontmatterState::Parsed(frontmatter) => Some(frontmatter),
        _ => None,
    };
    document_id(&workspace.root, &document.path, frontmatter)
}

pub fn normalize_ref(reference: &str) -> String {
    reference.trim().replace('\\', "/")
}

fn relative_path(from_dir: &Path, to: &Path) -> String {
    let from = normalize_path(from_dir);
    let to = normalize_path(to);
    let from_components = from.components().collect::<Vec<_>>();
    let to_components = to.components().collect::<Vec<_>>();

    let mut i = 0usize;
    while i < from_components.len()
        && i < to_components.len()
        && from_components[i] == to_components[i]
    {
        i += 1;
    }

    let mut parts = Vec::<String>::new();
    for component in &from_components[i..] {
        if matches!(component, Component::Normal(_)) {
            parts.push("..".to_string());
        }
    }
    for component in &to_components[i..] {
        if let Component::Normal(value) = component {
            parts.push(value.to_string_lossy().to_string());
        }
    }

    if parts.is_empty() {
        to.file_name()
            .map(|value| value.to_string_lossy().to_string())
            .unwrap_or_else(|| ".".to_string())
    } else {
        parts.join("/")
    }
}
