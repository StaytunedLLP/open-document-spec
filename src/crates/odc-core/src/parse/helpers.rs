use super::*;

fn parse_string_list(
    lines: &[&str],
    start: usize,
    min_indent: usize,
    inline: &str,
) -> (Vec<String>, usize) {
    if !inline.is_empty() {
        return (parse_inline_list(inline), start);
    }

    let mut index = start;
    let mut values = Vec::new();

    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }

        if indent(raw_line) < min_indent {
            break;
        }

        let trimmed = raw_line.trim_start();
        let Some(item) = trimmed.strip_prefix("- ") else {
            break;
        };

        values.push(unquote(item.trim()));
        index += 1;
    }

    (values, index)
}

fn parse_resources(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> Result<(Vec<ResourceRef>, usize), String> {
    let mut index = start;
    let mut resources = Vec::new();

    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }

        if indent(raw_line) < min_indent {
            break;
        }

        let trimmed = raw_line.trim_start();
        if let Some(item) = trimmed.strip_prefix("- ") {
            let item_trimmed = item.trim();
            if item_trimmed.contains(':') {
                let mut path = None;
                parse_resource_kv(item_trimmed, &mut path)?;
                index += 1;
                while let Some(inner) = lines.get(index) {
                    if inner.trim().is_empty() {
                        index += 1;
                        continue;
                    }
                    if indent(inner) < min_indent + 2 {
                        break;
                    }
                    parse_resource_kv(inner.trim(), &mut path)?;
                    index += 1;
                }

                if let Some(path) = path {
                    resources.push(ResourceRef { path });
                }
                continue;
            }

            resources.push(ResourceRef {
                path: PathBuf::from(unquote(item_trimmed)),
            });
            index += 1;
        } else {
            break;
        }
    }

    Ok((resources, index))
}

fn parse_code_refs(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> Result<(Vec<CodeRef>, usize), String> {
    let mut index = start;
    let mut code_refs = Vec::new();

    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }

        if indent(raw_line) < min_indent {
            break;
        }

        let trimmed = raw_line.trim_start();
        let Some(item) = trimmed.strip_prefix("- ") else {
            break;
        };

        let mut path = None;
        let mut symbol = None;
        let mut role = None;

        parse_code_kv(item.trim(), &mut path, &mut symbol, &mut role)?;
        index += 1;

        while let Some(inner) = lines.get(index) {
            if inner.trim().is_empty() {
                index += 1;
                continue;
            }

            if indent(inner) < min_indent + 2 {
                break;
            }

            parse_code_kv(inner.trim(), &mut path, &mut symbol, &mut role)?;
            index += 1;
        }

        let Some(path) = path else {
            return Err("code entry missing path".to_string());
        };
        let Some(role) = role else {
            return Err("code entry missing role".to_string());
        };

        code_refs.push(CodeRef { path, symbol, role });
    }

    Ok((code_refs, index))
}

pub(super) fn parse_context(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> Result<(ContextSpec, usize), String> {
    let mut index = start;
    let mut load = Vec::new();
    let mut ignore = Vec::new();
    let mut max_depth = None;

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
            index += 1;
            continue;
        };

        let key = key.trim();
        let rest = rest.trim();

        match key {
            "load" => {
                let (items, next) = parse_string_list(lines, index + 1, min_indent + 2, rest);
                load.extend(items.into_iter().map(|s| {
                    let normalized = s.replace('\\', "/");
                    if normalized.contains('.') {
                        normalized
                    } else {
                        normalized.to_lowercase()
                    }
                }));
                index = next;
                continue;
            }
            "ignore" => {
                let (items, next) = parse_string_list(lines, index + 1, min_indent + 2, rest);
                ignore.extend(items.into_iter().map(|s| s.replace('\\', "/")));
                index = next;
                continue;
            }
            "max-depth" => {
                max_depth = rest.parse::<usize>().ok();
            }
            _ => {}
        }

        index += 1;
    }

    Ok((
        ContextSpec {
            load,
            ignore,
            max_depth,
        },
        index,
    ))
}
