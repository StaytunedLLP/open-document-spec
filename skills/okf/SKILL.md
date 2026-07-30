---
name: okf
description: >-
  Author and validate Google OKF v0.2 knowledge bundles with OpenDocify (`odc okf`).
  Use when working with okf_version root markers, concept `type:` frontmatter,
  provenance/trust keys (sources, generated, verified), Attested Computation,
  or any `odc okf` command (init, lint, index, context, export, audit, watch).
---

# OKF v0.2 via OpenDocify (`odc okf`)

Native OpenDocify support for [Google Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md).

## CLI (mandatory namespace)

```bash
odc okf init [path]           # scaffold okf_version: "0.2" + sample concept
odc okf init --attested       # + Attested Computation stub
odc okf lint [path]           # validate concepts (type required)
odc okf index [path]          # progressive disclosure indexes
odc okf context [path] <id>   # concept + markdown link targets
odc okf export [path]         # okf-graph.md
odc okf fmt [path]            # normalize FM spacing (preserves unknown keys)
odc okf doctor [path]         # health / trust / stale
odc okf audit --write-report  # .odc/odc-errors.md
odc okf adopt --write         # draft type/title on plain .md
odc okf watch [path]          # re-lint on change
odc okf serve [path]          # headless re-lint poll loop
```

Bare `odc lint` is **not** valid — always use `odc okf …` or `odc ods …`.

## Key rules (summary)

| Key | Notes |
|---|---|
| `type` | **Required** on every concept |
| `title`, `description`, `resource`, `tags` | Recommended |
| `sources` / `usage_window` | Provenance + credibility signals |
| `generated` / `verified` | Trust; bare `verified: {by,at}` OK |
| `status` / `stale_after` | Lifecycle / freshness |
| `runtime` | **Required** when `type: Attested Computation` |
| `parameters`, `executor`, `attester`, `computation` | Attested Computation contract |
| `okf_version: "0.2"` | Root `index.md` only |

Unknown keys are preserved. Do **not** execute attesters/executors unless the user asks and tooling supports it.

## ODS vs OKF

| Spec | Namespace | Root marker |
|---|---|---|
| ODS (codebase docs) | `odc ods` | `ods:` + `ods-cli:` |
| OKF (knowledge) | `odc okf` | `okf_version: "0.2"` |

Full comparison: `docs/specs/frontmatter-keys-ods-vs-okf.md`.

## Agent notes

After structural edits: `odc okf lint`. For agent instruction files: `odc agents sync`.
