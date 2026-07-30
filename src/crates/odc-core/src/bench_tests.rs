#[cfg(test)]
mod tests {
    use super::*;
    use odc_test_support::temp_workspace;

    #[test]
    fn bench_strip_and_restore_cycle() {
        let dir = temp_workspace();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n- [doc.md](doc.md)\n",
        )
        .unwrap();
        fs::write(
            dir.join("doc.md"),
            "---\nprofile: note\nstatus: draft\ndescription: test doc\n---\n\n# Doc Body\n",
        )
        .unwrap();

        let report = bench_strip_workspace(&dir, BenchStripOptions { write: true, full: true, ..Default::default() }).unwrap();
        assert!(report.total_stripped >= 1);
        assert!(!dir.join("index.md").exists());

        let doc_after = fs::read_to_string(dir.join("doc.md")).unwrap();
        assert!(!doc_after.contains("profile: note"));
        assert!(doc_after.contains("# Doc Body"));

        let restore = bench_restore_workspace(&dir, None).unwrap();
        assert!(restore.total_restored >= 1);
        assert!(dir.join("index.md").exists());

        let doc_restored = fs::read_to_string(dir.join("doc.md")).unwrap();
        assert!(doc_restored.contains("profile: note"));
    }

    #[test]
    fn bench_stats_and_run_simulation() {
        let dir = temp_workspace();
        fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\nodc: \">=0.0.1\"\n---\n\n# Root\n",
        )
        .unwrap();
        fs::write(dir.join("a.md"), "---\nprofile: note\n---\n\n# A\n").unwrap();

        let stats = bench_calculate_stats(&dir).unwrap();
        assert!(stats.total_files >= 1);
        assert!(stats.estimated_total_tokens > 0);

        let run = bench_run_simulation(&dir, "summarize architecture", "openai/gpt-4o").unwrap();
        assert_eq!(run.provider, "openai/gpt-4o");
        assert!(run.simulated_output.contains("Simulated"));
    }
}
