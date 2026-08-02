//! Parallel parse stage: paths → Document values.

use crate::model::Document;
use crate::parse::parse_document_text;
use rayon::prelude::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Resolve parallel job count from `ODC_JOBS` (positive integer) or rayon default.
pub fn parse_pool_jobs() -> Option<usize> {
    std::env::var("ODC_JOBS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
}

/// Read and parse a single Markdown path.
///
/// Even when `include_body` is false, **`index.md` bodies are retained** so
/// index child-list lint and generators stay correct without holding every
/// note body in RAM.
pub fn parse_path(root: &Path, path: PathBuf, include_body: bool) -> io::Result<Document> {
    let text = fs::read_to_string(&path)?;
    let keep_body = include_body
        || path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case("index.md"));
    Ok(parse_document_text(root, path, &text, keep_body))
}

/// Parse many paths in parallel (order-preserving). Honors `ODC_JOBS` when set.
pub fn parse_paths_parallel(
    root: &Path,
    paths: &[PathBuf],
    include_body: bool,
) -> io::Result<Vec<Document>> {
    let root = root.to_path_buf();
    let run = || {
        paths
            .par_iter()
            .map(|path| parse_path(&root, path.clone(), include_body))
            .collect::<Result<Vec<_>, _>>()
    };

    match parse_pool_jobs() {
        Some(n) => {
            let pool = rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build()
                .map_err(|e| io::Error::other(e.to_string()))?;
            pool.install(run)
        }
        None => run(),
    }
}
