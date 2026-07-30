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
    if let Some(&idx) = workspace.by_path.get(doc_path)
        && let Some(fm) = parsed_fm(&workspace.documents[idx])
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
            && let Some(&idx) = workspace.by_path.get(&index_path)
            && let Some(fm) = parsed_fm(&workspace.documents[idx])
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
/// automatically gets its `ods`/`odc` workspace markers synthesized by
/// the regular index generator, the same way `odc init` would.
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
        if is_index_md(&document.path) {
            continue;
        }
        if !document.path.starts_with(&scope) {
            continue;
        }
        let rel = document
            .path
            .strip_prefix(&scope)
            .unwrap_or(&document.path)
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
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ods-share-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::canonicalize(&dir).unwrap_or(dir)
    }

    fn write(dir: &Path, rel: &str, content: &str) {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn effective_share_defaults_to_public() {
        let dir = temp_dir("default-public");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "a.md",
            "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("a.md"), &ws);
        assert_eq!(level, ShareLevel::Public);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_uses_own_frontmatter() {
        let dir = temp_dir("own-frontmatter");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "a.md",
            "---\nprofile: note\nstatus: draft\nid: a\nshare: private\n---\n\n# A\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("a.md"), &ws);
        assert_eq!(level, ShareLevel::Private);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_cascades_from_subdirectory_index() {
        let dir = temp_dir("cascade");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\nshare: org\n---\n\n# Sub\n",
        );
        write(
            dir.as_path(),
            "sub/doc.md",
            "---\nprofile: note\nstatus: draft\nid: sub-doc\n---\n\n# Doc\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("sub/doc.md"), &ws);
        assert_eq!(level, ShareLevel::Org);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_document_overrides_ancestor_cascade() {
        let dir = temp_dir("override");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\nshare: org\n---\n\n# Sub\n",
        );
        write(
            dir.as_path(),
            "sub/doc.md",
            "---\nprofile: note\nstatus: draft\nid: sub-doc\nshare: private\n---\n\n# Doc\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("sub/doc.md"), &ws);
        assert_eq!(level, ShareLevel::Private);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_nearest_ancestor_wins_over_farther_one() {
        let dir = temp_dir("nearest-wins");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\nshare: private\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\nshare: public\n---\n\n# Sub\n",
        );
        write(
            dir.as_path(),
            "sub/doc.md",
            "---\nprofile: note\nstatus: draft\nid: sub-doc\n---\n\n# Doc\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("sub/doc.md"), &ws);
        assert_eq!(level, ShareLevel::Public);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_stops_at_workspace_root() {
        let outer = temp_dir("outer");
        write(
            outer.as_path(),
            "index.md",
            "---\nprofile: index\nshare: private\n---\n\n# Outer\n",
        );
        let root = outer.join("workspace");
        fs::create_dir_all(&root).unwrap();
        write(
            &root,
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            &root,
            "a.md",
            "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
        );
        let ws = load_workspace(&root).unwrap();
        let level = effective_share(&root.join("a.md"), &ws);
        assert_eq!(level, ShareLevel::Public);
        let _ = fs::remove_dir_all(&outer);
    }

    #[test]
    fn effective_share_index_own_value_used_for_itself() {
        let dir = temp_dir("index-self");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\nshare: private\n---\n\n# Sub\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("sub/index.md"), &ws);
        assert_eq!(level, ShareLevel::Private);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn publish_excludes_private_and_org_by_default() {
        let dir = temp_dir("publish-default");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "public.md",
            "---\nprofile: note\nstatus: draft\nid: public\n---\n\n# Public\n",
        );
        write(
            dir.as_path(),
            "secret.md",
            "---\nprofile: note\nstatus: draft\nid: secret\nshare: private\n---\n\n# Secret\n",
        );
        write(
            dir.as_path(),
            "internal.md",
            "---\nprofile: note\nstatus: draft\nid: internal\nshare: org\n---\n\n# Internal\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let out = temp_dir("publish-default-out");
        let report = publish_workspace(&ws, &dir, &out, ShareOptions::default()).unwrap();

        assert!(report.written.contains(&PathBuf::from("public.md")));
        assert!(report.excluded.contains(&PathBuf::from("secret.md")));
        assert!(report.excluded.contains(&PathBuf::from("internal.md")));
        assert!(out.join("public.md").exists());
        assert!(!out.join("secret.md").exists());
        assert!(!out.join("internal.md").exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn publish_include_org_flag() {
        let dir = temp_dir("publish-org");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "internal.md",
            "---\nprofile: note\nstatus: draft\nid: internal\nshare: org\n---\n\n# Internal\n",
        );
        write(
            dir.as_path(),
            "secret.md",
            "---\nprofile: note\nstatus: draft\nid: secret\nshare: private\n---\n\n# Secret\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let out = temp_dir("publish-org-out");
        let report = publish_workspace(
            &ws,
            &dir,
            &out,
            ShareOptions {
                include_org: true,
                include_private: false,
            },
        )
        .unwrap();

        assert!(report.written.contains(&PathBuf::from("internal.md")));
        assert!(report.excluded.contains(&PathBuf::from("secret.md")));
        assert!(out.join("internal.md").exists());
        assert!(!out.join("secret.md").exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn publish_include_private_flag() {
        let dir = temp_dir("publish-private");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "secret.md",
            "---\nprofile: note\nstatus: draft\nid: secret\nshare: private\n---\n\n# Secret\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let out = temp_dir("publish-private-out");
        let report = publish_workspace(
            &ws,
            &dir,
            &out,
            ShareOptions {
                include_org: false,
                include_private: true,
            },
        )
        .unwrap();

        assert!(report.written.contains(&PathBuf::from("secret.md")));
        assert!(out.join("secret.md").exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn publish_respects_subtree_scope() {
        let dir = temp_dir("publish-scope");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "a.md",
            "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\n---\n\n# Sub\n",
        );
        write(
            dir.as_path(),
            "sub/b.md",
            "---\nprofile: note\nstatus: draft\nid: b\n---\n\n# B\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let out = temp_dir("publish-scope-out");
        let report =
            publish_workspace(&ws, dir.join("sub"), &out, ShareOptions::default()).unwrap();

        assert!(report.written.contains(&PathBuf::from("b.md")));
        assert!(out.join("b.md").exists());
        assert!(!out.join("a.md").exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn publish_regenerates_indexes_and_output_is_valid_workspace() {
        let dir = temp_dir("publish-regen");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "a.md",
            "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let out = temp_dir("publish-regen-out");
        publish_workspace(&ws, &dir, &out, ShareOptions::default()).unwrap();

        assert!(out.join("index.md").exists());
        let reloaded = load_workspace(&out).unwrap();
        assert!(crate::index::indexes_are_current(&reloaded).unwrap());
        assert!(ods_enabled_check(&out));

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    fn ods_enabled_check(root: &Path) -> bool {
        crate::fs::index_has_ods_field(&root.join("index.md"))
    }

    #[test]
    fn publish_cascade_directory_default_excludes_whole_subtree() {
        let dir = temp_dir("publish-cascade");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "secrets/index.md",
            "---\nprofile: index\nshare: private\n---\n\n# Secrets\n",
        );
        write(
            dir.as_path(),
            "secrets/a.md",
            "---\nprofile: note\nstatus: draft\nid: secrets-a\n---\n\n# A\n",
        );
        write(
            dir.as_path(),
            "public_docs/index.md",
            "---\nprofile: index\n---\n\n# Public docs\n",
        );
        write(
            dir.as_path(),
            "public_docs/b.md",
            "---\nprofile: note\nstatus: draft\nid: public-b\n---\n\n# B\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let out = temp_dir("publish-cascade-out");
        let report = publish_workspace(&ws, &dir, &out, ShareOptions::default()).unwrap();

        assert!(
            report
                .excluded
                .contains(&PathBuf::from("secrets").join("a.md"))
        );
        assert!(
            report
                .written
                .contains(&PathBuf::from("public_docs").join("b.md"))
        );
        assert!(!out.join("secrets/a.md").exists());
        assert!(out.join("public_docs/b.md").exists());

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn share_level_methods_and_edge_cases() {
        assert_eq!(ShareLevel::parse("invalid"), None);
        assert_eq!(ShareLevel::Public.as_str(), "public");
        assert_eq!(ShareLevel::Org.as_str(), "org");
        assert_eq!(ShareLevel::Private.as_str(), "private");

        let ws = Workspace::empty(PathBuf::from("/ws"));
        // parent is None
        assert_eq!(effective_share(Path::new(""), &ws), ShareLevel::Public);
        // parent outside workspace.root
        assert_eq!(effective_share(Path::new("/other/dir/doc.md"), &ws), ShareLevel::Public);
    }
}
