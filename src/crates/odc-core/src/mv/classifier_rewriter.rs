use super::*;

pub fn rewrite_path_prefix_in_text(text: &str, old_prefix: &str, new_prefix: &str) -> String {
    if old_prefix.is_empty() || old_prefix == new_prefix {
        return text.to_string();
    }
    let mut out = text.to_string();
    out = out.replace(&format!("]({old_prefix}/"), &format!("]({new_prefix}/"));
    out = out.replace(&format!("]({old_prefix})"), &format!("]({new_prefix})"));
    out = out.replace(
        &format!("]({old_prefix}.md)"),
        &format!("]({new_prefix}.md)"),
    );
    let mut lines = Vec::new();
    for line in out.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            let val = rest.trim();
            if val == old_prefix || val.starts_with(&format!("{old_prefix}/")) {
                let new_val = if val == old_prefix {
                    new_prefix.to_string()
                } else {
                    format!("{new_prefix}{}", &val[old_prefix.len()..])
                };
                let indent = &line[..line.len() - trimmed.len()];
                lines.push(format!("{indent}- {new_val}"));
                continue;
            }
        }
        if let Some(rest) = trimmed.strip_prefix("id:") {
            let val = rest.trim();
            if val == old_prefix || val.starts_with(&format!("{old_prefix}/")) {
                let new_val = if val == old_prefix {
                    new_prefix.to_string()
                } else {
                    format!("{new_prefix}{}", &val[old_prefix.len()..])
                };
                let indent = &line[..line.len() - trimmed.len()];
                lines.push(format!("{indent}id: {new_val}"));
                continue;
            }
        }
        lines.push(line.to_string());
    }
    let mut joined = lines.join("\n");
    if text.ends_with('\n') && !joined.ends_with('\n') {
        joined.push('\n');
    }
    joined
}

pub fn rewrite_relative_links_in_text(
    text: &str,
    doc_dir: &Path,
    root: &Path,
    path_pairs: &[(String, String)],
) -> String {
    if path_pairs.is_empty() {
        return text.to_string();
    }

    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(idx) = rest.find("](") {
        out.push_str(&rest[..idx]);
        out.push_str("](");
        rest = &rest[idx + 2..];
        let end = rest.find([')', '\n']).unwrap_or(rest.len());
        let target = &rest[..end];
        out.push_str(&rewrite_one_link_target(target, doc_dir, root, path_pairs));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}


