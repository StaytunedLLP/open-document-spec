---
name: ods
description: >-
  Install, run, update, and author with Open Document Spec (`ods`). ODS is the default
  engine (no --ods flag). Extra specs: --okf (Google OKF v0.2), --skills (Agent Skills).
  Use ods lsp for JSON-RPC editor support; ods lint / ods lint --okf / ods lint --skills;
  never use ods okf or ods ods namespaces.
---

# ODS — Open Document Spec

## 0. Glossary & Core Concepts

| Term | Meaning |
|---|---|
| **`ods`** | Unified native Rust CLI binary and engine (`ods`) |
| **`ods lsp`** | Native JSON-RPC 2.0 Language Server built into `ods` for Zed, VS Code, Neovim, Cursor |
| **`ods:` frontmatter** | ODS format root/nested engine keys (`ods: { profile: rfc, status: draft }`) |
| **`custom-profiles:`** | Root `index.md` array declaring custom profile schema definitions |
| **`ods context`** | Sub-5ms bounded AI context reader following graph `depends` / `related` chains |
| **`ods export graph`** | Single canonical graph exporter returning structured JSON (`--format json`), Markdown (`--format md`), or text (`--format text`) for `--spec ods` and `--spec okf` |

ODS is plain Markdown with **permissive** YAML frontmatter (`title:` and `name:` both supported), powered by a native Rust engine binary named **`ods`**. A **workspace** is any directory tree whose **root `index.md` carries an `ods:` key** (e.g. `ods: 0.1`).

---

## Multi-spec flags (locked)

| Flag | Meaning |
|---|---|
| *(none)* | **ODS** — default native product of this CLI |
| `--okf` | Enable **Google OKF v0.2** engine for this command |
| `--skills` | Enable **Agent Skills** package engine for this command |

There is **no** `--ods` flag and **no** `ods okf` / `ods ods` namespaces (`ods okf` is hard-removed; use flags).

---

## 1. 🤖 5W1H Directives for AI Agents

When assisting users inside an ODS workspace, follow these operational directives:

- ❓ **WHAT**: Recognize ODS workspaces by checking for `ods:` in root `index.md`. Keep files as `.md`. Permissively accept `title:` or `name:` at top-level frontmatter.
- 💡 **WHY**: Use `ods context <doc-id-or-path>` or `ods export graph --format json` whenever responding to complex requests to retrieve a sub-5ms bounded context graph or full JSON graph without token bloat.
- 👥 **WHO**: Operate seamlessly on behalf of non-technical or developer users without requiring manual terminal commands.
- 📍 **WHERE**: Inspect `.ods/profiles/` and `custom-profiles:` in root `index.md` to understand registered profile section schemas.
- ⏰ **WHEN**: Always run `ods lint` (or `ods lint --fix` to auto-repair missing keys) after creating/editing documents to ensure zero broken links. Use `ods mv <src> <dst>` when moving files to auto-heal graph dependencies.
- 🛠️ **HOW**: Execute `ods` commands directly via shell or scripts. Use `ods update` to ensure end-user CLI freshness.

---

## 2. End-User Installation & Machine Setup

The primary method to run ODS is via the single native `ods` CLI binary:

- **Linux / macOS**: Run `scripts/install.sh` or `curl -fsSL https://raw.githubusercontent.com/.../install.sh | bash`
- **Windows**: Run `scripts/install.ps1` or `iwr -useb https://raw.githubusercontent.com/.../install.ps1 | iex`
- **Self-Update**: `ods update` checks GitHub releases, self-updates the binary, and restarts any running background service.
- **Workspace Setup**: `ods setup [path]` initializes workspace boundaries, verifies version compatibility, and registers OS service.

---

## 3. Core Frontmatter & Key Placement Rules

1. **Custom Domain Keys (`title`, `name`, `author`, `reviewer`, `target_release`, `service`, `team`, `tags`)**:
   - Placed at the **top-level** of frontmatter (outside `ods:`). Permissively supports both `title:` and `name:`.
   - Keeps custom metadata clean, human-readable, and compatible with Static Site Generators (Docusaurus, Astro, Hugo, Next.js).
2. **ODS Engine Keys (`profile`, `status`, `id`, `share`, `depends`, `related`, `code`, `resources`)**:
   - Nested inside the **`ods:` map** (`ods.profile: rfc`, `ods.status: draft`).

```yaml
---
title: "Distributed Redis Caching Strategy"
name: "cache_strategy"
author: "Alice Smith"
reviewer: "Bob Jones"
target_release: "v2.4"
ods:
  profile: rfc
  status: draft
  depends:
    - docs/architecture/cache_overview.md
  related:
    - docs/specs/api_endpoint.md
---
```

---

## 4. Custom Profiles Specification

Custom profiles define domain schemas, required frontmatter keys, and starter templates.

### Single-Source Registration (`custom-profiles:` in Root `index.md`)

Custom profiles MUST be explicitly registered in root `index.md` under `custom-profiles:`:

```yaml
---
profile: index
ods: 0.1
custom-profiles:
  - .ods/profiles/rfc.md
  - docs/profiles/api_endpoint.md
---
```

---

## 5. Editor JSON-RPC Language Server (`ods lsp`)

`ods lsp` serves standard Language Server Protocol (JSON-RPC 2.0) over **stdio** (default) or **TCP socket** (`--port <PORT>`).

### Zed Configuration (`.zed/settings.json`)
```json
{
  "languages": {
    "Markdown": {
      "language_servers": ["ods-lsp"],
      "enable_language_server": true
    }
  },
  "lsp": {
    "ods-lsp": {
      "binary": {
        "path": "ods",
        "arguments": ["lsp"]
      }
    }
  }
}
```

