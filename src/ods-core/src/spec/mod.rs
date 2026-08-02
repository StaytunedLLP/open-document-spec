//! Spec key schemas and registry (single source: `schema.rs`).
//!
//! Dead parallel `SpecDescriptor` / `SpecKeyProcessor` tables were removed —
//! parse/lint do not use them; use `SpecSchemaRegistry` for schema-driven needs.

pub mod schema;

pub use schema::{KeyDefinition, KeyPlacement, KeyType, SpecKind, SpecSchema, SpecSchemaRegistry};
