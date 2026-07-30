---
title: "Tooling & Service Reference"
description: "CLI command matrix, service daemon vs watch comparison, CI integration, and updates."
status: "stable"
order: 4
ods:
  profile: "note"
  status: "stable"
---

# Tooling

Reference implementation: the **`odc` (legacy `ods`) CLI** only. There is no language server or editor extension in the production install.

---

## Production Checklist

| Check | Action |
| --- | --- |
| Version | `odc --version` |
| First-run setup | `odc setup` |
| Workspace | Root `index.md` with spec `ods: 0.1` and CLI requirement `odc: ">=0.0.1"` |
| Local clean | `odc ods index && odc ods lint` |
| CI | `odc ods index --check` + `odc ods lint` |
| Automation | `odc ods start` (background) or `odc ods watch` (foreground) |
| AI dump (optional) | `odc ods export` → `graph.md` |
| Doctor | `odc ods doctor` |
| Updates | `odc update` |

Happy path: [Quickstart Guide](/docs/quickstart).

---

## Commands (Day-to-Day)

| Command | Role & Syntax |
| --- | --- |
| `odc ods init [path]` | Make folder/repo ODS-compliant (creates root `index.md` + `ods:` spec, generates indexes). `--adopt` drafts frontmatter. |
| `odc setup [path]` | Set up machine background service for workspace, check updates, and run `odc ods doctor`. |
| `odc ods new <path>` | Scaffold a new Markdown document with inferred profile (`guide`, `feature`, etc.) and valid frontmatter. |
| `odc ods rm <path-or-id>` | Atomically delete document and scrub graph references (`depends`/`related`) workspace-wide. Alias: `odc ods remove`. |
| `odc ods archive <path-or-id>` | Set `status: archived` in frontmatter in place. |
| `odc ods start [path]` | Register + start **user OS service** (`systemd` / `launchd` / Windows Scheduled Task). `--status` checks status. |
| `odc ods stop [path]` | Stop running OS service. `--unregister` stops and removes registration completely. |
| `odc ods watch [path]` | Foreground live rename map + re-lint terminal loop. |
| `odc ods logs [path] [-f]` | Alias for `odc ods watch` (foreground re-lint loop). |
| `odc ods serve --root <path>` | Headless daemon loop executed by background service (`--mode auto\|watch\|poll`, `--memory-report`, `--poll-secs`). |
| `odc lint` / `odc ods lint [path]` | Validate graph & schemas (`--level 1\|3`, `--format text\|json`, `--canonical-refs`). Generates or clears `.odc/odc-errors.md`. |
| `odc ods index [path]` | Generate `index.md` lockfiles (`--check` exits 1 if stale in CI). |
| `odc ods mv <from> <to>` | Offline document move + rewrite graph references workspace-wide. |
| `odc ods sync [path]` | Reconcile git-tracked renames (`git status --porcelain`) and rewrite graph references. |
| `odc ods fmt [path]` | Reformat YAML frontmatter/body blank-line spacing. `--refs md-paths` converts extensionless IDs to relative `.md` paths. |
| `odc ods context [path] <id>` | Generate resolved bounded AI reading list (`--include-private` includes `share: private` documents). |
| `odc ods graph [path]` | Print `depends`/`related` edges as `path -> edge` lines. |
| `odc ods export [path]` | Export single-file Markdown graph snapshot (`--out PATH`, `--include-private`). |
| `odc ods share [path]` | Publish share-filtered workspace/subtree directory (`--out DIR`, `--include-org`, `--include-private`). |
| `odc ods disable [path]` | Opt-out / strip ODS metadata (dry-run; `--write`, `--keep-frontmatter`, `--remove-indexes`, `--remove-root-index`). Alias: `odc ods revert`. |
| `odc ods adopt [path]` | Draft frontmatter for existing Markdown files (dry-run; `--write`). |
| `odc ods profiles [path]` | List standard and custom profiles loaded in workspace and report schema conflicts. |
| `odc ods tags [path]` | List document tags with counts (`--all` includes default unused tags). |
| `odc ods find [path] --tag <t>` | Find and list documents by tag (repeat `--tag` for OR query). |
| `odc ods tag rename <old> <new>` | Workspace-wide tag rename (dry-run; `--write`). |
| `odc workspaces <subcommand>` | Manage globally tracked ODS workspaces in `~/.odc/odcconfig.toml` (`add`, `remove`, `list`, `path`; legacy `~/.ods` is read). |
| `odc pack <subcommand>` | Manage reusable ODS Packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| `odc ods bench <subcommand>` | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| `odc ods doctor [path]` | Workspace health check (version, doc count, index freshness, profile conflicts, service status, pending git renames). |
| `odc update` | Self-update CLI binary & restart background user service (`--check`, `--force`, `--version <tag>`). |

---

### `odc ods serve` vs. `odc ods watch` Comparison

| Dimension | `odc ods serve` (Background OS Service) | `odc ods watch` (Foreground Terminal Watcher) |
| :--- | :--- | :--- |
| **Execution Architecture** | Headless OS Daemon (systemd user unit, launchd agent, Windows Scheduled Task). Registered via `odc ods start` / `odc setup`. | Interactive Foreground Process running in an open terminal tab (`odc ods watch .`). |
| **User Visibility** | Invisible ("Set and Forget"). Zero terminal output. | Displays live event logs, rename mappings, and lint warnings in stdout. |
| **Workspace Scope** | Global machine service tracking registered workspace paths (`~/.odc/odcconfig.toml`). | Single workspace directory tree (defaults to `.`). |
| **Process Lifecycle** | Automatically starts on OS boot/login. Runs persistently in background. | Runs until terminal tab is closed or `Ctrl+C` is pressed. |
| **Extra Responsibilities** | Background auto-update check (~daily) and remote Git pack auto-sync (`odc pack sync`). | Dedicated filesystem event watching and instant re-linting. |
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

`path` is required, `role` is required and fixed, and `symbol` is optional. `odc ods lint` validates paths at Level 3.

---

## CI Integration

In CI pipelines:

```bash
odc ods index --check
odc ods lint --level 3
```

---

## Next

- [Profiles & Catalogs](/docs/profiles) · [Advanced Workspaces](/docs/advanced) · [FAQ](/docs/faq)
