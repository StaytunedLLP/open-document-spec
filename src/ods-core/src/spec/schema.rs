//! Declarative spec key schemas — single source of truth for dialect keys.
//!
//! Parse keeps a typed `Frontmatter` model; lint / `ods schema` / dialect
//! registration consult this registry so adding or updating keys is a schema
//! change rather than N hardcoded match arms.

use crate::model::{Diagnostic, Frontmatter, Severity};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecKind {
    Ods,
    Okf,
    Skills,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPlacement {
    /// Universal / domain keys at YAML top level (e.g. tags, description, OKF type).
    TopLevel,
    /// ODS engine keys nested under the `ods:` map.
    NestedEngineMap,
    /// Only valid on the workspace root index.
    RootIndexOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    String,
    List,
    Map,
    /// String or list of strings (e.g. owner).
    StringOrList,
    Enum(Vec<String>),
    Timestamp,
}

#[derive(Debug, Clone)]
pub struct KeyDefinition {
    pub name: String,
    pub placement: KeyPlacement,
    pub key_type: KeyType,
    pub required: bool,
    pub description: String,
    /// Alternate spellings accepted by parsers (e.g. created_at → created).
    pub aliases: Vec<String>,
}

impl KeyDefinition {
    fn new(
        name: impl Into<String>,
        placement: KeyPlacement,
        key_type: KeyType,
        required: bool,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            placement,
            key_type,
            required,
            description: description.into(),
            aliases: Vec::new(),
        }
    }

    fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| (*s).to_string()).collect();
        self
    }
}

#[derive(Debug, Clone)]
pub struct SpecSchema {
    pub kind: SpecKind,
    pub version: String,
    pub keys: HashMap<String, KeyDefinition>,
    /// When true, unknown keys are preserved (ODS / OKF / Skills default).
    pub preserve_unknown: bool,
}

impl SpecSchema {
    pub fn new(kind: SpecKind, version: impl Into<String>) -> Self {
        Self {
            kind,
            version: version.into(),
            keys: HashMap::new(),
            preserve_unknown: true,
        }
    }

    pub fn add_key(&mut self, def: KeyDefinition) {
        self.keys.insert(def.name.clone(), def);
    }

    pub fn key_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.keys.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    pub fn keys_with_placement(&self, placement: KeyPlacement) -> Vec<&KeyDefinition> {
        let mut out: Vec<&KeyDefinition> = self
            .keys
            .values()
            .filter(|k| k.placement == placement)
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpecSchemaRegistry {
    schemas: HashMap<String, SpecSchema>,
}

impl SpecSchemaRegistry {
    pub fn with_defaults() -> Self {
        let mut reg = Self::default();
        reg.register_ods_schema();
        reg.register_okf_schema();
        reg.register_skills_schema();
        reg
    }

    pub fn register_ods_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Ods, "0.1");

        // Universal top-level keys
        for def in [
            KeyDefinition::new(
                "description",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "One-line summary for indexes and previews",
            ),
            KeyDefinition::new(
                "tags",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Free-form taxonomy tags (never under ods:)",
            ),
            KeyDefinition::new(
                "owner",
                KeyPlacement::TopLevel,
                KeyType::StringOrList,
                false,
                "Responsible person or team",
            ),
            KeyDefinition::new(
                "created",
                KeyPlacement::TopLevel,
                KeyType::Timestamp,
                false,
                "Optional created timestamp",
            )
            .with_aliases(&["created_at", "date"]),
            KeyDefinition::new(
                "updated",
                KeyPlacement::TopLevel,
                KeyType::Timestamp,
                false,
                "Optional updated timestamp",
            )
            .with_aliases(&["last_updated", "updated_at"]),
        ] {
            schema.add_key(def);
        }

        // Engine keys under ods:
        for def in [
            KeyDefinition::new(
                "profile",
                KeyPlacement::NestedEngineMap,
                KeyType::String,
                false,
                "Document profile type",
            ),
            KeyDefinition::new(
                "status",
                KeyPlacement::NestedEngineMap,
                KeyType::Enum(vec![
                    "draft".into(),
                    "stable".into(),
                    "deprecated".into(),
                    "archived".into(),
                ]),
                false,
                "Document lifecycle status",
            ),
            KeyDefinition::new(
                "id",
                KeyPlacement::NestedEngineMap,
                KeyType::String,
                false,
                "Explicit document identifier",
            ),
            KeyDefinition::new(
                "share",
                KeyPlacement::NestedEngineMap,
                KeyType::Enum(vec!["public".into(), "org".into(), "private".into()]),
                false,
                "Document visibility level",
            ),
            KeyDefinition::new(
                "depends",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Required dependency IDs",
            ),
            KeyDefinition::new(
                "related",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Related document IDs",
            ),
            KeyDefinition::new(
                "code",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Code references",
            ),
            KeyDefinition::new(
                "resources",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Resource file references",
            ),
            KeyDefinition::new(
                "context",
                KeyPlacement::NestedEngineMap,
                KeyType::Map,
                false,
                "AI reading scope configuration",
            ),
        ] {
            schema.add_key(def);
        }

        // Root index only
        for def in [
            KeyDefinition::new(
                "ods",
                KeyPlacement::RootIndexOnly,
                KeyType::String,
                false,
                "ODS spec version marker on root index",
            ),
            KeyDefinition::new(
                "custom-profiles",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Workspace custom profile schema definitions",
            ),
            KeyDefinition::new(
                "profiles",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Legacy profile catalog roots",
            ),
            KeyDefinition::new(
                "packs",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Imported ODS packs",
            ),
            KeyDefinition::new(
                "ignore",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Ignore path prefixes",
            ),
            KeyDefinition::new(
                "aliases",
                KeyPlacement::RootIndexOnly,
                KeyType::Map,
                false,
                "Vocabulary section aliases",
            ),
            KeyDefinition::new(
                "specs",
                KeyPlacement::RootIndexOnly,
                KeyType::Map,
                false,
                "Multi-spec activation and lint config",
            ),
            KeyDefinition::new(
                "name",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Custom profile definition name",
            ),
            KeyDefinition::new(
                "expected_keys",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Required domain keys for a custom profile",
            )
            .with_aliases(&["expected-keys"]),
        ] {
            schema.add_key(def);
        }

        self.schemas.insert("ods".into(), schema);
    }

