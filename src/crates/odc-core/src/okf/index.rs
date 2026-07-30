use super::model::{OkfBundle, OkfFrontmatterState};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Generate progressive-disclosure `index.md` bodies for directories that contain concepts.
/// Root keeps `okf_version` frontmatter when present.
pub fn generate_okf_indexes(bundle: &OkfBundle) -> io::Result<Vec<PathBuf>> {
    let mut written = Vec::new();
    // Group concepts by parent directory
    use std::collections::BTreeMap;
    let mut by_dir: BTreeMap<PathBuf, Vec<&super::model::OkfDocument>> = BTreeMap::new();
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        let parent = doc
            .path
            .parent()
            .unwrap_or(bundle.root.as_path())
            .to_path_buf();
        by_dir.entry(parent).or_default().push(doc);
    }

    // Also ensure root index exists with version
    let root_index = bundle.root.join("index.md");
    if !root_index.exists() {
        let body = format!(
            "---\nokf_version: \"{}\"\n---\n\n# Knowledge bundle\n\n",
            super::model::current_okf_version()
        );
        fs::write(&root_index, body)?;
        written.push(root_index.clone());
    }

    for (dir, docs) in by_dir {
        let index_path = dir.join("index.md");
        let is_root = dir == bundle.root;
        let mut body = String::new();
        if is_root {
            let ver = bundle
                .okf_version
                .clone()
                .unwrap_or_else(|| super::model::current_okf_version().into());
            body.push_str(&format!("---\nokf_version: \"{ver}\"\n---\n\n"));
            body.push_str("# Knowledge bundle\n\n");
        } else {
            let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("index");
            body.push_str(&format!("# {name}\n\n"));
        }
        for doc in docs {
            let title = match &doc.frontmatter {
                OkfFrontmatterState::Parsed(fm) => fm
                    .title
                    .clone()
                    .or_else(|| fm.type_name.clone())
                    .unwrap_or_else(|| doc.concept_id.clone()),
                _ => doc.concept_id.clone(),
            };
            let desc = match &doc.frontmatter {
                OkfFrontmatterState::Parsed(fm) => fm.description.clone().unwrap_or_default(),
                _ => String::new(),
            };
            let rel = doc
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("concept.md");
            if desc.is_empty() {
                body.push_str(&format!("* [{title}]({rel})\n"));
            } else {
                body.push_str(&format!("* [{title}]({rel}) - {desc}\n"));
            }
        }
        // Preserve root log link etc. — rewrite full file for managed indexes
        fs::write(&index_path, body)?;
        written.push(index_path);
    }
    Ok(written)
}

pub fn okf_indexes_are_current(bundle: &OkfBundle) -> io::Result<bool> {
    let written = generate_okf_indexes_preview(bundle)?;
    for (path, expected) in written {
        let actual = fs::read_to_string(&path).unwrap_or_default();
        if actual != expected {
            return Ok(false);
        }
    }
    Ok(true)
}

fn generate_okf_indexes_preview(bundle: &OkfBundle) -> io::Result<Vec<(PathBuf, String)>> {
    // Dry-run style: compute expected content without writing (duplicate logic lightly)
    use std::collections::BTreeMap;
    let mut by_dir: BTreeMap<PathBuf, Vec<&super::model::OkfDocument>> = BTreeMap::new();
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        let parent = doc
            .path
            .parent()
            .unwrap_or(bundle.root.as_path())
            .to_path_buf();
        by_dir.entry(parent).or_default().push(doc);
    }
    let mut out = Vec::new();
    for (dir, docs) in by_dir {
        let index_path = dir.join("index.md");
        let is_root = dir == bundle.root;
        let mut body = String::new();
        if is_root {
            let ver = bundle
                .okf_version
                .clone()
                .unwrap_or_else(|| super::model::current_okf_version().into());
            body.push_str(&format!("---\nokf_version: \"{ver}\"\n---\n\n"));
            body.push_str("# Knowledge bundle\n\n");
        } else {
            let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("index");
            body.push_str(&format!("# {name}\n\n"));
        }
        for doc in docs {
            let title = match &doc.frontmatter {
                OkfFrontmatterState::Parsed(fm) => fm
                    .title
                    .clone()
                    .or_else(|| fm.type_name.clone())
                    .unwrap_or_else(|| doc.concept_id.clone()),
                _ => doc.concept_id.clone(),
            };
            let desc = match &doc.frontmatter {
                OkfFrontmatterState::Parsed(fm) => fm.description.clone().unwrap_or_default(),
                _ => String::new(),
            };
            let rel = doc
                .path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("concept.md");
            if desc.is_empty() {
                body.push_str(&format!("* [{title}]({rel})\n"));
            } else {
                body.push_str(&format!("* [{title}]({rel}) - {desc}\n"));
            }
        }
        out.push((index_path, body));
    }
    Ok(out)
}

