// ODS Benchmark Evaluation Binary in Rust
// Calculates context size, token volume, estimated cost, and reduction percentages
// comparing unstructured documentation dumps (without-ods) against targeted ODS graph context loads (with-ods).

use std::fs;
use std::path::{Path, PathBuf};

const PRICE_PER_1M_TOKENS: f64 = 5.00;

struct Scenario {
    name: &'static str,
    without_dir: PathBuf,
    with_target: PathBuf,
    depends_targets: Vec<PathBuf>,
}

fn count_tokens_in_file(path: &Path) -> usize {
    if !path.is_file() {
        return 0;
    }
    let Ok(content) = fs::read_to_string(path) else {
        return 0;
    };
    let words = content.split_whitespace().count();
    (words as f64 * 1.33) as usize
}

fn count_tokens_in_directory(dir: &Path, ignore_private: bool) -> usize {
    let mut total_tokens = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                total_tokens += count_tokens_in_directory(&path, ignore_private);
            } else if path.extension().is_some_and(|ext| ext == "md") {
                if ignore_private {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if content.contains("share: private") {
                            continue;
                        }
                    }
                }
                total_tokens += count_tokens_in_file(&path);
            }
        }
    }
    total_tokens
}

fn main() {
    let benchmark_dir = Path::new("ods-test/benchmarks");
    let without_ods = benchmark_dir.join("without-ods");
    let with_ods = benchmark_dir.join("with-ods");

    let scenarios = vec![
        Scenario {
            name: "Monolith E-Commerce Setup",
            without_dir: without_ods.join("example1-monolith"),
            with_target: with_ods.join("example1-monolith/checkout.md"),
            depends_targets: vec![with_ods.join("example1-monolith/auth.md")],
        },
        Scenario {
            name: "Cross-Repo Pack Sharing & Secrets",
            without_dir: without_ods.join("example2-packs"),
            with_target: with_ods.join("example2-packs/public-api.md"),
            depends_targets: vec![],
        },
        Scenario {
            name: "SOP & Onboarding Compliance",
            without_dir: without_ods.join("example3-compliance"),
            with_target: with_ods.join("example3-compliance/onboarding.md"),
            depends_targets: vec![with_ods.join("example3-compliance/security-policy.md")],
        },
    ];

    println!("=========================================================================");
    println!("               ODS CONTEXT & TOKEN SAVINGS BENCHMARK REPORT             ");
    println!("=========================================================================\n");

    println!(
        "{:<32} | {:<12} | {:<10} | {:<12} | {:<11}",
        "Example Scenario", "Without ODS", "With ODS", "Token Saving", "Reduction %"
    );
    println!("{}", "-".repeat(88));

    let mut total_without = 0;
    let mut total_with = 0;

    for sc in &scenarios {
        let tokens_without = count_tokens_in_directory(&sc.without_dir, false);
        let mut tokens_with = count_tokens_in_file(&sc.with_target);
        for dep in &sc.depends_targets {
            tokens_with += count_tokens_in_file(dep);
        }

        let saving = tokens_without.saturating_sub(tokens_with);
        let pct = if tokens_without > 0 {
            (saving as f64 / tokens_without as f64) * 100.0
        } else {
            0.0
        };

        total_without += tokens_without;
        total_with += tokens_with;

        println!(
            "{:<32} | {:>8} tok | {:>6} tok | {:>8} tok | {:>9.1}%",
            sc.name, tokens_without, tokens_with, saving, pct
        );
    }

    let total_saving = total_without.saturating_sub(total_with);
    let total_pct = if total_without > 0 {
        (total_saving as f64 / total_without as f64) * 100.0
    } else {
        0.0
    };

    println!("{}", "-".repeat(88));
    println!(
        "{:<32} | {:>8} tok | {:>6} tok | {:>8} tok | {:>9.1}%",
        "TOTAL (Cumulative Across Queries)", total_without, total_with, total_saving, total_pct
    );
    println!("=========================================================================\n");

    let cost_without = (total_without as f64 * 2000.0 / 1_000_000.0) * PRICE_PER_1M_TOKENS;
    let cost_with = (total_with as f64 * 2000.0 / 1_000_000.0) * PRICE_PER_1M_TOKENS;
    let cost_saving = cost_without - cost_with;

    println!("Enterprise Daily API Cost (100 devs @ 20 queries/day):");
    println!("  - Without ODS:  ${:.2} / day", cost_without);
    println!("  - With ODS:     ${:.2} / day", cost_with);
    println!("  - Daily Saving: ${:.2} / day", cost_saving);
    println!(
        "  - Annual Savings (250 days): ${:.2} / year\n",
        cost_saving * 250.0
    );
}
