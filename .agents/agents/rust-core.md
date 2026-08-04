# Subagent: rust-core

**Use when:** changing `src/ods-core` or `src/ods-cli` behavior that must stay aligned with specs.

**Process:**

1. Map modules under `src/ods-core/src/` before editing
2. Prefer functional style (`docs/maintainer/functional-style.md`)
3. Preserve CLI surface: no `--ods`, no namespaces; subcommand names after the verb (argv)
4. **Keys/dialects:** edit `spec/schema.rs` registry first; keep lint/`ods schema` schema-driven
5. Add/adjust tests (edge cases for flags + keys, not only happy path)
6. Run `SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh` (includes naming + odc residue)
7. If coverage-sensitive, `ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh`
8. If behavior is user-visible, update `specs` + guide via docs-sync
