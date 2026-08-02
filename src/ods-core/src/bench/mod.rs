// Benchmarking and Frontmatter Snapshot/Restore System for ODS.
use crate::fs::load_workspace;
use crate::parse::split_frontmatter;

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BenchStripOptions {
    pub write: bool,
    pub path_filter: Option<PathBuf>,
    pub strip_indexes: bool,
    pub strip_profiles: bool,
    pub full: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchStripReport {
    pub snapshot_id: String,
    pub snapshot_path: PathBuf,
    pub total_processed: usize,
    pub total_stripped: usize,
    pub total_indexes_deleted: usize,
    pub total_profiles_removed: usize,
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchRestoreReport {
    pub snapshot_id: String,
    pub snapshot_path: PathBuf,
    pub total_restored: usize,
    pub total_indexes_restored: usize,
    pub total_profiles_restored: usize,
}

#[derive(Debug, Clone)]
pub struct BenchStatsReport {
    pub total_files: usize,
    pub total_raw_bytes: usize,
    pub estimated_total_tokens: usize,
    pub avg_ods_context_tokens: usize,
    pub token_reduction_percentage: f64,
    pub est_monthly_cost_savings_usd: f64,
}

#[derive(Debug, Clone)]
pub struct BenchRunReport {
    pub prompt: String,
    pub provider: String,
    pub raw_context_tokens: usize,
    pub ods_context_tokens: usize,
    pub token_savings_pct: f64,
    pub est_raw_cost_usd: f64,
    pub est_ods_cost_usd: f64,
    pub simulated_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snapshot {
    #[serde(default)]
    snapshot_id: String,
    #[serde(default)]
    root: String,
    #[serde(default)]
    files: Vec<FileEntry>,
    #[serde(default)]
    deleted_indexes: Vec<ContentEntry>,
    #[serde(default)]
    profile_files: Vec<ContentEntry>,
    #[serde(default)]
    error_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FileEntry {
    path: String,
    frontmatter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentEntry {
    path: String,
    content: String,
}

include!("snapshot_helpers.rs");
include!("engine.rs");
include!("stats.rs");

#[cfg(test)]
include!("tests.rs");
