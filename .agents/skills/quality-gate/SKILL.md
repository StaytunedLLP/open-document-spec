---
name: quality-gate
description: >-
  Sequenced pre-commit and engine-quality work: baseline, odc/naming cleanup,
  schema alignment, edge tests, coverage ≥90%, local CI mirror. Use when making
  the tree commit-ready or after multi-file engine/CLI changes.
---

# Quality gate (maintainer)

## When to use

- “Make pipeline green / pre-commit / coverage / finish remaining work”
- Multi-phase engine + docs + tests changes
- After large renames or multi-spec surface work

## Fixed sequence (do not skip)

### 1. Baseline

```bash
cargo test --workspace --locked 2>&1 | tail -20
./src/scripts/check-naming.sh
./src/scripts/check-odc-residue.sh
# optional coverage snapshot
./src/scripts/coverage.sh   # writes .artifacts/coverage/missing_ranked.txt
```

### 2. Naming / legacy `odc`

- Class A: strip product/fixture `odc:` pins  
- Do **not** blind-replace archive plans, CHANGELOG history, or “do not invent” text  
- Dual-read `ODC_*` env only with legacy comments  
- Gate: `./src/scripts/check-odc-residue.sh`

### 3. Schema + keys alignment

- Registry: `src/ods-core/src/spec/schema.rs`  
- Docs: `specs/*/keys.md`  
- Rule: `.agents/rules/30-schema-keys.md`  
- If keys changed: unit + lint integration tests

### 4. Edge-case tests

- CLI flags: `--okf`, `--skills`, reject `--ods`, namespaces  
- Keys: invalid enums, placement, missing required  
- Prefer table-driven tests over coverage-only stubs  

### 5. Coverage

```bash
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
```

- Work `missing_ranked.txt` top-down  
- T3 excludes only for real effect edges; triple-sync docs/CI/script  

### 6. Full local gate + docs

```bash
SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh
```

- User-visible → `.agents/skills/release-docs/SKILL.md`  
- Install scripts: `src/scripts/install.*` → sync `app-web/public` + `skills/ods/scripts`

## Definition of Done

See `.agents/rules/40-quality-gates.md`. Do not report ready without green `check-local`.

## Out of scope

- Rewriting `docs/plan/archive/**` history  
- 100% line coverage including T3  
- End-user skill (`skills/ods`) process docs — keep product-only  
