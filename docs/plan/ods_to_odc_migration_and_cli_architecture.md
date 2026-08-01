---
profile: plan
status: stable
share: public
description: Final migration plan — ODS CLI to ODC multi-spec architecture, mandatory odc ods / odc okf namespaces, native OKF link, upgrade/audit, full touchpoints (tests/docs/CI/specs), and verification gates.
owner: team:opendocify
tags: [odc, ods, okf, migration, cli, architecture, plan, upgrade, audit, touchpoints]
---

# ODS to ODC Migration & Multi-Spec CLI Architecture Plan

> **Superseded for CLI UX and root keys** by [`odc_tool_keys_legacy_cleanup.md`](./odc_tool_keys_legacy_cleanup.md).
> Bare `odc lint` **auto-detects**; CLI pin is **`odc:`** (not `ods-cli:`). This file remains useful for historical migration touchpoints.

## Direct answers

### Even with native OKF, do we need `odc okf` commands?

**Optional force, not mandatory for everyday use.** Native = OKF code lives in the **same** `odc-core` engine/binary. Bare `odc lint` auto-selects from root markers; `odc ods` / `odc okf` force an engine (preferred in hybrid CI).

| Layer | Meaning |
|---|---|
| Engine | One binary: ODS modules + OKF modules |
| CLI | Bare auto-detect **or** explicit namespaces |

```
odc lint         # auto-detect ODS / OKF / hybrid
odc okf lint     # force OKF engine
odc ods lint     # force ODS engine
```

Different specs → different required keys, root markers, and lint rules.

**Related:** [Tool / keys cleanup](./odc_tool_keys_legacy_cleanup.md) · [Native OKF support plan](./okf_native_support.md) · [ODS vs OKF key comparison](../specs/frontmatter-keys-ods-vs-okf.md)

---

## Locked decisions

| Topic | Decision |
|---|---|
| CLI binary | `ods` → **`odc`** |
| ODS Markdown keys | Stay **`ods:` / `odc:`** only — never `odc-cli:` |
| Root ODS indexes (≈3 repos) | **Manual** edit |
| Dual-compat API | **Out** |
| Spec namespaces | **(historical row — superseded)** optional force `odc ods` / `odc okf` |
| Bare doc commands | **(historical — superseded)** auto-detect |
| Platform | `odc update`, `upgrade`, `setup`, `version`, `workspaces`, `skill` |
| OKF | **Native** in `odc-core` + **`odc okf *` CLI** |
| OKF keys | **All v0.2** frontmatter + bundle conventions |
| OKF runtime attesters | **Out** of v1 (parse/lint/audit/graph only) |
| Audit report | `.odc/odc-errors.md` via `odc ods audit` / `odc okf audit` |
| Touchpoints | All tests, docs, specs, CI, skills, install, site (§5) |
| Spec docs | Key comparison + purpose; update ODS specs CLI strings (§6) |

---

## 1. Naming policy

1. **ODS Markdown:** `ods:`, `odc:`, nested `ods:` — unchanged.
2. **OKF Markdown:** upstream OKF v0.2 keys (`type`, `sources`, `okf_version`, …) — not renamed to ODS keys.
3. **CLI binary:** `odc`. Optional `ods` → `odc` symlink maps to **`odc ods` only**.
4. **≈3 live ODS roots:** manual `index.md` / CI cutover.

---

## 2. Multi-spec CLI architecture

```
odc
├── odc ods <cmd>      # REQUIRED — ODS
├── odc okf <cmd>      # REQUIRED — OKF (native engine, separate commands)
├── odc agents <cmd>   # Agent graphs
└── odc <platform>     # update | upgrade | setup | version | workspaces | skill
```

### 2.1 Dispatch rules

1. Document ops **must** use second token `ods` | `okf` | `agents`.
2. Global `odc` never auto-detects which spec to lint/init.
3. Root markers **validate** the chosen namespace (wrong-spec → clear error).
4. Hybrid roots still require explicit namespace per command.

### 2.2 Command map

