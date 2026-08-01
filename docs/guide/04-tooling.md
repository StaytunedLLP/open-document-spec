---
description: "CLI command matrix, service daemon vs watch comparison, CI integration, and updates."
status: "stable"
order: 4
ods:
  profile: "note"
  status: "stable"
---

# Tooling

Reference implementation: the **`ods` (legacy `ods`) CLI** only. There is no language server or editor extension in the production install.

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

## Commands (Day-to-Day)

| Command | Role & Syntax |
| --- | --- |
| `ods init [path]` | Make folder/repo ODS-compliant (creates root `index.md` + `ods:` spec, generates indexes). `--adopt` drafts frontmatter. |
| `ods setup [path]` | Set up machine background service for workspace, check updates, and run `ods doctor`. |
| `ods new <path>` | Scaffold a new Markdown document with inferred profile (`guide`, `feature`, etc.) and valid frontmatter. |
| `ods rm <path-or-id>` | Atomically delete document and scrub graph references (`depends`/`related`) workspace-wide. Alias: `ods remove`. |
| `ods archive <path-or-id>` | Set `status: archived` in frontmatter in place. |
| `ods start [path]` | Register + start **user OS service** (`systemd` / `launchd` / Windows Scheduled Task). `--status` checks status. |
| `ods stop [path]` | Stop running OS service. `--unregister` stops and removes registration completely. |
| `ods watch [path]` | Foreground live rename map + re-lint terminal loop. |
| `ods logs [path] [-f]` | Show background service logs (`~/.ods/logs/ods-serve.log`); `-f` follows. Not a watch alias. |
| `ods serve --root <path>` | Headless daemon loop executed by background service (`--mode auto\|watch\|poll`, `--memory-report`, `--poll-secs`). |
| `ods lint` / `ods lint [path]` | Validate graph & schemas (`--level 1\|3`, `--format text\|json`, `--canonical-refs`). Generates or clears `.ods/ods-errors.md`. |
| `ods coverage [path]` | Documentation health % (`--write-report` → `.ods/coverage.md`; separate from lint `.ods/ods-errors.md`). |
| `ods index [path]` | Generate `index.md` lockfiles (`--check` exits 1 if stale in CI). |
| `ods mv <from> <to>` | Offline document move + rewrite graph references workspace-wide. |
| `ods sync [path]` | Reconcile git-tracked renames (`git status --porcelain`) and rewrite graph references. |
| `ods fmt [path]` | Reformat YAML frontmatter/body blank-line spacing. `--refs md-paths` converts extensionless IDs to relative `.md` paths. |
| `ods context [path] <id>` | Generate resolved bounded AI reading list (`--include-private` includes `share: private` documents). |
| `ods graph [path]` | Print `depends`/`related` edges as `path -> edge` lines. |
| `ods export [path]` | Export single-file Markdown graph snapshot (`--out PATH`, `--include-private`). |
| `ods share [path]` | Publish share-filtered workspace/subtree directory (`--out DIR`, `--include-org`, `--include-private`). |
| `ods disable [path]` | Opt-out / strip ODS metadata (dry-run; `--write`, `--keep-frontmatter`, `--remove-indexes`, `--remove-root-index`). Alias: `ods revert`. |
| `ods adopt [path]` | Draft frontmatter for existing Markdown files (dry-run; `--write`). |
| `ods profiles [path]` | List standard and custom profiles loaded in workspace and report schema conflicts. |
| `ods tags [path]` | List document tags with counts (`--all` includes default unused tags). |
| `ods find [path] --tag <t>` | Find and list documents by tag (repeat `--tag` for OR query). |
| `ods tag rename <old> <new>` | Workspace-wide tag rename (dry-run; `--write`). |
| `ods workspaces <subcommand>` | Manage globally tracked ODS workspaces in `~/.ods/odcconfig.toml` (`add`, `remove`, `list`, `path`; legacy `~/.ods` is read). |
| `ods pack <subcommand>` | Manage reusable ODS Packs (`add`, `sync`, `list`, `preview`, `remove`, `init`). |
| `ods bench <subcommand>` | ROI benchmarking & frontmatter snapshot (`stats`, `strip`, `restore`, `run`). |
| `ods doctor [path]` | Workspace health check (version, doc count, index freshness, profile conflicts, service status, pending git renames). |
| `ods update` | Self-update CLI binary & restart background user service (`--check`, `--force`, `--version <tag>`). |

---

### `ods serve` vs. `ods watch` Comparison

| Dimension | `ods serve` (Background OS Service) | `ods watch` (Foreground Terminal Watcher) |
| :--- | :--- | :--- |
| **Execution Architecture** | Headless OS Daemon (systemd user unit, launchd agent, Windows Scheduled Task). Registered via `ods start` / `ods setup`. | Interactive Foreground Process running in an open terminal tab (`ods watch .`). |
| **User Visibility** | Invisible ("Set and Forget"). Zero terminal output. | Displays live event logs, rename mappings, and lint warnings in stdout. |
| **Workspace Scope** | Global machine service tracking registered workspace paths (`~/.ods/odcconfig.toml`). | Single workspace directory tree (defaults to `.`). |
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

Bare `ods lint` / `doctor` / `audit` run **both** engines. Other dual-engine bare commands (`index`, `fmt`, `export`, `context`, `watch`, `serve`, `adopt`) require `ods …` or `ods okf …`.
