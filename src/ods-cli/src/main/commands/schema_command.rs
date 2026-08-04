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
    // Universal keys (tags, description, owner, …) are top-level only so any
    // SSG/CMS/tool can read them. Engine keys nest under `ods:` and must not
    // include `tags`.
    r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://opendocspec.org/schemas/v0.1/ods.schema.json",
  "title": "Open Document Spec (ODS) Frontmatter Schema",
  "description": "Frontmatter metadata validation schema for Open Document Spec Markdown files. Universal keys (tags, description, owner) are top-level; engine keys nest under ods:.",
  "type": "object",
  "properties": {
    "description": {
      "type": "string",
      "description": "Single-line summary (universal top-level; SSG meta / indexes)."
    },
    "tags": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Free-form taxonomy tags. MUST be top-level (never under ods:) so Obsidian, Hugo, Docusaurus, Astro, and other tools can read them."
    },
    "owner": {
      "oneOf": [
        { "type": "string" },
        { "type": "array", "items": { "type": "string" } }
      ],
      "description": "Responsible person or team (universal top-level)."
    },
    "title": {
      "type": "string",
      "description": "Document title override (optional; H1 heading preferred)."
    },
    "ods": {
      "oneOf": [
        {
          "type": "string",
          "description": "Root index only: ODS spec version marker (e.g. '0.1')."
        },
        {
          "type": "object",
          "description": "ODS engine metadata map. Do not put tags here — tags are top-level only.",
          "properties": {
            "profile": {
              "type": "string",
              "description": "Document profile (note, feature, guide, api, …)."
            },
            "status": {
              "type": "string",
              "enum": ["draft", "stable", "deprecated", "archived"],
              "default": "draft",
              "description": "Lifecycle status."
            },
            "id": {
              "type": "string",
              "description": "Stable unique document identifier override."
            },
            "share": {
              "type": "string",
              "enum": ["public", "org", "private"],
              "default": "public",
              "description": "Access control and pack export visibility."
            },
            "depends": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Hard dependency document refs."
            },
            "related": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Soft related document refs."
            },
            "resources": {
              "type": "array",
              "description": "Non-Markdown resource refs."
            },
            "code": {
              "type": "array",
              "description": "Code path bindings."
            },
            "context": {
              "type": "object",
              "description": "AI context pack directives."
            }
          },
          "additionalProperties": false
        }
      ]
    },
    "custom-profiles": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Custom profile catalog files registered on root index."
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
