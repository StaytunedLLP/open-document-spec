---
description: "CLI command matrix, service daemon vs watch comparison, CI integration, and updates."
status: "stable"
order: 4
ods:
  profile: "note"
  status: "stable"
---

# Tooling

Reference implementation: the single native **`ods` CLI**. ODS is the default engine (no `--ods` flag). Extra specs use flags only: `--okf`, `--skills`. Editor support is built in via **`ods lsp`** (JSON-RPC over stdio; not the same as `ods serve`).

---

## Production Checklist

| Check | Action |
| --- | --- |
| Version | `ods --version` |
| First-run setup | `ods setup` |
| Workspace | Root `index.md` with spec `ods: 0.1` and CLI requirement `ods: ">=0.0.1"` |
| Local clean | `ods index && ods lint` |
| CI | `ods index --check` + `ods lint` |
| Automation | `ods start` (background) or `ods watch` (foreground) |
| AI dump (optional) | `ods export` → `graph.md` |
| Doctor | `ods doctor` |
| Updates | `ods update` |

Happy path: [Quickstart Guide](/docs/quickstart).

---

## Commands Matrix (Novice to Expert)

| Command | Mastery Tier | Role & Syntax |
| --- | --- | --- |
| `ods init [path]` | 🏁 **Tier 1: Novice** | Make folder/repo ODS-compliant (creates root `index.md` + `ods:` spec, generates indexes). `--adopt` drafts frontmatter. |
| `ods setup [path]` | 🏁 **Tier 1: Novice** | Set up machine background service for workspace, check updates, and run `ods doctor`. `--git-hooks` installs pre-commit hook. `--editor zed\|vscode\|nvim\|cursor` writes `ods lsp` config. |
| `ods lsp` | 🏁 **Tier 1: Novice** | JSON-RPC Language Server (stdio / `--port`); not the same as `ods serve`. |
| `ods lint` / `ods lint [path]` | 🏁 **Tier 1: Novice** | Validate graph & schemas (`--level 1\|3`, `--format text\|json\|sarif`, `--canonical-refs`). Generates or clears `.ods/ods-errors.md`. |
| `ods new <path>` | 🛠️ **Tier 2: Practitioner** | Scaffold a new Markdown document with inferred profile (`guide`, `feature`, etc.) and valid frontmatter. |
| `ods index [path]` | 🛠️ **Tier 2: Practitioner** | Generate `index.md` lockfiles (`--check` exits 1 if stale in CI). |
| `ods mv <from> <to>` | 🛠️ **Tier 2: Practitioner** | Offline document move + rewrite graph references workspace-wide. |
| `ods sync [path]` | 🛠️ **Tier 2: Practitioner** | Reconcile git-tracked renames (`git status --porcelain`) and rewrite graph references. |
| `ods adopt [path]` | 🛠️ **Tier 2: Practitioner** | Draft frontmatter for existing Markdown files (dry-run; `--write`). |
| `ods rm <path-or-id>` | 🛠️ **Tier 2: Practitioner** | Atomically delete document and scrub graph references (`depends`/`related`) workspace-wide. Alias: `ods remove`. |
| `ods archive <path-or-id>` | 🛠️ **Tier 2: Practitioner** | Set `status: archived` in frontmatter in place. |
| `ods fmt [path]` | 🛠️ **Tier 2: Practitioner** | Reformat YAML frontmatter/body blank-line spacing. `--refs md-paths` converts extensionless IDs to relative `.md` paths. |
| `ods stats [path]` | 🛠️ **Tier 2: Practitioner** | Display workspace document telemetry, graph density, profile distribution, and health score (`--format text\|json`). |
| `ods tree [path]` | 🛠️ **Tier 2: Practitioner** | Display visual ASCII/Unicode hierarchy tree of index navigation and dependency graphs (`--format text\|json`). |
| `ods context [path] <id>` | 📋 **Tier 3: Power User** | Generate resolved bounded AI reading list (`--include-private` includes `share: private` documents). |
| `ods profiles [path]` | 📋 **Tier 3: Power User** | List standard and custom profiles loaded in workspace and report schema conflicts. |
| `ods tags [path]` | 📋 **Tier 3: Power User** | List document tags with counts (`--all` includes default unused tags). |
| `ods find [path] --tag <t>` | 📋 **Tier 3: Power User** | Find and list documents by tag (repeat `--tag` for OR query). |
| `ods tag rename <old> <new>` | 📋 **Tier 3: Power User** | Workspace-wide tag rename (dry-run; `--write`). |
| `ods schema [path]` | 📋 **Tier 3: Power User** | Export JSON Schema (`ods.schema.json`) for IDE frontmatter autocomplete and validation (`--write`, `--out PATH`). |
| `ods diff [target]` | 📋 **Tier 3: Power User** | Compare document graph dependencies and frontmatter changes against git commits or branches (`--format text\|json`). |
| `ods graph [path]` | 📋 **Tier 3: Power User** | Print `depends`/`related` edges as `path -> edge` lines. |
| `ods export [path]` | 📋 **Tier 3: Power User** | Export single-file Markdown graph snapshot (`--out PATH`, `--include-private`). |
| `ods share [path]` | 📋 **Tier 3: Power User** | Publish share-filtered workspace/subtree directory (`--out DIR`, `--include-org`, `--include-private`). |
| `ods pack <subcommand>` | 📋 **Tier 3: Power User** | Manage reusable ODS Packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| `ods bench <subcommand>` | 📋 **Tier 3: Power User** | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| `ods completion <shell>` | 🏢 **Tier 4: Enterprise Architect** | Generate shell autocompletion scripts (`bash`, `zsh`, `fish`, `powershell`). |
| `ods clean [path]` | 🏢 **Tier 4: Enterprise Architect** | Clean `.ods/ods-errors.md`, `.ods/coverage.md`, and diagnostic cache files. |
| `ods coverage [path]` | 🏢 **Tier 4: Enterprise Architect** | Documentation health % (`--write-report` → `.ods/coverage.md`; separate from lint `.ods/ods-errors.md`). |
| `ods start [path]` | 🏢 **Tier 4: Enterprise Architect** | Register + start **user OS service** (`systemd` / `launchd` / Windows Scheduled Task). `--status` checks status. |
| `ods stop [path]` | 🏢 **Tier 4: Enterprise Architect** | Stop running OS service. `--unregister` stops and removes registration completely. |
| `ods watch [path]` | 🏢 **Tier 4: Enterprise Architect** | Foreground live rename map + re-lint terminal loop. |
| `ods logs [path] [-f]` | 🏢 **Tier 4: Enterprise Architect** | Show background service logs (`~/.ods/logs/ods-serve.log`); `-f` follows. |
| `ods serve --root <path>` | 🏢 **Tier 4: Enterprise Architect** | Headless daemon loop executed by background service (`--mode auto\|watch\|poll`). |
| `ods workspaces <subcommand>` | 🏢 **Tier 4: Enterprise Architect** | Manage globally tracked ODS workspaces in `~/.ods/odsconfig.toml` (`add`, `remove`, `list`, `path`). |
| `ods disable [path]` | 🏢 **Tier 4: Enterprise Architect** | Opt-out / strip ODS metadata (dry-run; `--write`, `--keep-frontmatter`, `--remove-indexes`). Alias: `ods revert`. |
| `ods doctor [path]` | 🏢 **Tier 4: Enterprise Architect** | Workspace health check (version, doc count, index freshness, profile conflicts, service status). |
| `ods update` | 🏢 **Tier 4: Enterprise Architect** | Self-update CLI binary & restart background user service (`--check`, `--force`, `--version <tag>`). |

