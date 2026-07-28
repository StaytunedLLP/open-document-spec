// Benchmarking and Frontmatter Snapshot/Restore System for ODS.
// Allows users to take machine-level JSON snapshots of frontmatters,
// strip frontmatters, index lockfiles, profiles, and error artifacts across all Markdown files
// to test LLM/AI workflows without ODS, restore frontmatters and workspace artifacts back seamlessly,
// and calculate context token/cost savings.

use crate::fs::load_workspace;
use crate::parse::split_frontmatter;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
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

/// Compute a stable hash identifier for a repository root path.
pub fn compute_repo_hash(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let path_str = canonical.to_string_lossy();
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in path_str.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Resolve directory path for machine-level snapshots: ~/.ods/backups/<repo_hash>/
pub fn get_backup_dir(root: &Path) -> io::Result<PathBuf> {
    let repo_hash = compute_repo_hash(root);
    let base = if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".ods").join("backups")
    } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
        PathBuf::from(userprofile).join(".ods").join("backups")
    } else {
        std::env::temp_dir().join("ods_backups")
    };
    let dir = base.join(repo_hash);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Strip frontmatter across Markdown documents in workspace while generating a JSON snapshot backup.
pub fn bench_strip_workspace(
    root: &Path,
    options: BenchStripOptions,
) -> io::Result<BenchStripReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let workspace = load_workspace(&root)?;

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let snapshot_id = format!("snapshot_{timestamp}");
    let backup_dir = get_backup_dir(&root)?;
    let snapshot_path = backup_dir.join(format!("{snapshot_id}.json"));

    let do_strip_indexes = options.full || options.strip_indexes;
    let do_strip_profiles = options.full || options.strip_profiles;
    let do_strip_error = options.full;

    let mut snapshot_files: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut files_to_strip: Vec<(PathBuf, String)> = Vec::new();
    let mut deleted_indexes: BTreeMap<String, String> = BTreeMap::new();
    let mut indexes_to_delete: Vec<PathBuf> = Vec::new();
    let mut profile_files: BTreeMap<String, String> = BTreeMap::new();
    let mut profiles_to_delete: Vec<PathBuf> = Vec::new();
    let mut error_file_content: Option<String> = None;
    let mut error_file_to_delete: Option<PathBuf> = None;

    let mut total_processed = 0;

    for doc in &workspace.documents {
        let path = &doc.path;
        if let Some(ref filter) = options.path_filter {
            if !path.starts_with(filter) {
                continue;
            }
        }
        total_processed += 1;

        let relative_path = path
            .strip_prefix(&root)
            .unwrap_or(path)
            .to_string_lossy()
            .replace('\\', "/");

        // Check if non-root index.md
        if do_strip_indexes && relative_path != "index.md" && relative_path.ends_with("index.md") {
            if let Ok(content) = fs::read_to_string(path) {
                deleted_indexes.insert(relative_path.clone(), content);
                indexes_to_delete.push(path.clone());
            }
            continue;
        }

        let text = fs::read_to_string(path)?;
        let (frontmatter_block, body) = split_frontmatter(&text);

        if let Some(fm) = frontmatter_block {
            snapshot_files.insert(relative_path.clone(), Some(fm.to_string()));
            files_to_strip.push((path.clone(), body.to_string()));
        } else {
            snapshot_files.insert(relative_path, None);
        }
    }

    // Strip custom profiles in ods-profiles/
    if do_strip_profiles {
        let ods_profiles_dir = root.join("ods-profiles");
        if ods_profiles_dir.is_dir() {
            if let Ok(entries) = fs::read_dir(&ods_profiles_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        let rel = path
                            .strip_prefix(&root)
                            .unwrap_or(&path)
                            .to_string_lossy()
                            .replace('\\', "/");
                        if let Ok(content) = fs::read_to_string(&path) {
                            profile_files.insert(rel, content);
                            profiles_to_delete.push(path);
                        }
                    }
                }
            }
        }
    }

    // Strip ods-error.md if present
    if do_strip_error {
        let err_path = root.join("ods-error.md");
        if err_path.is_file() {
            if let Ok(content) = fs::read_to_string(&err_path) {
                error_file_content = Some(content);
                error_file_to_delete = Some(err_path);
            }
        }
    }

    let total_stripped = files_to_strip.len();
    let total_indexes_deleted = indexes_to_delete.len();
    let total_profiles_removed = profiles_to_delete.len();

    if options.write {
        // Save machine-level JSON snapshot
        let snapshot = Snapshot {
            snapshot_id: snapshot_id.clone(),
            root: root.display().to_string(),
            files: snapshot_files
                .into_iter()
                .map(|(path, frontmatter)| FileEntry { path, frontmatter })
                .collect(),
            deleted_indexes: deleted_indexes
                .into_iter()
                .map(|(path, content)| ContentEntry { path, content })
                .collect(),
            profile_files: profile_files
                .into_iter()
                .map(|(path, content)| ContentEntry { path, content })
                .collect(),
            error_file: error_file_content,
        };
        let json_content = serde_json::to_string_pretty(&snapshot)
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
        fs::write(&snapshot_path, json_content)?;

        // Apply stripped body back to files
        for (file_path, body) in &files_to_strip {
            let clean_body = body.trim_start_matches(['\r', '\n']);
            fs::write(file_path, clean_body)?;
        }

        // Delete non-root index lockfiles
        for path in &indexes_to_delete {
            let _ = fs::remove_file(path);
        }

        // Delete profile files and clean directory
        for path in &profiles_to_delete {
            let _ = fs::remove_file(path);
        }
        let ods_profiles_dir = root.join("ods-profiles");
        let _ = fs::remove_dir(ods_profiles_dir);

        // Delete error file
        if let Some(err_p) = error_file_to_delete {
            let _ = fs::remove_file(err_p);
        }
    }

    Ok(BenchStripReport {
        snapshot_id,
        snapshot_path,
        total_processed,
        total_stripped,
        total_indexes_deleted,
        total_profiles_removed,
        dry_run: !options.write,
    })
}

