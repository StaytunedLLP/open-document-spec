fn run_schema_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut write = false;
    let mut out_path = None;

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
                    .ok_or_else(|| usage("missing value for --out"))?;
                out_path = Some(PathBuf::from(p));
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }

    let schema_json = generate_ods_json_schema();

    if write || out_path.is_some() {
        let dest = out_path.unwrap_or_else(|| PathBuf::from(".ods/ods.schema.json"));
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&dest, &schema_json).map_err(|e| failure(format!("write {}: {e}", dest.display())))?;
        println!("wrote JSON Schema to {}", dest.display());
    } else {
        println!("{}", schema_json);
    }

    Ok(ExitCode::from(0))
}

fn generate_ods_json_schema() -> String {
    r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://opendocspec.org/schemas/v0.1/ods.schema.json",
  "title": "Open Document Spec (ODS) Frontmatter Schema",
  "description": "Frontmatter metadata validation schema for Open Document Spec Markdown files.",
  "type": "object",
  "required": ["profile"],
  "properties": {
    "ods": {
      "type": "string",
      "description": "Spec version marker (e.g. '0.1'). Required on workspace root index.md."
    },
    "profile": {
      "type": "string",
      "description": "Document profile catalog schema (e.g. index, note, feature, guide, policy, rfc, postmortem)."
    },
    "id": {
      "type": "string",
      "description": "Stable unique document identifier."
    },
    "title": {
      "type": "string",
      "description": "Document title override (optional; H1 heading preferred)."
    },
    "status": {
      "type": "string",
      "enum": ["draft", "stable", "deprecated", "archived"],
      "default": "draft",
      "description": "Lifecycle status of the document."
    },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Taxonomy tags associated with this document."
    },
    "depends": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Explicit prerequisite document dependency IDs."
    },
    "related": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Related document IDs for context expansion."
    },
    "share": {
      "type": "string",
      "enum": ["public", "org", "private"],
      "default": "public",
      "description": "Access control and pack export visibility."
    },
    "custom-profiles": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Custom profile catalog files registered on root index.md."
    },
    "packs": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Exported or imported ODS document pack paths."
    },
    "ignore": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Workspace file or folder ignore patterns."
    }
  },
  "additionalProperties": true
}"#.to_string()
}
