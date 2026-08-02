use super::model::{SkillFrontmatter, SkillPackage};
use crate::parse::split_frontmatter;
use std::fs;
use std::io;
use std::path::Path;

pub fn parse_skill_package(root: impl AsRef<Path>) -> io::Result<SkillPackage> {
    let root = root.as_ref();
    let skill_md = root.join("SKILL.md");
    let text = fs::read_to_string(&skill_md)?;
    let (fm_block, body) = split_frontmatter(&text);
    let frontmatter = match fm_block {
        Some(block) => parse_skill_frontmatter_block(block),
        None => SkillFrontmatter::default(),
    };
    let dir_name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();
    Ok(SkillPackage {
        root: root.to_path_buf(),
        skill_md,
        frontmatter,
        body: body.to_string(),
        dir_name,
    })
}

pub fn parse_skill_frontmatter_block(block: &str) -> SkillFrontmatter {
    let mut fm = SkillFrontmatter::default();
    let lines: Vec<&str> = block.lines().collect();
    let mut i = 0usize;
    while i < lines.len() {
        let line = lines[i].trim_end_matches('\r');
        i += 1;
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if line.starts_with(' ') || line.starts_with('\t') {
            continue;
        }
        let Some((key, rest)) = line.split_once(':') else {
            continue;
        };
        let key = key.trim();
        let rest = rest.trim();
        match key {
            "name" => fm.name = Some(unquote(rest)),
            "description" => {
                if rest.is_empty() || rest == ">" || rest == "|" {
                    // folded/literal block: collect indented lines
                    let mut parts = Vec::new();
                    while i < lines.len() {
                        let l = lines[i];
                        if l.starts_with(' ') || l.starts_with('\t') {
                            parts.push(l.trim());
                            i += 1;
                        } else if l.trim().is_empty() {
                            i += 1;
                            break;
                        } else {
                            break;
                        }
                    }
                    fm.description = Some(parts.join(" "));
                } else {
                    fm.description = Some(unquote(rest));
                }
            }
            "license" => fm.license = Some(unquote(rest)),
            "compatibility" => fm.compatibility = Some(unquote(rest)),
            "allowed-tools" => fm.allowed_tools = Some(unquote(rest)),
            "metadata" => {
                // map block
                while i < lines.len() {
                    let l = lines[i];
                    let t = l.trim_end_matches('\r');
                    if !(t.starts_with(' ') || t.starts_with('\t')) {
                        break;
                    }
                    if let Some((k, v)) = t.trim().split_once(':') {
                        fm.metadata
                            .insert(k.trim().to_string(), unquote(v.trim()));
                    }
                    i += 1;
                }
            }
            other => {
                fm.unknown.insert(other.to_string(), unquote(rest));
            }
        }
    }
    fm
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len().saturating_sub(1)].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal() {
        let fm = parse_skill_frontmatter_block(
            "name: pdf-processing\ndescription: Extract PDF text when needed.\n",
        );
        assert_eq!(fm.name.as_deref(), Some("pdf-processing"));
        assert!(fm.description.as_ref().unwrap().contains("PDF"));
    }

    #[test]
    fn parse_optional_fields() {
        let fm = parse_skill_frontmatter_block(
            "name: demo\ndescription: Demo skill description here.\nlicense: Apache-2.0\ncompatibility: Requires git\nallowed-tools: Bash Read\nmetadata:\n  author: org\n  version: \"1.0\"\n",
        );
        assert_eq!(fm.license.as_deref(), Some("Apache-2.0"));
        assert_eq!(fm.metadata.get("author").map(String::as_str), Some("org"));
        assert!(fm.allowed_tools.as_ref().unwrap().contains("Bash"));
    }
}
