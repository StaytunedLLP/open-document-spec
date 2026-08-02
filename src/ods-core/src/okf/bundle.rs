use super::model::{OkfBundle, OkfDocument, concept_id_for_path};
use super::parse::parse_okf_document_text;
use crate::parse::split_frontmatter;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn okf_enabled(root: &Path) -> bool {
    okf_version_from_root(root).is_some()
}

pub fn okf_version_from_root(root: &Path) -> Option<String> {
    let index = root.join("index.md");
    let text = fs::read_to_string(index).ok()?;
    let (fm, _) = split_frontmatter(&text);
    let block = fm?;
    for line in block.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("okf_version:") {
            let v = rest.trim().trim_matches('"').trim_matches('\'').to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

pub fn load_okf_bundle(root: &Path) -> io::Result<OkfBundle> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let okf_version = okf_version_from_root(&root);
    let mut documents = Vec::new();
    scan_md(&root, &root, &mut documents)?;
    Ok(OkfBundle {
        root,
        okf_version,
        documents,
    })
}

fn scan_md(root: &Path, dir: &Path, out: &mut Vec<OkfDocument>) -> io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            scan_md(root, &path, out)?;
            continue;
        }
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
        let is_reserved = name == "index.md" || name == "log.md";
        let text = fs::read_to_string(&path)?;
        let frontmatter = if is_reserved && name == "index.md" {
            // root index may only carry okf_version; still parse lightly
            parse_okf_document_text(&text)
        } else if is_reserved {
            super::model::OkfFrontmatterState::Absent
        } else {
            parse_okf_document_text(&text)
        };
        let (_, body) = split_frontmatter(&text);
        out.push(OkfDocument {
            path: path.clone(),
            concept_id: concept_id_for_path(root, &path),
            body: body.to_string(),
            frontmatter,
            is_reserved,
        });
    }
    Ok(())
}
