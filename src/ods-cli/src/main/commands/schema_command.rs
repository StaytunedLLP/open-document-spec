fn run_schema_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut write = false;
    let mut out_path = None;
    let mut dialect = "ods".to_string();

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--write" | "-w" => {
                write = true;
                i += 1;
            }
            "--out" | "-o" => {
                let p = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--out", "`ods schema --out schema.json`")))?;
                out_path = Some(PathBuf::from(p));
                i += 2;
            }
            "--okf" => {
                dialect = "okf".into();
                i += 1;
            }
            "--skills" => {
                dialect = "skills".into();
                i += 1;
            }
            "--spec" => {
                let p = args
                    .get(i + 1)
                    .ok_or_else(|| usage_msg(ods_core::missing_flag_value("--spec", "`ods schema --spec ods`")))?;
                dialect = p.clone();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let schema_json = match dialect.as_str() {
        "ods" => ods_core::generate_ods_json_schema(),
        other => {
            // Future: generate from registry for okf/skills; for now surface keys.
            let registry = ods_core::SpecSchemaRegistry::with_defaults();
            let schema = registry
                .get(other)
                .ok_or_else(|| {
                    usage_msg(ods_core::invalid_choice("--spec", other, "ods|okf|skills"))
                })?;
            let keys: Vec<_> = schema
                .keys
                .values()
                .map(|k| {
                    serde_json::json!({
                        "name": k.name,
                        "placement": format!("{:?}", k.placement),
                        "required": k.required,
                        "description": k.description,
                    })
                })
                .collect();
            serde_json::to_string_pretty(&serde_json::json!({
                "dialect": other,
                "version": schema.version,
                "keys": keys,
            }))
            .map_err(|e| fail_msg(ods_core::io_failed("serialize schema", e)))?
        }
    };

    if write || out_path.is_some() {
        let dest = out_path.unwrap_or_else(|| {
            if dialect == "ods" {
                PathBuf::from(".ods/ods.schema.json")
            } else {
                PathBuf::from(format!(".ods/{dialect}.schema.json"))
            }
        });
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&dest, &schema_json)
            .map_err(|e| fail_msg(ods_core::io_failed("write schema", e)))?;
        println!("wrote JSON Schema to {}", dest.display());
    } else {
        println!("{schema_json}");
    }

    Ok(ExitCode::from(0))
}
