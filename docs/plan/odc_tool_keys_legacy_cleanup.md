---
profile: plan
status: stable
share: public
description: Superseding plan — tool is odc; root keys ods/odc/okf_version; seamless bare CLI; full legacy inventory and cleanup phases for docs, install, config paths, and error reports.
owner: team:opendocify
tags: [odc, ods, okf, keys, legacy, migration, plan, cleanup]
---

# Plan: Tool = `odc`, root keys, legacy cleanup

**Supersedes** conflicting sections in:

- [`ods_to_odc_migration_and_cli_architecture.md`](./ods_to_odc_migration_and_cli_architecture.md) — especially “bare `odc lint` is ERROR” and “mandatory namespace only”
- [`okf_native_support.md`](./okf_native_support.md) — same bare-command policy
- [`docs/specs/frontmatter-keys-ods-vs-okf.md`](../specs/frontmatter-keys-ods-vs-okf.md) — “`odc lint` ERROR — pick a spec”

Those older docs remain useful for history and OKF engine detail; **this file is the source of truth** for product naming, root keys, CLI UX, and remaining cleanup.

---

## 0. Direct answers (locked)

| Question | Answer |
|---|---|
| What is the **tool**? | **`odc`** (OpenDocify). Ship primary binary `odc`. Optional legacy argv0/symlink **`ods`** → always means **ODS engine** (`odc ods …`). |
| What is **ODS**? | **Spec / document dialect** for codebase doc graphs. Markdown still uses **`ods:`** (never rename the nested engine map). |
| What is **OKF**? | **Google OKF v0.2** knowledge format. Root marker **`okf_version:`** only — **do not invent** `okf:` as a CLI pin. |
| Root keys? | **`ods:`** · **`odc:`** · **`okf_version:`** (see §1). |
| Bare `odc lint`? | **Seamless auto-detect** from root markers. Explicit `odc ods` / `odc okf` always available. Hybrid lint/doctor/audit may run both. |
| Breaking change for CLI pin? | **Yes, accepted.** `ods-cli:` → **`odc:`**. `odc upgrade --write` rewrites remaining roots. |
| Dual long-term compat for `ods-cli:`? | **No** in parse path after cutover; only upgrade rewrite + docs. |

---

## 1. Root frontmatter keys (canonical)

### 1.1 Three markers

| Key | Format | Where | Meaning |
|---|---|---|---|
| **`ods: <spec>`** | e.g. `0.1` | ODS root `index.md` | Workspace is ODS; pin **spec** version. Nested document map still under top-level **`ods:`** block in non-root docs (profile, depends, …). |
| **`odc: ">=x.y.z"`** | semver / range | ODS root `index.md` | Pin **minimum OpenDocify CLI** version. **Replaces** legacy `ods-cli:`. |
| **`okf_version: "0.2"`** | Google OKF string | OKF root `index.md` **only** | Bundle targets OKF v0.2. Unchanged Google vocabulary. |

**Hybrid root** may carry `ods:` + `odc:` + `okf_version:` together. Engines stay separate; bare multi-engine commands (lint/doctor/audit) may run **both** when both markers exist.

### 1.2 What is **not** a root key

| Rejected / legacy | Action |
|---|---|
| `ods-cli:` | **Legacy.** Upgrade rewrites → `odc:`. Docs must not teach it. |
| `odc-cli:` | Never introduce. |
| `okf:` as CLI pin | Never introduce (collides with mental model of Google keys). |
| Renaming nested `ods.profile` → `odc.profile` | **Out of scope forever** for v0.x — nested map stays `ods:`. |

### 1.3 Init writers (required behavior)

| Command | Writes |
|---|---|
| `odc init` / `odc ods init` | Root `ods: <current_spec>` + `odc: ">=…"` |
| `odc init --okf` / `odc okf init` | Root `okf_version: "0.2"` (+ OKF conventions) |
| `odc upgrade --write` | `ods-cli:` → `odc:`; optional `~/.ods` → `~/.odc` hints/copies |

### 1.4 Example root snippets

**ODS only**

```yaml
---
title: My repo
ods: 0.1
odc: ">=0.0.4"
---
```

**OKF only**

```yaml
---
title: Knowledge pack
okf_version: "0.2"
---
```

**Hybrid**

```yaml
---
title: Monorepo docs + knowledge
ods: 0.1
odc: ">=0.0.4"
okf_version: "0.2"
---
```

