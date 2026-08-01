/// Rewrite tag list items in frontmatter text. Returns None if no frontmatter.
pub fn rewrite_tags_in_text(text: &str, from: &str, to: &str) -> Option<String> {
    let (fm_raw, body) = split_frontmatter(text);
    let fm_raw = fm_raw?;
    let from_n = normalize_tag(from)?;
    let to_n = normalize_tag(to)?;

    let mut lines: Vec<String> = fm_raw.lines().map(|l| l.to_string()).collect();
    let mut in_tags = false;
    let mut tags_indent = 0usize;
    let mut changed = false;

    for line in &mut lines {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();

        if !in_tags {
            if let Some(rest) = trimmed.strip_prefix("tags:") {
                let rest = rest.trim();
                // Inline form: tags: [a, b] or tags: a
                if !rest.is_empty() && !rest.starts_with('#') {
                    if let Some(new_inline) = rewrite_inline_tags(rest, &from_n, &to_n) {
                        *line = format!("{}tags: {}", " ".repeat(indent), new_inline);
                        changed = true;
                    }
                    in_tags = false;
                } else {
                    in_tags = true;
                    tags_indent = indent;
                }
            }
            continue;
        }

        // Still in tags block?
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Nested list item under tags
        if trimmed.starts_with('-') && indent > tags_indent {
            if let Some(value) = list_item_scalar(trimmed)
                && normalize_tag(&value).as_deref() == Some(from_n.as_str())
            {
                // Preserve quote style loosely
                let new_item = format_list_tag_item(trimmed, &to_n);
                *line = format!("{}{}", " ".repeat(indent), new_item);
                changed = true;
            }
            continue;
        }
        // Next top-level key ends tags
        if indent <= tags_indent && trimmed.contains(':') && !trimmed.starts_with('-') {
            in_tags = false;
            // re-process this line as potential new key? skip for simplicity
        }
    }

    if !changed {
        return Some(text.to_string());
    }

    let new_fm = lines.join("\n");
    // Preserve trailing newline style of original frontmatter block
    let composed = format!("---\n{new_fm}\n---\n{body}");
    Some(composed)
}

fn list_item_scalar(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix('-')?.trim();
    if rest.is_empty() {
        return None;
    }
    Some(unquote_simple(rest))
}

fn format_list_tag_item(original_trimmed: &str, new_tag: &str) -> String {
    let rest = original_trimmed
        .strip_prefix('-')
        .map(str::trim)
        .unwrap_or(original_trimmed);
    let quoted = rest.starts_with('"') || rest.starts_with('\'');
    if quoted {
        format!("- \"{new_tag}\"")
    } else {
        format!("- {new_tag}")
    }
}

fn rewrite_inline_tags(rest: &str, from: &str, to: &str) -> Option<String> {
    let rest = rest.trim();
    if rest.starts_with('[') && rest.ends_with(']') {
        let inner = &rest[1..rest.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(str::trim).collect();
        let mut any = false;
        let mut out = Vec::new();
        for p in parts {
            if p.is_empty() {
                continue;
            }
            let bare = unquote_simple(p);
            if normalize_tag(&bare).as_deref() == Some(from) {
                out.push(to.to_string());
                any = true;
            } else {
                out.push(bare);
            }
        }
        // dedupe after rename
        let out = normalize_tag_list(out);
        if any {
            return Some(format!("[{}]", out.join(", ")));
        }
        return None;
    }
    // single scalar
    if normalize_tag(rest).as_deref() == Some(from) {
        return Some(to.to_string());
    }
    None
}

fn unquote_simple(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// Whether a tag is a default ODS suggestion.
pub fn is_builtin_tag(tag: &str) -> bool {
    normalize_tag(tag)
        .map(|n| builtin_tags().contains(&n.as_str()))
        .unwrap_or(false)
}
