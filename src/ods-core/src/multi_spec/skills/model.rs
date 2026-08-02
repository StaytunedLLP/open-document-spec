use std::collections::BTreeMap;
use std::path::PathBuf;

/// Parsed SKILL.md frontmatter (Agent Skills spec).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SkillFrontmatter {
    pub name: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: BTreeMap<String, String>,
    pub allowed_tools: Option<String>,
    /// Unknown keys preserved.
    pub unknown: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillPackage {
    pub root: PathBuf,
    pub skill_md: PathBuf,
    pub frontmatter: SkillFrontmatter,
    pub body: String,
    pub dir_name: String,
}
