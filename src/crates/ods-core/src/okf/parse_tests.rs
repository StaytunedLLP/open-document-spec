#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_type() {
        let fm = parse_okf_frontmatter_block("type: Metric\n").unwrap();
        assert_eq!(fm.type_name.as_deref(), Some("Metric"));
    }

    #[test]
    fn parse_generated_and_bare_verified() {
        let block = r#"
type: Metric
generated: { by: agent/v1, at: 2026-06-20T22:53:05Z }
verified: { by: human:alice, at: 2026-06-25T09:00:00Z }
stale_after: 2026-12-31
"#;
        let fm = parse_okf_frontmatter_block(block).unwrap();
        assert_eq!(fm.generated.as_ref().unwrap().by, "agent/v1");
        assert_eq!(fm.verified.len(), 1);
        assert!(fm.verified[0].by.starts_with("human:"));
        assert_eq!(fm.stale_after.as_deref(), Some("2026-12-31"));
    }

    #[test]
    fn parse_attested_computation() {
        let block = r#"
type: Attested Computation
runtime: bigquery
parameters:
  - { name: year, type: integer, required: true }
executor:
  resource: references/skills/run-on-bq.md
  receipt: [job_id, executed_sql, result]
attester:
  resource: references/attesters/sql-equality.py
"#;
        let fm = parse_okf_frontmatter_block(block).unwrap();
        assert_eq!(fm.type_name.as_deref(), Some("Attested Computation"));
        assert_eq!(fm.runtime.as_deref(), Some("bigquery"));
        assert_eq!(fm.parameters.len(), 1);
        assert_eq!(fm.parameters[0].name, "year");
        assert_eq!(
            fm.executor.resource.as_deref(),
            Some("references/skills/run-on-bq.md")
        );
        assert!(fm.executor.receipt.contains(&"job_id".into()));
    }

    #[test]
    fn parse_sources() {
        let block = r#"
type: BigQuery Table
sources:
  - id: rev-policy
    resource: https://example.com/policy
    title: Policy
    author: team:finance
    last_modified: 2026-04-02
"#;
        let fm = parse_okf_frontmatter_block(block).unwrap();
        assert_eq!(fm.sources.len(), 1);
        assert_eq!(fm.sources[0].id.as_deref(), Some("rev-policy"));
        assert!(fm.sources[0].resource.as_ref().unwrap().contains("example"));
    }

    #[test]
    fn preserves_unknown_keys() {
        let fm = parse_okf_frontmatter_block("type: X\nfoo: bar\n").unwrap();
        assert_eq!(fm.unknown.get("foo").map(String::as_str), Some("bar"));
    }
}
