# AGENTS.md — `src/ods-core`

- Domain folders: `graph/`, `lint/`, `parse/`, `index/`, `okf/`, `multi_spec/`, `spec/`, …
- Functional style: pure data + free functions; IO at edges
- Multi-spec: `multi_spec/` + flags only (no namespaces)
- Frontmatter model: universal top-level tags; engine under nested `ods:`
- Index canonical name: prefer `index.ods.md` (also accept `index.md`)
- Root marker: scalar `ods` version on root index; custom profiles key `custom-profiles`
- Spec docs: `specs/ods/` (not a runtime dependency — keep behavior aligned)

## Keys = schema registry

- **Source of truth:** `spec/schema.rs` → `SpecSchemaRegistry` (ODS / OKF / Skills + custom profiles)
- `validate_ods_frontmatter` + `generate_ods_json_schema` are registry-driven
- Do not reintroduce dead parallel key tables; unknown keys stay preserved
- Guide: `docs/maintainer/schema-driven-keys.md`
- Tests: unit tests in `schema.rs`; integration under `tests/` for lint enums + multi_spec
