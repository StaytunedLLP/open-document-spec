use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct SkillsInitOptions {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct SkillsInitReport {
    pub root: PathBuf,
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

/// Scaffold an Agent Skills package directory with SKILL.md.
pub fn init_skill_package(
    root: impl AsRef<Path>,
    opts: SkillsInitOptions,
) -> io::Result<SkillsInitReport> {
    let root = root.as_ref().to_path_buf();
    fs::create_dir_all(&root)?;
    let name = opts
        .name
        .or_else(|| {
            root.file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "my-skill".into());
    let name = sanitize_name(&name);

    let mut report = SkillsInitReport {
        root: root.clone(),
        ..Default::default()
    };

    let skill_md = root.join("SKILL.md");
    if skill_md.exists() {
        report.skipped.push(skill_md);
    } else {
        let body = format!(
            "---\n\
             name: {name}\n\
             description: >-\n\
               Describe what this skill does and when to use it. Include keywords\n\
               that help agents select this skill for relevant tasks.\n\
             ---\n\
             \n\
             # {name}\n\
             \n\
             ## Instructions\n\
             \n\
             1. Step one\n\
             2. Step two\n\
             \n\
             ## Examples\n\
             \n\
             Provide input/output examples here.\n"
        );
        fs::write(&skill_md, body)?;
        report.created.push(skill_md);
    }

    for sub in ["scripts", "references", "assets"] {
        let p = root.join(sub);
        if !p.exists() {
            fs::create_dir_all(&p)?;
            report.created.push(p);
        } else {
            report.skipped.push(p);
        }
    }

    Ok(report)
}

fn sanitize_name(raw: &str) -> String {
    let mut s: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_uppercase() {
                c.to_ascii_lowercase()
            } else if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    while s.contains("--") {
        s = s.replace("--", "-");
    }
    s = s.trim_matches('-').to_string();
    if s.is_empty() {
        "my-skill".into()
    } else {
        s.chars().take(64).collect()
    }
}