---

### `ods serve` vs. `ods watch` Comparison

| Dimension | `ods serve` (Background OS Service) | `ods watch` (Foreground Terminal Watcher) |
| :--- | :--- | :--- |
| **Execution Architecture** | Headless OS Daemon (systemd user unit, launchd agent, Windows Scheduled Task). Registered via `ods start` / `ods setup`. | Interactive Foreground Process running in an open terminal tab (`ods watch .`). |
| **User Visibility** | Invisible ("Set and Forget"). Zero terminal output. | Displays live event logs, rename mappings, and lint warnings in stdout. |
| **Workspace Scope** | Global machine service tracking registered workspace paths (`~/.ods/odsconfig.toml`). | Single workspace directory tree (defaults to `.`). |
| **Process Lifecycle** | Automatically starts on OS boot/login. Runs persistently in background. | Runs until terminal tab is closed or `Ctrl+C` is pressed. |
| **Extra Responsibilities** | Background auto-update check (~daily) and remote Git pack auto-sync (`ods pack sync`). | Dedicated filesystem event watching and instant re-linting. |
| **Ideal For** | Everyday background automation for devs and non-tech users. | Active refactoring sessions, debugging graph renames, containerized environments. |

---

## Code References

ODS documents can declare implementation locations with `code:`:

```yaml
code:
  - path: apps/web/src/routes/checkout.tsx
    symbol: CheckoutRoute
    role: entrypoint
  - path: apps/web/src/features/checkout/pricing.ts
    symbol: calculateCheckoutTotal
    role: implementation
  - path: apps/web/src/features/checkout/pricing.test.ts
    symbol: calculateCheckoutTotal
    role: test
```

`path` is required, `role` is required and fixed, and `symbol` is optional. `ods lint` validates paths at Level 3.

---

## CI Integration

In CI pipelines:

```bash
ods index --check
ods lint --level 3
```

---

## Next

- [Profiles & Catalogs](/docs/profiles) · [Advanced Workspaces](/docs/advanced) · [FAQ](/docs/faq)


## Hybrid workspaces (ODS + OKF markers)

Bare `ods lint` / `doctor` / `audit` run **ODS only**. Pass `--okf` (and/or `--skills`) to enable extra specs. Pure OKF trees: `ods lint --okf`. Pure skills: `ods lint --skills`. Editors: `ods lsp` (not `ods serve`).
