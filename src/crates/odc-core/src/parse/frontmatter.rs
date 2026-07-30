use crate::model::{
    CodeRef, CodeRole, ContextSpec, Document, Frontmatter, FrontmatterState, ResourceRef,
};
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn parse_document(root: &Path, path: PathBuf) -> io::Result<Document> {
    let text = fs::read_to_string(&path)?;
    Ok(parse_document_text(root, path, &text, true))
}

pub fn parse_document_text(root: &Path, path: PathBuf, text: &str, include_body: bool) -> Document {
    let (frontmatter, body) = split_frontmatter(text);
    let headings = extract_headings(body);
    let directory = path.parent().unwrap_or(root).to_path_buf();

    Document {
        path,
        directory,
        body: if include_body {
            body.to_string()
        } else {
            String::new()
        },
        headings,
        frontmatter: match frontmatter {
            Some(block) => match parse_frontmatter(block) {
                Ok(parsed) => FrontmatterState::Parsed(parsed),
                Err(err) => FrontmatterState::Invalid(err),
            },
            None => FrontmatterState::Absent,
        },
    }
}

pub fn split_frontmatter(text: &str) -> (Option<&str>, &str) {
    if !text.starts_with("---") {
        return (None, text);
    }

    let mut lines = text.split('\n');
    let first = lines.next().unwrap();
    if first.trim_end_matches('\r') != "---" {
        return (None, text);
    }

    let mut current_offset = first.len() + 1;
    let mut found_end_offset = None;

    for line in lines {
        let line_len_with_nl = line.len() + 1;
        if line.trim_end_matches('\r') == "---" {
            found_end_offset = Some(current_offset);
            break;
        }
        current_offset += line_len_with_nl;
    }

    match found_end_offset {
        Some(end_offset) => {
            let frontmatter = &text[first.len() + 1..end_offset];
            // frontmatter block trim trailing \n or \r\n
            let frontmatter = frontmatter.trim_end_matches('\r').trim_end_matches('\n');

            // body starts after the "---" line we just found
            let body_start = end_offset + 3; // "---" has length 3
            let body = if body_start < text.len() {
                let mut b = &text[body_start..];
                if b.starts_with('\r') {
                    b = &b[1..];
                }
                if b.starts_with('\n') {
                    b = &b[1..];
                }
                b
            } else {
                ""
            };
            (Some(frontmatter), body)
        }
        None => (Some(&text[first.len() + 1..]), ""),
    }
}

pub fn extract_headings(body: &str) -> Vec<String> {
    extract_heading_groups(body)
        .into_iter()
        .filter_map(|group| group.into_iter().next())
        .collect()
}

pub fn extract_heading_groups(body: &str) -> Vec<Vec<String>> {
    body.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if let Some(h) = trimmed.strip_prefix("## ") {
                Some(h.trim())
            } else if let Some(h) = trimmed.strip_prefix("### ") {
                Some(h.trim())
            } else {
                None
            }
        })
        .filter(|heading| !heading.is_empty())
        .map(parse_heading_group)
        .collect()
}

pub fn split_markdown_link_target(text: &str) -> Option<String> {
    let start = text.find("](")? + 2;
    let rest = text.get(start..)?;
    let end = rest.find(')')?;
    Some(rest[..end].trim().to_string())
}

pub fn document_id(root: &Path, path: &Path, frontmatter: Option<&Frontmatter>) -> String {
    if let Some(id) = frontmatter.and_then(|fm| fm.id.as_ref()) {
        return id.replace("\\", "/").to_lowercase();
    }

    let relative = path.strip_prefix(root).unwrap_or(path);
    let without_ext = relative.with_extension("");
    without_ext
        .iter()
        .map(|component| component.to_string_lossy().to_string().to_lowercase())
        .collect::<Vec<_>>()
        .join("/")
}

