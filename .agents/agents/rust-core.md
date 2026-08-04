# Subagent: rust-core

**Use when:** changing `src/ods-core` or `src/ods-cli` behavior that must stay aligned with specs.

**Process:** follow `.agents/skills/quality-gate/SKILL.md` when the change is multi-step.

1. Map modules under `src/ods-core/src/` before editing  
2. Functional style (`docs/maintainer/functional-style.md`)  
3. CLI locks: no `--ods`, no namespaces; subcommand args after the verb  
4. Keys/dialects → `spec/schema.rs` first (`.agents/rules/30-schema-keys.md`)  
5. Edge-case tests (flags + keys), not only happy path  
6. **Forbidden:** expand T3 coverage excludes without updating `coverage.sh` + `pr.yml` + `coverage-excludes.md` together  
7. Handoff: `.agents/hooks/scripts/pre-handoff.sh --full`  
8. User-visible → docs-sync / release-docs skill  
