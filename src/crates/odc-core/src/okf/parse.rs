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



include!("parse_sources.rs");

#[cfg(test)]
include!("parse_tests.rs");
