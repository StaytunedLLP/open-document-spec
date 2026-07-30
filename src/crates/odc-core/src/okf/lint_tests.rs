#[cfg(test)]
mod tests {
    use super::*;

    fn doc(body: &str) -> OkfDocument {
        let root = std::path::Path::new("/b");
        let path = PathBuf::from("/b/doc.md");
        let (fm, body_str) = crate::parse::split_frontmatter(body);
        let frontmatter = match fm {
            Some(b) => match crate::okf::parse_okf_frontmatter_block(b) {
                Ok(f) => OkfFrontmatterState::Parsed(f),
                Err(e) => OkfFrontmatterState::Invalid(e),
            },
            None => OkfFrontmatterState::Absent,
        };
        OkfDocument {
            path: path.clone(),
            concept_id: "doc".into(),
            body: body_str.to_string(),
            frontmatter,
            is_reserved: false,
        }
    }

    #[test]
    fn missing_type_errors() {
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![doc("---\ntitle: X\n---\n\n# X\n")],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(
            diags.iter().any(|d| d.message.contains("type")),
            "{diags:?}"
        );
    }

    #[test]
    fn attested_requires_runtime() {
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![doc(
                "---\ntype: Attested Computation\ntitle: R\n---\n\n# R\n",
            )],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(
            diags.iter().any(|d| d.message.contains("runtime")),
            "{diags:?}"
        );
    }

    #[test]
    fn type_only_is_ok() {
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![doc("---\ntype: Metric\n---\n\n# M\n")],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(
            !diags.iter().any(|d| d.severity == Severity::Error),
            "{diags:?}"
        );
    }

    #[test]
    fn warns_on_missing_or_other_okf_version() {
        let mut bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: None,
            documents: vec![],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(diags.iter().any(|d| d.message.contains("okf_version")));

        bundle.okf_version = Some("0.1".into());
        let diags = lint_okf_bundle(&bundle);
        assert!(diags.iter().any(|d| d.message.contains("0.1")));
    }

    #[test]
    fn absent_and_invalid_frontmatter() {
        let mut d = doc("# no fm\n");
        d.frontmatter = OkfFrontmatterState::Absent;
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![d],
        };
        assert!(lint_okf_bundle(&bundle)
            .iter()
            .any(|x| x.message.contains("missing YAML")));

        let mut d2 = doc("---\ntype: Metric\n---\n\n# M\n");
        d2.frontmatter = OkfFrontmatterState::Invalid("boom".into());
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![d2],
        };
        assert!(lint_okf_bundle(&bundle)
            .iter()
            .any(|x| x.message.contains("invalid OKF")));
    }

    #[test]
    fn level1_skips_shape_checks() {
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![doc(
                "---\ntype: Metric\ngenerated:\n  by: \"\"\n  at: 2020-01-01\nstale_after: not-a-date\n---\n\n# M\n",
            )],
        };
        let diags = lint_okf_bundle_with_level(&bundle, OkfLintLevel::Level1);
        assert!(
            !diags.iter().any(|d| d.message.contains("generated.by")),
            "{diags:?}"
        );
    }

    #[test]
    fn level3_sources_empty_resource_and_stale() {
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![doc(
                "---\ntype: Metric\ngenerated:\n  by: agent/x\n  at: 2020-01-01T00:00:00Z\nverified:\n  - by: human:y\n    at: 2020-01-02T00:00:00Z\nsources:\n  - id: s1\n    resource: \"\"\nstale_after: 2000-01-01\n---\n\n# M\n",
            )],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(
            diags.iter().any(|d| d.message.contains("sources[0].resource")),
            "{diags:?}"
        );
        assert!(
            diags.iter().any(|d| d.message.contains("stale")),
            "{diags:?}"
        );
    }

    #[test]
    fn level3_empty_generated_by_via_mutated_fm() {
        let mut d = doc("---\ntype: Metric\n---\n\n# M\n");
        if let OkfFrontmatterState::Parsed(ref mut fm) = d.frontmatter {
            fm.generated = Some(crate::okf::ActorEvent {
                by: String::new(),
                at: Some("2020-01-01T00:00:00Z".into()),
            });
            fm.verified = vec![crate::okf::ActorEvent {
                by: String::new(),
                at: Some("2020-01-01T00:00:00Z".into()),
            }];
        }
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![d],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(diags.iter().any(|d| d.message.contains("generated.by")), "{diags:?}");
        assert!(diags.iter().any(|d| d.message.contains("verified[0].by")), "{diags:?}");
    }

    #[test]
    fn stale_after_bad_format_warns() {
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![doc(
                "---\ntype: Metric\nstale_after: tomorrow\n---\n\n# M\n",
            )],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(
            diags.iter().any(|d| d.message.contains("YYYY-MM-DD")),
            "{diags:?}"
        );
    }

    #[test]
    fn reserved_docs_skipped() {
        let mut d = doc("---\ntype: Metric\n---\n\n# M\n");
        d.is_reserved = true;
        d.frontmatter = OkfFrontmatterState::Absent;
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![d],
        };
        let diags = lint_okf_bundle(&bundle);
        assert!(!diags.iter().any(|x| x.message.contains("missing YAML")));
    }

    #[test]
    fn civil_from_days_smoke() {
        let (y, m, d) = civil_from_days(0);
        assert_eq!((y, m, d), (1970, 1, 1));
        let (y_neg, _, _) = civil_from_days(-800000);
        assert!(y_neg < 0);
        assert!(is_yyyy_mm_dd("2024-06-15"));
        assert!(!is_yyyy_mm_dd("2024-6-15"));
        let today = chrono_like_today();
        assert_eq!(today.len(), 10);
    }

    #[test]
    fn level1_attested_computation_early_return() {
        let d = doc("---\ntype: Attested Computation\n---\n\n# AC\n");
        let bundle = OkfBundle {
            root: PathBuf::from("/b"),
            okf_version: Some("0.2".into()),
            documents: vec![d],
        };
        let diags = lint_okf_bundle_with_level(&bundle, OkfLintLevel::Level1);
        assert!(diags.iter().any(|x| x.message.contains("requires runtime")));
    }

    #[test]
    fn pre_epoch_system_time_fallback() {
        let before_epoch = std::time::UNIX_EPOCH - std::time::Duration::from_secs(100);
        assert_eq!(system_time_to_today(before_epoch), "1970-01-01");
    }
}
