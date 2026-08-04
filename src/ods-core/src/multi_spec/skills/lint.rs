use super::model::SkillPackage;
use crate::model::{Diagnostic, Severity};
use std::path::PathBuf;

/// Lint an Agent Skills package against agentskills.md constraints.
pub fn lint_skill_package(pkg: &SkillPackage) -> Vec<Diagnostic> {
    lint_skill_package_with_config(pkg, &crate::model::SpecLintConfig::default())
}

pub fn lint_skill_package_with_config(
    pkg: &SkillPackage,
    config: &crate::model::SpecLintConfig,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let path = pkg.skill_md.clone();
    let fm = &pkg.frontmatter;

    if config.check_keys {
        match fm.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            None => {
                if !config.ignore_keys.contains("name") {
                    out.push(diag_msg(path.clone(), crate::error::skills_missing_name()));
                }
            }
            Some(name) => {
                if name.len() > 64 {
                    out.push(diag_msg(
                        path.clone(),
                        crate::error::skills_name_too_long(name.len()),
                    ));
                }
                if !is_valid_skill_name(name) {
                    out.push(diag_msg(path.clone(), crate::error::skills_name_invalid()));
                }
                if !pkg.dir_name.is_empty() && name != pkg.dir_name {
                    out.push(diag_msg(
                        path.clone(),
                        crate::error::skills_name_dir_mismatch(name, &pkg.dir_name),
                    ));
                }
            }
        }

        match fm
            .description
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            None => {
                if !config.ignore_keys.contains("description") {
                    out.push(diag_msg(
                        path.clone(),
                        crate::error::skills_missing_description(),
                    ));
                }
            }
            Some(desc) => {
                if desc.len() > 1024 {
                    out.push(diag_msg(
                        path.clone(),
                        crate::error::skills_description_too_long(desc.len()),
                    ));
                }
            }
        }
    }

    if let Some(c) = fm.compatibility.as_deref() {
        if c.len() > 500 {
            out.push(diag_msg(
                path.clone(),
                crate::error::skills_compatibility_too_long(c.len()),
            ));
        }
    }

    // Progressive disclosure advisory
    let body_lines = pkg.body.lines().count();
    if body_lines > 500 {
        out.push(Diagnostic {
            path: path.clone(),
            severity: Severity::Warning,
            message: crate::error::skills_body_too_long(body_lines),
        });
    }

    out
}

fn is_valid_skill_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 64 {
        return false;
    }
    if name.starts_with('-') || name.ends_with('-') || name.contains("--") {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn diag_msg(path: PathBuf, message: String) -> Diagnostic {
    Diagnostic {
        path,
        severity: Severity::Error,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_spec::skills::model::SkillFrontmatter;
    use std::path::PathBuf;

    fn pkg(name: &str, dir: &str, desc: &str) -> SkillPackage {
        SkillPackage {
            root: PathBuf::from(dir),
            skill_md: PathBuf::from(dir).join("SKILL.md"),
            frontmatter: SkillFrontmatter {
                name: Some(name.into()),
                description: Some(desc.into()),
                ..Default::default()
            },
            body: "# Hi\n".into(),
            dir_name: dir.into(),
        }
    }

    #[test]
    fn valid_package() {
        let d = lint_skill_package(&pkg(
            "demo",
            "demo",
            "A valid skill description for testing.",
        ));
        assert!(d.iter().all(|x| x.severity != Severity::Error), "{d:?}");
    }

    #[test]
    fn invalid_name_case() {
        let d = lint_skill_package(&pkg(
            "Demo",
            "Demo",
            "A valid skill description for testing.",
        ));
        assert!(d.iter().any(|x| x.message.contains("lowercase")));
    }

    #[test]
    fn name_dir_mismatch() {
        let d = lint_skill_package(&pkg(
            "demo",
            "other",
            "A valid skill description for testing.",
        ));
        assert!(d.iter().any(|x| x.message.contains("parent directory")));
    }
}
