use super::*;

pub(super) fn extract_prose(existing: &str, entries: &[IndexEntry]) -> (String, String) {
    let mut header_lines = Vec::new();
    let mut footer_lines = Vec::new();
    let mut title_found = false;
    let mut first_link_idx = None;
    let mut last_link_idx = None;
    let mut in_frontmatter = false;

    let lines: Vec<&str> = existing.lines().collect();

    let mut is_link_line = vec![false; lines.len()];
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed == "---" {
            in_frontmatter = !in_frontmatter;
            continue;
        }
        if in_frontmatter {
            continue;
        }

        if !title_found {
            if trimmed.starts_with("# ") {
                title_found = true;
            }
            continue;
        }

        if trimmed.starts_with("- [") {
            for entry in entries {
                let target_pattern = format!("]({})", entry.target);
                if trimmed.contains(&target_pattern) {
                    is_link_line[idx] = true;
                    if first_link_idx.is_none() {
                        first_link_idx = Some(idx);
                    }
                    last_link_idx = Some(idx);
                    break;
                }
            }
        }
    }

    if let (Some(first), Some(last)) = (first_link_idx, last_link_idx) {
        let mut title_idx = None;
        in_frontmatter = false;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "---" {
                in_frontmatter = !in_frontmatter;
                continue;
            }
            if in_frontmatter {
                continue;
            }
            if trimmed.starts_with("# ") {
                title_idx = Some(idx);
                break;
            }
        }

        if let Some(t_idx) = title_idx {
            if t_idx + 1 < first {
                header_lines.extend_from_slice(&lines[(t_idx + 1)..first]);
            }
        }

        if last + 1 < lines.len() {
            footer_lines.extend_from_slice(&lines[(last + 1)..]);
        }
    } else {
        let mut title_idx = None;
        in_frontmatter = false;
        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed == "---" {
                in_frontmatter = !in_frontmatter;
                continue;
            }
            if in_frontmatter {
                continue;
            }
            if trimmed.starts_with("# ") {
                title_idx = Some(idx);
                break;
            }
        }
        if let Some(t_idx) = title_idx {
            if t_idx + 1 < lines.len() {
                header_lines.extend_from_slice(&lines[(t_idx + 1)..]);
            }
        }
    }

    fn clean_prose(lines: Vec<&str>) -> String {
        let mut start = 0;
        while start < lines.len() && lines[start].trim().is_empty() {
            start += 1;
        }
        let mut end = lines.len();
        while end > start && lines[end - 1].trim().is_empty() {
            end -= 1;
        }
        if start < end {
            lines[start..end].join("\n")
        } else {
            String::new()
        }
    }

    (clean_prose(header_lines), clean_prose(footer_lines))
}
