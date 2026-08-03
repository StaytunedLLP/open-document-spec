use crate::lifecycle::ods_enabled;
use crate::okf::okf_enabled;
use std::path::{Path, PathBuf};

/// Which product dialects are present under `root`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Detected {
    pub ods: bool,
    pub okf: bool,
    pub skills: bool,
}

/// Detect ODS / OKF / Agent Skills markers under `root`.
///
/// Pure of side effects beyond filesystem reads.
pub fn detect_workspace(root: impl AsRef<Path>) -> Detected {
    let root = root.as_ref();
    Detected {
        ods: ods_enabled(root),
        okf: okf_enabled(root),
        skills: skills_enabled(root),
    }
}

/// True when `root` is (or contains) an Agent Skills package (`SKILL.md`).
///
/// Recognizes `SKILL.md` files located anywhere in the directory tree (zero-config auto-discovery).
pub fn skills_enabled(root: impl AsRef<Path>) -> bool {
    !skill_package_roots(root).is_empty()
}

/// Collect skill package roots under `workspace` (for later lint).
///
/// Discovers SKILL.md packages by:
/// 1. Recursively walking workspace directory for any `SKILL.md` file at any depth.
/// 2. Reading explicit `skills:` array from root `index.ods.md` / `index.md` frontmatter (if defined).
pub fn skill_package_roots(workspace: impl AsRef<Path>) -> Vec<PathBuf> {
    let workspace = workspace.as_ref();
    let mut out = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. Check explicit skills: in root index.ods.md / index.md frontmatter
    let index_paths = [workspace.join("index.ods.md"), workspace.join("index.md")];
    for idx_path in &index_paths {
        if idx_path.is_file() {
            if let Ok(text) = std::fs::read_to_string(idx_path) {
                let (fm, _) = crate::parse::split_frontmatter(&text);
                if let Some(fm) = fm {
                    for line in fm.lines() {
                        let trimmed = line.trim();
                        if trimmed.starts_with("- ")
                            && (trimmed.ends_with(".md") || trimmed.contains("SKILL"))
                        {
                            let rel = trimmed
                                .trim_start_matches("- ")
                                .trim_matches(|c| c == '\'' || c == '"');
                            let p = workspace.join(rel);
                            let pkg_dir = if p.is_file() {
                                p.parent().unwrap_or(workspace).to_path_buf()
                            } else {
                                p
                            };
                            if pkg_dir.join("SKILL.md").is_file() && seen.insert(pkg_dir.clone()) {
                                out.push(pkg_dir);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Recursive zero-config auto-discovery
    fn walk_dir(dir: &Path, out: &mut Vec<PathBuf>, seen: &mut std::collections::HashSet<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.')
                || name == "node_modules"
                || name == "target"
                || name == "vendor"
                || name == "tmp"
            {
                continue;
            }
            if path.is_dir() {
                if path.join("SKILL.md").is_file() && seen.insert(path.clone()) {
                    out.push(path.clone());
                }
                walk_dir(&path, out, seen);
            } else if name == "SKILL.md" {
                if let Some(parent) = path.parent() {
                    if seen.insert(parent.to_path_buf()) {
                        out.push(parent.to_path_buf());
                    }
                }
            }
        }
    }

    if workspace.join("SKILL.md").is_file() && seen.insert(workspace.to_path_buf()) {
        out.push(workspace.to_path_buf());
    }

    walk_dir(workspace, &mut out, &mut seen);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detect_empty() {
        let td = tempdir().unwrap();
        let d = detect_workspace(td.path());
        assert!(!d.ods && !d.okf && !d.skills);
    }

    #[test]
    fn detect_ods_marker() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        let d = detect_workspace(td.path());
        assert!(d.ods);
        assert!(!d.okf);
    }

    #[test]
    fn detect_okf_marker() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# K\n",
        )
        .unwrap();
        let d = detect_workspace(td.path());
        assert!(!d.ods);
        assert!(d.okf);
    }

    #[test]
    fn detect_skill_package_root() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("SKILL.md"),
            "---\nname: demo\ndescription: A demo skill for tests.\n---\n\n# Demo\n",
        )
        .unwrap();
        assert!(skills_enabled(td.path()));
        assert!(detect_workspace(td.path()).skills);
    }

    #[test]
    fn detect_skills_subdir() {
        let td = tempdir().unwrap();
        let pkg = td.path().join("skills").join("demo");
        fs::create_dir_all(&pkg).unwrap();
        fs::write(
            pkg.join("SKILL.md"),
            "---\nname: demo\ndescription: Nested skill package.\n---\n\n# Demo\n",
        )
        .unwrap();
        let roots = skill_package_roots(td.path());
        assert_eq!(roots.len(), 1);
        assert!(detect_workspace(td.path()).skills);
    }
}