---

## 2. CLI product model (tool = odc)

```
odc                          # primary product binary
├── bare <cmd>               # auto-detect ODS / OKF / hybrid / init default ODS
├── odc ods <cmd>            # force ODS engine
├── odc okf <cmd>            # force OKF engine
├── odc agents <cmd>         # agent graphs
└── platform: update | upgrade | setup | version | workspaces | skill | pack
```

| Invocation | Behavior |
|---|---|
| `odc lint` (ODS root) | ODS lint |
| `odc lint` (OKF root) | OKF lint |
| `odc lint` (hybrid) | Both (lint/doctor/audit family) |
| `odc lint` (no markers) | Clear error → `odc init` or `odc init --okf` |
| `odc init` | ODS |
| `odc init --okf` | OKF |
| `ods lint` (legacy binary name) | Always ODS (argv0 compat) |
| `odc ods …` / `odc okf …` | Always explicit; preferred in CI for clarity |

**Docs teaching:** primary path is **`odc …`**. Namespaces are **escape hatches** and CI clarity, not friction for day-one users.

---

## 3. Already done (do not re-litigate)

Codebase progress (verify in CI; treat as baseline):

- [x] Crates renamed: `odc` / `odc-core` / `odc-test-support`
- [x] Root pin field `odc` in model/parse/lint/init (`current_odc_requirement`, etc.)
- [x] Bare auto-detect in `entry.rs` (`dispatch_auto_detect`)
- [x] OKF native modules under `odc-core::okf`
- [x] `odc upgrade` rewrites `ods-cli:` → `odc:`
- [x] Dual binary build: `--bin odc --bin ods`
- [x] Partial CHANGELOG / frontmatter-keys updates

---

## 4. Legacy inventory — what still needs updating

Legend: **P0** blocks correct UX · **P1** user-facing docs · **P2** internal names / polish · **Keep** intentional (format name ODS, nested `ods:`, skill folder may stay `skills/ods` until product rename).

### 4.1 P0 — Wrong product behavior or install path

| Area | Current debt | Target |
|---|---|---|
| `app-web/public/install.sh` | Still “ODS installer”, `cargo install ods` / `ods-cli` fallbacks | Install **`odc`** (+ optional `ods` symlink); repo `StaytunedLLP/open-document-spec` / release assets `odc-*` |
| `app-web/public/install.ps1` | `cargo install ods-cli` | Same as sh |
| `src/scripts/install.sh` header | “downloads prebuilt `ods`” | “prebuilt `odc` (+ `ods` alias)” |
| `src/action/README.md` | Install `ods`, run `ods lint` | `odc` / `odc ods lint` or bare `odc lint` |
| `action.yml` / `src/action/action.yml` | `ods-path` as primary mental model | Prefer `odc-path`; keep `ods-path` as deprecated alias if needed |
| Plans / key doc | Teach “`odc lint` ERROR — pick a spec” | Teach seamless auto-detect (§2) |

### 4.2 P1 — User-facing docs still on old names or mandatory namespace-only story

| Path | Issues |
|---|---|
| `README.md` | “legacy `ods` skill”, version banner `ods v0.1.x`, primary examples only `odc ods …` without bare path |
| `docs/specs/frontmatter-keys-ods-vs-okf.md` | § CLI table still “mandatory” + ERROR on bare lint; § out-of-scope still bans auto-pick |
| `docs/plan/ods_to_odc_migration_and_cli_architecture.md` | Locked “mandatory namespaces / bare error” — add banner → this plan |
| `docs/plan/okf_native_support.md` | Same bare-error policy |
| `docs/plan/external_repo_cutover_checklist.md` | Align root key `odc:` + bare vs namespaced CI |
| `skills/ods/SKILL.md` | Still `ods-error.md`, `~/.ods/odsconfig.toml` as only paths |
| `skills/ods/references/spec.md` | `ods lint` / `ods setup` strings; registry `~/.ods` |
| `skills/ods/scripts/bootstrap.sh` | “install `ods` binary” messaging |
| `app-web/src/content/docs/*` | `ods-error.md`, `~/.ods/odsconfig.toml`, mixed messaging |
| `specs/indexes.md` | `ods workspaces`, `ods start`, `~/.ods/odsconfig.toml` as sole machine path |
| `specs/profiles.md` | `~/.ods/packs/` only |
| `specs/validation.md` | OK on `odc:` keys; still “legacy: `ods lint`” — fine if secondary |
| `CONTRIBUTING.md` | Dual bin install OK; ensure docs say product is odc |
| `CHANGELOG.md` | Older bullets still “self-update installs `ods` only” — historical OK; Unreleased must match product |

