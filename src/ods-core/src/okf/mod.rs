//! Native Google OKF v0.2 support (parse, lint, audit, init).
//! Runtime executor/attester execution is intentionally out of scope.

mod audit;
mod bundle;
mod index;
mod init;
mod lint;
mod model;
mod parse;

pub use audit::{
    OkfAuditClass, OkfAuditItem, OkfAuditReport, audit_okf_bundle, render_okf_audit_markdown,
};
pub use bundle::{load_okf_bundle, okf_enabled, okf_version_from_root};
pub use index::{
    export_okf_graph, fmt_okf_bundle, generate_okf_indexes, okf_context, okf_indexes_are_current,
};
pub use init::{OkfInitOptions, OkfInitReport, init_okf_bundle};
pub use lint::{lint_okf_bundle, lint_okf_bundle_with_config, lint_okf_bundle_with_level};
pub use model::{
    ActorEvent, DateRange, OkfBundle, OkfDocument, OkfFrontmatter, OkfFrontmatterState,
    OkfLintLevel, OkfParameter, OkfSource, OkfStatus, OkfTrustTier, ResourceRefFields,
    concept_id_for_path, current_okf_version, derive_trust_tier,
};
pub use parse::parse_okf_frontmatter_block;