/// Resolve a simple reading list: target concept + markdown link targets in body.
pub fn okf_context(bundle: &OkfBundle, id_or_path: &str) -> Vec<PathBuf> {
    let needle = id_or_path.trim().trim_end_matches(".md");
    let mut out = Vec::new();
    let Some(doc) = bundle.documents.iter().find(|d| {
        d.concept_id == needle
            || d.concept_id.ends_with(needle)
            || d.path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s == needle)
    }) else {
        return out;
    };
    out.push(doc.path.clone());
    for target in extract_md_links(&doc.body) {
        let resolved = resolve_link(&bundle.root, &doc.path, &target);
        if resolved.exists() && !out.contains(&resolved) {
            out.push(resolved);
        }
    }
    out
}

fn extract_md_links(body: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = body;
    while let Some(start) = rest.find("](") {
        let after = &rest[start + 2..];
        if let Some(end) = after.find(')') {
            let target = after[..end].trim();
            if !target.is_empty()
                && !target.starts_with("http://")
                && !target.starts_with("https://")
                && !target.starts_with('#')
            {
                links.push(target.to_string());
            }
            rest = &after[end + 1..];
        } else {
            break;
        }
    }
    links
}

fn resolve_link(root: &Path, from: &Path, target: &str) -> PathBuf {
    if let Some(stripped) = target.strip_prefix('/') {
        return root.join(stripped);
    }
    let base = from.parent().unwrap_or(root);
    base.join(target)
}

/// Export a simple graph markdown of concepts and link edges.
pub fn export_okf_graph(bundle: &OkfBundle, out: &Path) -> io::Result<PathBuf> {
    let mut md = String::from("# OKF graph\n\n## Concepts\n\n");
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        let ty = match &doc.frontmatter {
            OkfFrontmatterState::Parsed(fm) => fm.type_name.clone().unwrap_or_default(),
            _ => String::new(),
        };
        md.push_str(&format!("- `{}` ({ty})\n", doc.concept_id));
    }
    md.push_str("\n## Edges (markdown links)\n\n");
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        for target in extract_md_links(&doc.body) {
            md.push_str(&format!("- `{}` → `{}`\n", doc.concept_id, target));
        }
    }
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(out, md)?;
    Ok(out.to_path_buf())
}

/// Normalize trailing whitespace in concept frontmatter blocks (spacing only).
pub fn fmt_okf_bundle(bundle: &OkfBundle) -> io::Result<Vec<PathBuf>> {
    let mut changed = Vec::new();
    for doc in &bundle.documents {
        if doc.is_reserved && doc.path.file_name().and_then(|s| s.to_str()) == Some("log.md") {
            continue;
        }
        let text = fs::read_to_string(&doc.path)?;
        let normalized = normalize_fm_spacing(&text);
        if normalized != text {
            fs::write(&doc.path, normalized)?;
            changed.push(doc.path.clone());
        }
    }
    Ok(changed)
}

fn normalize_fm_spacing(text: &str) -> String {
    let (fm, body) = crate::parse::split_frontmatter(text);
    let Some(block) = fm else {
        return text.to_string();
    };
    let mut lines: Vec<String> = block.lines().map(|l| l.trim_end().to_string()).collect();
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    let mut out = String::from("---\n");
    out.push_str(&lines.join("\n"));
    if !lines.is_empty() {
        out.push('\n');
    }
    out.push_str("---\n");
    if !body.is_empty() {
        if !body.starts_with('\n') {
            out.push('\n');
        }
        out.push_str(body.trim_start_matches('\n'));
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}
