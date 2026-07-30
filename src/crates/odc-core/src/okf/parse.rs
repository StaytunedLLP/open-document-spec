use super::model::{
    ActorEvent, DateRange, OkfFrontmatter, OkfFrontmatterState, OkfParameter, OkfSource, OkfStatus,
    ResourceRefFields,
};
use crate::parse::split_frontmatter;
use std::collections::BTreeMap;

const KNOWN: &[&str] = &[
    "type",
    "title",
    "description",
    "resource",
    "tags",
    "sources",
    "usage_window",
    "generated",
    "verified",
    "status",
    "stale_after",
    "runtime",
    "parameters",
    "computation",
    "executor",
    "attester",
    "timestamp",
    "okf_version",
];

pub fn parse_okf_document_text(text: &str) -> OkfFrontmatterState {
    let (fm, _) = split_frontmatter(text);
    match fm {
        None => OkfFrontmatterState::Absent,
        Some(block) => match parse_okf_frontmatter_block(block) {
            Ok(parsed) => OkfFrontmatterState::Parsed(parsed),
            Err(e) => OkfFrontmatterState::Invalid(e),
        },
    }
}

pub fn parse_okf_frontmatter_block(block: &str) -> Result<OkfFrontmatter, String> {
    let lines: Vec<&str> = block.lines().collect();
    let mut fm = OkfFrontmatter::default();
    let mut i = 0;
    while i < lines.len() {
        let raw = lines[i];
        let line = raw.trim_end_matches('\r');
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            i += 1;
            continue;
        }
        // nested list/map continuation is handled by specialists
        if line.starts_with(' ') || line.starts_with('\t') {
            i += 1;
            continue;
        }
        let Some((key, rest)) = split_key(line) else {
            i += 1;
            continue;
        };
        let key = key.trim();
        let rest = rest.trim();
        match key {
            "type" => fm.type_name = Some(unquote(rest)),
            "title" => fm.title = Some(unquote(rest)),
            "description" => fm.description = Some(unquote(rest)),
            "resource" => fm.resource = Some(unquote(rest)),
            "stale_after" => fm.stale_after = Some(unquote(rest)),
            "runtime" => fm.runtime = Some(unquote(rest)),
            "computation" => fm.computation = Some(unquote(rest)),
            "timestamp" => fm.timestamp = Some(unquote(rest)),
            "okf_version" => {
                // root-only key; store as unknown if on concept, still ok
                fm.unknown.insert("okf_version".into(), unquote(rest));
            }
            "status" => {
                let v = unquote(rest);
                fm.status = OkfStatus::parse(&v);
                if fm.status.is_none() && !v.is_empty() {
                    fm.unknown.insert("status".into(), v);
                }
            }
            "tags" => {
                let (tags, next) = parse_string_list(rest, &lines, i + 1);
                fm.tags = tags;
                i = next.max(i) + 1;
                continue;
            }
            "generated" => {
                fm.generated = parse_actor_inline_or_block(rest, &lines, &mut i)?;
            }
            "verified" => {
                fm.verified = parse_verified(rest, &lines, &mut i)?;
            }
            "usage_window" => {
                fm.usage_window = parse_date_range_inline_or_block(rest, &lines, &mut i);
            }
            "sources" => {
                let (sources, next) = parse_sources(&lines, i + 1, rest)?;
                fm.sources = sources;
                i = next.max(i) + 1;
                continue;
            }
            "parameters" => {
                let (params, next) = parse_parameters(&lines, i + 1, rest)?;
                fm.parameters = params;
                i = next.max(i) + 1;
                continue;
            }
            "executor" => {
                let (fields, next) = parse_resource_ref_fields(&lines, i + 1, rest)?;
                fm.executor = fields;
                i = next.max(i) + 1;
                continue;
            }
            "attester" => {
                let (fields, next) = parse_resource_ref_fields(&lines, i + 1, rest)?;
                fm.attester = fields;
                i = next.max(i) + 1;
                continue;
            }
            other => {
                if !KNOWN.contains(&other) && !rest.is_empty() {
                    fm.unknown.insert(other.to_string(), unquote(rest));
                }
            }
        }
        i += 1;
    }
    Ok(fm)
}

fn split_key(line: &str) -> Option<(&str, &str)> {
    let line = line.trim_end();
    let idx = line.find(':')?;
    Some((&line[..idx], &line[idx + 1..]))
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn indent_of(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ' || *c == '\t').count()
}

fn parse_string_list(rest: &str, lines: &[&str], start: usize) -> (Vec<String>, usize) {
    let rest = rest.trim();
    if rest.starts_with('[') && rest.ends_with(']') {
        let inner = &rest[1..rest.len() - 1];
        let tags = inner
            .split(',')
            .map(|s| unquote(s.trim()))
            .filter(|s| !s.is_empty())
            .collect();
        return (tags, start - 1);
    }
    let mut tags = Vec::new();
    let mut i = start;
    while i < lines.len() {
        let line = lines[i].trim_end_matches('\r');
        if indent_of(line) == 0 && !line.trim().is_empty() {
            break;
        }
        let t = line.trim();
        if let Some(item) = t.strip_prefix("- ") {
            tags.push(unquote(item));
        }
        i += 1;
    }
    (tags, i - 1)
}

fn parse_flow_map(s: &str) -> BTreeMap<String, String> {
    let s = s.trim();
    let s = s
        .strip_prefix('{')
        .and_then(|x| x.strip_suffix('}'))
        .unwrap_or(s);
    let mut map = BTreeMap::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((k, v)) = part.split_once(':') {
            map.insert(k.trim().to_string(), unquote(v.trim()));
        }
    }
    map
}

