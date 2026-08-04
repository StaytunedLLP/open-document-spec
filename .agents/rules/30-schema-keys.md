# Schema-driven keys

**Engine SoT:** `src/ods-core/src/spec/schema.rs` → `SpecSchemaRegistry`

| Dialect | Flag | Docs |
|---|---|---|
| ODS | *(default)* | `specs/ods/keys.md` |
| OKF | `--okf` | `specs/okf/keys.md` |
| Skills | `--skills` | `specs/skills/keys.md` |
| Custom profiles | `expected_keys` | `specs/ods/profiles.md` |

## Must

- Add/change keys in the registry **first**
- Keep `specs/*/keys.md` and registry in the **same** change set
- Drive lint enums / `ods schema` from the registry — no new parallel `match key` catalogs
- Unknown keys remain preserved

## New dialect checklist

1. `register_<name>_schema()` + `with_defaults()`
2. Thin engine (`multi_spec/` or sibling)
3. CLI flag in `ExtraSpecs` (never `--ods`)
4. `specs/<name>/{intro,keys}.md`
5. Tests: schema unit + lint + CLI flag

Detail: `docs/maintainer/schema-driven-keys.md`
