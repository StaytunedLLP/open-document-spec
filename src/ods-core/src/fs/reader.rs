use crate::model::FrontmatterState;
use serde::{Deserialize, Serialize};

/// Options for fine-grained section and content reading.
#[derive(Clone, Debug, Default)]
pub struct ReadOptions {
    /// Target section heading or slug (e.g. "## Architecture" or "architecture").
    pub section: Option<String>,
    /// Extract summary outline (headings and metadata) only.
    pub summary_only: bool,
    /// Soft token budget limit (bytes / 4).
    pub max_tokens: Option<usize>,
}

/// Outline heading entry in document structure.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionOutline {
    pub title: String,
    pub level: usize,
    pub line_number: usize,
}

/// Result of reading document content.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadResult {
    pub id: String,
    pub path: PathBuf,
    pub profile: Option<String>,
    pub status: Option<String>,
    pub title: Option<String>,
    pub content: String,
    pub outline: Vec<SectionOutline>,
    pub token_estimate: usize,
    pub truncated: bool,
}

/// Read and filter a document's content by section, summary, or token budget.
pub fn read_document_content(
    workspace: &Workspace,
    query: &str,
    options: &ReadOptions,
) -> Result<ReadResult, String> {
    let doc = find_document(workspace, query)
        .ok_or_else(|| format!("document not found matching '{query}' in workspace"))?;

    let fm_opt = match &doc.frontmatter {
        FrontmatterState::Parsed(fm) => Some(fm),
        _ => None,
    };

    let doc_id = crate::parse::document_id(&workspace.root, &doc.path, fm_opt);
    let (profile, status, title, tags) = match fm_opt {
        Some(fm) => (
            fm.profile.clone(),
            fm.status.clone(),
            fm.title.clone(),
            fm.tags.clone(),
        ),
        None => (None, None, None, Vec::new()),
    };

    let full_text = fs::read_to_string(&doc.path)
        .map_err(|e| format!("failed to read file {}: {e}", doc.path.display()))?;

    let outline = extract_outline(&full_text);

    let (mut extracted_content, mut is_truncated) = if options.summary_only {
        (render_summary(&doc_id, doc, &profile, &status, &tags, &outline), false)
    } else if let Some(ref sec_target) = options.section {
        extract_section_content(&full_text, sec_target, &outline)
    } else {
        (full_text.clone(), false)
    };

    if let Some(max_tok) = options.max_tokens {
        let max_bytes = max_tok * 4;
        if extracted_content.len() > max_bytes {
            let mut char_count = 0;
            let mut truncate_pos = extracted_content.len();
            for (idx, _ch) in extracted_content.char_indices() {
                if idx >= max_bytes {
                    truncate_pos = idx;
                    break;
                }
                char_count += 1;
            }
            let _ = char_count;
            extracted_content.truncate(truncate_pos);
            extracted_content.push_str(&format!("\n\n... [truncated to {max_tok} max tokens]"));
            is_truncated = true;
        }
    }

    let token_est = extracted_content.len().div_ceil(4);

    Ok(ReadResult {
        id: doc_id,
        path: doc.path.clone(),
        profile,
        status,
        title,
        content: extracted_content,
        outline,
        token_estimate: token_est,
        truncated: is_truncated,
    })
}

fn find_document<'a>(workspace: &'a Workspace, query: &str) -> Option<&'a Document> {
    if let Some(&idx) = workspace.by_id.get(query) {
        return workspace.documents.get(idx);
    }
    let norm_q = crate::fs::normalize_path(Path::new(query));
    if let Some(&idx) = workspace.by_path.get(&norm_q) {
        return workspace.documents.get(idx);
    }
    let target = workspace.root.join(query);
    let norm_target = crate::fs::normalize_path(&target);
    if norm_target.starts_with(&workspace.root) {
        if let Some(&idx) = workspace.by_path.get(&norm_target) {
            return workspace.documents.get(idx);
        }
    }
    // Substring fallback (only within workspace documents)
    workspace.documents.iter().find(|d| {
        let fm_opt = match &d.frontmatter {
            FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        };
        let id = crate::parse::document_id(&workspace.root, &d.path, fm_opt);
        id.eq_ignore_ascii_case(query)
            || d.path.to_string_lossy().ends_with(query)
            || id.ends_with(query)
    })
}

