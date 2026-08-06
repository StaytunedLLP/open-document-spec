# Rust style (ods-core / ods-cli)

- Prefer functional style: data + free functions, pipelines, effects at the edge
- See `docs/maintainer/functional-style.md`
- Keep public re-exports stable when moving modules
- Tests live next to domains under `src/ods-core/tests/` and `src/ods-cli/tests/`
- **User-facing strings:** `src/ods-core/src/error/messages.rs` only (CLI constructs `CliError` via catalog helpers)

## File Size & Modularity

- **Maximum File Length**: Keep individual Rust source files strictly under **500 lines**, targeting **~300 lines** per file.
- When a module/file approaches 500 lines, refactor by splitting subcommands or logical sub-domains into dedicated submodules within a folder module directory (e.g. `foo/mod.rs`, `foo/init.rs`, `foo/inspect.rs`).

## Schema-driven keys

- Dialect keys live in `src/ods-core/src/spec/schema.rs` (`SpecSchemaRegistry`)
- Lint enums / `ods schema` consume the registry — avoid new parallel `match key` catalogs
- Adding a dialect: register schema → thin engine module → CLI flag in `ExtraSpecs` → `specs/<d>/keys.md` → tests
- Detail: `docs/maintainer/schema-driven-keys.md`

## Quality before handoff

See `.agents/rules/40-quality-gates.md` and skill `quality-gate`.

```bash
.agents/hooks/scripts/pre-handoff.sh --full
```

- Prefer table-driven edge tests (flags, keys, invalid enums) over coverage-only noise