    pub fn register_okf_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Okf, "0.2");

        for def in [
            KeyDefinition::new(
                "okf_version",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "Google OKF bundle version",
            ),
            KeyDefinition::new(
                "type",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "Concept kind (routes/filters)",
            ),
            KeyDefinition::new(
                "title",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Display title",
            ),
            KeyDefinition::new(
                "description",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Short summary",
            ),
            KeyDefinition::new(
                "tags",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Free-form facets",
            ),
            KeyDefinition::new(
                "resource",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Canonical URI of underlying asset",
            ),
            KeyDefinition::new(
                "concept_id",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "OKF concept identifier",
            ),
            KeyDefinition::new(
                "sources",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Provenance sources",
            ),
            KeyDefinition::new(
                "generated",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Who/what produced the content",
            ),
            KeyDefinition::new(
                "verified",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Independent confirmations",
            ),
            KeyDefinition::new(
                "status",
                KeyPlacement::TopLevel,
                KeyType::Enum(vec!["draft".into(), "stable".into(), "deprecated".into()]),
                false,
                "OKF lifecycle (top-level)",
            ),
            KeyDefinition::new(
                "stale_after",
                KeyPlacement::TopLevel,
                KeyType::Timestamp,
                false,
                "Absolute freshness deadline",
            ),
            KeyDefinition::new(
                "trust_tier",
                KeyPlacement::TopLevel,
                KeyType::Enum(vec![
                    "verified".into(),
                    "provisional".into(),
                    "untrusted".into(),
                ]),
                false,
                "Derived trust level",
            ),
            KeyDefinition::new(
                "attested",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Attested computation block",
            ),
            KeyDefinition::new(
                "date_range",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Temporal applicability range",
            ),
            KeyDefinition::new(
                "runtime",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Attested computation runtime",
            ),
            KeyDefinition::new(
                "parameters",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Named typed parameters",
            ),
            KeyDefinition::new(
                "computation",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Path to computation file",
            ),
            KeyDefinition::new(
                "executor",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "How to run + receipt shape",
            ),
            KeyDefinition::new(
                "attester",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Deterministic check of a run receipt",
            ),
        ] {
            schema.add_key(def);
        }

        self.schemas.insert("okf".into(), schema);
    }

    pub fn register_skills_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Skills, "1.0");

        for def in [
            KeyDefinition::new(
                "name",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "Skill identity (must match parent directory)",
            ),
            KeyDefinition::new(
                "description",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "What the skill does and when to use it",
            ),
            KeyDefinition::new(
                "license",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "License name or path",
            ),
            KeyDefinition::new(
                "compatibility",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Environment requirements",
            ),
            KeyDefinition::new(
                "metadata",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Extra structured labels",
            ),
            KeyDefinition::new(
                "allowed-tools",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Experimental pre-approved tools list",
            ),
        ] {
            schema.add_key(def);
        }

        self.schemas.insert("skills".into(), schema);
    }

    pub fn register_custom_profile(&mut self, name: &str, expected_keys: &[String]) {
        let mut schema = SpecSchema::new(SpecKind::Custom(name.into()), "1.0");

        for key_name in expected_keys {
            schema.add_key(KeyDefinition::new(
                key_name.clone(),
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                format!("Required custom profile domain key '{key_name}'"),
            ));
        }

        self.schemas.insert(name.into(), schema);
    }

    pub fn get(&self, name: &str) -> Option<&SpecSchema> {
        self.schemas.get(name)
    }

    pub fn dialect_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.schemas.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