/// Restore frontmatters, index lockfiles, and workspace artifacts from JSON snapshot backup in ~/.ods/backups/<repo_hash>/.
pub fn bench_restore_workspace(
    root: &Path,
    snapshot_id: Option<&str>,
) -> io::Result<BenchRestoreReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let backup_dir = get_backup_dir(&root)?;

    let target_snapshot_path = match snapshot_id {
        Some(id) => backup_dir.join(if id.ends_with(".json") {
            id.to_string()
        } else {
            format!("{id}.json")
        }),
        None => {
            // Find latest snapshot file
            let mut entries: Vec<PathBuf> = fs::read_dir(&backup_dir)?
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
                .collect();
            entries.sort();
            entries
                .pop()
                .ok_or_else(|| io::Error::other("No ODS benchmark snapshots found to restore"))?
        }
    };

    let content = fs::read_to_string(&target_snapshot_path)?;
    let parsed_data: Snapshot = serde_json::from_str(&content)
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    let snapshot_id_final = if parsed_data.snapshot_id.is_empty() {
        target_snapshot_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "snapshot".to_string())
    } else {
        parsed_data.snapshot_id
    };

    let mut total_restored = 0;
    for entry in parsed_data.files {
        let full_path = root.join(&entry.path);
        if !full_path.exists() {
            continue;
        }

        if let Some(fm) = entry.frontmatter {
            let current_text = fs::read_to_string(&full_path)?;
            let (_, body) = split_frontmatter(&current_text);
            let clean_body = body.trim_start_matches(['\r', '\n']);
            let restored_content = format!("---\n{fm}\n---\n\n{clean_body}");
            fs::write(&full_path, restored_content)?;
            total_restored += 1;
        }
    }

    let mut total_indexes_restored = 0;
    for entry in parsed_data.deleted_indexes {
        let full_path = root.join(&entry.path);
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&full_path, entry.content)?;
        total_indexes_restored += 1;
    }

    let mut total_profiles_restored = 0;
    for entry in parsed_data.profile_files {
        let full_path = root.join(&entry.path);
        if let Some(parent) = full_path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        fs::write(&full_path, entry.content)?;
        total_profiles_restored += 1;
    }

    if let Some(err_content) = parsed_data.error_file {
        let err_path = root.join("ods-error.md");
        let _ = fs::write(err_path, err_content);
    }

    Ok(BenchRestoreReport {
        snapshot_id: snapshot_id_final,
        snapshot_path: target_snapshot_path,
        total_restored,
        total_indexes_restored,
        total_profiles_restored,
    })
}

