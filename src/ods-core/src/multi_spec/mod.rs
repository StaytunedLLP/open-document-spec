//! Multi-spec detection and engine selection (functional).
//!
//! ODS is the default native product. Extra specs (OKF, Agent Skills) are
//! enabled only via CLI flags (`--okf`, `--skills`) — never a `--ods` flag.

mod detect;
mod scope;
pub mod skills;

pub use detect::{Detected, detect_workspace, skill_package_roots, skills_enabled};
pub use scope::{
    ActiveEngines, ExtraSpecs, ScopeResolveError, parse_extra_spec_flags, resolve_engines,
};
pub use skills::{
    SkillFrontmatter, SkillPackage, SkillsInitOptions, SkillsInitReport, init_skill_package,
    lint_skill_package, lint_skill_package_with_config, parse_skill_package,
};
