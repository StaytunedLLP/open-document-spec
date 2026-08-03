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
    pub fn message(&self) -> String {
        match self {
            ScopeResolveError::NotOdsWorkspace {
                hint_okf,
                hint_skills,
            } => {
                let mut msg = String::from(
                    "not an ODS workspace (no root index.ods.md with 'ods:' marker).\n\n\
                     To fix:\n\
                     • Run `ods init` here to make this folder ODS-compliant",
                );
                if *hint_okf {
                    msg.push_str(
                        "\n• Or pass `--okf` for a Google OKF v0.2 bundle (`ods init --okf`, then `ods lint --okf`)",
                    );
                }
                if *hint_skills {
                    msg.push_str(
                        "\n• Or pass `--skills` for Agent Skills packages (`ods init --skills`, then `ods lint --skills`)",
                    );
                }
                msg
            }
            ScopeResolveError::NotOkfBundle => {
                "not an OKF bundle: no root index.md with okf_version.\n\
                 Run `ods init --okf` to create an OKF v0.2 bundle."
                    .into()
            }
            ScopeResolveError::NoSkillsPackage => {
                "no Agent Skills package found (expected SKILL.md at root or under skills/).\n\
                 Run `ods init --skills` to scaffold a skill package."
                    .into()
            }
            ScopeResolveError::ForbiddenOdsFlag => "unknown flag: --ods\n\n\
                 ODS is the default native engine of this CLI — no flag is needed.\n\
                 Use bare `ods <cmd>` for ODS.\n\
                 Use `--okf` or `--skills` only for other specs."
                .into(),
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
/// - OKF / Skills run when their flags are set OR when declared enabled in root `index.ods.md` `specs:`.
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
    let index_paths = [root.join("index.ods.md"), root.join("index.md")];
    for idx_path in &index_paths {
        if idx_path.is_file() {
            if let Ok(text) = std::fs::read_to_string(idx_path) {
                let doc = crate::parse::parse_document_text(root, idx_path.clone(), &text, false);
                if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
                    return fm.specs;
                }
            }
        }
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
