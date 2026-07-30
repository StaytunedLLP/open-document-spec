use super::*;

pub fn parse_nested_ods_map(
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

pub fn scalar_value(text: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }

    Some(unquote(text))
}

pub fn parse_heading_group(heading: &str) -> Vec<String> {
    heading
        .split('|')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(unquote)
        .collect()
}
