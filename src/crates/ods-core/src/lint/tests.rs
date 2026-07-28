fn frontmatter(document: &Document) -> Option<&crate::model::Frontmatter> {
    match &document.frontmatter {
        FrontmatterState::Parsed(frontmatter) => Some(frontmatter),
        _ => None,
    }
}
fn is_resource_like(text: &str) -> bool {
    text.contains('.')
}
