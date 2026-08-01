use std::collections::BTreeMap;

pub mod schema;

pub use schema::{KeyDefinition, KeyPlacement, SpecSchema, SpecSchemaRegistry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecKind {
    Ods,
    Okf,
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Scalar,
    StringList,
    Map,
    ResourceList,
    CodeList,
    ContextSpec,
    ActorEvent,
    DateRange,
    UnknownPassthrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySchema {
    pub name: String,
    pub key_type: KeyType,
    pub aliases: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecDescriptor {
    pub kind: SpecKind,
    pub keys: BTreeMap<String, KeySchema>,
    pub alias_map: BTreeMap<String, String>,
}

impl SpecDescriptor {
    pub fn ods_descriptor() -> Self {
        let mut keys = BTreeMap::new();
        let mut alias_map = BTreeMap::new();

        let schemas = vec![
            KeySchema {
                name: "profile".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Document profile type".into(),
            },
            KeySchema {
                name: "status".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Document lifecycle status".into(),
            },
            KeySchema {
                name: "created".into(),
                key_type: KeyType::Scalar,
                aliases: vec!["created_at".into(), "date".into()],
                description: "Document creation date".into(),
            },
            KeySchema {
                name: "updated".into(),
                key_type: KeyType::Scalar,
                aliases: vec!["last_updated".into(), "updated_at".into()],
                description: "Document last update date".into(),
            },
            KeySchema {
                name: "share".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Share visibility (public/internal/private)".into(),
            },
            KeySchema {
                name: "description".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Short document summary".into(),
            },
            KeySchema {
                name: "id".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Explicit document identifier".into(),
            },
            KeySchema {
                name: "owner".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Document owner or team".into(),
            },
            KeySchema {
                name: "ods".into(),
                key_type: KeyType::Map,
                aliases: vec![],
                description: "Nested ODS engine map".into(),
            },
            KeySchema {
                name: "ods".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "ODS CLI version requirement".into(),
            },
            KeySchema {
                name: "profiles".into(),
                key_type: KeyType::StringList,
                aliases: vec![],
                description: "Custom profile catalog roots".into(),
            },
            KeySchema {
                name: "packs".into(),
                key_type: KeyType::StringList,
                aliases: vec![],
                description: "Imported ODS packs".into(),
            },
            KeySchema {
                name: "depends".into(),
                key_type: KeyType::StringList,
                aliases: vec!["dependencies".into()],
                description: "Prerequisite document dependencies".into(),
            },
            KeySchema {
                name: "related".into(),
                key_type: KeyType::StringList,
                aliases: vec![],
                description: "Related reference documents".into(),
            },
            KeySchema {
                name: "tags".into(),
                key_type: KeyType::StringList,
                aliases: vec![],
                description: "Categorization tags".into(),
            },
            KeySchema {
                name: "ignore".into(),
                key_type: KeyType::StringList,
                aliases: vec![],
                description: "Path prefixes to ignore".into(),
            },
            KeySchema {
                name: "resources".into(),
                key_type: KeyType::ResourceList,
                aliases: vec![],
                description: "Resource file references".into(),
            },
            KeySchema {
                name: "code".into(),
                key_type: KeyType::CodeList,
                aliases: vec![],
                description: "Code implementation references".into(),
            },
            KeySchema {
                name: "context".into(),
                key_type: KeyType::ContextSpec,
                aliases: vec![],
                description: "Context loading specification".into(),
            },
            KeySchema {
                name: "aliases".into(),
                key_type: KeyType::Map,
                aliases: vec![],
                description: "Custom heading section aliases".into(),
            },
        ];

        for s in schemas {
            for alias in &s.aliases {
                alias_map.insert(alias.clone(), s.name.clone());
            }
            keys.insert(s.name.clone(), s);
        }

        Self {
            kind: SpecKind::Ods,
            keys,
            alias_map,
        }
    }

    pub fn okf_descriptor() -> Self {
        let mut keys = BTreeMap::new();
        let mut alias_map = BTreeMap::new();

        let schemas = vec![
            KeySchema {
                name: "type".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "OKF concept type".into(),
            },
            KeySchema {
                name: "title".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "OKF concept title".into(),
            },
            KeySchema {
                name: "description".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "OKF description".into(),
            },
            KeySchema {
                name: "resource".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Target resource".into(),
            },
            KeySchema {
                name: "tags".into(),
                key_type: KeyType::StringList,
                aliases: vec![],
                description: "OKF tags".into(),
            },
            KeySchema {
                name: "sources".into(),
                key_type: KeyType::ResourceList,
                aliases: vec![],
                description: "OKF sources".into(),
            },
            KeySchema {
                name: "usage_window".into(),
                key_type: KeyType::DateRange,
                aliases: vec![],
                description: "OKF usage window".into(),
            },
            KeySchema {
                name: "generated".into(),
                key_type: KeyType::ActorEvent,
                aliases: vec![],
                description: "OKF generation metadata".into(),
            },
            KeySchema {
                name: "verified".into(),
                key_type: KeyType::ActorEvent,
                aliases: vec![],
                description: "OKF verification metadata".into(),
            },
            KeySchema {
                name: "status".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "OKF status".into(),
            },
            KeySchema {
                name: "stale_after".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Staleness threshold".into(),
            },
            KeySchema {
                name: "runtime".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Runtime environment".into(),
            },
            KeySchema {
                name: "parameters".into(),
                key_type: KeyType::Map,
                aliases: vec![],
                description: "OKF parameters".into(),
            },
            KeySchema {
                name: "computation".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "Computation script".into(),
            },
            KeySchema {
                name: "executor".into(),
                key_type: KeyType::Map,
                aliases: vec![],
                description: "OKF executor ref".into(),
            },
            KeySchema {
                name: "attester".into(),
                key_type: KeyType::Map,
                aliases: vec![],
                description: "OKF attester ref".into(),
            },
            KeySchema {
                name: "timestamp".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "OKF timestamp".into(),
            },
            KeySchema {
                name: "okf_version".into(),
                key_type: KeyType::Scalar,
                aliases: vec![],
                description: "OKF version".into(),
            },
        ];

        for s in schemas {
            for alias in &s.aliases {
                alias_map.insert(alias.clone(), s.name.clone());
            }
            keys.insert(s.name.clone(), s);
        }

        Self {
            kind: SpecKind::Okf,
            keys,
            alias_map,
        }
    }

    pub fn canonicalize_key<'a>(&'a self, key: &'a str) -> &'a str {
        if let Some(canonical) = self.alias_map.get(key) {
            canonical.as_str()
        } else {
            key
        }
    }

    pub fn get_schema(&self, key: &str) -> Option<&KeySchema> {
        let canonical = self.canonicalize_key(key);
        self.keys.get(canonical)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtractedKeys {
    pub scalars: BTreeMap<String, String>,
    pub lists: BTreeMap<String, Vec<String>>,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub unknown: BTreeMap<String, String>,
}

pub struct SpecKeyProcessor {
    descriptor: SpecDescriptor,
}

impl SpecKeyProcessor {
    pub fn new(descriptor: SpecDescriptor) -> Self {
        Self { descriptor }
    }

    pub fn descriptor(&self) -> &SpecDescriptor {
        &self.descriptor
    }

    pub fn process_line_scalar(&self, raw_key: &str, raw_val: &str, out: &mut ExtractedKeys) {
        let key = raw_key.trim();
        let val = unquote_scalar(raw_val);
        let canonical = self.descriptor.canonicalize_key(key);

        if let Some(schema) = self.descriptor.get_schema(key) {
            match schema.key_type {
                KeyType::Scalar => {
                    out.scalars.insert(canonical.to_string(), val);
                }
                _ => {
                    out.unknown.insert(key.to_string(), val);
                }
            }
        } else {
            out.unknown.insert(key.to_string(), val);
        }
    }
}

pub fn unquote_scalar(s: &str) -> String {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ods_descriptor_aliases() {
        let desc = SpecDescriptor::ods_descriptor();
        assert_eq!(desc.canonicalize_key("last_updated"), "updated");
        assert_eq!(desc.canonicalize_key("created_at"), "created");
        assert_eq!(desc.canonicalize_key("dependencies"), "depends");
        assert_eq!(desc.canonicalize_key("profile"), "profile");
    }

    #[test]
    fn test_unquote_scalar() {
        assert_eq!(unquote_scalar("\"hello\""), "hello");
        assert_eq!(unquote_scalar("'world'"), "world");
        assert_eq!(unquote_scalar("plain"), "plain");
    }
}
