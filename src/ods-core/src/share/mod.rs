//! Share-aware workspace publishing (`ods share`).
//!
//! A document's `share` visibility is set on the document frontmatter only
//! (default `public`). Nested index inheritance was removed.
//! `publish_workspace` copies visible documents and writes `ods.toml` at out.

#[allow(unused_imports)]
use crate::fs::load_workspace;
use crate::model::{Document, Frontmatter, FrontmatterState, Workspace};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShareLevel {
    #[default]
    Public,
    Org,
    Private,
}

impl ShareLevel {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "public" => Some(Self::Public),
            "org" => Some(Self::Org),
            "private" => Some(Self::Private),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Org => "org",
            Self::Private => "private",
        }
    }
}

fn parsed_fm(document: &Document) -> Option<&Frontmatter> {
    match &document.frontmatter {
        FrontmatterState::Parsed(fm) => Some(fm),
        _ => None,
    }
}

fn is_index_md(path: &Path) -> bool {
    path.file_name().and_then(|n| n.to_str()).is_some_and(|n| {
        n.eq_ignore_ascii_case("index.md") || n.eq_ignore_ascii_case("index.ods.md")
    })
}

/// Resolve the effective share visibility for a document.
///
/// Document frontmatter `share` only (nested indexes removed — no folder inheritance).
pub fn effective_share(doc_path: &Path, workspace: &Workspace) -> ShareLevel {
    if let Some(doc) = workspace.document_by_path(doc_path)
        && let Some(fm) = parsed_fm(doc)
        && let Some(level) = fm.share.as_deref().and_then(ShareLevel::parse)
    {
        return level;
    }
    ShareLevel::Public
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ShareOptions {
    pub include_org: bool,
    pub include_private: bool,
}

#[derive(Debug, Clone, Default)]
pub struct SharePublishReport {
    /// Relative (to `scope`) paths of documents copied into the output directory.
    pub written: Vec<PathBuf>,
    /// Relative (to `scope`) paths of documents excluded by share visibility.
    pub excluded: Vec<PathBuf>,
}

fn included(level: ShareLevel, options: ShareOptions) -> bool {
    match level {
        ShareLevel::Public => true,
        ShareLevel::Org => options.include_org,
        ShareLevel::Private => options.include_private,
    }
}

/// Copy the subset of `workspace` under `scope` that passes share-visibility
/// filtering into `out`, then regenerate `out`'s own `index.md` files so the
/// result is a standalone, lint-clean ODS workspace directory.
///
/// `index.md` files are never copied verbatim — their child lists are
/// specific to the source tree, so they are always regenerated from what
/// actually landed in `out`. This also means a promoted subdirectory root
/// automatically gets its `ods`/`ods` workspace markers synthesized by
/// the regular index generator, the same way `ods init` would.
pub fn publish_workspace(
    workspace: &Workspace,
    scope: impl AsRef<Path>,
    out: impl AsRef<Path>,
    options: ShareOptions,
) -> io::Result<SharePublishReport> {
    let scope = scope.as_ref();
    let scope = scope.canonicalize().unwrap_or_else(|_| scope.to_path_buf());
    let out = out.as_ref();

    fs::create_dir_all(out)?;

    let mut report = SharePublishReport::default();

    for document in &workspace.documents {
        let doc_path = document
            .path
            .canonicalize()
            .unwrap_or_else(|_| document.path.clone());
        if is_index_md(&doc_path) {
            continue;
        }
        if !doc_path.starts_with(&scope) {
            continue;
        }
        let rel = doc_path
            .strip_prefix(&scope)
            .unwrap_or(&doc_path)
            .to_path_buf();

        let level = effective_share(&document.path, workspace);
        if !included(level, options) {
            report.excluded.push(rel);
            continue;
        }

        let dest = out.join(&rel);
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = fs::read(&document.path)?;
        fs::write(&dest, bytes)?;
        report.written.push(rel);
    }

    // Materialize workspace marker at out root (no nested indexes).
    let mut out_cfg = workspace.config.clone();
    if out_cfg.spec.trim().is_empty() {
        out_cfg.spec = crate::model::current_ods_spec_version().to_string();
    }
    let _ = crate::config::write_ods_toml(out.as_ref(), &out_cfg);

    Ok(report)
}

#[cfg(test)]
include!("tests.rs");