### VS Code & Neovim
- **VS Code**: Configure path `ods`, args `["lsp"]`.
- **Neovim**: Configure `cmd = { "ods", "lsp" }`.

---

## 6. Complete CLI Workflow Matrix (`ods`)

| Command | Mastery Tier | Role & Syntax |
|---|---|---|
| **`ods lsp [--port N]`** | 🏁 Tier 1 (Novice) | Native JSON-RPC 2.0 Language Server for real-time editor lints, hover, definition, and completion. |
| **`ods init [path]`** | 🏁 Tier 1 (Novice) | Initialize root `index.md` with `ods: 0.1` spec marker. `--adopt` drafts frontmatter on plain `.md` files. |
| **`ods setup [path]`** | 🏁 Tier 1 (Novice) | Verify workspace boundary, check updates, register OS daemon. `--git-hooks` installs pre-commit hook. `--editor zed\|vscode\|nvim\|cursor` writes `ods lsp` config. |
| **`ods lint [path]`** | 🏁 Tier 1 (Novice) | ODS validation by default (`--mode strict\|standard`, `--fix`, `--format text\|json\|sarif`). Add `--okf` / `--skills` for other specs. Never `--ods`. |
| **`ods export graph`** | 🏁 Tier 1 (Novice) | Export workspace knowledge graph in structured JSON (`--format json`), Markdown (`--format md`), or text (`--format text`) for `--spec ods` (default) or `--spec okf`. |
| **`ods new <path>`** | 🛠️ Tier 2 (Practitioner) | Scaffold new Markdown document from profile template with starter HTML comments. |
| **`ods index [path]`** | 🛠️ Tier 2 (Practitioner) | Generate navigation `index.md` lockfiles (`--check` verifies freshness). |
| **`ods mv <from> <to>`** | 🛠️ Tier 2 (Practitioner) | Atomic document move + workspace-wide graph reference rewriting. |
| **`ods sync [path]`** | 🛠️ Tier 2 (Practitioner) | Reconcile git-tracked renames (`git status --porcelain`) and rewrite graph links. |
| **`ods adopt [path]`** | 🛠️ Tier 2 (Practitioner) | Draft frontmatter (`ods: { profile: note, status: draft }`) on legacy Markdown trees based on prose headings. |
| **`ods rm <path>`** | 🛠️ Tier 2 (Practitioner) | Atomically delete document and scrub graph references workspace-wide. |
| **`ods archive <path>`** | 🛠️ Tier 2 (Practitioner) | Set `status: archived` in frontmatter in place. |
| **`ods fmt [path]`** | 🛠️ Tier 2 (Practitioner) | Reformat YAML frontmatter and body spacing (`--refs md-paths` converts IDs to relative `.md` paths). |
| **`ods stats [path]`** | 🛠️ Tier 2 (Practitioner) | Workspace document telemetry, graph density, profile distribution, and health score %. |
| **`ods tree [path]`** | 🛠️ Tier 2 (Practitioner) | Visual ASCII/Unicode hierarchy tree of index navigation and dependency graphs. |
| **`ods context <id>`** | 📋 Tier 3 (Power User) | Sub-5ms bounded AI context reading list following `depends` / `related` chains (`--max-tokens N` caps budget). |
| **`ods profile list/init`** | 📋 Tier 3 (Power User) | List registered profiles or scaffold new custom profile schemas in `.ods/profiles/`. |
| **`ods tags [path]`** | 📋 Tier 3 (Power User) | Workspace-wide tag catalog and document count breakdown. |
| **`ods tag rename`** | 📋 Tier 3 (Power User) | Perform workspace-wide tag renaming (dry-run; `--write`). |
| **`ods schema [path]`** | 📋 Tier 3 (Power User) | Export official JSON Schema (`ods.schema.json`) for IDE frontmatter validation (`--write`, `--out`). |
| **`ods diff [target]`** | 📋 Tier 3 (Power User) | Compare document graph dependencies and frontmatter changes against git commits or branches. |
| **`ods share [path]`** | 📋 Tier 3 (Power User) | Publish share-filtered workspace/subtree directory (`--out DIR`). |
| **`ods pack`** | 📋 Tier 3 (Power User) | Manage reusable ODS Packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| **`ods … --okf`** | 📋 Tier 3 (Power User) | Google OKF v0.2: `init --okf`, `lint --okf`, `audit --okf`, `export --okf` (flag-only; no namespace). |
| **`ods … --skills`** | 📋 Tier 3 (Power User) | Agent Skills packages: `init --skills`, `lint --skills` (name/description/license/…). |
| **`ods agents sync`** | 📋 Tier 3 (Power User) | Write AGENTS.md + editor agent snippets. |
| **`ods bench`** | 📋 Tier 3 (Power User) | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| **`ods completion <shell>`**| 🏢 Tier 4 (Architect) | Generate shell autocompletion scripts for `bash`, `zsh`, `fish`, `powershell`. |
| **`ods clean [path]`** | 🏢 Tier 4 (Architect) | Clean `.ods/ods-errors.md`, `.ods/coverage.md`, and diagnostic cache files. |
| **`ods coverage [path]`** | 🏢 Tier 4 (Architect) | Documentation health % (`--write-report` → `.ods/coverage.md`). |
| **`ods start / stop`** | 🏢 Tier 4 (Architect) | Manage background user OS service (`systemd` / `launchd` / Windows Scheduled Task). |
| **`ods serve --root <path>`**| 🏢 Tier 4 (Architect) | Headless daemon loop executed by background service. |
| **`ods doctor [path]`** | 🏢 Tier 4 (Architect) | Workspace health check (version, doc count, index freshness, profile conflicts, service status). |
| **`ods update`** | 🏢 Tier 4 (Architect) | Self-update CLI binary & restart background user service. |