| Op | ODS | OKF | Platform |
|---|---|---|---|
| init | `odc ods init` → `ods:` + `odc:` | `odc okf init` → `okf_version: "0.2"` | — |
| lint / index / context / doctor | `odc ods …` | `odc okf …` | — |
| adopt / audit / fmt / export | `odc ods …` | `odc okf …` | — |
| serve / watch / start / stop | `odc ods …` | `odc okf …` as needed | — |
| update | — | — | `odc update` |
| upgrade | readiness hints | readiness hints | `odc upgrade` |
| workspaces / skill / setup | — | — | `odc …` |

Legacy surface: every old `ods <cmd>` → **`odc ods <cmd>`** (same flags/exit codes unless versioned removal).

### 2.3 Native OKF

Full design: **[okf_native_support.md](./okf_native_support.md)**.  
Key comparison for authors: **[frontmatter-keys-ods-vs-okf.md](../specs/frontmatter-keys-ods-vs-okf.md)**.

---

## 3. `odc update` vs `odc upgrade`

| Command | Role |
|---|---|
| `odc update` | Binary self-update from GitHub Releases; restart service |
| `odc upgrade` | Dry-run default; `--write` machine forward (`~/.odc`); optional `--migrate-fm` for ODS canonical frontmatter; never invents `odc-cli:` |

```
odc upgrade [path]
odc upgrade [path] --write
odc upgrade [path] --check
odc upgrade [path] --migrate-fm
```

Exit: `0` clean, `1` work remaining/warnings, `2` usage, `3` hard failure.

### Typical cutover

```bash
# install odc
odc upgrade --write
odc upgrade --migrate-fm --write   # only if legacy flat ODS FM
odc ods audit --write-report
odc ods adopt --write
odc ods lint
# manual: ≈3 root index.md pins if needed
```

---

## 4. Audit (`odc ods audit` / `odc okf audit`)

```
odc ods audit [path] --write-report
odc okf audit [path] --write-report
odc ods audit [path] --fail-on plain|invalid|any
odc ods audit [path] --report-path <file>
```

Bare `odc audit` → usage error.

**Default report:** `.odc/odc-errors.md`  
**Classes:** compliant | plain | invalid | partial | skipped

| Command | Role |
|---|---|
| `odc ods lint` / `odc okf lint` | Schema correctness |
| `odc ods audit` / `odc okf audit` | Inventory + report file |
| `odc ods adopt` / `odc okf adopt` | Fix plains |
| `odc update` | Binary |
| `odc upgrade` | Forward cutover helpers |

---

## 5. All touchpoints (must update)

### 5.1 Code & tests

| Area | Work |
|---|---|
| `ods-core` → `odc-core` | OKF modules; ODS keys unchanged; shared scan |
| CLI crate | Dispatcher: `odc ods` / `odc okf` / platform |
| Unit tests | OKF key families; bare `verified`; unknown key preserve |
| Integration / production matrix | Binary `odc`; namespace required |
| CLI tests | **(superseded)** bare `odc lint` auto-detects; namespaced paths green |
| Fixtures | ODS fixtures + OKF mini-bundle + hybrid root |
| `ods-test/**` smoke | `odc ods …` |
| Post-rename | All imports / binary names |

### 5.2 Scripts, CI, packaging

| Area | Work |
|---|---|
| install.sh / install.ps1 / skill bootstrap | Install `odc`; optional `ods`→`odc` |
| GitHub Action / action.yml | `odc ods …` (document inputs) |
| smoke-ods.sh → smoke-odc.sh | Namespaced commands |
| check-local.sh | Update |
| Release assets | `odc-*` |
| Install one-liners / NPM docs | `odc` |

### 5.3 Product docs & skills

| Area | Work |
|---|---|
| README, CHANGELOG, CONTRIBUTING | `odc ods` / `odc okf`; update vs upgrade |
| app-web docs / quickstart / tooling | Multi-spec examples |
| skills (ODS/ODC) | Namespaced CLI; link OKF skill |
| New skills/okf | Authoring + `odc okf` |
| `--help` | Spec namespaces first |

