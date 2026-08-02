---
profile: plan
status: stable
share: public
description: Final migration plan — ODS CLI to ODS multi-spec architecture, mandatory ods ods / ods okf namespaces, native OKF link, upgrade/audit, full touchpoints (tests/docs/CI/specs), and verification gates.
owner: team:opendocify
tags: [ods, ods, okf, migration, cli, architecture, plan, upgrade, audit, touchpoints]
---

# ODS to ODS Migration & Multi-Spec CLI Architecture Plan

> **SUPERSEDED for CLI UX:** product surface is flag-only — bare `ods` = ODS (no `--ods`); extra specs `--okf` / `--skills`. Namespaces (`ods okf`, `ods ods`) are deprecated. See session public-launch plan.


> **Superseded for CLI UX and root keys** by [`odc_tool_keys_legacy_cleanup.md`](./odc_tool_keys_legacy_cleanup.md).
> Bare `ods lint` **auto-detects**; CLI pin is **`ods:`** (not `ods-cli:`). This file remains useful for historical migration touchpoints.

## Direct answers

### Even with native OKF, do we need `ods okf` commands?

**Optional force, not mandatory for everyday use.** Native = OKF code lives in the **same** `ods-core` engine/binary. Bare `ods lint` auto-selects from root markers; `ods ods` / `ods okf` force an engine (preferred in hybrid CI).

| Layer | Meaning |
|---|---|
| Engine | One binary: ODS modules + OKF modules |
| CLI | Bare auto-detect **or** explicit namespaces |

```
ods lint         # auto-detect ODS / OKF / hybrid
ods okf lint     # force OKF engine
ods ods lint     # force ODS engine
```

Different specs → different required keys, root markers, and lint rules.

**Related:** [Tool / keys cleanup](./odc_tool_keys_legacy_cleanup.md) · [Native OKF support plan](./okf_native_support.md) · [ODS vs OKF key comparison](../../other-specs/frontmatter-keys-ods-vs-okf.md)

---

## Locked decisions

| Topic | Decision |
|---|---|
| CLI binary | `ods` → **`ods`** |
| ODS Markdown keys | Stay **`ods:` / `ods:`** only — never `ods-cli:` |
| Root ODS indexes (≈3 repos) | **Manual** edit |
| Dual-compat API | **Out** |
| Spec namespaces | **(historical row — superseded)** optional force `ods ods` / `ods okf` |
| Bare doc commands | **(historical — superseded)** auto-detect |
| Platform | `ods update`, `upgrade`, `setup`, `version`, `workspaces`, `skill` |
| OKF | **Native** in `ods-core` + **`ods okf *` CLI** |
| OKF keys | **All v0.2** frontmatter + bundle conventions |
| OKF runtime attesters | **Out** of v1 (parse/lint/audit/graph only) |
| Audit report | `.ods/ods-errors.md` via `ods ods audit` / `ods okf audit` |
| Touchpoints | All tests, docs, specs, CI, skills, install, site (§5) |
| Spec docs | Key comparison + purpose; update ODS specs CLI strings (§6) |

---

## 1. Naming policy

1. **ODS Markdown:** `ods:`, `ods:`, nested `ods:` — unchanged.
2. **OKF Markdown:** upstream OKF v0.2 keys (`type`, `sources`, `okf_version`, …) — not renamed to ODS keys.
3. **CLI binary:** `ods`. Optional `ods` → `ods` symlink maps to **`ods ods` only**.
4. **≈3 live ODS roots:** manual `index.md` / CI cutover.

---

## 2. Multi-spec CLI architecture

```
ods
├── ods ods <cmd>      # REQUIRED — ODS
├── ods okf <cmd>      # REQUIRED — OKF (native engine, separate commands)
├── ods agents <cmd>   # Agent graphs
└── ods <platform>     # update | upgrade | setup | version | workspaces | skill
```

### 2.1 Dispatch rules

1. Document ops **must** use second token `ods` | `okf` | `agents`.
2. Global `ods` never auto-detects which spec to lint/init.
3. Root markers **validate** the chosen namespace (wrong-spec → clear error).
4. Hybrid roots still require explicit namespace per command.

### 2.2 Command map

