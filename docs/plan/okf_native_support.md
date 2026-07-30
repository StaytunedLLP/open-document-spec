---
profile: plan
status: stable
share: public
description: Native Google OKF v0.2 in odc-core — full frontmatter vocabulary, mandatory odc okf CLI namespace (even though native), phases, touchpoints, and acceptance gates.
owner: team:opendocify
tags: [odc, okf, native, plan, lint, audit, multi-spec]
---

# Native Google OKF v0.2 Support in OpenDocify (`odc-core` / `odc okf`)

## Direct answer: native + separate CLI?

**Yes — both.**

| | |
|---|---|
| **Native** | OKF parse/lint/model lives **inside** `odc-core` (same binary as ODS) |
| **Separate CLI** | Users **must** call **`odc okf <command>`** — never rely on bare `odc lint` auto-detect |

```
odc okf lint     # correct
odc ods lint     # ODS only
odc lint         # usage error
```

**Why:** native is about *where the code lives*; namespaces are about *which spec rules apply*. See [migration plan](./ods_to_odc_migration_and_cli_architecture.md) and [key comparison](../specs/frontmatter-keys-ods-vs-okf.md).

---

## Decision (locked)

| Question | Answer |
|---|---|
| Support OKF natively? | **Yes** — `odc-core` |
| Invoke OKF | **`odc okf <command>`** (mandatory) |
| Invoke ODS | **`odc ods <command>`** (mandatory) |
| Keys | **All OKF v0.2** frontmatter + bundle conventions |
| Out of scope v1 | Execute executors/attesters, receipt wire protocol, attester sandbox |

**Normative upstream:** [OKF SPEC v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md).

---

## 1. Architecture

```
odc (CLI)
├── odc ods <cmd>     → ODS engine
├── odc okf <cmd>     → OKF engine (this plan)
├── odc agents <cmd>
└── odc <platform>    → update | upgrade | setup | version | workspaces | skill

odc-core
├── ods/   (existing)
├── okf/   (new)
│   ├── model.rs | parse.rs | lint.rs | bundle.rs
│   ├── trust.rs | paths.rs | audit.rs
└── shared: fs scan, markdown links, export helpers
```

| Root marker | Valid for |
|---|---|
| `ods:` (+ `ods-cli:`) | `odc ods *` |
| `okf_version: "0.2"` | `odc okf *` |
| both | hybrid; explicit namespace per command |

---

## 2. Complete OKF v0.2 key support

### 2.1 Bundle / reserved

| Item | Support |
|---|---|
| Root `okf_version: "0.2"` | `odc okf init` write; lint warn if missing |
| `index.md` | Reserved; progressive disclosure |
| `log.md` | Reserved; not a concept |
| `references/` | Convention; normal path resolve |

### 2.2 Identity

| Key | Required |
|---|---|
| `type` | **Yes** (only always-required concept key) |
| `title`, `description`, `resource`, `tags` | Recommended / optional |

### 2.3 Provenance

`sources[]` with `resource` (required in entry), `id`, `title`, `author`, `usage_count`, `last_modified`, optional per-entry `usage_window`; sibling `usage_window: { from, to }`.

### 2.4 Trust

`generated: { by, at }` · `verified` list **or** bare map · derived trust tier (unverified / machine-confirmed / human-reviewed) — advisory only.

### 2.5 Lifecycle

`status`: draft | stable | deprecated (absent ⇒ stable) · `stale_after`: YYYY-MM-DD.

### 2.6 Attested Computation (`type: Attested Computation`)

| Key | Required |
|---|---|
| `runtime` | **Yes for this type** |
| `parameters[]` | Optional `{ name, type, required }` |
| `computation` | Optional path vs body `# Computation` |
| `executor.resource` / `executor.receipt` | Optional |
| `attester.resource` | Optional |

Plus all provenance/trust/lifecycle keys on the same concept.

### 2.7 Legacy read

`timestamp` → fallback for `generated.at` · body `# Citations` optional.

### 2.8 Conformance (SPEC §11)

