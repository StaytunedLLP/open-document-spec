//! User-facing error and diagnostic message catalog.
//!
//! **Source of truth** for short, directive CLI and engine messages.
//! Prefer builders here over ad-hoc `format!` strings in CLI commands.

mod messages;

pub use messages::*;
