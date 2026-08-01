---
profile: plan
status: stable
share: public
description: Comparison of ODS vs Google OKF v0.2 frontmatter keys — which keys exist in which spec, purpose of each family, root markers, and seamless ods CLI.
owner: team:opendocify
tags: [ods, okf, frontmatter, keys, multi-spec, reference, ods]
---

# Frontmatter keys: ODS vs OKF v0.2

This document is the **author-facing map** of which metadata keys belong to which Open Document Spec spec, what they are for, and how the **`ods`** CLI routes them.

| Spec | CLI | Best for |
|---|---|---|
| **ODS** (Open Document Spec) | `ods …` (auto) or `ods ods <cmd>` | Codebase documentation graphs: profiles, depends/related, code refs, indexes |
| **OKF v0.2** (Open Knowledge Format) | `ods …` (auto) or `ods okf <cmd>` | Knowledge/metric/asset catalogs: trust, provenance, freshness, attested computations |

**Tool** = **`ods`**. **Native OKF** means both engines ship in one binary. Frontmatter dialects stay separate.

```
ods lint         # auto-detect ODS / OKF / hybrid from root markers
ods ods lint     # force ODS rules + ODS keys
ods okf lint     # force OKF rules + OKF keys
ods init         # ODS workspace (ods: + ods:)
ods init --okf   # OKF bundle (okf_version: "0.2")
```

**Source of truth for product naming:** [ods tool / keys / legacy cleanup plan](../plan/odc_tool_keys_legacy_cleanup.md).

**Normative references**

