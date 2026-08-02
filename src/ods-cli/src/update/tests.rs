#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_ordering() {
        assert_eq!(cmp_semver("0.1.0", "0.1.1"), std::cmp::Ordering::Less);
        assert_eq!(cmp_semver("0.2.0", "0.1.9"), std::cmp::Ordering::Greater);
        assert_eq!(cmp_semver("v1.0.0", "1.0.0"), std::cmp::Ordering::Equal);
        assert_eq!(cmp_semver("0.1.5", "0.1.5"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn normalize_tags() {
        assert_eq!(normalize_tag("0.1.5"), "v0.1.5");
        assert_eq!(normalize_tag("v0.1.5"), "v0.1.5");
    }

    #[test]
    fn host_target_is_known() {
        // Must not panic; may be Err only on exotic OS in CI (unlikely).
        let t = host_target();
        assert!(t.is_ok(), "{t:?}");
    }

    #[test]
    fn checksum_line_parse() {
        let sums = "abc123  ods-v0.1.5-linux-x86_64.tar.gz\ndef  other\n";
        assert_eq!(
            find_checksum(sums, "ods-v0.1.5-linux-x86_64.tar.gz").as_deref(),
            Some("abc123")
        );
    }

    #[test]
    fn windows_asset_detection() {
        assert!(is_windows_target("windows-x86_64"));
        assert!(is_windows_target("windows-arm64"));
        assert!(!is_windows_target("linux-x86_64"));
        assert!(!is_windows_target("macos-arm64"));
    }

    #[test]
    fn find_asset_id_before_name() {
        let json = r#"{"assets":[{"url":"x","id": 42, "node_id":"n","name": "ods-v0.1.5-linux-x86_64.tar.gz","size":1}]}"#;
        assert_eq!(
            find_asset_id(json, "ods-v0.1.5-linux-x86_64.tar.gz"),
            Some(42)
        );
        assert_eq!(find_asset_id(json, "missing"), None);
    }
}
