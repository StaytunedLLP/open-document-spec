

pub fn existing_index_links(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(split_markdown_link_target)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::load_workspace;
    use ods_test_support::temp_workspace;
    use std::fs;

    #[test]
    fn root_index_render_injects_cli_requirement() {
        let dir = temp_workspace();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nprofiles:\n  - docs/guide/07-examples/ecommerce/ods-profiles\nignore:\n  - skills\n  - src/zed-ods-lsp\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();
        fs::write(
            dir.join("note.md"),
            "---\nprofile: note\nstatus: draft\n---\n\n# N\n",
        )
        .unwrap();

        let ws = load_workspace(&dir).unwrap();
        let rendered = render_index(
            &ws,
            &ws.root,
            Some(&fs::read_to_string(dir.join("index.md")).unwrap()),
        );

        assert!(
            rendered.contains(&format!(
                "ods-cli: \"{}\"",
                crate::model::current_ods_cli_requirement()
            )),
            "{rendered}"
        );
    }

    #[test]
    fn nested_docs_get_ancestor_indexes() {
        let dir = temp_workspace();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Root\n",
        )
        .unwrap();
        fs::create_dir_all(dir.join("a/b")).unwrap();
        fs::write(
            dir.join("a/b/doc.md"),
            "---\nprofile: note\nstatus: draft\ndescription: Hello\n---\n\n# Doc\n",
        )
        .unwrap();

        let ws = load_workspace(&dir).unwrap();
        let touched = generate_indexes(&ws).unwrap();
        assert!(touched.iter().any(|p| p.ends_with("a/index.md")));
        assert!(touched.iter().any(|p| p.ends_with("a/b/index.md")));
        let ab = fs::read_to_string(dir.join("a/b/index.md")).unwrap();
        assert!(ab.contains("doc.md") && ab.contains("Hello"), "{ab}");
        let a = fs::read_to_string(dir.join("a/index.md")).unwrap();
        assert!(a.contains("b/index.md"), "{a}");
        assert!(indexes_are_current(&load_workspace(&dir).unwrap()).unwrap());
    }

    #[test]
    fn orphan_index_pruned_when_docs_removed() {
        let dir = temp_workspace();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Root\n",
        )
        .unwrap();
        fs::create_dir_all(dir.join("gone")).unwrap();
        fs::write(
            dir.join("gone/index.md"),
            "---\nprofile: index\n---\n\n# gone\n\n- [x.md](x.md)\n",
        )
        .unwrap();
        // no x.md — orphan managed index
        let ws = load_workspace(&dir).unwrap();
        let touched = generate_indexes(&ws).unwrap();
        assert!(
            touched.iter().any(|p| p.ends_with("gone/index.md")),
            "expected prune of orphan: {touched:?}"
        );
        assert!(!dir.join("gone/index.md").exists());
        assert!(dir.join("index.md").exists());
    }

    #[test]
    fn description_change_updates_parent_index() {
        let dir = temp_workspace();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Root\n",
        )
        .unwrap();
        fs::write(
            dir.join("note.md"),
            "---\nprofile: note\nstatus: draft\ndescription: Old\n---\n\n# N\n",
        )
        .unwrap();
        let ws = load_workspace(&dir).unwrap();
        generate_indexes(&ws).unwrap();
        let idx = fs::read_to_string(dir.join("index.md")).unwrap();
        assert!(idx.contains("Old"), "{idx}");

        fs::write(
            dir.join("note.md"),
            "---\nprofile: note\nstatus: draft\ndescription: New desc\n---\n\n# N\n",
        )
        .unwrap();
        let ws = load_workspace(&dir).unwrap();
        generate_indexes(&ws).unwrap();
        let idx = fs::read_to_string(dir.join("index.md")).unwrap();
        assert!(idx.contains("New desc"), "{idx}");
        assert!(!idx.contains("Old"), "{idx}");
    }
}
