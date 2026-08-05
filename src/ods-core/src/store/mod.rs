//! Incremental frontmatter store for progressive discovery and low-memory serve.
//!
//! Engine model:
//! - **Incremental:** on path add/change/remove, reparse only that file’s frontmatter and patch maps.
//! - **Progressive:** CLI queries (`overview` / `find` / `context`) read this store without dumping bodies.

mod patch;

pub use patch::{DocMeta, StorePatch, WorkspaceStore};