### 5.4 Specs

See §6 and [frontmatter-keys-ods-vs-okf.md](../specs/frontmatter-keys-ods-vs-okf.md).

### 5.5 Touchpoint acceptance

- [ ] Tests green under `odc` + namespaces  
- [x] Bare `odc lint` is primary (auto-detect); namespaces for CI/hybrid  
- [ ] CI/action uses namespaced commands  
- [ ] Key comparison doc published  
- [ ] ODS SPEC CLI strings use `odc ods`  
- [ ] OKF skill or docs section for `odc okf`  

---

## 6. Specs & key comparison

### 6.1 Comparison doc (required)

**[docs/specs/frontmatter-keys-ods-vs-okf.md](../specs/frontmatter-keys-ods-vs-okf.md)** — when to use which spec, root markers, full key availability table, purpose of each family, CLI mapping.

### 6.2 ODS `specs/*` updates

| File | Updates |
|---|---|
| `specs/SPEC.md` | CLI → `odc ods …`; multi-spec pointer; link comparison; **do not** rename ODS key model |
| `specs/validation.md` | Namespaced lint; pointer to `odc okf lint` |
| `specs/graph.md`, `indexes.md`, `context.md`, `profiles.md`, `resources-and-code.md` | Examples → `odc ods …` |
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

1. **Phase A** — crates/binary rename (`odc-core`, `odc`, …)  
2. **Phase B** — install, action, scripts, skills, `~/.odc`, `ODC_*`  
3. **Phase C** — namespace dispatcher, audit/upgrade, native OKF  
4. **Phase D** — docs/site, comparison doc, ODS specs CLI, ≈3-repo cutover  

Maintainer script: manifest renames; never rewrite user `ods:` keys.

---

## 8. Manual cutover (≈3 ODS repos)

1. List repos/owners.  
2. Confirm root `ods:` / `odc:`.  
3. CI → `odc ods lint` (etc.).  
4. Run audit/doctor once.  
5. No mass automated marker rewrite product.

---

## 9. Verification

| Scenario | Expected |
|---|---|
| `odc ods lint` on ODS fixture | Green / parity with old `ods lint` |
| `odc lint` | **(superseded)** auto-detect ODS/OKF/hybrid |
| `odc ods init` | Root has `ods:` + `odc:` only |
| `odc okf init` | Root has `okf_version: "0.2"` |
| OKF full-key fixture | `odc okf lint` green |
| Audit `--write-report` | `.odc/odc-errors.md` |
| Install + `odc update` | Works |
| Specs + comparison | CLI strings + key table published |

### Smoke

```bash
cargo test --workspace
odc ods lint && odc ods index --check && odc ods doctor
odc ods audit --write-report
odc upgrade --check
odc okf init /tmp/okf-demo && odc okf lint /tmp/okf-demo
```

### Release blockers

- [ ] Workspace tests green after rename  
- [ ] Namespaces enforced  
- [ ] ODS Markdown keys still `ods:` / `odc:`  
- [ ] OKF MVP gates (see OKF plan)  
- [ ] Touchpoints §5.5  
- [ ] Comparison + ODS SPEC CLI updates  
- [ ] ≈3-repo manual checklist  

---

## 10. Implementation phasing

```
Phase 0  Docs lock (this file + okf_native_support + key comparison)
Phase 1  Namespace dispatcher + odc ods path; audit/upgrade
Phase 2  Crate/binary rename; installers/CI
Phase 3  Native OKF (okf_native_support phases) + odc agents trail
Phase 4  All touchpoints §5; specs §6; site; 3-repo cutover; release
```

---

## 11. Out of scope

- Dual-read `ods-cli`/`odc-cli` or long dual config promise  
- Auto-detect primary lint path  
- Mass root rewrites  
- Running OKF executors/attesters in v1  
- Unifying ODS+OKF into one frontmatter dialect in v1 (comparison only)
