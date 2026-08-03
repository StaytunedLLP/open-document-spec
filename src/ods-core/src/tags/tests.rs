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

    #[test]
    fn tag_suggestions_and_rewrite_edge_cases() {
        assert!(is_builtin_tag("oncall"));
        assert!(!is_builtin_tag("custom_tag_123"));
        assert!(!is_builtin_tag(""));

        // Single scalar inline
        let text_scalar = "---\ntags: old\n---\n";
        let out = rewrite_tags_in_text(text_scalar, "old", "new").unwrap();
        assert!(out.contains("tags: new"));

        // Unmatched inline bracket
        let text_unmatched = "---\ntags: [x, y]\n---\n";
        let out = rewrite_tags_in_text(text_unmatched, "old", "new").unwrap();
        assert_eq!(out, text_unmatched);

        // Quoted tag item and comments/empty line/next key under tags
        let text_complex = "---\ntags:\n  - \"old\"\n  - \nprofile: note\n---\n";
        let out = rewrite_tags_in_text(text_complex, "old", "new").unwrap();
        assert!(out.contains("- \"new\""));
    }

    #[test]
    fn tags_catalog_all_helpers_and_warnings_test() {
        let dir = ods_test_support::temp_workspace();
        std::fs::write(dir.join("index.md"), "---\nprofile: index\nods: 0.1\n---\n\n# R\n").unwrap();
        std::fs::write(
            dir.join("a.md"),
            "---\nprofile: note\ntags:\n  - \"tag with spaces\"\n  - draft\n  - feature\n---\n\n# A\n",
        )
        .unwrap();

        let ws = crate::fs::load_workspace(&dir).unwrap();

        assert_eq!(observed_tags(&ws), vec!["draft".to_string(), "feature".to_string(), "tag with spaces".to_string()]);
        assert!(!docs_with_any_tag(&ws, &["draft".to_string()]).is_empty());
        assert!(!tag_usage_with_builtins(&ws, true).is_empty());

        let doc_a = ws.document_by_path(&dir.join("a.md")).unwrap();
        let diags = lint_document_tags(doc_a);
        assert!(diags.iter().any(|d| d.message.contains("tag has spaces")));
        assert!(diags.iter().any(|d| d.message.contains("tag collides with status value")));
        assert!(diags.iter().any(|d| d.message.contains("tag collides with profile name")));
    }
}
