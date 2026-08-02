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
/// Recognizes:
/// - `root/SKILL.md` (package root)
/// - `root/skills/*/SKILL.md` (workspace skill tree)
pub fn skills_enabled(root: impl AsRef<Path>) -> bool {
    let root = root.as_ref();
    if root.join("SKILL.md").is_file() {
        return true;
    }
    let skills_dir = root.join("skills");
    if !skills_dir.is_dir() {
        return false;
    }
    let Ok(entries) = std::fs::read_dir(&skills_dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            return true;
        }
    }
    false
}

/// Collect skill package roots under `workspace` (for later lint).
pub fn skill_package_roots(workspace: impl AsRef<Path>) -> Vec<PathBuf> {
    let workspace = workspace.as_ref();
    let mut out = Vec::new();
    if workspace.join("SKILL.md").is_file() {
        out.push(workspace.to_path_buf());
    }
    let skills_dir = workspace.join("skills");
    if let Ok(entries) = std::fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("SKILL.md").is_file() {
                out.push(path);
            }
        }
    }
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
            td.path().join("index.md"),
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