- ODS: [`specs/SPEC.md`](../../specs/SPEC.md) and sibling modules under `specs/`
- OKF: [Google OKF SPEC v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
- Implementation plans: [ODS migration](../plan/ods_to_odc_migration_and_cli_architecture.md), [Native OKF](../plan/okf_native_support.md)

---

## 1. When to use which spec

| Use ODS when… | Use OKF when… |
|---|---|
| Docs describe **software systems** (services, guides, RFCs, APIs) | Docs describe **data/knowledge assets** (tables, metrics, playbooks, policies) |
| You need **typed graph edges** (`depends`, `related`) and **code bindings** | You need **trust signals** (who generated/verified) and **staleness** |
| Progressive disclosure via **ODS indexes** + profiles | Progressive disclosure via OKF **index.md** / **log.md** conventions |
| Workspace root is an engineering repo | Bundle is a knowledge pack (often agent-maintained) |

**Hybrid repos** may carry both root markers.

| Bare command on hybrid | Behavior |
|---|---|
| `lint` / `doctor` / `audit` | Run **both** engines (ODS then OKF) |
| `index` / `fmt` / `export` / `context` / `watch` / `serve` / `adopt` | **Error** — use `ods ods …` or `ods okf …` (Hybrid R1) |
| ODS-only cmds (`mv`, `tags`, …) | ODS engine |

Use `ods ods` or `ods okf` whenever you mean one engine only.

---

## 2. Root markers (workspace / bundle identity)

| Key | Spec | Where | Purpose |
|---|---|---|---|
| `ods: <version>` e.g. `0.1` | ODS | Root `index.md` | Declares ODS workspace boundary and spec version |
| `ods: ">=x.y.z"` | ODS | Root `index.md` | Minimum CLI version required for this workspace |
| `okf_version: "0.2"` | OKF | Root `index.md` only | Declares OKF bundle targets v0.2 |
| Both | Hybrid | Root `index.md` | Both engines may apply; still use `ods ods` vs `ods okf` per command |

**Reserved filenames (OKF):** `index.md`, `log.md` are not concepts.  
**ODS:** root/non-root `index.md` follow ODS index rules (`profile: index`, etc.).

---

## 3. Key comparison table (availability)

Legend: **Yes** = first-class in that spec · **No** = not part of that spec’s model · **Related** = similar idea under a different shape.

| Key / concept | ODS | OKF v0.2 | Purpose (short) |
|---|---|---|---|
| **Root `ods:`** | Yes | No | ODS workspace version / boundary |
| **`ods:`** | Yes | No | Pin minimum CLI for ODS workspace |
| **Nested `ods.profile`** | Yes | No | Document shape (guide, api, feature, …) |
| **Nested `ods.status`** | Yes | Related* | ODS doc lifecycle (`draft` / `stable` / …) |
| **Nested `ods.id`** | Yes | No† | Stable id override (path is default id) |
| **Nested `ods.share`** | Yes | No | Publish/share policy for export |
| **Nested `ods.depends`** | Yes | No† | Hard dependency edges |
| **Nested `ods.related`** | Yes | No† | Soft relationship edges |
| **Nested `ods.resources`** | Yes | Related | Non-code resource refs |
| **Nested `ods.code`** | Yes | No | Code path bindings + roles |
| **Nested `ods.context`** | Yes | No | Context pack for `ods ods context` |
| **Universal top-level** (`title`, `description`, `owner`, `tags`, …) | Yes (SSG/CMS-friendly) | Partial (`title`, `description`, `tags` recommended) | Human/agent display metadata |
| **`type`** | No | **Required** | Concept kind (Metric, BigQuery Table, …) |
| **`resource`** | Via resources/code | Yes optional | Canonical URI of underlying asset |
| **`sources`** (+ credibility) | No (v1) | Yes | Provenance: what this concept was built from |
| **`usage_window`** | No | Yes | Window framing `usage_count` |
| **`generated` `{by,at}`** | No (v1) | Yes | Who/what wrote the concept, when |
| **`verified` `[{by,at}]`** | No (v1) | Yes | Independent confirmations; trust tier |
| **Top-level `status`** | No (use `ods.status`) | Yes | `draft` \| `stable` \| `deprecated` |
| **`stale_after`** | No (v1) | Yes | Absolute freshness deadline |
| **`runtime`** | No | Attested Computation | How the computation runs |
| **`parameters`** | No | Attested Computation | Named typed holes for the runtime |
| **`computation`** | No | Attested Computation | Path to computation file (alt to body) |
| **`executor`** | No | Attested Computation | How to run + receipt shape |
| **`attester`** | No | Attested Computation | Deterministic check of a run receipt |
| **`okf_version`** | No | Root | Bundle OKF version |
| **`timestamp`** (legacy) | No | Read fallback | v0.1; prefer `generated.at` |
| **Markdown links** | Yes | Yes | Relationships (OKF: untyped edges in prose) |
| **Unknown custom keys** | Preserve | Preserve | Extensions; must not be rejected |

\* **Status:** ODS keeps lifecycle under **`ods.status`**. OKF uses **top-level `status`**. Do not assume they are the same field in the engine without an explicit mapping.

† **IDs and edges:** OKF concept id is the path without `.md`. Relationships are ordinary markdown links, not `depends`/`related` arrays.

---

## 4. Purpose of key families

### 4.1 ODS engine keys (nested under `ods:`)

| Family | Purpose |
|---|---|
| **profile / status** | Tell tools and agents *what kind of doc* this is and whether it is ready |
| **id / share** | Stable identity and publish boundaries |
| **depends / related** | Build a **queryable graph** for reading order and impact |
| **code / resources** | Bind docs to real implementation and artifacts |
| **context** | Define bounded packs for prompt assembly (`ods ods context`) |
| **Root ods / ods** | Mark the tree as an ODS workspace and gate CLI version |

**Goal:** efficient, structured **engineering documentation** for humans and coding agents.

### 4.2 OKF identity & description

| Key | Purpose |
|---|---|
| **type** | Route/filter concepts (only required field) |
| **title / description / tags** | Discovery, indexes, previews |
| **resource** | Point at the real system object (table URI, API, …) |

### 4.3 OKF provenance (`sources`)

Records **what materials** a concept was derived from, with optional credibility signals (`author`, `usage_count`, `last_modified`) — **not** a stored credibility score. Footnotes `[^id]` join body claims to `sources[].id`.

### 4.4 OKF trust (`generated`, `verified`)

| Key | Purpose |
|---|---|
| **generated** | How the content was produced (agent/human/process) and when it last meaningfully changed |
| **verified** | Independent confirmations; consumers derive **trust tiers** (unverified → machine-confirmed → human-reviewed) |

Absence of `verified` is meaningful (unverified) but **not** a hard failure under OKF conformance.

### 4.5 OKF lifecycle (`status`, `stale_after`)

| Key | Purpose |
|---|---|
| **status** | draft → stable → deprecated |
| **stale_after** | Absolute date; concept is stale when `today >= stale_after` |

### 4.6 OKF Attested Computation

Answers: *was this number produced the way we said it must be?*  
Frontmatter carries the **contract** (`runtime`, `parameters`, `executor`, `attester`, optional `computation` path). Open Document Spec v1 **lints the contract**; it does not execute attesters.

### 4.7 Shared conventions

| Convention | Purpose |
|---|---|
| YAML frontmatter + Markdown body | Human-readable, agent-parseable, git-diffable |
| Preserve unknown keys | Allow extensions without forking the specs |
| Progressive disclosure indexes | Cheap navigation before loading full docs |

---

## 5. CLI mapping

| Goal | Command |
|---|---|
| Create ODS workspace | `ods init` or `ods ods init` |
| Create OKF bundle | `ods init --okf` or `ods okf init` |
| Validate (auto) | `ods lint` |
| Validate ODS only | `ods ods lint` |
| Validate OKF only | `ods okf lint` |
| Inventory non-compliant Markdown | `ods audit --write-report` → `.ods/ods-errors.md` |
| Draft frontmatter on plains | `ods adopt` / `ods ods adopt` / `ods okf adopt` |
| Resolve reading list | `ods context` / `ods ods context` / `ods okf context` |
| Binary self-update | `ods update` |
| Machine/workspace forward helpers | `ods upgrade` (`ods-cli:` → `ods:`, `~/.ods` → `~/.ods`) |

---

## 6. Example side-by-side

### ODS document (sketch)

```yaml
---
title: Checkout Setup
owner: team:payments
ods:
  profile: guide
  status: stable
  depends:
    - products/payments/index
  code:
    - path: src/checkout/mod.rs
      role: implementation
---
```

### OKF concept (sketch)

```yaml
---
type: Metric
title: Revenue
description: Recognized revenue for a fiscal year.
tags: [finance, revenue]
status: stable
generated: { by: reference_agent/gemini-2.5-pro, at: 2026-06-20T22:53:05Z }
verified: { by: human:reviewer, at: 2026-06-25T09:00:00Z }
stale_after: 2026-12-31
sources:
  - id: rev-policy
    resource: https://wiki.example/finance/revenue-recognition
    title: Revenue recognition policy
---
```

Same file format family (Markdown + YAML). **Different keys, different lint rules.** One tool (`ods`) routes by root markers or explicit namespace.

---

## 7. What v1 will not do

- Merge ODS and OKF into one frontmatter dialect  
- Invent `okf:` as a CLI pin (Google uses `okf_version:`)  
- Rename nested ODS engine keys from `ods:` to `ods:`  
- Run OKF executors/attesters  
- Silently map `ods.status` ↔ OKF `status` without a future explicit design  

If Open Document Spec later adopts shared optional keys into ODS (e.g. `stale_after`, `sources`), that is a **separate ODS version bump** documented in `specs/` — not implied by native OKF support alone.

---

## 8. See also

- ODS modules: [`specs/`](../../specs/) (`SPEC.md`, `graph.md`, `validation.md`, `profiles.md`, …)  
- Plans: [Tool / keys / legacy cleanup](../plan/odc_tool_keys_legacy_cleanup.md), [ODS migration](../plan/ods_to_odc_migration_and_cli_architecture.md), [Native OKF](../plan/okf_native_support.md)  
- Upstream OKF: [SPEC.md v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
