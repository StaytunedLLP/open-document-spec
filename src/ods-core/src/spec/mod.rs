//! Spec key schemas and registry (single source: `schema.rs`).
//!
//! `SpecSchemaRegistry` is the authoritative key catalog for ODS, OKF, and
//! Skills. Lint enum/placement checks and `ods schema` JSON emission are driven
//! from the registry so new dialects/keys land in one place.

pub mod schema;

pub use schema::{
    KeyDefinition, KeyPlacement, KeyType, SchemaIssue, SpecKind, SpecSchema, SpecSchemaRegistry,
    generate_ods_json_schema, validate_ods_frontmatter,
};