fn actor_from_map(map: &BTreeMap<String, String>) -> Result<ActorEvent, String> {
    let by = map
        .get("by")
        .cloned()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "generated/verified missing by".to_string())?;
    Ok(ActorEvent {
        by,
        at: map.get("at").cloned().filter(|s| !s.is_empty()),
    })
}

fn parse_actor_inline_or_block(
    rest: &str,
    lines: &[&str],
    i: &mut usize,
) -> Result<Option<ActorEvent>, String> {
    let rest = rest.trim();
    if rest.starts_with('{') {
        let map = parse_flow_map(rest);
        return Ok(Some(actor_from_map(&map)?));
    }
    if rest.is_empty() {
        // block form
        let mut map = BTreeMap::new();
        let base = *i + 1;
        let mut j = base;
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
        *i = j - 1;
        if map.is_empty() {
            return Ok(None);
        }
        return Ok(Some(actor_from_map(&map)?));
    }
    Ok(None)
}

fn parse_verified(rest: &str, lines: &[&str], i: &mut usize) -> Result<Vec<ActorEvent>, String> {
    let rest = rest.trim();
    // bare mapping: verified: { by: x, at: y }
    if rest.starts_with('{') {
        return Ok(vec![actor_from_map(&parse_flow_map(rest))?]);
    }
    let mut out = Vec::new();
    let mut j = *i + 1;
    while j < lines.len() {
        let line = lines[j].trim_end_matches('\r');
        if indent_of(line) == 0 && !line.trim().is_empty() {
            break;
        }
        let t = line.trim();
        if let Some(item) = t.strip_prefix("- ") {
            let item = item.trim();
            if item.starts_with('{') {
                out.push(actor_from_map(&parse_flow_map(item))?);
            }
        }
        j += 1;
    }
    *i = j.saturating_sub(1);
    Ok(out)
}

fn parse_date_range_inline_or_block(
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

fn parse_sources(
    lines: &[&str],
    start: usize,
    rest: &str,
) -> Result<(Vec<OkfSource>, usize), String> {
    if !rest.is_empty() && rest != "[]" {
        // rare inline — ignore empty
    }
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

fn parse_parameters(
    lines: &[&str],
    start: usize,
    rest: &str,
) -> Result<(Vec<OkfParameter>, usize), String> {
    let mut params = Vec::new();
    if rest.starts_with('[') {
        // skip complex inline
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

fn parse_resource_ref_fields(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_type() {
        let fm = parse_okf_frontmatter_block("type: Metric\n").unwrap();
        assert_eq!(fm.type_name.as_deref(), Some("Metric"));
    }

    #[test]
    fn parse_generated_and_bare_verified() {
        let block = r#"
type: Metric
generated: { by: agent/v1, at: 2026-06-20T22:53:05Z }
verified: { by: human:alice, at: 2026-06-25T09:00:00Z }
stale_after: 2026-12-31
"#;
        let fm = parse_okf_frontmatter_block(block).unwrap();
        assert_eq!(fm.generated.as_ref().unwrap().by, "agent/v1");
        assert_eq!(fm.verified.len(), 1);
        assert!(fm.verified[0].by.starts_with("human:"));
        assert_eq!(fm.stale_after.as_deref(), Some("2026-12-31"));
    }

    #[test]
    fn parse_attested_computation() {
        let block = r#"
type: Attested Computation
runtime: bigquery
parameters:
  - { name: year, type: integer, required: true }
executor:
  resource: references/skills/run-on-bq.md
  receipt: [job_id, executed_sql, result]
attester:
  resource: references/attesters/sql-equality.py
"#;
        let fm = parse_okf_frontmatter_block(block).unwrap();
        assert_eq!(fm.type_name.as_deref(), Some("Attested Computation"));
        assert_eq!(fm.runtime.as_deref(), Some("bigquery"));
        assert_eq!(fm.parameters.len(), 1);
        assert_eq!(fm.parameters[0].name, "year");
        assert_eq!(
            fm.executor.resource.as_deref(),
            Some("references/skills/run-on-bq.md")
        );
        assert!(fm.executor.receipt.contains(&"job_id".into()));
    }

    #[test]
    fn parse_sources() {
        let block = r#"
type: BigQuery Table
sources:
  - id: rev-policy
    resource: https://example.com/policy
    title: Policy
    author: team:finance
    last_modified: 2026-04-02
"#;
        let fm = parse_okf_frontmatter_block(block).unwrap();
        assert_eq!(fm.sources.len(), 1);
        assert_eq!(fm.sources[0].id.as_deref(), Some("rev-policy"));
        assert!(fm.sources[0].resource.as_ref().unwrap().contains("example"));
    }

    #[test]
    fn preserves_unknown_keys() {
        let fm = parse_okf_frontmatter_block("type: X\nfoo: bar\n").unwrap();
        assert_eq!(fm.unknown.get("foo").map(String::as_str), Some("bar"));
    }
}
