use crate::model::{Diagnostic, Severity};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub fn current_okf_version() -> &'static str {
    "0.2"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OkfComplianceMode {
    #[default]
    Strict,
    Standard,
}

impl OkfComplianceMode {
    #[allow(non_upper_case_globals)]
    pub const Level3: Self = Self::Strict;
    #[allow(non_upper_case_globals)]
    pub const Level1: Self = Self::Standard;
}

pub type OkfLintLevel = OkfComplianceMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OkfStatus {
    Draft,
    Stable,
    Deprecated,
}

impl OkfStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            OkfStatus::Draft => "draft",
            OkfStatus::Stable => "stable",
            OkfStatus::Deprecated => "deprecated",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "draft" => Some(OkfStatus::Draft),
            "stable" => Some(OkfStatus::Stable),
            "deprecated" => Some(OkfStatus::Deprecated),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OkfTrustTier {
    Unverified,
    MachineConfirmed,
    HumanReviewed,
}

impl OkfTrustTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            OkfTrustTier::Unverified => "unverified",
            OkfTrustTier::MachineConfirmed => "machine-confirmed",
            OkfTrustTier::HumanReviewed => "human-reviewed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActorEvent {
    pub by: String,
    pub at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateRange {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfSource {
    pub id: Option<String>,
    pub resource: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub usage_count: Option<i64>,
    pub last_modified: Option<String>,
    pub usage_window: Option<DateRange>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OkfParameter {
    pub name: String,
    pub type_name: Option<String>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResourceRefFields {
    pub resource: Option<String>,
    pub receipt: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OkfFrontmatter {
    pub type_name: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub resource: Option<String>,
    pub tags: Vec<String>,
    pub sources: Vec<OkfSource>,
    pub usage_window: Option<DateRange>,
    pub generated: Option<ActorEvent>,
    pub verified: Vec<ActorEvent>,
    pub status: Option<OkfStatus>,
    pub stale_after: Option<String>,
    pub runtime: Option<String>,
    pub parameters: Vec<OkfParameter>,
    pub computation: Option<String>,
    pub executor: ResourceRefFields,
    pub attester: ResourceRefFields,
    /// Legacy OKF v0.1
    pub timestamp: Option<String>,
    /// Producer extensions — preserved, not rejected
    pub unknown: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum OkfFrontmatterState {
    Parsed(OkfFrontmatter),
    Invalid(String),
    Absent,
}

#[derive(Debug, Clone)]
pub struct OkfDocument {
    pub path: PathBuf,
    pub concept_id: String,
    pub body: String,
    pub frontmatter: OkfFrontmatterState,
    pub is_reserved: bool,
}

#[derive(Debug, Clone)]
pub struct OkfBundle {
    pub root: PathBuf,
    pub okf_version: Option<String>,
    pub documents: Vec<OkfDocument>,
}

pub fn derive_trust_tier(verified: &[ActorEvent]) -> OkfTrustTier {
    if verified.is_empty() {
        return OkfTrustTier::Unverified;
    }
    if verified.iter().any(|v| v.by.starts_with("human:")) {
        OkfTrustTier::HumanReviewed
    } else {
        OkfTrustTier::MachineConfirmed
    }
}

pub fn concept_id_for_path(root: &std::path::Path, path: &std::path::Path) -> String {
    let rel = path.strip_prefix(root).unwrap_or(path);
    let s = rel.to_string_lossy().replace('\\', "/");
    s.strip_suffix(".md").unwrap_or(&s).to_string()
}

pub(crate) fn diag(path: PathBuf, severity: Severity, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        path,
        severity,
        message: message.into(),
    }
}