fn extract_outline(text: &str) -> Vec<SectionOutline> {
    let mut outline = Vec::new();
    let mut in_code_block = false;

    for (idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }

        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|&c| c == '#').count();
            if level > 0 && level <= 6 {
                let title = trimmed[level..].trim().to_string();
                outline.push(SectionOutline {
                    title,
                    level,
                    line_number: idx + 1,
                });
            }
        }
    }
    outline
}

fn render_summary(
    doc_id: &str,
    doc: &Document,
    profile: &Option<String>,
    status: &Option<String>,
    tags: &[String],
    outline: &[SectionOutline],
) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Outline Summary: {}\n\n", doc_id));
    out.push_str(&format!("• Path: {}\n", doc.path.display()));
    if let Some(p) = profile {
        out.push_str(&format!("• Profile: {p}\n"));
    }
    if let Some(s) = status {
        out.push_str(&format!("• Status: {s}\n"));
    }
    if !tags.is_empty() {
        out.push_str(&format!("• Tags: {}\n", tags.join(", ")));
    }
    out.push_str("\n## Document Structure:\n");
    if outline.is_empty() {
        out.push_str("  (no section headings found)\n");
    } else {
        for sec in outline {
            let indent = "  ".repeat(sec.level.saturating_sub(1));
            out.push_str(&format!(
                "{}• {} (line {})\n",
                indent, sec.title, sec.line_number
            ));
        }
    }
    out
}

fn extract_section_content(
    text: &str,
    target: &str,
    outline: &[SectionOutline],
) -> (String, bool) {
    let clean_target = target.trim().trim_start_matches('#').trim();
    let norm_target = slugify(clean_target);

    let found = outline.iter().find(|sec| {
        sec.title.eq_ignore_ascii_case(clean_target)
            || slugify(&sec.title) == norm_target
            || sec.title.to_lowercase().contains(&clean_target.to_lowercase())
    });

    let Some(target_sec) = found else {
        return (
            format!(
                "# Section Not Found\n\nNo section matching '{target}' found. Available sections:\n"
            ) + &outline
                .iter()
                .map(|s| format!("  - {}", s.title))
                .collect::<Vec<_>>()
                .join("\n"),
            false,
        );
    };

    let start_line = target_sec.line_number;
    let target_level = target_sec.level;

    let end_line = outline
        .iter()
        .find(|sec| sec.line_number > start_line && sec.level <= target_level)
        .map(|sec| sec.line_number)
        .unwrap_or(usize::MAX);

    let lines: Vec<&str> = text.lines().collect();
    let section_lines: Vec<&str> = lines
        .into_iter()
        .enumerate()
        .filter(|(idx, _)| {
            let line_num = idx + 1;
            line_num >= start_line && line_num < end_line
        })
        .map(|(_, line)| line)
        .collect();

    (section_lines.join("\n"), false)
}

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests_reader {
    use super::*;

    #[test]
    fn test_extract_outline_and_slugify() {
        let sample = "---\nprofile: note\n---\n\n# Main Title\n\nSome text.\n\n## Section One\n\nContent 1.\n\n### Subsection\n\nContent 2.\n\n## Section Two\n\nContent 3.\n";
        let outline = extract_outline(sample);
        assert_eq!(outline.len(), 4);
        assert_eq!(outline[0].title, "Main Title");
        assert_eq!(outline[1].title, "Section One");
        assert_eq!(slugify("Section One"), "section-one");

        let (sec_content, _) = extract_section_content(sample, "Section One", &outline);
        assert!(sec_content.contains("Section One"));
        assert!(sec_content.contains("Subsection"));
        assert!(!sec_content.contains("Section Two"));
    }
}
