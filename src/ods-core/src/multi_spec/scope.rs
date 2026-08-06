use super::detect::Detected;

/// Extra-spec flags from argv. ODS has **no** flag — it is the default product.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ExtraSpecs {
    pub okf: bool,
    pub skills: bool,
}

/// Which engines should run for a command after resolving flags × detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActiveEngines {
    pub ods: bool,
    pub okf: bool,
    pub skills: bool,
}

impl ActiveEngines {
    pub fn any(self) -> bool {
        self.ods || self.okf || self.skills
    }
}

/// Why engine resolution failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScopeResolveError {
    /// No ODS workspace and no extra-spec flags.
    NotOdsWorkspace { hint_okf: bool, hint_skills: bool },
    /// `--okf` set but tree is not an OKF bundle (for ops that require one).
    NotOkfBundle,
    /// `--skills` set but no skill package found (for ops that require one).
    NoSkillsPackage,
    /// User passed forbidden `--ods`.
    ForbiddenOdsFlag,
}

impl ScopeResolveError {
    /// User-facing message from the central catalog (`crate::error::messages`).
    pub fn message(&self) -> String {
        match self {
            ScopeResolveError::NotOdsWorkspace {
                hint_okf,
                hint_skills,
            } => crate::error::not_ods_workspace(*hint_okf, *hint_skills).render_error(),
            ScopeResolveError::NotOkfBundle => crate::error::not_okf_bundle().render_error(),
            ScopeResolveError::NoSkillsPackage => crate::error::no_skills_package().render_error(),
            // Flag parse error → usage-style text (CLI maps this to exit 2).
            ScopeResolveError::ForbiddenOdsFlag => {
                crate::error::forbidden_ods_flag().render_usage()
            }
        }
    }
}

/// Parse `--okf` / `--skills` from argv. Rejects `--ods` as a usage error signal.
///
/// Returns `Err(ScopeResolveError::ForbiddenOdsFlag)` if `--ods` appears.
pub fn parse_extra_spec_flags<'a>(
    args: impl IntoIterator<Item = &'a str>,
) -> Result<ExtraSpecs, ScopeResolveError> {
    let mut extra = ExtraSpecs::default();
    for a in args {
        match a {
            "--ods" => return Err(ScopeResolveError::ForbiddenOdsFlag),
            "--okf" => extra.okf = true,
            "--skills" => extra.skills = true,
            _ => {}
        }
    }
    Ok(extra)
}

/// Resolve which engines run for a read/validate-style command (lint/doctor/audit).
///
/// Policy (locked):
/// - ODS runs when `detected.ods` (default product — no flag).
/// - OKF / Skills run when their flags are set OR when declared enabled in root `ods.toml` `[specs.*]`.
/// - Pure other-spec trees: flags alone enable that engine.
/// - If nothing to run: `NotOdsWorkspace` (with hints when markers suggest other specs).
///
/// When `require_present` is true, `--okf` / `--skills` require the dialect to exist.
pub fn resolve_engines(
    extra: ExtraSpecs,
    detected: Detected,
    require_present: bool,
) -> Result<ActiveEngines, ScopeResolveError> {
    resolve_engines_with_config(extra, detected, None, require_present)
}

pub fn resolve_engines_with_config(
    extra: ExtraSpecs,
    detected: Detected,
    config: Option<&crate::model::WorkspaceSpecsConfig>,
    require_present: bool,
) -> Result<ActiveEngines, ScopeResolveError> {
    let okf_enabled = extra.okf || config.is_some_and(|c| c.okf.enabled);
    let skills_enabled = extra.skills || config.is_some_and(|c| c.skills.enabled);

    let engines = ActiveEngines {
        ods: detected.ods,
        okf: okf_enabled,
        skills: skills_enabled,
    };

    if require_present {
        if engines.okf && !detected.okf {
            return Err(ScopeResolveError::NotOkfBundle);
        }
        if engines.skills && !detected.skills {
            return Err(ScopeResolveError::NoSkillsPackage);
        }
    }

    if !engines.any() {
        return Err(ScopeResolveError::NotOdsWorkspace {
            hint_okf: detected.okf,
            hint_skills: detected.skills,
        });
    }

    Ok(engines)
}

pub fn load_root_specs_config(root: &std::path::Path) -> crate::model::WorkspaceSpecsConfig {
    if let Ok(cfg) = crate::config::load_workspace_config(root) {
        return cfg.to_workspace_specs();
    }
    crate::model::WorkspaceSpecsConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn det(ods: bool, okf: bool, skills: bool) -> Detected {
        Detected { ods, okf, skills }
    }

    #[test]
    fn bare_ods_workspace() {
        let e = resolve_engines(ExtraSpecs::default(), det(true, false, false), true).unwrap();
        assert!(e.ods && !e.okf && !e.skills);
    }

    #[test]
    fn hybrid_bare_is_ods_only() {
        let e = resolve_engines(ExtraSpecs::default(), det(true, true, false), true).unwrap();
        assert!(e.ods && !e.okf);
    }

    #[test]
    fn hybrid_with_okf_flag_runs_both() {
        let e = resolve_engines(
            ExtraSpecs {
                okf: true,
                skills: false,
            },
            det(true, true, false),
            true,
        )
        .unwrap();
        assert!(e.ods && e.okf);
    }

    #[test]
    fn pure_okf_bare_errors_with_hint() {
        let err =
            resolve_engines(ExtraSpecs::default(), det(false, true, false), true).unwrap_err();
        match err {
            ScopeResolveError::NotOdsWorkspace { hint_okf, .. } => assert!(hint_okf),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn pure_okf_with_flag() {
        let e = resolve_engines(
            ExtraSpecs {
                okf: true,
                skills: false,
            },
            det(false, true, false),
            true,
        )
        .unwrap();
        assert!(!e.ods && e.okf);
    }

    #[test]
    fn okf_flag_without_bundle() {
        let err = resolve_engines(
            ExtraSpecs {
                okf: true,
                skills: false,
            },
            det(false, false, false),
            true,
        )
        .unwrap_err();
        assert!(matches!(err, ScopeResolveError::NotOkfBundle));
    }

    #[test]
    fn parse_flags_okf_skills() {
        let e = parse_extra_spec_flags(["lint", "--okf", "--skills", "path"]).unwrap();
        assert!(e.okf && e.skills);
    }

    #[test]
    fn parse_flags_rejects_ods() {
        let err = parse_extra_spec_flags(["lint", "--ods"]).unwrap_err();
        assert!(matches!(err, ScopeResolveError::ForbiddenOdsFlag));
    }
}
