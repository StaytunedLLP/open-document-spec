//! Incremental workspace updates (explicit mutation boundary for watch).

use crate::fs::rebuild_indexes;
use crate::model::{Document, Workspace};
use std::path::Path;

/// Upsert many documents, then rebuild indexes once.
pub fn apply_document_upserts(workspace: &mut Workspace, documents: Vec<Document>) {
    if documents.is_empty() {
        return;
    }
    for document in documents {
        upsert_document_no_reindex(workspace, document);
    }
    rebuild_indexes(workspace);
}

/// Remove many documents by path, then rebuild indexes once if anything changed.
pub fn apply_document_removes(workspace: &mut Workspace, paths: &[&Path]) -> usize {
    if paths.is_empty() {
        return 0;
    }
    let before = workspace.documents.len();
    workspace
        .documents
        .retain(|doc| !paths.iter().any(|p| doc.path == *p));
    let removed = before.saturating_sub(workspace.documents.len());
    if removed > 0 {
        rebuild_indexes(workspace);
    }
    removed
}

fn upsert_document_no_reindex(workspace: &mut Workspace, document: Document) {
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
}
