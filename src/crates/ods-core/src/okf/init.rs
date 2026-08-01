use super::model::current_okf_version;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct OkfInitOptions {
    pub write_sample_concept: bool,
    pub write_attested_stub: bool,
    pub write_log: bool,
}

impl Default for OkfInitOptions {
    fn default() -> Self {
        Self {
            write_sample_concept: true,
            write_attested_stub: false,
            write_log: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct OkfInitReport {
    pub root: PathBuf,
    pub created: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
}

pub fn init_okf_bundle(root: &Path, opts: OkfInitOptions) -> io::Result<OkfInitReport> {
    fs::create_dir_all(root)?;
    let mut report = OkfInitReport {
        root: root.to_path_buf(),
        ..Default::default()
    };

    let index = root.join("index.md");
    if index.exists() {
        report.skipped.push(index);
    } else {
        let body = format!(
            r#"---
okf_version: "{ver}"
---

# Knowledge bundle

* [Sample metric](metrics/sample-metric.md) - Example OKF concept
"#,
            ver = current_okf_version()
        );
        fs::write(&index, body)?;
        report.created.push(index);
    }

    if opts.write_sample_concept {
        let metrics = root.join("metrics");
        fs::create_dir_all(&metrics)?;
        let sample = metrics.join("sample-metric.md");
        if sample.exists() {
            report.skipped.push(sample);
        } else {
            fs::write(
                &sample,
                r#"---
type: Metric
title: Sample metric
description: Example OKF v0.2 concept for Open Document Spec.
tags: [example]
status: draft
generated: { by: ods/okf-init, at: 2026-01-01T00:00:00Z }
---

# Definition

Replace this sample with a real knowledge concept.
"#,
            )?;
            report.created.push(sample);
        }
    }

    if opts.write_attested_stub {
        let computations = root.join("computations");
        fs::create_dir_all(&computations)?;
        let path = computations.join("sample-computation.md");
        if path.exists() {
            report.skipped.push(path);
        } else {
            fs::write(
                &path,
                r#"---
type: Attested Computation
title: Sample computation
description: Contract-only stub (Open Document Spec does not execute attesters in v1).
status: draft
runtime: bigquery
parameters:
  - { name: year, type: integer, required: true }
executor:
  resource: references/skills/run-on-bq.md
  receipt: [job_id, executed_sql, result]
attester:
  resource: references/attesters/sql-equality.py
---

# Computation

    SELECT 1 AS placeholder
"#,
            )?;
            report.created.push(path);
        }
    }

    if opts.write_log {
        let log = root.join("log.md");
        if log.exists() {
            report.skipped.push(log);
        } else {
            fs::write(
                &log,
                "# Bundle update log\n\n## 2026-01-01\n* **Initialization**: Created OKF bundle via `ods okf init`.\n",
            )?;
            report.created.push(log);
        }
    }

    Ok(report)
}
