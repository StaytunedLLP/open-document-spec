#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    pub(super) fn temp_dir(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ods-share-{label}-{}-{}",
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::canonicalize(&dir).unwrap_or(dir)
    }

    pub(super) fn write(dir: &Path, rel: &str, content: &str) {
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
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
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
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
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
        let dir = temp_dir("sub-index");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
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
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\nshare: private\n---\n\n# Sub\n",
        );
        write(
            dir.as_path(),
            "sub/doc.md",
            "---\nprofile: note\nstatus: draft\nid: sub-doc\nshare: public\n---\n\n# Doc\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("sub/doc.md"), &ws);
        assert_eq!(level, ShareLevel::Public);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_walks_up_multiple_directory_levels() {
        let dir = temp_dir("multi-level");
        write(
            dir.as_path(),
            "index.md",
            "---\nprofile: index\nods: 0.1\nshare: org\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "a/b/c/doc.md",
            "---\nprofile: note\nstatus: draft\nid: deep\n---\n\n# Deep\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("a/b/c/doc.md"), &ws);
        assert_eq!(level, ShareLevel::Org);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn effective_share_stops_at_workspace_root() {
        let outer = temp_dir("outside-root");
        let root = outer.join("ws");
        write(
            &outer,
            "index.md",
            "---\nprofile: index\nshare: private\n---\n\n# Outer\n",
        );
        write(
            &root,
            "index.md",
            "---\nprofile: index\nods: 0.1\n---\n\n# Inner\n",
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
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "sub/index.md",
            "---\nprofile: index\nshare: private\n---\n\n# Sub\n",
        );
        let ws = load_workspace(&dir).unwrap();
        let level = effective_share(&dir.join("sub/index.ods.md"), &ws);
        assert_eq!(level, ShareLevel::Private);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn publish_public_only_excludes_org_and_private() {
        let dir = temp_dir("publish-public");
        write(
            dir.as_path(),
            "index.ods.md",
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        );
        write(
            dir.as_path(),
            "pub.md",
            "---\nprofile: note\nstatus: draft\nid: pub\n---\n\n# Public\n",
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
        let out = temp_dir("publish-public-out");
        let report = publish_workspace(
            &ws,
            &dir,
            &out,
            ShareOptions {
                include_org: false,
                include_private: false,
            },
        )
        .unwrap();

        assert_eq!(report.written.len(), 1);
        assert!(out.join("index.ods.md").exists());
        assert!(out.join("pub.md").exists());
        assert!(!out.join("internal.md").exists());
        assert!(!out.join("secret.md").exists());
        assert_eq!(report.excluded.len(), 2);

        let out_ws = load_workspace(&out).unwrap();
        let out_idx = fs::read_to_string(out.join("index.ods.md")).unwrap();
        assert!(out_idx.contains("pub.md"));
        assert!(!out_idx.contains("internal.md"));
        assert!(!out_idx.contains("secret.md"));
        assert_eq!(out_ws.documents.len(), 2);

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn share_level_parse_and_as_str() {
        assert_eq!(ShareLevel::parse("public"), Some(ShareLevel::Public));
        assert_eq!(ShareLevel::parse("org"), Some(ShareLevel::Org));
        assert_eq!(ShareLevel::parse("private"), Some(ShareLevel::Private));
        assert_eq!(ShareLevel::parse("unknown"), None);

        assert_eq!(ShareLevel::Public.as_str(), "public");
        assert_eq!(ShareLevel::Org.as_str(), "org");
        assert_eq!(ShareLevel::Private.as_str(), "private");
    }

    #[test]
    fn publish_workspace_includes_org_and_private() {
        let dir = temp_dir("publish-all");
        write(
            dir.as_path(),
            "index.ods.md",
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
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
        let out = temp_dir("publish-all-out");
        let report = publish_workspace(
            &ws,
            &dir,
            &out,
            ShareOptions {
                include_org: true,
                include_private: true,
            },
        )
        .unwrap();

        assert_eq!(report.written.len(), 2);
        assert!(out.join("internal.md").exists());
        assert!(out.join("secret.md").exists());
        assert_eq!(report.excluded.len(), 0);

        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&out);
    }

    include!("pack_tests.rs");
}
