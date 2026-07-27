//! Manual scale benchmark for the workspace load / index-generate / lint pipeline.
//!
//! Not part of `cargo test` — run explicitly:
//!
//! ```text
//! cargo run -p ods-core --release --example scale_bench -- 100000
//! cargo run -p ods-core --release --example scale_bench -- 1000000 200
//! ```
//!
//! Args: `<doc_count> [docs_per_dir]` (default docs_per_dir=100).
//!
//! Set `ODS_BENCH_DIR` to generate under a specific (disk-backed) directory
//! instead of the OS temp dir — useful when the temp dir is a small tmpfs
//! (RAM-backed) mount and `doc_count` is large enough that the generated
//! files themselves would pressure system memory.
//!
//! Generates `doc_count` synthetic Markdown documents under a temp workspace,
//! then times `load_workspace` -> `generate_indexes` -> `load_workspace` ->
//! `lint_workspace` (the same sequence `large_workspace_with_10k_documents_lints`
//! exercises), so results are directly comparable to that test's timing.
//! See docs/maintainer/production-readiness-audit-2026-07-22.md for recorded
//! results and the O(N^2) index-rebuild fix this benchmark validates.

use ods_core::{generate_indexes, lint_workspace, load_workspace};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

fn main() {
    let mut args = std::env::args().skip(1);
    let doc_count: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(10_000);
    let docs_per_dir: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(100);
    let dir_count = doc_count.div_ceil(docs_per_dir);

    let base_dir = std::env::var("ODS_BENCH_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    let root = base_dir.join(format!("ods-scale-bench-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create bench root");

    println!(
        "Generating {doc_count} documents across {dir_count} directories ({docs_per_dir}/dir) at {}...",
        root.display()
    );
    let gen_start = Instant::now();
    let mut written = 0usize;
    for group in 0..dir_count {
        let dir = root.join(format!("group-{group:06}"));
        fs::create_dir_all(&dir).expect("group dir");
        for item in 0..docs_per_dir {
            if written >= doc_count {
                break;
            }
            let path = dir.join(format!("doc-{item:06}.md"));
            fs::write(
                &path,
                format!(
                    "---\nprofile: note\nstatus: draft\n---\n\n# Doc {group}-{item}\n\n## Overview\n"
                ),
            )
            .expect("write doc");
            written += 1;
        }
    }
    let root_index = format!(
        "---\nprofile: index\nods: 0.1\nods-cli: \">=0.1.18\"\n---\n\n# Scale Bench ({doc_count} docs)\n"
    );
    fs::write(root.join("index.md"), root_index).expect("root index");
    println!("  file generation: {:?}", gen_start.elapsed());

    time_phase("load_workspace #1", &root, || {
        load_workspace(&root).expect("load #1");
    });

    let workspace = load_workspace(&root).expect("load for generate_indexes");
    time_phase("generate_indexes", &root, || {
        generate_indexes(&workspace).expect("generate indexes");
    });

    let workspace2 = time_phase_ret("load_workspace #2 (post-index)", &root, || {
        load_workspace(&root).expect("load #2")
    });

    time_phase("lint_workspace", &root, || {
        let diagnostics = lint_workspace(&workspace2);
        let errors = diagnostics
            .iter()
            .filter(|d| d.severity == ods_core::Severity::Error)
            .count();
        if errors > 0 {
            eprintln!("  warning: {errors} lint errors (unexpected for synthetic docs)");
        }
    });

    let _ = fs::remove_dir_all(&root);
    println!("Done. Cleaned up {}.", root.display());
}

fn time_phase(label: &str, _root: &PathBuf, f: impl FnOnce()) {
    let start = Instant::now();
    f();
    println!("  {label}: {:?}", start.elapsed());
}

fn time_phase_ret<T>(label: &str, _root: &PathBuf, f: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = f();
    println!("  {label}: {:?}", start.elapsed());
    result
}
