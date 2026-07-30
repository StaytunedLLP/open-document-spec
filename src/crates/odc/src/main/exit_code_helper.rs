fn graph_lines(workspace: &odc_core::Workspace) -> Vec<String> {
    workspace
        .documents
        .iter()
        .flat_map(|document| {
            let prefix = document.path.display().to_string();
            let edges = match &document.frontmatter {
                odc_core::FrontmatterState::Parsed(frontmatter) => frontmatter
                    .depends
                    .iter()
                    .chain(frontmatter.related.iter())
                    .map(|reference| {
                        odc_core::canonical_document_ref_for_reference(
                            workspace,
                            document,
                            reference,
                        )
                        .unwrap_or_else(|| reference.clone())
                    })
                    .collect::<Vec<_>>(),
                _ => Vec::new(),
            };
            edges
                .into_iter()
                .map(move |edge| format!("{prefix} -> {edge}"))
        })
        .collect()
}

#[cfg(test)]
mod test_exit_code_helper {
    use super::*;

    #[test]
    fn graph_lines_dangling_reference_unwraps() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        std::fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n",
        ).unwrap();
        std::fs::write(
            root.join("a.md"),
            "---\nprofile: note\ndepends:\n  - dangling_doc_id\n---\n\n# A\n",
        ).unwrap();

        let ws = odc_core::load_workspace(root).unwrap();
        let lines = graph_lines(&ws);
        assert!(lines.iter().any(|l| l.contains("dangling_doc_id")));
    }
}
