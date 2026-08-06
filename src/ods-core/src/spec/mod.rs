//! Spec key schemas and registry (single source: `schema.rs`).
//!
//! `SpecSchemaRegistry` is the authoritative key catalog for ODS, OKF, and
//! Skills. Lint enum/placement checks and `ods schema` JSON emission are driven
//! from the registry so new dialects/keys land in one place.

pub mod schema;

pub use schema::{
    KeyDefinition, KeyPlacement, KeyType, SchemaIssue, SpecKind, SpecSchema, SpecSchemaRegistry,
    evaluate_document_key_query, evaluate_single_key_clause, filter_documents_by_keys,
    generate_ods_json_schema, get_document_key_values, validate_ods_frontmatter,
};
