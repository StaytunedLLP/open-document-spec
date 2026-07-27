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
        let Some(rest) = trimmed.strip_prefix("- ") else {
            break;
        };

        let mut path = None::<PathBuf>;

        if !rest.trim().is_empty() {
            parse_resource_kv(rest.trim(), &mut path)?;
        }

        index += 1;

        while let Some(inner) = lines.get(index) {
            if inner.trim().is_empty() {
                index += 1;
                continue;
            }

            let inner_indent = indent(inner);
            if inner_indent < min_indent + 2 {
                break;
            }

            let inner_trimmed = inner.trim_start();
            if inner_indent == min_indent && inner_trimmed.starts_with("- ") {
                break;
            }

            parse_resource_kv(inner_trimmed, &mut path)?;
            index += 1;
        }

        let Some(path) = path else {
            return Err("resource entry missing path".to_string());
        };

        resources.push(ResourceRef { path });
    }

    Ok((resources, index))
}

fn parse_code_refs(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> Result<(Vec<CodeRef>, usize), String> {
    let mut index = start;
    let mut refs = Vec::new();

    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }

        if indent(raw_line) < min_indent {
            break;
        }

        let trimmed = raw_line.trim_start();
        let Some(rest) = trimmed.strip_prefix("- ") else {
            break;
        };

        let mut path = None::<PathBuf>;
        let mut symbol = None::<String>;
        let mut role = None::<CodeRole>;

        if !rest.trim().is_empty() {
            parse_code_kv(rest.trim(), &mut path, &mut symbol, &mut role)?;
        }

        index += 1;

        while let Some(inner) = lines.get(index) {
            if inner.trim().is_empty() {
                index += 1;
                continue;
            }

            let inner_indent = indent(inner);
            if inner_indent < min_indent + 2 {
                break;
            }

            let inner_trimmed = inner.trim_start();
            if inner_indent == min_indent && inner_trimmed.starts_with("- ") {
                break;
            }

            if let Some(rest_sym) = inner_trimmed.strip_prefix("symbol:") {
                let rest_sym = rest_sym.trim();
                if rest_sym.is_empty() {
                    let (items, next) = parse_string_list(lines, index + 1, inner_indent + 2, "");
                    if !items.is_empty() {
                        symbol = Some(items.join(", "));
                    }
                    index = next;
                    continue;
                }
            }

            parse_code_kv(inner_trimmed, &mut path, &mut symbol, &mut role)?;
            index += 1;
        }

        let Some(path) = path else {
            return Err("code entry missing path".to_string());
        };
        let Some(role) = role else {
            return Err("code entry missing role".to_string());
        };

        refs.push(CodeRef { path, symbol, role });
    }

    Ok((refs, index))
}

fn parse_context(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> Result<(ContextSpec, usize), String> {
    let mut index = start;
    let mut load = Vec::new();
    let mut ignore = Vec::new();
    let mut max_depth = None::<usize>;

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

fn parse_aliases(
    lines: &[&str],
    start: usize,
    min_indent: usize,
) -> (BTreeMap<String, Vec<String>>, usize) {
    let mut index = start;
    let mut aliases = BTreeMap::new();

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
        let mut values = Vec::new();
        index += 1;

        if !rest.trim().is_empty() {
            values.extend(parse_inline_list(rest));
        } else {
            while let Some(inner) = lines.get(index) {
                if inner.trim().is_empty() {
                    index += 1;
                    continue;
                }

                if indent(inner) < min_indent + 2 {
                    break;
                }

                let inner_trimmed = inner.trim_start();
                if let Some(item) = inner_trimmed.strip_prefix("- ") {
                    values.push(unquote(item.trim()));
                    index += 1;
                } else {
                    break;
                }
            }
        }

        if !key.is_empty() {
            aliases.insert(key.to_string(), values);
        }
    }

    (aliases, index)
}

fn parse_passthrough_block(lines: &[&str], start: usize, min_indent: usize) -> ((), usize) {
    let mut index = start;
    while let Some(raw_line) = lines.get(index) {
        if raw_line.trim().is_empty() {
            index += 1;
            continue;
        }
        if indent(raw_line) < min_indent {
            break;
        }
        index += 1;
    }
    ((), index)
}

fn parse_resource_kv(text: &str, path: &mut Option<PathBuf>) -> Result<(), String> {
    let Some((key, rest)) = text.split_once(':') else {
        return Err(format!("invalid resource entry: {text}"));
    };

    if key.trim() == "path" {
        *path = Some(PathBuf::from(unquote(rest.trim())));
    }

    Ok(())
}

fn parse_code_kv(
    text: &str,
    path: &mut Option<PathBuf>,
    symbol: &mut Option<String>,
    role: &mut Option<CodeRole>,
) -> Result<(), String> {
    let Some((key, rest)) = text.split_once(':') else {
        return Err(format!("invalid code entry: {text}"));
    };

    match key.trim() {
        "path" => {
            *path = Some(PathBuf::from(unquote(rest.trim()).replace('\\', "/")));
        }
        "symbol" => {
            let trimmed = rest.trim();
            if trimmed.starts_with('[') {
                let items = parse_inline_list(trimmed);
                if !items.is_empty() {
                    *symbol = Some(items.join(", "));
                }
            } else {
                let value = unquote(trimmed);
                if !value.is_empty() {
                    *symbol = Some(value);
                }
            }
        }
        "role" => {
            let value = unquote(rest.trim()).to_lowercase();
            let Some(parsed) = CodeRole::parse(&value) else {
                return Err(format!("invalid code role: {value}"));
            };
            *role = Some(parsed);
        }
        _ => {}
    }

    Ok(())
}
fn parse_inline_list(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    let inner = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']'));
    inner
        .map(|items| {
            items
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(unquote)
                .collect()
        })
        .unwrap_or_else(|| vec![unquote(trimmed)])
}

fn indent(text: &str) -> usize {
    text.chars().take_while(|ch| *ch == ' ').count()
}

fn unquote(text: &str) -> String {
    let trimmed = text.trim();
    let maybe_stripped = trimmed.strip_prefix('"').and_then(|s| s.strip_suffix('"'));
    maybe_stripped
        .or_else(|| {
            trimmed
                .strip_prefix('\'')
                .and_then(|s| s.strip_suffix('\''))
        })
        .unwrap_or(trimmed)
        .to_string()
}

#[cfg(test)]
mod parse_v02_tests {
    use super::*;

    #[test]
    fn test_parse_share_and_packs() {
        let text = r#"---
profile: guide
status: stable
share: private
packs:
  - vendor/engineering-pack
  - ../shared-pack
---
# Test Doc
"#;
        let doc = parse_document_text(Path::new("/workspace"), PathBuf::from("/workspace/doc.md"), text, true);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            assert_eq!(fm.share.as_deref(), Some("private"));
            assert_eq!(fm.packs, vec!["vendor/engineering-pack", "../shared-pack"]);
        } else {
            panic!("expected parsed frontmatter");
        }
    }

    #[test]
    fn test_parse_share_org() {
        let text = r#"---
profile: decision
status: stable
share: org
---
# Internal Decision
"#;
        let doc = parse_document_text(Path::new("/workspace"), PathBuf::from("/workspace/doc.md"), text, true);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            assert_eq!(fm.share.as_deref(), Some("org"));
        } else {
            panic!("expected parsed frontmatter");
        }
    }
}