| Op | ODS | OKF | Platform |
|---|---|---|---|
| init | `ods ods init` → `ods:` + `ods:` | `ods okf init` → `okf_version: "0.2"` | — |
| lint / index / context / doctor | `ods ods …` | `ods okf …` | — |
| adopt / audit / fmt / export | `ods ods …` | `ods okf …` | — |
| serve / watch / start / stop | `ods ods …` | `ods okf …` as needed | — |
| update | — | — | `ods update` |
| upgrade | readiness hints | readiness hints | `ods upgrade` |
| workspaces / skill / setup | — | — | `ods …` |

Legacy surface: every old `ods <cmd>` → **`ods ods <cmd>`** (same flags/exit codes unless versioned removal).

### 2.3 Native OKF

Full design: **[okf_native_support.md](./okf_native_support.md)**.  
Key comparison for authors: **[frontmatter-keys-ods-vs-okf.md](../../other-specs/frontmatter-keys-ods-vs-okf.md)**.

---

## 3. `ods update` vs `ods upgrade`

| Command | Role |
|---|---|
| `ods update` | Binary self-update from GitHub Releases; restart service |
| `ods upgrade` | Dry-run default; `--write` machine forward (`~/.ods`); optional `--migrate-fm` for ODS canonical frontmatter; never invents `ods-cli:` |

```
ods upgrade [path]
ods upgrade [path] --write
ods upgrade [path] --check
ods upgrade [path] --migrate-fm
```

Exit: `0` clean, `1` work remaining/warnings, `2` usage, `3` hard failure.

### Typical cutover

```bash
# install ods
ods upgrade --write
ods upgrade --migrate-fm --write   # only if legacy flat ODS FM
ods ods audit --write-report
ods ods adopt --write
ods ods lint
# manual: ≈3 root index.md pins if needed
```

---

## 4. Audit (`ods ods audit` / `ods okf audit`)

```
ods ods audit [path] --write-report
ods okf audit [path] --write-report
ods ods audit [path] --fail-on plain|invalid|any
ods ods audit [path] --report-path <file>
```

Bare `ods audit` → usage error.

**Default report:** `.ods/ods-errors.md`  
**Classes:** compliant | plain | invalid | partial | skipped

| Command | Role |
|---|---|
| `ods ods lint` / `ods okf lint` | Schema correctness |
| `ods ods audit` / `ods okf audit` | Inventory + report file |
| `ods ods adopt` / `ods okf adopt` | Fix plains |
| `ods update` | Binary |
| `ods upgrade` | Forward cutover helpers |

---

## 5. All touchpoints (must update)

### 5.1 Code & tests

| Area | Work |
|---|---|
| `ods-core` → `ods-core` | OKF modules; ODS keys unchanged; shared scan |
| CLI crate | Dispatcher: `ods ods` / `ods okf` / platform |
| Unit tests | OKF key families; bare `verified`; unknown key preserve |
| Integration / production matrix | Binary `ods`; namespace required |
| CLI tests | **(superseded)** bare `ods lint` auto-detects; namespaced paths green |
| Fixtures | ODS fixtures + OKF mini-bundle + hybrid root |
| `ods-test/**` smoke | `ods ods …` |
| Post-rename | All imports / binary names |

### 5.2 Scripts, CI, packaging

| Area | Work |
|---|---|
| install.sh / install.ps1 / skill bootstrap | Install `ods`; optional `ods`→`ods` |
| GitHub Action / action.yml | `ods ods …` (document inputs) |
| smoke-ods.sh → smoke-ods.sh | Namespaced commands |
| check-local.sh | Update |
| Release assets | `ods-*` |
| Install one-liners / NPM docs | `ods` |

### 5.3 Product docs & skills

| Area | Work |
|---|---|
| README, CHANGELOG, CONTRIBUTING | `ods ods` / `ods okf`; update vs upgrade |
| app-web docs / quickstart / tooling | Multi-spec examples |
| skills (ODS/ODS) | Namespaced CLI; link OKF skill |
| New skills/okf | Authoring + `ods okf` |
| `--help` | Spec namespaces first |

### 5.4 Specs

See §6 and [frontmatter-keys-ods-vs-okf.md](../../other-specs/frontmatter-keys-ods-vs-okf.md).

### 5.5 Touchpoint acceptance

