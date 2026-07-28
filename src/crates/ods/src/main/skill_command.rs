fn run_skill_command(args: &[String]) -> Result<ExitCode, CliError> {
    let sub = args.get(2).map(String::as_str).unwrap_or("help");
    if sub == "help" || sub == "-h" || sub == "--help" {
        print_skill_help();
        return Ok(ExitCode::from(0));
    }
    if sub != "install" {
        return Err(usage(format!(
            "unknown skill subcommand: {sub} (use install or help)"
        )));
    }

    let agent = parse_flag_val(args, "--agent").ok_or_else(|| {
        usage("missing required --agent parameter (e.g. --agent claude-code)")
    })?;

    let scope_val = parse_flag_val(args, "--scope");
    let scope = match scope_val.as_deref() {
        Some("project") => "project",
        Some("user") => "user",
        Some(other) => {
            return Err(usage(format!(
                "invalid scope: {other} (use project or user)"
            )));
        }
        None => {
            // Default scopes per agent
            match agent.as_str() {
                "claude-code" | "antigravity" | "codex" | "gemini-cli" => "user",
                "cursor" | "copilot" | "windsurf" => "project",
                _ => "project",
            }
        }
    };

    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| failure("could not resolve home directory"))?;

    let target = match agent.as_str() {
        "claude-code" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".claude/skills/ods")
            } else {
                PathBuf::from(".claude/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "cursor" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".cursor/rules/ods.mdc")
            } else {
                PathBuf::from(".cursor/rules/ods.mdc")
            };
            SkillInstallTarget::File {
                path,
                content: CURSOR_MDC_TEMPLATE.as_bytes(),
            }
        }
        "antigravity" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".gemini/config/skills/ods")
            } else {
                PathBuf::from(".gemini/config/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "codex" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".codex/skills/ods")
            } else {
                PathBuf::from(".codex/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "gemini-cli" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".gemini/skills/ods")
            } else {
                PathBuf::from(".gemini/skills/ods")
            };
            SkillInstallTarget::Bundle(path)
        }
        "windsurf" => {
            let path = if scope == "user" {
                PathBuf::from(&home).join(".codeium/windsurf/memories/global_rules.md")
            } else {
                PathBuf::from(".windsurf/rules/ods.md")
            };
            SkillInstallTarget::File {
                path,
                content: WINDSURF_RULE_TEMPLATE.as_bytes(),
            }
        }
        "copilot" => {
            if scope == "user" {
                eprintln!("warning: GitHub Copilot only reads workspace-level instructions. Writing to project scope instead.");
            }
            SkillInstallTarget::File {
                path: PathBuf::from(".github/copilot-instructions.md"),
                content: SKILL_BUNDLE[0].1,
            }
        }
        other => {
            return Err(usage(format!(
                "unknown agent: {other} (use claude-code, cursor, antigravity, codex, gemini-cli, windsurf, or copilot)"
            )));
        }
    };

    let dest_path = match target {
        SkillInstallTarget::Bundle(path) => {
            install_skill_bundle(&path)?;
            path
        }
        SkillInstallTarget::File { path, content } => {
            write_install_file(&path, content)?;
            path
        }
    };

    println!(
        "✓ ODS skill successfully installed for agent '{}' under '{}' scope (path: {})",
        agent,
        scope,
        dest_path.display()
    );

    Ok(ExitCode::from(0))
}

enum SkillInstallTarget {
    Bundle(PathBuf),
    File {
        path: PathBuf,
        content: &'static [u8],
    },
}

fn install_skill_bundle(destination: &Path) -> Result<(), CliError> {
    for (relative_path, contents) in SKILL_BUNDLE {
        write_install_file(&destination.join(relative_path), contents)?;
    }
    Ok(())
}

fn write_install_file(path: &Path, contents: &[u8]) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            failure(format!(
                "failed to create destination directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    fs::write(path, contents)
        .map_err(|e| failure(format!("failed to write skill file to {}: {e}", path.display())))
}

fn print_skill_help() {
    println!(
        "ods skill <command> [flags]

Commands:
  install                  Install ODS skill or rules configuration into an AI agent's directory.
  help                     Print this help message.

Flags:
  --agent <name>           The AI agent target (claude-code, cursor, antigravity, codex, gemini-cli, windsurf, copilot)
  --scope <project|user>   Install to project workspace or global home directory (optional)"
    );
}

const CURSOR_MDC_TEMPLATE: &str = r#"---
description: Activates when editing files inside an ODS workspace. Enforces frontmatter standards, dependency graph integrity, and runs ODS CLI check commands.
globs: ["*.md"]
---
# Open Document Specs (ODS) Rules

When editing Markdown files in this project:
1. **Frontmatter Integrity**: Keep ODS metadata (`profile`, `status`, `share`, `depends`, `related`, `code`) consistent.
2. **Context Resolution**: Use `ods context <file>` to resolve dependency graphs before sending text to the LLM prompt.
3. **Automatic Link Repair**: Use `ods mv <src> <dst>` instead of `mv` when renaming or moving documents.
4. **Validation**: Run `ods lint` to check for broken links and structural compliance.
"#;

const WINDSURF_RULE_TEMPLATE: &str = r#"---
trigger: always_on
description: Keep Markdown documentation compliant with Open Document Specs.
---
# Open Document Specs (ODS) Rules

When editing Markdown files in this project:
1. Keep ODS metadata (`profile`, `status`, `share`, `depends`, `related`, `code`) consistent.
2. Run `ods context <file>` before loading a document's dependency graph into the agent context.
3. Use `ods mv <src> <dst>` instead of `mv` when renaming or moving documents.
4. Run `ods lint` to check links and structural compliance.
"#;

const SKILL_BUNDLE: &[(&str, &[u8])] = &[
    ("SKILL.md", include_bytes!("../../../../../skills/ods/SKILL.md")),
    ("CHANGELOG.md", include_bytes!("../../../../../skills/ods/CHANGELOG.md")),
    ("index.md", include_bytes!("../../../../../skills/ods/index.md")),
    ("evals/evals.json", include_bytes!("../../../../../skills/ods/evals/evals.json")),
    ("references/index.md", include_bytes!("../../../../../skills/ods/references/index.md")),
    ("references/non-goals.md", include_bytes!("../../../../../skills/ods/references/non-goals.md")),
    ("references/spec.md", include_bytes!("../../../../../skills/ods/references/spec.md")),
    ("scripts/bootstrap.ps1", include_bytes!("../../../../../skills/ods/scripts/bootstrap.ps1")),
    ("scripts/bootstrap.sh", include_bytes!("../../../../../skills/ods/scripts/bootstrap.sh")),
    ("scripts/install-from-release.sh", include_bytes!("../../../../../skills/ods/scripts/install-from-release.sh")),
];
