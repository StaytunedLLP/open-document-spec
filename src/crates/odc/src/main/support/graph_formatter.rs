fn print_profiles(workspace: &odc_core::Workspace, format: OutputFormat) {
    match format {
        OutputFormat::Text => {
            println!("profiles:");
            for (name, definition) in &workspace.profiles.definitions {
                let layer = profile_layer(&definition.source);
                println!("  {name}: [{layer}] {}", definition.source.display());
            }

            if workspace.profiles.conflicts.is_empty() {
                return;
            }

            println!("profile conflicts:");
            for conflict in &workspace.profiles.conflicts {
                println!(
                    "  {}: kept {}, ignored {}",
                    conflict.name,
                    conflict.kept.display(),
                    conflict.ignored.display()
                );
            }
        }
        OutputFormat::Json => {
            let profiles: Vec<_> = workspace
                .profiles
                .definitions
                .iter()
                .map(|(name, definition)| {
                    let layer = profile_layer(&definition.source);
                    format!(
                        r#"{{"name":{},"layer":{},"source":{}}}"#,
                        json_escape(name),
                        json_escape(layer),
                        json_escape(&definition.source.display().to_string())
                    )
                })
                .collect();
            let conflicts: Vec<_> = workspace
                .profiles
                .conflicts
                .iter()
                .map(|c| {
                    format!(
                        r#"{{"name":{},"kept":{},"ignored":{}}}"#,
                        json_escape(&c.name),
                        json_escape(&c.kept.display().to_string()),
                        json_escape(&c.ignored.display().to_string())
                    )
                })
                .collect();
            println!(
                r#"{{"profiles":[{}],"conflicts":[{}]}}"#,
                profiles.join(","),
                conflicts.join(",")
            );
        }
    }
}

fn exit_code(diagnostics: &[Diagnostic]) -> ExitCode {
    if diagnostics
        .iter()
        .any(|d| matches!(d.severity, Severity::Error))
    {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}
