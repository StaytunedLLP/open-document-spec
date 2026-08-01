

pub(super) fn parse_date_range_inline_or_block(
    rest: &str,
    lines: &[&str],
    i: &mut usize,
) -> Option<DateRange> {
    let rest = rest.trim();
    if rest.starts_with('{') {
        let map = parse_flow_map(rest);
        return Some(DateRange {
            from: map.get("from").cloned(),
            to: map.get("to").cloned(),
        });
    }
    let mut map = BTreeMap::new();
    let mut j = *i + 1;
    while j < lines.len() {
        let line = lines[j].trim_end_matches('\r');
        if indent_of(line) == 0 && !line.trim().is_empty() {
            break;
        }
        let t = line.trim();
        if let Some((k, v)) = split_key(t) {
            map.insert(k.trim().to_string(), unquote(v.trim()));
        }
        j += 1;
    }
    *i = j.saturating_sub(1);
    if map.is_empty() {
        None
    } else {
        Some(DateRange {
            from: map.get("from").cloned(),
            to: map.get("to").cloned(),
        })
    }
}

pub(super) fn parse_sources(
    lines: &[&str],
    start: usize,
    _rest: &str,
) -> Result<(Vec<OkfSource>, usize), String> {
    let mut sources = Vec::new();
    let mut i = start;
    let mut current: Option<OkfSource> = None;
    while i < lines.len() {
        let line = lines[i].trim_end_matches('\r');
        if indent_of(line) == 0 && !line.trim().is_empty() {
            break;
        }
        let t = line.trim();
        if let Some(item) = t.strip_prefix("- ") {
            if let Some(src) = current.take() {
                sources.push(src);
            }
            let mut src = OkfSource {
                id: None,
                resource: None,
                title: None,
                author: None,
                usage_count: None,
                last_modified: None,
                usage_window: None,
            };
            let item = item.trim();
            if item.starts_with('{') {
                let map = parse_flow_map(item);
                fill_source_from_map(&mut src, &map);
                sources.push(src);
            } else if let Some((k, v)) = split_key(item) {
                apply_source_field(&mut src, k.trim(), unquote(v.trim()));
                current = Some(src);
            } else {
                current = Some(src);
            }
        } else if let Some(src) = current.as_mut() {
            if let Some((k, v)) = split_key(t) {
                apply_source_field(src, k.trim(), unquote(v.trim()));
            }
        }
        i += 1;
    }
    if let Some(src) = current {
        sources.push(src);
    }
    Ok((sources, i - 1))
}

fn fill_source_from_map(src: &mut OkfSource, map: &BTreeMap<String, String>) {
    for (k, v) in map {
        apply_source_field(src, k, v.clone());
    }
}

fn apply_source_field(src: &mut OkfSource, key: &str, value: String) {
    match key {
        "id" => src.id = Some(value),
        "resource" => src.resource = Some(value),
        "title" => src.title = Some(value),
        "author" => src.author = Some(value),
        "usage_count" => src.usage_count = value.parse().ok(),
        "last_modified" => src.last_modified = Some(value),
        _ => {}
    }
}

pub(super) fn parse_parameters(
    lines: &[&str],
    start: usize,
    rest: &str,
) -> Result<(Vec<OkfParameter>, usize), String> {
    let mut params = Vec::new();
    if rest.starts_with('[') {
        return Ok((params, start - 1));
    }
    let mut i = start;
    while i < lines.len() {
        let line = lines[i].trim_end_matches('\r');
        if indent_of(line) == 0 && !line.trim().is_empty() {
            break;
        }
        let t = line.trim();
        if let Some(item) = t.strip_prefix("- ") {
            let item = item.trim();
            if item.starts_with('{') {
                let map = parse_flow_map(item);
                if let Some(name) = map.get("name").cloned() {
                    params.push(OkfParameter {
                        name,
                        type_name: map.get("type").cloned(),
                        required: map.get("required").and_then(|s| match s.as_str() {
                            "true" => Some(true),
                            "false" => Some(false),
                            _ => None,
                        }),
                    });
                }
            }
        }
        i += 1;
    }
    Ok((params, i - 1))
}

pub(super) fn parse_resource_ref_fields(
    lines: &[&str],
    start: usize,
    rest: &str,
) -> Result<(ResourceRefFields, usize), String> {
    let mut fields = ResourceRefFields::default();
    if rest.starts_with('{') {
        let map = parse_flow_map(rest);
        fields.resource = map.get("resource").cloned();
        return Ok((fields, start - 1));
    }
    let mut i = start;
    while i < lines.len() {
        let line = lines[i].trim_end_matches('\r');
        if indent_of(line) == 0 && !line.trim().is_empty() {
            break;
        }
        let t = line.trim();
        if let Some((k, v)) = split_key(t) {
            match k.trim() {
                "resource" => fields.resource = Some(unquote(v.trim())),
                "receipt" => {
                    let (list, next) = parse_string_list(v.trim(), lines, i + 1);
                    fields.receipt = list;
                    i = next;
                }
                _ => {}
            }
        }
        i += 1;
    }
    Ok((fields, i - 1))
}
