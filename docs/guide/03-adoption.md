---
title: "Adopting ODS"
description: "Enriching existing Markdown repositories progressively without migration overhead."
status: "stable"
order: 3
ods:
  profile: "note"
  status: "stable"
---

# Adopting ODS

ODS adoption is **enrichment**, never migration. Plain Markdown is already 100% valid (Level 0).

> [!TIP]
> **Zero-Effort Adoption in 10 Seconds**:
> Run `odc setup .` once. ODS automatically adopts existing files, starts the OS background service (`odc ods serve`), and indexes your workspace. From that moment on, the background service and your AI assistant (`odc skill`) maintain frontmatter, indexes, and links **automatically with zero extra effort from you**.

Empty tree? Use [Quickstart Guide](/docs/quickstart) first.

---

## Installing Tools Does Not Rewrite Your Repo

Installing `odc` (legacy `ods`) does **not** change Markdown until you opt in. Plain files stay Level 0.

| Goal | Command |
| --- | --- |
| Turn ODS on | `odc ods init .` or `odc ods init . --adopt` |
| Turn ODS off (dry-run) | `odc ods disable .` |
| Apply leave / strip metadata | `odc ods disable . --write` |

---

## Start in Place

| Step | Action |
| --- | --- |
| 1 | `odc setup` — check/update path and create or repair root `ods:` / `odc:` |
| 2 | `odc ods init .` — explicit opt-in alternative for root `ods:` + indexes |
| 3 | Optional: `odc ods init . --adopt` or `odc ods adopt --write` — draft `profile` + `status: draft` |
| 4 | Root `ignore:` for code trees if needed, then `odc ods index` |
| 5 | Add `depends` / `related` where you know relationships |
| 6 | `odc setup` or `odc ods start .` — ensure background service |
| 7 | CI: `odc ods index --check` + `odc ods lint` |

---

## Leaving ODS

```bash
odc ods disable .                 # dry-run
odc ods disable . --write         # strip ODS keys; keep prose
odc ods disable . --write --remove-indexes
```

Remove `odc ods lint` / `odc ods index --check` from CI if you no longer want enforcement. Stop any service: `odc ods stop --unregister .`.

---

## Tooling Surface

| Command | Behavior |
| --- | --- |
| `odc setup` | Check updates, workspace state, service, and doctor |
| `odc ods init` | Create/ensure root `index.md` + `ods:` |
| `odc ods adopt` / `--write` | Dry-run or draft minimal frontmatter |
| `odc ods index` / `--check` | Generate indexes / CI stale check |
| `odc ods lint` | Validate |
| `odc ods start` / `stop` | Background watch service |
| `odc ods watch` | Foreground automation |
| `odc ods mv` | Offline move + rewrite |
| `odc ods sync` | Reconcile git-tracked renames |
| `odc ods fmt` | Normalize frontmatter/body spacing |
| `odc ods tags` / `find` | Inspect tags and find tagged Documents |
| `odc ods export` | Optional AI graph file |
| `odc ods share` | Publish a share-filtered directory to git-publish yourself |
| `odc ods graph` / `context` | Edges / reading list |

Practical rules:

- Generated index **child lists** beat hand-maintained maps.
- With `odc ods start` or `odc ods watch`, renames keep refs and indexes current.
- Markdown language servers are optional and **not** required for ODS.

---

## Progressive Enrichment

| When | Add |
| --- | --- |
| Day 0 | Plain Markdown |
| Day 1 | Root `index.md` + `ods:` via `odc ods init` |
| Early | `profile` + `status` |
| As you link | `depends` / `related` |
| When skimming | `description` |
| Before agents | `context` / `odc ods export` |
| Full trust | Level 3 CI |

Full catalog: [Features](/docs/features).

---

## Next

- [Tooling Reference](/docs/tooling) · [Profiles & Catalogs](/docs/profiles) · [FAQ](/docs/faq)
