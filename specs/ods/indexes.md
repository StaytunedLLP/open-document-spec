---
description: "Indexes removed — workspace marker is ods.toml; discovery is CLI-only."
status: "stable"
order: 6
ods:
  profile: "note"
  status: "stable"
  depends:
    - keys.md
  related:
    - core.md
    - validation.md
---

# ODS · Workspace config & discovery

> **Breaking change:** nested `index.ods.md` navigation files are **removed**.
> The workspace marker is root **`ods.toml`**. Progressive discovery is via the `ods` CLI.

## 1. Workspace marker: `ods.toml`

```toml
# ods.toml — repository root only
spec = "0.1"

ignore = ["src", "target"]
custom_profiles = ["docs/profiles/rfc.md"]
packs = ["vendor/engineering-pack"]

[aliases]
Goal = ["Objective", "Purpose"]

[specs.okf]
enabled = false

[service]
mode = "poll"
poll_secs = 2
max_rss_mb = 10
```

| Field | Purpose |
| :--- | :--- |
| `spec` | Spec version (e.g. `"0.1"`) — required workspace marker |
| `ignore` | Path prefixes excluded from scan |
| `custom_profiles` | Profile catalog paths |
| `packs` | Imported pack roots |
| `aliases` | Section-heading synonyms |
| `specs.*` | Extra dialect activation |
| `service.*` | `ods serve` defaults (poll mode, RSS budget) |

`ods init` writes `ods.toml`. `ods init` migrates a legacy root index to `ods.toml` when present.

## 2. Progressive discovery (CLI only)

Do **not** commit folder indexes. Discover on demand:

```bash
ods overview
ods ls [path]
ods tree --depth 2
ods find --key status=draft
ods tag list
ods context <id> --max-tokens 2000
```

Cold-start for agents: `overview` → `find` / `tag` / `ls` → `context` (read **only** returned paths).

## 3. Incremental engine

File change handling is **incremental** (reparse changed frontmatter only) and discovery is **progressive** (bounded CLI queries). Service soft budget: **`service.max_rss_mb = 10`** by default.

## 4. Scan ignore defaults

Tools MUST still ignore base names: `target`, `node_modules`, `dist`, `build`, `.artifacts`, `.git`, `.hg`, `.svn`, `.jj`, `__pycache__`, `.venv`, `venv`, `vendor`, and names starting with `.`, plus `ods.toml` `ignore` prefixes.
