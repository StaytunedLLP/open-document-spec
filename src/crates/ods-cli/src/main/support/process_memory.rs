fn print_path_change_report(
    root: &Path,
    from: &str,
    to: &str,
    report: &ods_core::PathChangeReport,
    format: OutputFormat,
    verb: &str,
) {
    match format {
        OutputFormat::Text => {
            if verb == "moved" {
                println!("{verb} {from} → {to} ({})", report.summary());
            } else {
                println!("{verb} ({})", report.summary());
            }
            for path in &report.rewritten_files {
                if let Ok(rel) = path.strip_prefix(root) {
                    println!("  rewrote {}", rel.display());
                } else {
                    println!("  rewrote {}", path.display());
                }
            }
            for w in &report.warnings {
                eprintln!("warning: {w}");
            }
            for e in &report.errors {
                eprintln!("error: {e}");
            }
        }
        OutputFormat::Json => {
            let rewritten: Vec<_> = report
                .rewritten_files
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            let warnings: Vec<_> = report.warnings.iter().map(|w| json_escape(w)).collect();
            let errors: Vec<_> = report.errors.iter().map(|e| json_escape(e)).collect();
            println!(
                r#"{{"from":{},"to":{},"rewritten":[{}],"indexes":{},"moves":{},"warnings":[{}],"errors":[{}]}}"#,
                json_escape(from),
                json_escape(to),
                rewritten.join(","),
                report.indexes.len(),
                report.moves.len(),
                warnings.join(","),
                errors.join(",")
            );
        }
    }
}

fn print_aliases(workspace: &ods_core::Workspace) {
    let aliases = workspace_aliases(workspace);
    if aliases.is_empty() {
        return;
    }

    println!("workspace aliases:");
    for (canonical, values) in aliases {
        println!(
            "  {canonical}: {}",
            values.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
}

fn print_alias_suggestions(workspace: &ods_core::Workspace) {
    let suggestions = workspace_alias_suggestions(workspace);
    if suggestions.is_empty() {
        return;
    }

    println!("alias suggestions:");
    for (canonical, values) in suggestions {
        println!(
            "  {canonical}: {}",
            values.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
}
fn print_tags(workspace: &ods_core::Workspace, include_all: bool, format: OutputFormat) {
    let rows = tag_usage_with_builtins(workspace, include_all);
    match format {
        OutputFormat::Text => {
            if rows.is_empty() {
                println!("(no project tags)");
                return;
            }
            let width = rows
                .iter()
                .map(|(t, _, _)| t.len())
                .max()
                .unwrap_or(4)
                .max(4);
            for (tag, count, is_default_unused) in &rows {
                if *is_default_unused {
                    println!("{tag:<width$}  {count}  (default ODS, unused)");
                } else {
                    println!("{tag:<width$}  {count}");
                }
            }
        }
        OutputFormat::Json => {
            let items: Vec<_> = rows
                .iter()
                .map(|(tag, count, is_default)| {
                    format!(
                        r#"{{"tag":{},"count":{},"default_unused":{}}}"#,
                        json_escape(tag),
                        count,
                        if *is_default { "true" } else { "false" }
                    )
                })
                .collect();
            println!(r#"{{"tags":[{}]}}"#, items.join(","));
        }
    }
}
