//! Mutation helpers: adopt, path changes (mv), rename observation.

mod adopt;

pub use adopt::*;
// Re-export sibling engines for a single mutation-facing namespace.
pub use crate::mv::*;
pub use crate::observe::*;
