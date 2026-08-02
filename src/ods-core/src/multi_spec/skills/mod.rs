//! Agent Skills package support (parse / lint / init).
//! Activated only via CLI flag `--skills`.

mod init;
mod lint;
mod model;
mod parse;

pub use init::{SkillsInitOptions, SkillsInitReport, init_skill_package};
pub use lint::lint_skill_package;
pub use model::{SkillFrontmatter, SkillPackage};
pub use parse::parse_skill_package;
