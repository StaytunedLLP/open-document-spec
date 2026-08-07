---
description: "OKF v0.2 frontmatter keys: purpose and examples as used by the ods CLI."
ods:
  profile: "note"
  status: "stable"
  depends:
    - intro.md
---

# OKF · Keys

Author-facing dictionary for **Google OKF v0.2** keys recognized by `ods … --okf`. This is the ODS CLI dialect view—not a fork of Google’s document. Full upstream prose: `docs/other-specs/okf-0.2.md`.

---

## Root marker

| Key | Required | Purpose | Example |
| :--- | :---: | :--- | :--- |
| `okf_version` | Yes (root) | Bundle targets OKF v0.2 | `okf_version: "0.2"` |

```yaml
---
okf_version: "0.2"
---

# Knowledge Bundle
```

---

## Identity & description

| Key | Required | Purpose | Example |
| :--- | :---: | :--- | :--- |
| `type` | **Yes** | Concept kind (routes/filters) | `type: Metric` |
| `title` | No | Display title (OKF allows; ODS forbids FM title) | `title: Monthly churn` |
| `description` | No | Short summary | `description: 30-day churn rate` |
| `tags` | No | Free-form facets | `tags: [finance, kpi]` |
| `resource` | No | Canonical URI of underlying asset | `resource: bigquery://proj.dataset.table` |

```yaml
---
type: Metric
title: Monthly churn
description: Fraction of customers who cancel in a calendar month.
tags:
  - finance
  - kpi
resource: bigquery://acme.analytics.churn_monthly
---
```

---

## Provenance (`sources`)

Records **what materials** a concept was derived from (not a stored credibility score). Optional credibility signals: `author`, `usage_count`, `last_modified`. Body footnotes `[^id]` join claims to `sources[].id`.

```yaml
sources:
  - id: warehouse
    uri: bigquery://acme.analytics.events
    credibility:
      author: data-platform
      usage_count: 12
```

---

## Trust (`generated`, `verified`)

| Key | Purpose |
| :--- | :--- |
| `generated` | Who/what produced the content and when (`by`, `at`) |
| `verified` | Independent confirmations; consumers derive trust tiers |

```yaml
generated:
  by: agent/catalog-writer
  at: 2026-01-15T12:00:00Z
verified:
  - by: human/alice
    at: 2026-01-16T09:00:00Z
```

Absence of `verified` means unverified—not necessarily a hard lint failure.

---

## Lifecycle

| Key | Purpose | Values / notes |
| :--- | :--- | :--- |
| `status` | Lifecycle (top-level in OKF) | `draft` \| `stable` \| `deprecated` |
| `stale_after` | Absolute freshness deadline | Concept stale when `today >= stale_after` |

> **ODS contrast:** ODS lifecycle is **`ods.status`**, not top-level `status`. Do not assume engines map them without an explicit rule.

---

## Attested Computation

Contract keys (linted by Open Document Spec tooling; not executed by the core CLI):

| Key | Purpose |
| :--- | :--- |
| `runtime` | How the computation runs |
| `parameters` | Named typed holes for the runtime |
| `computation` | Path to computation file (alternative to body) |
| `executor` | How to run + receipt shape |
| `attester` | Deterministic check of a run receipt |

```yaml
type: Attested Computation
runtime: python
parameters:
  - name: year
    type: integer
    required: true
```

Suppress key checks if needed:

```bash
ods lint --okf --skip-frontmatter-keys
ods lint --okf --ignore-keys runtime,sources
```

Or root ODS workspace:

```yaml
specs:
  okf:
    enabled: true
    lint:
      check_keys: false
      ignore_keys: [runtime, sources]
```

---

## Shared conventions

- YAML frontmatter + Markdown body
- Preserve unknown keys
- Progressive disclosure via indexes
- Relationships primarily via Markdown links (not ODS `depends`/`related` arrays)
