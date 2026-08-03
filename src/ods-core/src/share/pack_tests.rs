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

    assert_eq!(report.written.len(), 1);
    assert!(out.join("internal.md").exists());
    assert!(!out.join("secret.md").exists());
    assert_eq!(report.excluded.len(), 1);

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn publish_include_private_flag_includes_everything() {
    let dir = temp_dir("publish-all");
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
    let out = temp_dir("publish-all-out");
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

    assert_eq!(report.written.len(), 1);
    assert!(out.join("secret.md").exists());
    assert_eq!(report.excluded.len(), 0);

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn publish_directory_share_cascade_filters_entire_subtree() {
    let dir = temp_dir("publish-sub-cascade");
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
    write(
        dir.as_path(),
        "sub/doc.md",
        "---\nprofile: note\nstatus: draft\nid: sub-doc\n---\n\n# Doc\n",
    );
    let ws = load_workspace(&dir).unwrap();
    let out = temp_dir("publish-sub-out");
    let report = publish_workspace(&ws, &dir, &out, ShareOptions::default()).unwrap();

    assert_eq!(report.written.len(), 0);
    assert!(!out.join("sub/doc.md").exists());
    assert_eq!(report.excluded.len(), 1);

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn publish_document_override_overrides_parent_directory_share() {
    let dir = temp_dir("publish-override");
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
    write(
        dir.as_path(),
        "sub/public_doc.md",
        "---\nprofile: note\nstatus: draft\nid: pub-doc\nshare: public\n---\n\n# Public Doc\n",
    );
    let ws = load_workspace(&dir).unwrap();
    let out = temp_dir("publish-override-out");
    let report = publish_workspace(&ws, &dir, &out, ShareOptions::default()).unwrap();

    assert_eq!(report.written.len(), 1);
    assert!(out.join("sub/index.ods.md").exists());
    assert!(out.join("sub/public_doc.md").exists());

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}

#[test]
fn publish_subtree_path_relative_to_root() {
    let dir = temp_dir("publish-subtree");
    write(
        dir.as_path(),
        "index.ods.md",
        "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# R\n",
    );
    write(
        dir.as_path(),
        "sub/index.ods.md",
        "---\nprofile: index\n---\n\n# Sub\n",
    );
    write(
        dir.as_path(),
        "sub/doc.md",
        "---\nprofile: note\nstatus: draft\nid: sub-doc\n---\n\n# Doc\n",
    );
    write(
        dir.as_path(),
        "other.md",
        "---\nprofile: note\nstatus: draft\nid: other\n---\n\n# Other\n",
    );

    let ws = load_workspace(&dir).unwrap();
    let out = temp_dir("publish-sub-only");
    let report = publish_workspace(&ws, dir.join("sub"), &out, ShareOptions::default()).unwrap();

    assert_eq!(report.written.len(), 1);
    assert!(out.join("index.ods.md").exists());
    assert!(out.join("doc.md").exists());
    assert!(!out.join("other.md").exists());

    let _ = fs::remove_dir_all(&dir);
    let _ = fs::remove_dir_all(&out);
}
