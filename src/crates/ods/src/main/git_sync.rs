fn doctor_workspace(root: &Path) -> Result<DoctorReport, CliError> {
    let mut lines = Vec::new();
    let mut json_fields = Vec::new();
    let mut has_error = false;
    lines.push(format!("workspace: {}", root.display()));
    json_fields.push(format!(
        r#""workspace":{}"#,
        json_escape(&root.display().to_string())
    ));
    lines.push(format!("ods cli version: {}", env!("CARGO_PKG_VERSION")));
    json_fields.push(format!(
        r#""ods_cli_version":{}"#,
        json_escape(env!("CARGO_PKG_VERSION"))
    ));

    match load_workspace(root) {
        Ok(workspace) => {
            lines.push(format!("documents: {}", workspace.documents.len()));
            json_fields.push(format!(r#""documents":{}"#, workspace.documents.len()));
            let root_meta = workspace
                .document_by_path(&workspace.root.join("index.md"))
                .and_then(|doc| match &doc.frontmatter {
                    ods_core::FrontmatterState::Parsed(fm) => {
                        Some((fm.ods.as_deref(), fm.ods_cli.as_deref()))
                    }
                    _ => None,
                });
            let (root_ods, root_ods_cli) = root_meta.unwrap_or((None, None));
            match root_ods {
                Some(version) if version == ods_core::current_ods_spec_version() => {
                    lines.push(format!("root ods spec: {version}"));
                    json_fields.push(format!(r#""root_ods":{}"#, json_escape(version)));
                    json_fields.push(r#""root_ods_current":true"#.to_string());
                }
                Some(version) => {
                    has_error = true;
                    lines.push(format!(
                        "root ods spec: {version} (expected {})",
                        ods_core::current_ods_spec_version()
                    ));
                    json_fields.push(format!(r#""root_ods":{}"#, json_escape(version)));
                    json_fields.push(r#""root_ods_current":false"#.to_string());
                }
                None => {
                    has_error = true;
                    lines.push(format!(
                        "root ods: missing (expected {})",
                        ods_core::current_ods_spec_version()
                    ));
                    json_fields.push(r#""root_ods":null"#.to_string());
                    json_fields.push(r#""root_ods_current":false"#.to_string());
                }
            }
            match root_ods_cli {
                Some(requirement) => match ods_core::ods_cli_requirement_satisfied(requirement) {
                    Ok(true) => {
                        lines.push(format!("root ods-cli: {requirement}"));
                        json_fields
                            .push(format!(r#""root_ods_cli":{}"#, json_escape(requirement)));
                        json_fields.push(r#""root_ods_cli_satisfied":true"#.to_string());
                    }
                    Ok(false) => {
                        has_error = true;
                        lines.push(format!(
                            "root ods-cli: {requirement} (installed {})",
                            ods_core::current_ods_version()
                        ));
                        json_fields
                            .push(format!(r#""root_ods_cli":{}"#, json_escape(requirement)));
                        json_fields.push(r#""root_ods_cli_satisfied":false"#.to_string());
                    }
                    Err(err) => {
                        has_error = true;
                        lines.push(format!("root ods-cli: invalid ({err})"));
                        json_fields
                            .push(format!(r#""root_ods_cli":{}"#, json_escape(requirement)));
                        json_fields.push(r#""root_ods_cli_satisfied":false"#.to_string());
                    }
                },
                None => {
                    has_error = true;
                    lines.push(format!(
                        "root ods-cli: missing (expected \"{}\")",
                        ods_core::current_ods_cli_requirement()
                    ));
                    json_fields.push(r#""root_ods_cli":null"#.to_string());
                    json_fields.push(r#""root_ods_cli_satisfied":false"#.to_string());
                }
            }
            let current =
                indexes_are_current(&workspace).map_err(|err| failure(err.to_string()))?;
            lines.push(if current {
                "indexes: current".to_string()
            } else {
                has_error = true;
                "indexes: stale (run `ods index`)".to_string()
            });
            json_fields.push(format!(
                r#""indexes_current":{}"#,
                if current { "true" } else { "false" }
            ));

            let conflicts = workspace.profiles.conflicts.len();
            if conflicts > 0 {
                has_error = true;
                lines.push(format!("profile conflicts: {conflicts}"));
            } else {
                lines.push("profile conflicts: none".to_string());
            }
            json_fields.push(format!(r#""profile_conflicts":{conflicts}"#));
        }
        Err(err) => {
            has_error = true;
            lines.push(format!("load: failed ({err})"));
            json_fields.push(format!(r#""load_error":{}"#, json_escape(&err.to_string())));
        }
    }

    let st = service::service_status(root);
    lines.push(format!(
        "service: installed={} running={} ({})",
        st.installed, st.running, st.detail
    ));
    json_fields.push(format!(
        r#""service_installed":{},"service_running":{}"#,
        st.installed, st.running
    ));

    match git_detect_renames(root) {
        Ok(Some(renames)) if renames.is_empty() => {
            lines.push("git renames: none".to_string());
            json_fields.push(r#""git_renames_pending":0"#.to_string());
        }
        Ok(Some(renames)) => {
            has_error = true;
            lines.push(format!("git renames pending: {}", renames.len()));
            json_fields.push(format!(r#""git_renames_pending":{}"#, renames.len()));
        }
        Ok(None) => {
            lines.push("git renames: unavailable".to_string());
            json_fields.push(r#""git_renames_pending":null"#.to_string());
        }
        Err(err) => {
            lines.push(format!("git renames: unavailable ({})", err.message()));
            json_fields.push(r#""git_renames_pending":null"#.to_string());
        }
    }

    json_fields.push(format!(
        r#""ok":{}"#,
        if has_error { "false" } else { "true" }
    ));

    Ok(DoctorReport {
        text: lines.join("\n"),
        json: format!("{{{}}}", json_fields.join(",")),
        has_error,
    })
}