- [ ] Tests green under `ods` + namespaces  
- [x] Bare `ods lint` is primary (auto-detect); namespaces for CI/hybrid  
- [ ] CI/action uses namespaced commands  
- [ ] Key comparison doc published  
- [ ] ODS SPEC CLI strings use `ods ods`  
- [ ] OKF skill or docs section for `ods okf`  

---

## 6. Specs & key comparison

### 6.1 Comparison doc (required)

**[docs/specs/frontmatter-keys-ods-vs-okf.md](../../other-specs/frontmatter-keys-ods-vs-okf.md)** — when to use which spec, root markers, full key availability table, purpose of each family, CLI mapping.

### 6.2 ODS `specs/*` updates

| File | Updates |
|---|---|
| `specs/SPEC.md` | CLI → `ods ods …`; multi-spec pointer; link comparison; **do not** rename ODS key model |
| `specs/validation.md` | Namespaced lint; pointer to `ods okf lint` |
| `specs/graph.md`, `indexes.md`, `context.md`, `profiles.md`, `resources-and-code.md` | Examples → `ods ods …` |
| `specs/index.md` | Multi-spec + comparison entry |
| `specs/non-goals.md` | OKF is sibling native spec; runtime attestation still non-goal of ODS |

**v1:** OKF keys stay OKF-only; ODS SPEC stays ODS keys; comparison doc is the bridge. Unifying shared vocabulary = later ODS version bump, not sneaked into OKF-native work.

### 6.3 OKF reference

- Upstream [OKF SPEC v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md) is normative.  
- Optional local inventory: `docs/specs/okf-v0.2-keys.md`.  
- Engine detail: [okf_native_support.md](./okf_native_support.md).

---

## 7. Repository / crate migration

Phased PR stack (not whole-repo sed):

1. **Phase A** — crates/binary rename (`ods-core`, `ods`, …)  
2. **Phase B** — install, action, scripts, skills, `~/.ods`, `ODC_*`  
3. **Phase C** — namespace dispatcher, audit/upgrade, native OKF  
4. **Phase D** — docs/site, comparison doc, ODS specs CLI, ≈3-repo cutover  

Maintainer script: manifest renames; never rewrite user `ods:` keys.

---

## 8. Manual cutover (≈3 ODS repos)

1. List repos/owners.  
2. Confirm root `ods:` / `ods:`.  
3. CI → `ods ods lint` (etc.).  
4. Run audit/doctor once.  
5. No mass automated marker rewrite product.

---

## 9. Verification

| Scenario | Expected |
|---|---|
| `ods ods lint` on ODS fixture | Green / parity with old `ods lint` |
| `ods lint` | **(superseded)** auto-detect ODS/OKF/hybrid |
| `ods ods init` | Root has `ods:` + `ods:` only |
| `ods okf init` | Root has `okf_version: "0.2"` |
| OKF full-key fixture | `ods okf lint` green |
| Audit `--write-report` | `.ods/ods-errors.md` |
| Install + `ods update` | Works |
| Specs + comparison | CLI strings + key table published |

### Smoke

```bash
cargo test --workspace
ods ods lint && ods ods index --check && ods ods doctor
ods ods audit --write-report
ods upgrade --check
ods okf init /tmp/okf-demo && ods okf lint /tmp/okf-demo
```

### Release blockers

- [ ] Workspace tests green after rename  
- [ ] Namespaces enforced  
- [ ] ODS Markdown keys still `ods:` / `ods:`  
- [ ] OKF MVP gates (see OKF plan)  
- [ ] Touchpoints §5.5  
- [ ] Comparison + ODS SPEC CLI updates  
- [ ] ≈3-repo manual checklist  

---

## 10. Implementation phasing

```
Phase 0  Docs lock (this file + okf_native_support + key comparison)
Phase 1  Namespace dispatcher + ods ods path; audit/upgrade
Phase 2  Crate/binary rename; installers/CI
Phase 3  Native OKF (okf_native_support phases) + ods agents trail
Phase 4  All touchpoints §5; specs §6; site; 3-repo cutover; release
```

---

## 11. Out of scope

- Dual-read `ods-cli`/`ods-cli` or long dual config promise  
- Auto-detect primary lint path  
- Mass root rewrites  
- Running OKF executors/attesters in v1  
- Unifying ODS+OKF into one frontmatter dialect in v1 (comparison only)