### 4.3 P1 — Machine paths & error report naming (product consistency)

| Legacy | Intended modern | Notes |
|---|---|---|
| `~/.ods/` | **`~/.odc/`** | Config, backups, packs. Read legacy + write modern; `upgrade --write` migrates. |
| `~/.ods/odsconfig.toml` | **`~/.odc/odcconfig.toml`** | Dual-read during transition. |
| `~/.ods/packs/` | **`~/.odc/packs/`** | Same. |
| `~/.ods/backups/` | **`~/.odc/backups/`** | Bench snapshots. |
| Root **`ods-error.md`** | **`.odc/odc-errors.md`** (or keep root file but rename) | Plans already chose `.odc/odc-errors.md` for audit; lint report should align. Dual-write/read one release if needed. |
| Service unit names `ods-watch` / “ods serve” comments | `odc-watch` / `odc serve` | Rename carefully (user machines). |

**Decision needed only if contested:** whether lint continues writing root `ods-error.md` for one minor release. **Recommendation:** prefer `.odc/odc-errors.md` as canonical; dual-read old path; document in CHANGELOG as breaking for scripts that `cat ods-error.md`.

### 4.4 P2 — Internal code identifiers (no user-facing break)

| Location | Debt | Target |
|---|---|---|
| `odc-core/src/index/checker.rs` | Locals `ods_cli` | Rename → `odc_pin` / `odc_req` |
| `share.rs` comments | `ods`/`ods-cli` markers | `ods`/`odc` |
| `service/launchers.rs` comment | `ods serve` | `odc serve` / `odc ods serve` |
| `upgrade_command.rs` help text | “never assume bare `odc lint`” | Align with auto-detect |
| `entry.rs` outdated comment L76 | “require explicit namespace” | “auto-detect or explicit” |
| Fixture dirs `ods-test/` | Name is fine (ODS **format** fixtures) | Optional later `fixtures/` — low priority |
| Crate path `skills/ods/` | Skill for ODS dialect | Keep until product skill rename; binary install must be `odc` |

### 4.5 Intentional “ods” that must **stay**

- Nested / root **spec key** `ods:`
- Product prose “ODS (Open Document Spec)”
- CLI namespace token `odc ods`
- Google OKF keys unchanged
- Historical CHANGELOG entries
- Research notes under `docs/research/` (stamp “historical” if wrong)

### 4.6 Ops outside monorepo (checklist, not code)

See [`external_repo_cutover_checklist.md`](./external_repo_cutover_checklist.md):

1. ≈3 live roots: ensure `odc:` not `ods-cli:`
2. CI: `odc lint` or `odc ods lint`
3. GitHub Release assets named `odc-*` (and optional `ods` alias packaging)
4. Action consumers pin to version that installs `odc`

---

## 5. Documentation truth table (what to teach)

| Topic | Teach this | Do not teach |
|---|---|---|
| Install | `curl … \| sh` → `odc` on PATH | `cargo install ods-cli` as primary |
| First command | `odc init` then `odc lint` | Must type `odc ods` for every call |
| Explicit namespaces | Recommended in **CI** and hybrid repos | Required for all humans always |
| Root keys | `ods` + `odc` for ODS; `okf_version` for OKF | `ods-cli`, `odc-cli`, `okf:` pin |
| Config home | `~/.odc` | Only `~/.ods` without migration note |
| Lint report | `.odc/odc-errors.md` (after P1) | Permanent `ods-error.md` as brand |
| Binary alias | `ods` = ODS-only argv0 | `ods` as the product name |

---

## 6. Execution phases

### Phase A — Align plans & key reference (docs only, 1 PR-sized)

1. Mark this plan as **stable** once approved.
2. Banner on old migration + OKF plans: “CLI UX superseded by `odc_tool_keys_legacy_cleanup.md`.”
3. Rewrite `frontmatter-keys-ods-vs-okf.md` § CLI:
   - Seamless bare commands
   - Root key table matches §1
   - Remove “ERROR — pick a spec”
4. README hero: tool `odc`, keys one-liner, both bare + namespaced examples.

