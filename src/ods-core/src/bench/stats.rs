use crate::graph::{ContextOptions, estimate_path_tokens, resolve_context_with_options};
use crate::model::FrontmatterState;
use crate::parse::document_id;
// note: included into bench/mod.rs — fs/io/Path already imported there

/// Calculate token & cost ROI statistics for current workspace.
///
/// `avg_ods_context_tokens` is the **mean** of real `resolve_context` sizes
/// (file bytes/4) across documents — not total_repo/n_docs.
pub fn bench_calculate_stats(root: &Path) -> io::Result<crate::bench::BenchStatsReport> {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let workspace = crate::fs::load_workspace(&root)?;

    let mut total_raw_bytes = 0usize;
    for doc in &workspace.documents {
        if let Ok(meta) = fs::metadata(&doc.path) {
            total_raw_bytes += meta.len() as usize;
        }
    }

    let estimated_total_tokens = total_raw_bytes / 4;

    let mut context_token_samples = Vec::new();
    for doc in &workspace.documents {
        let fm = match &doc.frontmatter {
            FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        };
        let id = document_id(&workspace.root, &doc.path, fm);
        let result = resolve_context_with_options(
            &workspace,
            &id,
            &ContextOptions {
                include_private: false,
                include_code: false,
                include_related: false,
                max_tokens: None,
            },
        );
        if result.paths.is_empty() {
            // Fallback: single-file estimate when id resolution fails.
            context_token_samples.push(estimate_path_tokens(&doc.path));
        } else {
            context_token_samples.push(result.token_estimate);
        }
    }

    let avg_ods_context_tokens = if context_token_samples.is_empty() {
        0
    } else {
        context_token_samples.iter().sum::<usize>() / context_token_samples.len()
    };

    let token_reduction_percentage = if estimated_total_tokens == 0 {
        0.0
    } else {
        ((estimated_total_tokens.saturating_sub(avg_ods_context_tokens)) as f64
            / estimated_total_tokens as f64)
            * 100.0
    };

    let est_monthly_cost_savings_usd =
        ((estimated_total_tokens.saturating_sub(avg_ods_context_tokens)) as f64 / 1_000_000.0)
            * 5.0
            * 100.0;

    Ok(crate::bench::BenchStatsReport {
        total_files: workspace.documents.len(),
        total_raw_bytes,
        estimated_total_tokens,
        avg_ods_context_tokens,
        token_reduction_percentage,
        est_monthly_cost_savings_usd,
    })
}

/// Simulate an AI agent prompt execution measuring raw vs ODS context token savings.
pub fn bench_run_simulation(
    root: &Path,
    prompt: &str,
    provider: &str,
) -> io::Result<crate::bench::BenchRunReport> {
    let stats = bench_calculate_stats(root)?;
    let provider_name = if provider.is_empty() {
        "openai/gpt-4o"
    } else {
        provider
    };

    let raw_cost = (stats.estimated_total_tokens as f64 / 1_000_000.0) * 5.0;
    let ods_cost = (stats.avg_ods_context_tokens as f64 / 1_000_000.0) * 5.0;

    let output_msg = format!(
        "[Simulated estimate — no live LLM API call made] Benchmarked prompt: '{prompt}' across {} repository docs.\n\
         Without ODS Context: ~{} tokens (sum of all doc bytes/4)\n\
         With ODS Bounded Graph: ~{} tokens (mean resolve_context size, code edges off, ~${ods_cost:.4} at $5/1M)\n\
         Estimated Token Savings: {:.1}%\n\n\
         This is a local estimate only; ods does not currently call OPENAI_API_KEY, ANTHROPIC_API_KEY, or GEMINI_API_KEY.",
        stats.total_files,
        stats.estimated_total_tokens,
        stats.avg_ods_context_tokens,
        stats.token_reduction_percentage
    );

    Ok(crate::bench::BenchRunReport {
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
