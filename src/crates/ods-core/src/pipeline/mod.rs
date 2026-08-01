//! Functional load/apply stages: discover → parse → index → lint.
//! Free functions only; no Manager/Service types.

pub mod apply;
pub mod discover;
pub mod parse_stage;

pub use apply::{apply_document_removes, apply_document_upserts};
pub use discover::discover_markdown_paths;
pub use parse_stage::{parse_path, parse_paths_parallel, parse_pool_jobs};
