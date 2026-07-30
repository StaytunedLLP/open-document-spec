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
    // Avoid extra deps: use UTC date via system if available; fallback never stale
    use std::time::{SystemTime, UNIX_EPOCH};
    let Ok(dur) = SystemTime::now().duration_since(UNIX_EPOCH) else {
        return "1970-01-01".into();
    };
    // Approximate civil date from days since epoch (adequate for staleness warning)
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
}