### Phase B — Install / Action / packaging (P0)

1. Rewrite `app-web/public/install.{sh,ps1}` from current `src/scripts` sources (single source of truth → copy or generate).
2. Ensure release asset names documented: `odc-linux-…`, symlink/copy `ods`.
3. Action README + descriptions: install `odc`; outputs `odc-path` primary.
4. Smoke: install script dry messages never say “ods-cli crate” as primary path.

### Phase C — Machine path + error report migration (P1, may be breaking)

1. Dual-read `~/.odc` then `~/.ods` for config/packs/backups.
2. Prefer write to `~/.odc`.
3. `odc upgrade --write` completes copy + documents next steps.
4. Lint/audit report path → `.odc/odc-errors.md`; dual-read `ods-error.md` one release; update bench/tests.
5. Specs (`indexes`, `profiles`, `validation`), skills, app-web troubleshooting catalog.

### Phase D — Sweep docs/skills/specs strings

Systematic replace/teach (careful: do **not** replace nested key `ods:`):

| Pattern | Replacement policy |
|---|---|
| User-facing `` `ods lint` `` as primary | `` `odc lint` `` or `` `odc ods lint` `` |
| `` `ods-error.md` `` | `` `.odc/odc-errors.md` `` after Phase C |
| `` `~/.ods/odsconfig.toml` `` | dual paths documented |
| “mandatory namespace / bare errors” | auto-detect + optional namespaces |
| “legacy `ods` as product” | “legacy argv0 alias for ODS” |

Files: README, CONTRIBUTING, skills/*, specs/*, app-web docs, guide 07, action README, plan external checklist.

### Phase E — Internal renames + tests

1. `ods_cli` locals → `odc_*` in checker.
2. Comments in entry/upgrade/share/launchers.
3. Tests: bare `odc lint` ODS + OKF + hybrid; upgrade rewrites `ods-cli:`; config dual-read.
4. `cargo test --workspace` + `cargo clippy -p odc -p odc-core --all-targets -- -D warnings`.

### Phase F — Ops handoff

1. Publish GitHub Release with `odc-*` assets.
2. External ≈3-repo root `odc:` + CI.
3. Optional: service unit rename `ods-watch` → `odc-watch` with migration note.

---

## 7. Verification gates

| Gate | Pass criteria |
|---|---|
| Keys | Fresh `odc init` root has `ods:` + `odc:` only (no `ods-cli:`) |
| OKF keys | `odc init --okf` has `okf_version: "0.2"`; no invented OKF keys |
| Seamless | `odc lint` green on ODS fixture and OKF fixture without namespace |
| Explicit | `odc ods lint` / `odc okf lint` still work |
| Hybrid | Hybrid root lint runs both without panic |
| Upgrade | Fixture with `ods-cli:` → `odc upgrade --write` → `odc:` |
| Install docs | No primary path to `ods-cli` crate name |
| Config | New writes under `~/.odc` (after Phase C) |
| Report | Lint report path matches taught docs |
| Suite | Full workspace tests + clippy -D warnings green |

---

## 8. Out of scope

- Merging ODS and OKF frontmatter dialects into one schema
- Executing OKF executors/attesters (parse/lint/graph/audit only)
- Renaming nested document key `ods:` to `odc:`
- Renaming test tree `ods-test/` (optional later)
- Long dual-read of `ods-cli:` in normal parse (upgrade-only)

---

## 9. Suggested PR slice order

1. **Docs plan + key reference** (Phase A) — this file + frontmatter-keys + README banner  
2. **Install/action P0** (Phase B)  
3. **Error report + `~/.odc`** (Phase C) with tests  
4. **Docs/skills/specs sweep** (Phase D)  
5. **Internal polish + full CI** (Phase E)  
6. **Release + external cutover** (Phase F)

---

## 10. One-page cheat sheet

```
Tool:     odc
Formats:  ODS (ods: + nested ods) · OKF (okf_version + Google keys)
CLI pin:  odc: ">=x.y.z"     # not ods-cli:
OKF pin:  okf_version: "0.2" # not okf:
Bare UX:  odc lint | init | doctor | audit  → auto-detect
Force:    odc ods … | odc okf …
Alias:    ods … → ODS only
Home:     ~/.odc  (read ~/.ods during migration)
Report:   .odc/odc-errors.md
Upgrade:  odc upgrade --write  # ods-cli→odc, config hints
```