/// Lightweight schema issue before conversion to workspace `Diagnostic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIssue {
    pub severity: Severity,
    pub message: String,
}

impl SchemaIssue {
    pub fn to_diagnostic(self, path: PathBuf) -> Diagnostic {
        Diagnostic {
            path,
            severity: self.severity,
            message: self.message,
        }
    }
}

/// Validate known ODS engine enums and placement flags against the registry.
///
/// Graph/profile/pack rules stay outside the schema layer.
pub fn validate_ods_frontmatter(frontmatter: &Frontmatter) -> Vec<SchemaIssue> {
    let registry = SpecSchemaRegistry::with_defaults();
    let Some(schema) = registry.get("ods") else {
        return Vec::new();
    };
    let mut issues = Vec::new();

    if let Some(status) = &frontmatter.status
        && let Some(def) = schema.keys.get("status")
        && let KeyType::Enum(allowed) = &def.key_type
        && !allowed.iter().any(|v| v == status)
    {
        let hint = status_alias_hint(status);
        issues.push(SchemaIssue {
            severity: Severity::Error,
            message: if let Some(h) = hint {
                format!("invalid status: {status} (did you mean `{h}`? allowed: draft|stable|deprecated|archived)")
            } else {
                format!(
                    "invalid status: {status} (allowed: draft|stable|deprecated|archived)"
                )
            },
        });
    }

    if let Some(share) = &frontmatter.share
        && let Some(def) = schema.keys.get("share")
        && let KeyType::Enum(allowed) = &def.key_type
        && !allowed.iter().any(|v| v == share)
    {
        issues.push(SchemaIssue {
            severity: Severity::Error,
            message: format!("invalid share value: {share} (allowed: public|org|private)"),
        });
    }

    // Prefer H1 title; frontmatter title: is preserved but discouraged for ODS purity.
    if frontmatter.title.is_some() {
        issues.push(SchemaIssue {
            severity: Severity::Warning,
            message: "frontmatter `title:` is discouraged for ODS docs — use the first `# H1` as the document title (value is preserved)".into(),
        });
    }

    // tags_misplaced is reported by tags::lint_document_tags (single message).
    issues
}

fn status_alias_hint(status: &str) -> Option<&'static str> {
    match status.to_ascii_lowercase().as_str() {
        "wip" | "in-progress" | "in_progress" | "todo" | "working" | "dev" => Some("draft"),
        "done" | "complete" | "completed" | "ready" | "ga" | "released" => Some("stable"),
        "old" | "obsolete" | "sunset" => Some("deprecated"),
        "archive" | "archived" | "inactive" => Some("archived"),
        _ => None,
    }
}

fn key_type_to_json_schema(key_type: &KeyType, description: &str) -> Value {
    let mut obj = match key_type {
        KeyType::String | KeyType::Timestamp => json!({ "type": "string" }),
        KeyType::List => json!({
            "type": "array",
            "items": { "type": "string" }
        }),
        KeyType::Map => json!({ "type": "object" }),
        KeyType::StringOrList => json!({
            "oneOf": [
                { "type": "string" },
                { "type": "array", "items": { "type": "string" } }
            ]
        }),
        KeyType::Enum(values) => json!({
            "type": "string",
            "enum": values
        }),
    };
    if let Some(map) = obj.as_object_mut() {
        map.insert("description".into(), Value::String(description.into()));
    }
    obj
}

