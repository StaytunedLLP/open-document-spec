#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_basic() {
        assert_eq!(normalize_tag("  Billing ").as_deref(), Some("billing"));
        assert_eq!(normalize_tag("   "), None);
    }

    #[test]
    fn normalize_list_dedupes() {
        assert_eq!(
            normalize_tag_list(["Billing", "billing", "oncall", ""]),
            vec!["billing".to_string(), "oncall".to_string()]
        );
    }

    #[test]
    fn rewrite_list_tags() {
        let text = "---\nprofile: note\ntags:\n  - billing\n  - old-tag\n---\n\n# Doc\n";
        let out = rewrite_tags_in_text(text, "old-tag", "new-tag").unwrap();
        assert!(out.contains("- new-tag"), "{out}");
        assert!(!out.contains("old-tag"), "{out}");
    }

    #[test]
    fn rewrite_inline_tags_bracket() {
        let text = "---\ntags: [a, old, b]\n---\n\n# D\n";
        let out = rewrite_tags_in_text(text, "old", "new").unwrap();
        assert!(out.contains("new"), "{out}");
        assert!(!out.contains("old"), "{out}");
    }
}
