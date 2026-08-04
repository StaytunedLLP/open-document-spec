# AGENTS.md

Rules for coding agents working in this Open Document Spec repository.

## CLI product surface (locked)

- Binary: **`ods`**
- **ODS is default** — bare `ods <cmd>` (there is **no** `--ods` flag)
- Extra specs use flags only:
  - `--okf` — Google OKF v0.2
  - `--skills` — Agent Skills (`SKILL.md` packages)
- Do **not** use namespaces `ods okf` / `ods ods` / `ods skills` (deprecated if present)
- Editors: **`ods lsp`** (JSON-RPC). Background watcher: `ods serve` / `ods start` — not an LSP
- Do not invent `odc:` or `ods-cli:` frontmatter keys
- Subcommands: after a verb like `profile init`, the **name is argv index 3** (`ods profile init <name>`), not index 2

## Specs layout (normative)

Single tree at **repo root** `specs/` (not under `src/`; no symlink):

```
specs/
  ods/     # default dialect — start at intro.md, keys.md, core.md
  okf/     # --okf dialect — intro.md, keys.md
  skills/  # --skills dialect — intro.md, keys.md
```

- Authoring keys: read `specs/ods/keys.md` (not buried in core).
- After renames, update `app-web` nav + sitemap + `docs/guide` + `skills/ods/references` + `llms.txt`.
- Maintainer agent skills: `.agents/skills/` (this repo). End-user skill: `skills/ods/`.

## Schema-driven keys (engine)

**Single source of truth for dialect keys:** `src/ods-core/src/spec/schema.rs` (`SpecSchemaRegistry`).

- Add/change keys or dialects there first (ODS / OKF / Skills + custom profiles).
- Wire lint / `ods schema` from the registry — do **not** grow new hardcoded key tables in parse/lint.
- Normative prose: `specs/*/keys.md`. How-to: `docs/maintainer/schema-driven-keys.md`.
- `ods schema` emits registry-driven JSON Schema; `ods schema --okf` / `--skills` list dialect keys.

## Naming / legacy `odc` (do not thrash)

- Product name and binary are **`ods`**. Never teach `odc` as primary.
- **Do not** blind `s/odc/ods/g` (breaks archive history, dual-read, “do not invent” text).
- Strip `odc:` pins from **new** fixtures; one intentional preserve-unknown test is OK.
- Legacy env dual-read (`ODC_JOBS`, `ODC_AUTO_UPDATE`, …) may remain with “legacy” comments.
- Gate: `./src/scripts/check-odc-residue.sh` and `./src/scripts/check-naming.sh`.

## After document / skill edits

```bash
ods lint
ods lint --okf      # if OKF trees changed
ods lint --skills   # if SKILL.md packages changed
```

## Before commit (local pipeline)

Mirrors CI quality gate. Prefer this over ad-hoc partial checks:

```bash
SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh   # fmt, clippy -D warnings, tests, fixtures, naming, odc residue
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh   # workspace ≥90% lines with T3 excludes
```

- Coverage policy: `docs/maintainer/coverage.md` + `coverage-excludes.md` (T3 = network/OS/watch/LSP/git-update edges).
- Ratchet CI `--fail-under-lines` **up only**. Keep ignore regex in sync: `coverage.sh`, `pr.yml`, excludes doc.
- Install script source of truth: `src/scripts/install.*` — sync copies under `app-web/public/` and `skills/ods/scripts/`.

## Code style

Prefer functional style in `ods-core` / CLI: data + free functions, pipelines, effects at the edge.
See `docs/maintainer/functional-style.md`.

## Useful commands

| Goal | Command |
|---|---|
| Validate ODS | `ods lint` |
| OKF | `ods lint --okf` / `ods init --okf` |
| Skills | `ods lint --skills` / `ods init --skills` |
| Context | `ods context <id>` |
| Editor LSP | `ods lsp` |
| JSON Schema | `ods schema` / `ods schema --okf` / `--skills` |
| Install skill into host | `ods skill install --agent <name>` |
| Local quality gate | `./src/scripts/check-local.sh` |
| Coverage | `./src/scripts/coverage.sh` |
| Refresh this file | `ods agents sync` |