fn parse_frontmatter(block: &str) -> Result<Frontmatter, String> {
    let lines = block.lines().collect::<Vec<_>>();
    let mut index = 0usize;
    let mut frontmatter = Frontmatter::default();

    while let Some(raw_line) = lines.get(index) {
        let line = raw_line.trim();
        index += 1;

        if line.is_empty() {
            continue;
        }

        let Some((key, rest)) = line.split_once(':') else {
            return Err(format!("invalid frontmatter line: {line}"));
        };

        let key = key.trim();
        let rest = rest.trim();

        if key == "title" {
            return Err("frontmatter MUST NOT contain a title field (title is derived from first H1 header)".to_string());
        }

        match key {
            "profile" => frontmatter.profile = scalar_value(rest).map(|s| s.to_lowercase()),
            "status" => frontmatter.status = scalar_value(rest).map(|s| s.to_lowercase()),
            "share" => frontmatter.share = scalar_value(rest).map(|s| s.to_lowercase()),
            "description" => frontmatter.description = scalar_value(rest),
            "id" => {
                frontmatter.id = scalar_value(rest).map(|s| s.replace('\\', "/").to_lowercase())
            }
            "owner" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                if !items.is_empty() {
                    frontmatter.owner = Some(items.join(", "));
                    index = next;
                } else {
                    frontmatter.owner = scalar_value(rest);
                }
            }
            "ods" => {
                if !rest.is_empty() {
                    frontmatter.ods = scalar_value(rest).map(|s| s.to_lowercase());
                } else {
                    // Parse nested ods: map block
                    let (nested_fm, next) = parse_nested_ods_map(&lines, index, 2)?;
                    if nested_fm.profile.is_some() {
                        frontmatter.profile = nested_fm.profile;
                    }
                    if nested_fm.status.is_some() {
                        frontmatter.status = nested_fm.status;
                    }
                    if nested_fm.share.is_some() {
                        frontmatter.share = nested_fm.share;
                    }
                    if nested_fm.id.is_some() {
                        frontmatter.id = nested_fm.id;
                    }
                    if !nested_fm.depends.is_empty() {
                        frontmatter.depends.extend(nested_fm.depends);
                    }
                    if !nested_fm.related.is_empty() {
                        frontmatter.related.extend(nested_fm.related);
                    }
                    if !nested_fm.resources.is_empty() {
                        frontmatter.resources.extend(nested_fm.resources);
                    }
                    if !nested_fm.code.is_empty() {
                        frontmatter.code.extend(nested_fm.code);
                    }
                    if nested_fm.context.is_some() {
                        frontmatter.context = nested_fm.context;
                    }
                    index = next;
                }
            }
            "odc" => frontmatter.odc = scalar_value(rest),
            "profiles" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.profiles.extend(items);
                index = next;
            }
            "packs" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.packs.extend(items);
                index = next;
            }
            "ignore" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.ignore.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace('\\', "/").trim_end_matches('/').to_string())
                        .filter(|s| !s.is_empty()),
                );
                index = next;
            }
            "depends" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.depends.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace("\\", "/").to_lowercase()),
                );
                index = next;
            }
            "related" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                frontmatter.related.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace("\\", "/").to_lowercase()),
                );
                index = next;
            }
            "tags" => {
                let (items, next) = parse_string_list(&lines, index, 2, rest);
                // Normalize each entry; keep duplicates so lint can warn.
                for item in items {
                    if let Some(n) = crate::tags::normalize_tag(&item) {
                        frontmatter.tags.push(n);
                    }
                }
                index = next;
            }
            "resources" => {
                let (items, next) = parse_resources(&lines, index, 2)?;
                frontmatter.resources.extend(items);
                index = next;
            }
            "code" => {
                let (items, next) = parse_code_refs(&lines, index, 2)?;
                frontmatter.code.extend(items);
                index = next;
            }
            "context" => {
                let (context, next) = parse_context(&lines, index, 2)?;
                frontmatter.context = Some(context);
                index = next;
            }
            "aliases" => {
                let (aliases, next) = parse_aliases(&lines, index, 2);
                frontmatter.aliases.extend(aliases);
                index = next;
            }
            _ => {
                let (_, next) = parse_passthrough_block(&lines, index, 2);
                index = next;
            }
        }
    }

    Ok(frontmatter)
}

fn parse_nested_ods_map(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> Result<(Frontmatter, usize), String> {
    let mut index = start;
    let mut frontmatter = Frontmatter::default();

    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }

        if indent(raw_line) < min_indent {
            break;
        }

        let trimmed = raw_line.trim_start();
        let Some((key, rest)) = trimmed.split_once(':') else {
            break;
        };

        let key = key.trim();
        let rest = rest.trim();
        let item_indent = min_indent + 2;

        match key {
            "profile" => {
                frontmatter.profile = scalar_value(rest).map(|s| s.to_lowercase());
                index += 1;
            }
            "status" => {
                frontmatter.status = scalar_value(rest).map(|s| s.to_lowercase());
                index += 1;
            }
            "share" => {
                frontmatter.share = scalar_value(rest).map(|s| s.to_lowercase());
                index += 1;
            }
            "id" => {
                frontmatter.id = scalar_value(rest).map(|s| s.replace('\\', "/").to_lowercase());
                index += 1;
            }
            "depends" => {
                let (items, next) = parse_string_list(lines, index + 1, item_indent, rest);
                frontmatter.depends.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace('\\', "/").to_lowercase()),
                );
                index = next;
            }
            "related" => {
                let (items, next) = parse_string_list(lines, index + 1, item_indent, rest);
                frontmatter.related.extend(
                    items
                        .into_iter()
                        .map(|s| s.replace('\\', "/").to_lowercase()),
                );
                index = next;
            }
            "resources" => {
                let (items, next) = parse_resources(lines, index + 1, item_indent)?;
                frontmatter.resources.extend(items);
                index = next;
            }
            "code" => {
                let (items, next) = parse_code_refs(lines, index + 1, item_indent)?;
                frontmatter.code.extend(items);
                index = next;
            }
            "context" => {
                let (context, next) = parse_context(lines, index + 1, item_indent)?;
                frontmatter.context = Some(context);
                index = next;
            }
            _ => {
                let (_, next) = parse_passthrough_block(lines, index + 1, item_indent);
                index = next;
            }
        }
    }

    Ok((frontmatter, index))
}

fn scalar_value(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    Some(unquote(text))
}

fn parse_heading_group(heading: &str) -> Vec<String> {
    heading
        .split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(unquote)
        .collect()
}
