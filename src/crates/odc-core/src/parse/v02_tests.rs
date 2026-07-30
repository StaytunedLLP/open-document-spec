#[cfg(test)]
mod parse_v02_tests {
    use super::*;

    #[test]
    fn test_parse_share_and_packs() {
        let text = r#"---
profile: guide
status: stable
share: private
packs:
  - vendor/engineering-pack
  - ../shared-pack
---
# Test Doc
"#;
        let doc = parse_document_text(Path::new("/workspace"), PathBuf::from("/workspace/doc.md"), text, true);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            assert_eq!(fm.share.as_deref(), Some("private"));
            assert_eq!(fm.packs, vec!["vendor/engineering-pack", "../shared-pack"]);
        } else {
            panic!("expected parsed frontmatter");
        }
    }

    #[test]
    fn test_parse_share_org() {
        let text = r#"---
profile: decision
status: stable
share: org
---
# Internal Decision
"#;
        let doc = parse_document_text(Path::new("/workspace"), PathBuf::from("/workspace/doc.md"), text, true);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            assert_eq!(fm.share.as_deref(), Some("org"));
        } else {
            panic!("expected parsed frontmatter");
        }
    }
}
