# Rust style (ods-core / ods-cli)

- Prefer functional style: data + free functions, pipelines, effects at the edge
- See `docs/maintainer/functional-style.md`
- Keep public re-exports stable when moving modules
- Tests live next to domains under `src/ods-core/tests/` and `src/ods-cli/tests/`

## Schema-driven keys

- Dialect keys live in `src/ods-core/src/spec/schema.rs` (`SpecSchemaRegistry`)
- Lint enums / `ods schema` consume the registry — avoid new parallel `match key` catalogs
- Adding a dialect: register schema → thin engine module → CLI flag in `ExtraSpecs` → `specs/<d>/keys.md` → tests
- Detail: `docs/maintainer/schema-driven-keys.md`

## Quality before handoff

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
# or full gate:
SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
```

- Workspace coverage bar: **≥90% lines** with T3 excludes (`docs/maintainer/coverage-excludes.md`)
- Prefer table-driven edge tests (flags, keys, invalid enums) over coverage-only noise