/// Calculate token & cost ROI statistics for current workspace.
pub fn bench_calculate_stats(root: &Path) -> io::Result<BenchStatsReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let workspace = load_workspace(&root)?;

    let mut total_raw_bytes = 0;
    for doc in &workspace.documents {
        if let Ok(meta) = fs::metadata(&doc.path) {
            total_raw_bytes += meta.len() as usize;
        }
    }

    let estimated_total_tokens = total_raw_bytes / 4;
    let avg_ods_context_tokens = if workspace.documents.is_empty() {
        0
    } else {
        (estimated_total_tokens / workspace.documents.len().max(1)).max(500)
    };

    let reduction_pct = if estimated_total_tokens > 0 {
        ((estimated_total_tokens.saturating_sub(avg_ods_context_tokens)) as f64
            / estimated_total_tokens as f64)
            * 100.0
    } else {
        0.0
    };

    let cost_per_million_tokens = 5.0; // Standard $5.00 / 1M tokens (Claude 3.5 Sonnet / GPT-4o)
    let queries_per_month = 100.0;
    let saved_tokens_per_query =
        estimated_total_tokens.saturating_sub(avg_ods_context_tokens) as f64;
    let est_monthly_cost_savings_usd =
        (saved_tokens_per_query * queries_per_month / 1_000_000.0) * cost_per_million_tokens;

    Ok(BenchStatsReport {
        total_files: workspace.documents.len(),
        total_raw_bytes,
        estimated_total_tokens,
        avg_ods_context_tokens,
        token_reduction_percentage: reduction_pct,
        est_monthly_cost_savings_usd,
    })
}

/// Print a simulated token/cost comparison for a prompt, with and without ODS context.
///
/// This is a local, offline estimate only: it never makes a network call or reads any
/// `*_API_KEY` environment variable. Token counts are a `bytes / 4` heuristic, not a
/// real tokenizer, and costs use a flat placeholder $5/1M-token rate.
pub fn bench_run_simulation(
    root: &Path,
    prompt: &str,
    provider: Option<&str>,
) -> io::Result<BenchRunReport> {
    let stats = bench_calculate_stats(root)?;
    let provider_name = provider.unwrap_or("simulated (no live API call)");

    let raw_cost = (stats.estimated_total_tokens as f64 / 1_000_000.0) * 5.0;
    let ods_cost = (stats.avg_ods_context_tokens as f64 / 1_000_000.0) * 5.0;

    let output_msg = format!(
        "[Simulated estimate — no live LLM API call made] Benchmarked prompt: '{prompt}' across {} repository docs.\n\
         Without ODS Context: ~{} tokens (byte-count heuristic, ~${raw_cost:.4} at a placeholder $5/1M-token rate)\n\
         With ODS Bounded Graph: ~{} tokens (~${ods_cost:.4} at the same placeholder rate)\n\
         Estimated Token Savings: {:.1}%\n\n\
         This is a local estimate only; ods does not currently call OPENAI_API_KEY, ANTHROPIC_API_KEY, or GEMINI_API_KEY.",
        stats.total_files,
        stats.estimated_total_tokens,
        stats.avg_ods_context_tokens,
        stats.token_reduction_percentage
    );

    Ok(BenchRunReport {
        prompt: prompt.to_string(),
        provider: provider_name.to_string(),
        raw_context_tokens: stats.estimated_total_tokens,
        ods_context_tokens: stats.avg_ods_context_tokens,
        token_savings_pct: stats.token_reduction_percentage,
        est_raw_cost_usd: raw_cost,
        est_ods_cost_usd: ods_cost,
        simulated_output: output_msg,
    })
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
