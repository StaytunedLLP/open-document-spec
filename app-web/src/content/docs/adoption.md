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
> Run `ods setup .` once. ODS automatically adopts existing files, starts the OS background service (`ods serve`), and indexes your workspace. From that moment on, the background service and your AI assistant (`ods skill`) maintain frontmatter, indexes, and links **automatically with zero extra effort from you**.

Empty tree? Use [Quickstart Guide](/docs/quickstart) first.

---

## Installing Tools Does Not Rewrite Your Repo

Installing `ods` does **not** change Markdown until you opt in. Plain files stay Level 0.

| Goal | Command |
| --- | --- |
| Turn ODS on | `ods init .` or `ods init . --adopt` |
| Turn ODS off (dry-run) | `ods disable .` |
| Apply leave / strip metadata | `ods disable . --write` |

---

## Start in Place

| Step | Action |
| --- | --- |
| 1 | `ods setup` — check/update path and create or repair root `ods:` / `ods-cli:` |
| 2 | `ods init .` — explicit opt-in alternative for root `ods:` + indexes |
| 3 | Optional: `ods init . --adopt` or `ods adopt --write` — draft `profile` + `status: draft` |
| 4 | Root `ignore:` for code trees if needed, then `ods index` |
| 5 | Add `depends` / `related` where you know relationships |
| 6 | `ods setup` or `ods start .` — ensure background service |
| 7 | CI: `ods index --check` + `ods lint` |

---

## Leaving ODS

```bash
ods disable .                 # dry-run
ods disable . --write         # strip ODS keys; keep prose
ods disable . --write --remove-indexes
```

Remove `ods lint` / `ods index --check` from CI if you no longer want enforcement. Stop any service: `ods stop --unregister .`.

---

## Tooling Surface

| Command | Behavior |
| --- | --- |
| `ods setup` | Check updates, workspace state, service, and doctor |
| `ods init` | Create/ensure root `index.md` + `ods:` |
| `ods adopt` / `--write` | Dry-run or draft minimal frontmatter |
| `ods index` / `--check` | Generate indexes / CI stale check |
| `ods lint` | Validate |
| `ods start` / `stop` | Background watch service |
| `ods watch` | Foreground automation |
| `ods mv` | Offline move + rewrite |
| `ods sync` | Reconcile git-tracked renames |
| `ods fmt` | Normalize frontmatter/body spacing |
| `ods tags` / `find` | Inspect tags and find tagged Documents |
| `ods export` | Optional AI graph file |
| `ods share` | Publish a share-filtered directory to git-publish yourself |
| `ods graph` / `context` | Edges / reading list |

Practical rules:

- Generated index **child lists** beat hand-maintained maps.
- With `ods start` or `ods watch`, renames keep refs and indexes current.
- Markdown language servers are optional and **not** required for ODS.

---

## Progressive Enrichment

| When | Add |
| --- | --- |
| Day 0 | Plain Markdown |
| Day 1 | Root `index.md` + `ods:` via `ods init` |
| Early | `profile` + `status` |
| As you link | `depends` / `related` |
| When skimming | `description` |
| Before agents | `context` / `ods export` |
| Full trust | Level 3 CI |

Full catalog: [Features](/docs/features).

---

## Next

- [Tooling Reference](/docs/tooling) · [Profiles & Catalogs](/docs/profiles) · [FAQ](/docs/faq)
