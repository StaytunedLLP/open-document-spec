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
