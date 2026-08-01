use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecKind {
    Ods,
    Okf,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPlacement {
    TopLevel,
    NestedEngineMap,
    RootIndexOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    String,
    List,
    Map,
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
}

#[derive(Debug, Clone)]
pub struct SpecSchema {
    pub kind: SpecKind,
    pub version: String,
    pub keys: HashMap<String, KeyDefinition>,
}

impl SpecSchema {
    pub fn new(kind: SpecKind, version: impl Into<String>) -> Self {
        Self {
            kind,
            version: version.into(),
            keys: HashMap::new(),
        }
    }

    pub fn add_key(&mut self, def: KeyDefinition) {
        self.keys.insert(def.name.clone(), def);
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
        reg
    }

    pub fn register_ods_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Ods, "0.1");

        let ods_keys = [
            (
                "profile",
                KeyPlacement::NestedEngineMap,
                KeyType::String,
                false,
                "Document profile type",
            ),
            (
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
            (
                "id",
                KeyPlacement::NestedEngineMap,
                KeyType::String,
                false,
                "Explicit document identifier",
            ),
            (
                "share",
                KeyPlacement::NestedEngineMap,
                KeyType::Enum(vec!["public".into(), "org".into(), "private".into()]),
                false,
                "Document visibility level",
            ),
            (
                "depends",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Required dependency IDs",
            ),
            (
                "related",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Related document IDs",
            ),
            (
                "code",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Code references",
            ),
            (
                "resources",
                KeyPlacement::NestedEngineMap,
                KeyType::List,
                false,
                "Resource file references",
            ),
            (
                "context",
                KeyPlacement::NestedEngineMap,
                KeyType::Map,
                false,
                "AI reading scope configuration",
            ),
        ];

        for (name, placement, key_type, required, desc) in ods_keys {
            schema.add_key(KeyDefinition {
                name: name.into(),
                placement,
                key_type,
                required,
                description: desc.into(),
            });
        }

        let root_keys = [
            (
                "custom-profiles",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Workspace custom profile schema definitions",
            ),
            (
                "profiles",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Legacy profile catalog roots",
            ),
            (
                "packs",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Imported ODS packs",
            ),
            (
                "ignore",
                KeyPlacement::RootIndexOnly,
                KeyType::List,
                false,
                "Ignore path prefixes",
            ),
            (
                "aliases",
                KeyPlacement::RootIndexOnly,
                KeyType::Map,
                false,
                "Vocabulary section aliases",
            ),
        ];

        for (name, placement, key_type, required, desc) in root_keys {
            schema.add_key(KeyDefinition {
                name: name.into(),
                placement,
                key_type,
                required,
                description: desc.into(),
            });
        }

        self.schemas.insert("ods".into(), schema);
    }

    pub fn register_okf_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Okf, "0.2");

        let okf_keys = [
            (
                "okf_version",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "Google OKF bundle version",
            ),
            (
                "concept_id",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "OKF concept identifier",
            ),
            (
                "trust_tier",
                KeyPlacement::TopLevel,
                KeyType::Enum(vec![
                    "verified".into(),
                    "provisional".into(),
                    "untrusted".into(),
                ]),
                false,
                "Trust level",
            ),
            (
                "attested",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Attested computation block",
            ),
            (
                "date_range",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Temporal applicability range",
            ),
        ];

        for (name, placement, key_type, required, desc) in okf_keys {
            schema.add_key(KeyDefinition {
                name: name.into(),
                placement,
                key_type,
                required,
                description: desc.into(),
            });
        }

        self.schemas.insert("okf".into(), schema);
    }

    pub fn register_custom_profile(&mut self, name: &str, expected_keys: &[String]) {
        let mut schema = SpecSchema::new(SpecKind::Custom(name.into()), "1.0");

        for key_name in expected_keys {
            schema.add_key(KeyDefinition {
                name: key_name.clone(),
                placement: KeyPlacement::TopLevel,
                key_type: KeyType::String,
                required: true,
                description: format!("Required custom profile domain key '{key_name}'"),
            });
        }

        self.schemas.insert(name.into(), schema);
    }

    pub fn get(&self, name: &str) -> Option<&SpecSchema> {
        self.schemas.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_registry_defaults() {
        let registry = SpecSchemaRegistry::with_defaults();
        let ods = registry.get("ods").expect("ods schema registered");
        assert!(ods.keys.contains_key("profile"));
        assert!(ods.keys.contains_key("custom-profiles"));

        let okf = registry.get("okf").expect("okf schema registered");
        assert!(okf.keys.contains_key("okf_version"));
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
}