/// Emit draft-07 JSON Schema for the ODS dialect from the registry.
pub fn generate_ods_json_schema() -> String {
    let registry = SpecSchemaRegistry::with_defaults();
    let schema = registry
        .get("ods")
        .expect("ods schema registered in with_defaults");

    let mut top_props = serde_json::Map::new();
    let mut engine_props = serde_json::Map::new();

    for def in schema.keys.values() {
        match def.placement {
            KeyPlacement::TopLevel => {
                top_props.insert(
                    def.name.clone(),
                    key_type_to_json_schema(&def.key_type, &def.description),
                );
            }
            KeyPlacement::NestedEngineMap => {
                engine_props.insert(
                    def.name.clone(),
                    key_type_to_json_schema(&def.key_type, &def.description),
                );
            }
            KeyPlacement::RootIndexOnly => {
                if def.name == "ods" {
                    continue; // handled as oneOf below
                }
                top_props.insert(
                    def.name.clone(),
                    key_type_to_json_schema(&def.key_type, &def.description),
                );
            }
        }
    }

    let root = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "$id": "https://opendocspec.org/schemas/v0.1/ods.schema.json",
        "title": "Open Document Spec (ODS) Frontmatter Schema",
        "description": "Frontmatter metadata validation schema for Open Document Spec Markdown files. Universal keys (tags, description, owner) are top-level; engine keys nest under ods:. Generated from SpecSchemaRegistry.",
        "type": "object",
        "properties": {
            "description": top_props.get("description").cloned().unwrap_or(json!({"type":"string"})),
            "tags": top_props.get("tags").cloned().unwrap_or(json!({"type":"array"})),
            "owner": top_props.get("owner").cloned().unwrap_or(json!({"type":"string"})),
            "created": top_props.get("created").cloned().unwrap_or(json!({"type":"string"})),
            "updated": top_props.get("updated").cloned().unwrap_or(json!({"type":"string"})),
            "ods": {
                "oneOf": [
                    {
                        "type": "string",
                        "description": "Root index only: ODS spec version marker (e.g. '0.1')."
                    },
                    {
                        "type": "object",
                        "description": "ODS engine metadata map. Do not put tags here — tags are top-level only.",
                        "properties": engine_props,
                        "additionalProperties": false
                    }
                ]
            },
            "custom-profiles": top_props.get("custom-profiles").cloned().unwrap_or(json!({"type":"array"})),
            "profiles": top_props.get("profiles").cloned().unwrap_or(json!({"type":"array"})),
            "packs": top_props.get("packs").cloned().unwrap_or(json!({"type":"array"})),
            "ignore": top_props.get("ignore").cloned().unwrap_or(json!({"type":"array"})),
            "aliases": top_props.get("aliases").cloned().unwrap_or(json!({"type":"object"})),
            "specs": top_props.get("specs").cloned().unwrap_or(json!({"type":"object"}))
        },
        "additionalProperties": true
    });

    serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Frontmatter;

    #[test]
    fn test_schema_registry_defaults() {
        let registry = SpecSchemaRegistry::with_defaults();
        let ods = registry.get("ods").expect("ods schema registered");
        assert!(ods.keys.contains_key("profile"));
        assert!(ods.keys.contains_key("custom-profiles"));
        assert!(ods.keys.contains_key("tags"));
        assert!(ods.keys.contains_key("description"));
        assert!(ods.keys.contains_key("specs"));

        let okf = registry.get("okf").expect("okf schema registered");
        assert!(okf.keys.contains_key("okf_version"));
        assert!(okf.keys.contains_key("type"));
        assert!(okf.keys.contains_key("sources"));

        let skills = registry.get("skills").expect("skills schema registered");
        assert!(skills.keys.contains_key("name"));
        assert!(skills.keys.contains_key("description"));
        assert!(skills.keys.contains_key("allowed-tools"));
    }

    #[test]
    fn test_custom_profile_registration() {
        let mut registry = SpecSchemaRegistry::with_defaults();
        registry
            .register_custom_profile("api_endpoint", &["endpoint_url".into(), "service".into()]);

        let custom = registry
            .get("api_endpoint")
            .expect("custom schema registered");
        assert!(custom.keys.contains_key("endpoint_url"));
        assert!(custom.keys.contains_key("service"));
    }

    #[test]
    fn validate_rejects_invalid_status_and_share() {
        let fm = Frontmatter {
            status: Some("nope".into()),
            share: Some("secret".into()),
            ..Default::default()
        };
        let issues = validate_ods_frontmatter(&fm);
        assert!(
            issues.iter().any(|i| i.message.contains("invalid status")),
            "{issues:?}"
        );
        assert!(
            issues.iter().any(|i| i.message.contains("invalid share")),
            "{issues:?}"
        );
    }

    #[test]
    fn generate_json_schema_contains_engine_and_universal_keys() {
        let raw = generate_ods_json_schema();
        let v: Value = serde_json::from_str(&raw).expect("valid json");
        let props = v.get("properties").expect("properties");
        assert!(props.get("tags").is_some());
        assert!(props.get("description").is_some());
        assert!(props.get("ods").is_some());
        assert!(props.get("custom-profiles").is_some());
        assert!(raw.contains("SpecSchemaRegistry") || raw.contains("Open Document Spec"));
    }

    #[test]
    fn ods_engine_keys_match_canonical_set() {
        let registry = SpecSchemaRegistry::with_defaults();
        let ods = registry.get("ods").unwrap();
        for key in [
            "profile",
            "status",
            "id",
            "share",
            "depends",
            "related",
            "code",
            "resources",
            "context",
        ] {
            let def = ods.keys.get(key).unwrap_or_else(|| panic!("missing {key}"));
            assert_eq!(def.placement, KeyPlacement::NestedEngineMap, "{key}");
        }
    }
}