- Preserve unknown keys  
- Do not reject unknown `type`, broken links, missing optional families  
- Shape-check when families present  
- Actors: `<producer>/<version>`, `human:<id>`, `process:<id>`  
- Paths: URL | `/bundle-relative` | relative  

### 2.9 Non-goals v1

Execute executor/attester · central type registry · replace domain schemas.

**Authors:** purpose and ODS-vs-OKF availability → [frontmatter-keys-ods-vs-okf.md](../specs/frontmatter-keys-ods-vs-okf.md).

---

## 3. CLI surface (`odc okf`)

| Command | Behavior |
|---|---|
| `odc okf init` | Scaffold + `okf_version: "0.2"` |
| `odc okf lint` | Full shape + conformance |
| `odc okf index` / `--check` | Progressive disclosure indexes |
| `odc okf audit --write-report` | Inventory → `.odc/odc-errors.md` |
| `odc okf adopt [--write]` | Draft `{ type, title }` |
| `odc okf context` | Link-based reading list |
| `odc okf doctor` | Version, stale, trust tiers, missing `runtime` |
| `odc okf export` | Concept graph |
| `odc okf fmt` | Normalize spacing; never drop keys |
| `odc okf watch` / `serve` | Optional later |

---

## 4. Data model (sketch)

```text
OkfBundle { root, okf_version, concepts, indexes, logs }
OkfDocument { path, concept_id, frontmatter, body, unknown, links }
OkfFrontmatter {
  type_name, title, description, resource, tags,
  sources, usage_window, generated, verified,
  status, stale_after,
  runtime, parameters, computation, executor, attester,
  timestamp  // legacy
}
```

---

## 5. Lint levels

| Level | Checks |
|---|---|
| L1 | Parseable FM; non-empty `type`; reserved files; Attested Computation has `runtime` |
| L3 (default) | L1 + shapes; date formats; soft path warnings; stale/trust warnings |

Exit: `0` clean, `1` diagnostics, `2` usage.

---

## 6. Phases

```
OKF-0  Docs (this plan + comparison doc + migration link)
OKF-1  Model + parse + lint + unit tests
OKF-2  CLI MVP: init, lint, audit, doctor + fixture bundle
OKF-3  index, adopt, context, export, fmt + hybrid fixture
OKF-4  Polish, skills/okf, website, full touchpoints
```

May land under current `ods-core` crate name before binary rename; product docs always say `odc okf`.

---

## 7. Touchpoints for OKF work

| Area | Update |
|---|---|
| Tests | Golden parse per key family; CLI namespace; wrong-spec errors |
| Fixtures | Mini acme_retail-style bundle under e.g. `ods-test/` or `odc-test/` |
| README / app-web / CHANGELOG | `odc okf` examples |
| skills/okf | New skill package |
| CI | Optional job: `odc okf lint` on OKF fixture |
| Specs | Comparison doc; ODS specs do not absorb OKF keys in v1 |

Full multi-spec touchpoint list: [migration plan §5](./ods_to_odc_migration_and_cli_architecture.md).

---

## 8. Verification & acceptance

- [ ] All §2 keys parse into model  
- [ ] only-`type` concept valid  
- [ ] bare `verified` map accepted  
- [ ] missing `runtime` on Attested Computation → error  
- [ ] unknown keys preserved  
- [ ] `odc okf init|lint|audit|doctor` shipped  
- [ ] Fixture green in CI  
- [ ] `odc lint` without namespace → usage error  
- [ ] Wrong-spec errors for `odc okf` on pure ODS and reverse  
- [ ] Comparison doc lists OKF keys + purposes  

```bash
odc okf init /tmp/okf-demo
odc okf lint /tmp/okf-demo
odc okf audit /tmp/okf-demo --write-report
odc okf doctor /tmp/okf-demo
odc lint   # must fail with hint
```

---

## 9. Defaults

| Topic | Default |
|---|---|
| Root version key | `okf_version: "0.2"` only |
| Bare doc command | Hard error |
| Stale | Lint warning; doctor counts |
| Trust tier | Advisory only |
| Report path | `.odc/odc-errors.md` |
