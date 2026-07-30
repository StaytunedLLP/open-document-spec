use super::model::{
    OkfBundle, OkfDocument, OkfFrontmatter, OkfFrontmatterState, OkfLintLevel, OkfTrustTier,
    derive_trust_tier, diag,
};
use crate::model::{Diagnostic, Severity};

pub fn lint_okf_bundle(bundle: &OkfBundle) -> Vec<Diagnostic> {
    lint_okf_bundle_with_level(bundle, OkfLintLevel::Level3)
}

pub fn lint_okf_bundle_with_level(bundle: &OkfBundle, level: OkfLintLevel) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    match bundle.okf_version.as_deref() {
        Some("0.2") => {}
        Some(other) => out.push(diag(
            bundle.root.join("index.md"),
            Severity::Warning,
            format!("okf_version is {other:?}; engine targets 0.2"),
        )),
        None => out.push(diag(
            bundle.root.join("index.md"),
            Severity::Warning,
            "root index.md missing okf_version: \"0.2\"",
        )),
    }

    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        out.extend(lint_concept(doc, level));
    }
    out
}

fn lint_concept(doc: &OkfDocument, level: OkfLintLevel) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    match &doc.frontmatter {
        OkfFrontmatterState::Absent => {
            out.push(diag(
                doc.path.clone(),
                Severity::Error,
                "OKF concept missing YAML frontmatter",
            ));
        }
        OkfFrontmatterState::Invalid(err) => {
            out.push(diag(
                doc.path.clone(),
                Severity::Error,
                format!("invalid OKF frontmatter: {err}"),
            ));
        }
        OkfFrontmatterState::Parsed(fm) => {
            out.extend(lint_parsed(doc, fm, level));
        }
    }
    out
}

fn lint_parsed(doc: &OkfDocument, fm: &OkfFrontmatter, level: OkfLintLevel) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let type_name = fm.type_name.as_deref().unwrap_or("").trim();
    if type_name.is_empty() {
        out.push(diag(
            doc.path.clone(),
            Severity::Error,
            "missing required frontmatter field: type",
        ));
    }

    if type_name.eq_ignore_ascii_case("Attested Computation")
        && fm.runtime.as_deref().unwrap_or("").trim().is_empty()
    {
        out.push(diag(
            doc.path.clone(),
            Severity::Error,
            "Attested Computation requires runtime",
        ));
    }

    if matches!(level, OkfLintLevel::Level1) {
        return out;
    }

    // Level 3 shape checks when families present
    if let Some(generated) = &fm.generated {
        if generated.by.trim().is_empty() {
            out.push(diag(
                doc.path.clone(),
                Severity::Error,
                "generated.by is required when generated is present",
            ));
        }
    }
    for (idx, v) in fm.verified.iter().enumerate() {
        if v.by.trim().is_empty() {
            out.push(diag(
                doc.path.clone(),
                Severity::Error,
                format!("verified[{idx}].by is required"),
            ));
        }
    }
    for (idx, src) in fm.sources.iter().enumerate() {
        if src.resource.as_deref().unwrap_or("").trim().is_empty() {
            out.push(diag(
                doc.path.clone(),
                Severity::Error,
                format!("sources[{idx}].resource is required within a sources entry"),
            ));
        }
    }
    if let Some(date) = &fm.stale_after {
        if !is_yyyy_mm_dd(date) {
            out.push(diag(
                doc.path.clone(),
                Severity::Warning,
                format!("stale_after should be YYYY-MM-DD, got {date:?}"),
            ));
        } else if is_stale(date) {
            out.push(diag(
                doc.path.clone(),
                Severity::Warning,
                format!("concept is stale (stale_after: {date})"),
            ));
        }
    }
    // trust tier is advisory (derived for doctor/CLI consumers; not a lint failure)
    let _tier: OkfTrustTier = derive_trust_tier(&fm.verified);
    let _ = _tier;
    out
}

fn is_yyyy_mm_dd(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b[0..4].iter().all(|c| c.is_ascii_digit())
        && b[5..7].iter().all(|c| c.is_ascii_digit())
        && b[8..10].iter().all(|c| c.is_ascii_digit())
}

fn is_stale(stale_after: &str) -> bool {
    // Compare as YYYY-MM-DD strings (ISO order)
    let today = chrono_like_today();
    today.as_str() >= stale_after
}

fn chrono_like_today() -> String {
    system_time_to_today(std::time::SystemTime::now())
}

fn system_time_to_today(now: std::time::SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let Ok(dur) = now.duration_since(UNIX_EPOCH) else {
        return "1970-01-01".into();
    };
    let days = dur.as_secs() / 86400;
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant civil_from_days (proleptic Gregorian)
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::okf::parse::parse_okf_document_text;
    use std::path::PathBuf;

    fn doc(text: &str) -> OkfDocument {
        OkfDocument {
            path: PathBuf::from("metrics/revenue.md"),
            concept_id: "metrics/revenue".into(),
            body: String::new(),
            frontmatter: parse_okf_document_text(text),
            is_reserved: false,
        }
    }

    #[test]
    fn requires_type() {
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
