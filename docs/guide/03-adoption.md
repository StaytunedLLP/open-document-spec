---
description: "Enriching existing Markdown repositories progressively without migration overhead."
ods:
  profile: "note"
  status: "stable"
---

# Adopting ODS

ODS adoption is **enrichment**, never migration. Plain Markdown is already 100% valid (plain Markdown).

> [!TIP]
> **Zero-Effort Adoption in 10 Seconds**:
> Run `ods setup .` once. ODS writes **`ods.toml`**, can adopt plain Markdown, and can start the OS background service (`ods serve`). Discovery is CLI-only (`overview` / `find` / `tree` / `context`) — **no nested index lockfiles**. From that moment on, the service and your AI assistant (`ods skill`) help keep frontmatter and links consistent.

Empty tree? Use [Quickstart Guide](/docs/quickstart) first.

---

## Installing Tools Does Not Rewrite Your Repo

Installing `ods`  does **not** change Markdown until you opt in. Plain files stay plain Markdown.

| Goal | Command |
| --- | --- |
| Turn ODS on | `ods init .` or `ods init . --adopt` |
| Turn ODS off (dry-run) | `ods disable .` |
| Apply leave / strip metadata | `ods disable . --write` |

---

## Start in Place

| Step | Action |
| --- | --- |
| 1 | `ods setup` — check/update path and create or repair root `ods.toml` |
| 2 | `ods init .` — explicit opt-in (writes root `ods.toml`) |
| 3 | Optional: `ods init . --adopt` or `ods adopt --write` — draft `profile` + `status: draft` |
| 4 | Root `ignore` in `ods.toml` for code trees if needed, then `ods overview` / `ods find` |
| 5 | Add `depends` / `related` where you know relationships |
| 6 | `ods setup` or `ods start .` — ensure background service |
| 7 | CI: `ods lint` |

---

## Leaving ODS

```bash
ods disable .                 # dry-run
ods disable . --write         # strip ODS keys; keep prose
ods disable . --write --remove-indexes
```

Remove `ods lint` / `ods lint` from CI if you no longer want enforcement. Stop any service: `ods stop --unregister .`.

---

## Tooling Surface

| Command | Behavior |
| --- | --- |
| `ods setup` | Check updates, workspace state, service, and doctor |
| `ods init` | Create/ensure root `ods.toml` |
| `ods adopt` / `--write` | Dry-run or draft minimal frontmatter |
| `ods lint` | Validate graph / links (CI gate) |
| `ods overview` / `find` / `tree` | Progressive discovery (no nested indexes) |
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

- Do **not** commit nested `index.ods.md` lockfiles — discovery is CLI-only.
- With `ods start` or `ods watch`, renames keep refs and graph intact.
- Markdown language servers are optional and **not** required for ODS.
- Non-ODS frontmatter keys (SSG metadata) are preserved; mutations only touch ODS keys.

---

## Progressive Enrichment

| When | Add |
| --- | --- |
| Day 0 | Plain Markdown |
| Day 1 | Root `ods.toml` via `ods init` |
| Early | `profile` + `status` |
| As you link | `depends` / `related` |
| When skimming | `description` |
| Before agents | `context` / `ods export` |
| Full trust | CI lint (compliant) |

Full catalog: [Features](/docs/features).

---

## Next

- [Tooling Reference](/docs/tooling) · [Profiles & Catalogs](/docs/profiles) · [FAQ](/docs/faq)
