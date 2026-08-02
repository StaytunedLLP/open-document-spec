//! Share-aware workspace publishing (`ods share`).
//!
//! A document's `share` visibility is either set directly on the document, or
//! inherited from the nearest ancestor directory's `index.md` `share` value
//! (a directory-level default), falling back to `public` when nothing is set
//! anywhere. `publish_workspace` uses this to copy only the documents that
//! should be visible into a fresh, git-ready output directory.

use crate::fs::load_workspace;
use crate::index::generate_indexes;
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
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case("index.md"))
}

/// Resolve the effective share visibility for a document.
///
/// Precedence: the document's own `share` frontmatter wins if set; otherwise
/// the nearest ancestor directory's `index.md` `share` value is used as a
/// directory-level default (walking up to, and including, `workspace.root`);
/// otherwise `ShareLevel::Public`.
pub fn effective_share(doc_path: &Path, workspace: &Workspace) -> ShareLevel {
    if let Some(doc) = workspace.document_by_path(doc_path)
        && let Some(fm) = parsed_fm(doc)
        && let Some(level) = fm.share.as_deref().and_then(ShareLevel::parse)
    {
        return level;
    }

    let Some(mut current) = doc_path.parent().map(Path::to_path_buf) else {
        return ShareLevel::Public;
    };

    loop {
        let index_path = current.join("index.md");
        if index_path != doc_path
            && let Some(doc) = workspace.document_by_path(&index_path)
            && let Some(fm) = parsed_fm(doc)
            && let Some(level) = fm.share.as_deref().and_then(ShareLevel::parse)
        {
            return level;
        }

        if current == workspace.root {
            break;
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => break,
        }
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

    let published = load_workspace(out)?;
    let _ = generate_indexes(&published)?;

    Ok(report)
}

#[cfg(test)]
include!("tests.rs");
