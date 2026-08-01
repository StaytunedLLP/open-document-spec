use super::model::{OkfBundle, OkfFrontmatterState};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OkfAuditClass {
    Compliant,
    Plain,
    Invalid,
    Partial,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct OkfAuditItem {
    pub path: PathBuf,
    pub class: OkfAuditClass,
    pub note: String,
}

#[derive(Debug, Clone, Default)]
pub struct OkfAuditReport {
    pub items: Vec<OkfAuditItem>,
    pub total_md: usize,
    pub compliant: usize,
    pub plain: usize,
    pub invalid: usize,
    pub partial: usize,
    pub skipped: usize,
}

pub fn audit_okf_bundle(bundle: &OkfBundle) -> OkfAuditReport {
    let mut report = OkfAuditReport::default();
    for doc in &bundle.documents {
        report.total_md += 1;
        if doc.is_reserved {
            report.skipped += 1;
            report.items.push(OkfAuditItem {
                path: doc.path.clone(),
                class: OkfAuditClass::Skipped,
                note: "reserved index.md/log.md".into(),
            });
            continue;
        }
        match &doc.frontmatter {
            OkfFrontmatterState::Absent => {
                report.plain += 1;
                report.items.push(OkfAuditItem {
                    path: doc.path.clone(),
                    class: OkfAuditClass::Plain,
                    note: "no frontmatter".into(),
                });
            }
            OkfFrontmatterState::Invalid(err) => {
                report.invalid += 1;
                report.items.push(OkfAuditItem {
                    path: doc.path.clone(),
                    class: OkfAuditClass::Invalid,
                    note: err.clone(),
                });
            }
            OkfFrontmatterState::Parsed(fm) => {
                let type_ok = fm
                    .type_name
                    .as_deref()
                    .map(|t| !t.trim().is_empty())
                    .unwrap_or(false);
                if !type_ok {
                    report.partial += 1;
                    report.items.push(OkfAuditItem {
                        path: doc.path.clone(),
                        class: OkfAuditClass::Partial,
                        note: "missing type".into(),
                    });
                } else if fm.type_name.as_deref() == Some("Attested Computation")
                    && fm.runtime.as_deref().unwrap_or("").trim().is_empty()
                {
                    report.partial += 1;
                    report.items.push(OkfAuditItem {
                        path: doc.path.clone(),
                        class: OkfAuditClass::Partial,
                        note: "Attested Computation missing runtime".into(),
                    });
                } else {
                    report.compliant += 1;
                    report.items.push(OkfAuditItem {
                        path: doc.path.clone(),
                        class: OkfAuditClass::Compliant,
                        note: String::new(),
                    });
                }
            }
        }
    }
    report
}

pub fn render_okf_audit_markdown(bundle_root: &std::path::Path, report: &OkfAuditReport) -> String {
    let mut md = String::new();
    md.push_str("---\n");
    md.push_str("generated_by: ods okf audit\n");
    md.push_str(&format!("workspace: {}\n", bundle_root.display()));
    md.push_str("summary:\n");
    md.push_str(&format!("  total_md: {}\n", report.total_md));
    md.push_str(&format!("  compliant: {}\n", report.compliant));
    md.push_str(&format!("  plain: {}\n", report.plain));
    md.push_str(&format!("  invalid: {}\n", report.invalid));
    md.push_str(&format!("  partial: {}\n", report.partial));
    md.push_str("---\n\n# ODS OKF Audit Report\n\n");
    md.push_str("## Summary\n\n| Class | Count |\n|---|---|\n");
    md.push_str(&format!("| compliant | {} |\n", report.compliant));
    md.push_str(&format!("| plain | {} |\n", report.plain));
    md.push_str(&format!("| invalid | {} |\n", report.invalid));
    md.push_str(&format!("| partial | {} |\n", report.partial));
    md.push_str(&format!("| skipped | {} |\n\n", report.skipped));

    fn section(md: &mut String, title: &str, report: &OkfAuditReport, class: OkfAuditClass) {
        md.push_str(&format!("## {title}\n\n"));
        let mut any = false;
        for item in &report.items {
            if item.class == class {
                any = true;
                if item.note.is_empty() {
                    md.push_str(&format!("- `{}`\n", item.path.display()));
                } else {
                    md.push_str(&format!("- `{}` — {}\n", item.path.display(), item.note));
                }
            }
        }
        if !any {
            md.push_str("_None._\n");
        }
        md.push('\n');
    }
    section(
        &mut md,
        "Compliant concepts",
        report,
        OkfAuditClass::Compliant,
    );
    section(
        &mut md,
        "Plain Markdown (adoption candidates)",
        report,
        OkfAuditClass::Plain,
    );
    section(
        &mut md,
        "Invalid Frontmatter",
        report,
        OkfAuditClass::Invalid,
    );
    section(
        &mut md,
        "Partial / Policy Gaps",
        report,
        OkfAuditClass::Partial,
    );
    md.push_str(
        "## Suggested next commands\n\n```bash\nods adopt --okf --write\nods lint --okf\n```\n",
    );
    md
}
