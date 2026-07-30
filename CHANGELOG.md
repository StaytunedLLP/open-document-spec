# Changelog

All notable changes to **OpenDocify (`odc`)** are documented in this file.

The format is inspired by [Keep a Changelog](https://keepachangelog.com/),
and this project follows [Semantic Versioning](https://semver.org/).

GitHub Releases use GitHub’s auto-generated notes. Edit this file by hand when useful.

## [Unreleased]

### Breaking

- Root CLI pin key renamed: `ods-cli:` → **`odc:`** (tool is OpenDocify). Spec marker remains `ods:`. OKF remains `okf_version:`.
- Lint health report path: root **`ods-error.md`** → **`.odc/odc-errors.md`** (legacy file is cleared when present).
- Machine config / logs / backups prefer **`~/.odc/`** (legacy `~/.ods/` still read; `odc upgrade --write` migrates).
- OS service unit names: `odc-watch-*` / `llp.odc.watch.*` (replaces `ods-watch-*` / `llp.ods.watch.*`).

### Added

- **Crate rename** — workspace packages `odc`, `odc-core`, `odc-test-support` (was `ods` / `ods-core` / `ods-test-support`).
- **OKF full CLI surface** — `odc okf index|context|export|fmt|watch|serve` plus existing init/lint/doctor/audit/adopt.
- **Seamless bare CLI** — `odc lint|init|doctor|audit|…` auto-detects ODS vs OKF from root markers; explicit `odc ods` / `odc okf` still available.
- **Cutover checklist** — `docs/plan/external_repo_cutover_checklist.md` for the ≈3 external ODS workspaces.
- **Tool/keys plan** — `docs/plan/odc_tool_keys_legacy_cleanup.md` (source of truth for naming + legacy inventory).
- **`skills/okf`** — agent skill for OKF via `odc okf`; ODS skill updated for OpenDocify.
- **`odc agents sync`** — writes `AGENTS.md` plus optional `.claude` / `.cursor` snippets.
- **OpenDocify multi-spec CLI (`odc`)** — primary binary `odc`:
  - Bare document commands auto-detect; namespaces force an engine
  - `odc ods <cmd>` — ODS document graphs (Markdown keys still `ods:` / nested `ods`)
  - `odc okf <cmd>` — **native Google OKF v0.2**
  - `odc agents <cmd>` — agent graph MVP (`sync`)
  - Platform: `odc update`, `odc upgrade` (`ods-cli:`→`odc:`; `~/.ods`→`~/.odc`), `setup`, `workspaces`, `skill`
- Legacy binary **`ods`** still ships (same code); bare `ods lint` ≡ ODS engine only.
- `odc audit` / `odc ods audit` / `odc okf audit` — compliance inventory; `--write-report` → `.odc/odc-errors.md`
- Plans & key map: `docs/plan/odc_tool_keys_legacy_cleanup.md`, migration + OKF plans, `docs/specs/frontmatter-keys-ods-vs-okf.md`
- `ods bench` — benchmarking and frontmatter snapshot/restore system (`stats`, `strip`, `restore`, `run`). Allows teams to take machine-level JSON snapshots (`~/.ods/backups/<repo-hash>/`), temporarily strip frontmatter, index lockfiles, and profiles to test AI task performance without ODS, restore workspace artifacts losslessly (`ods bench restore`), and calculate token & API cost ROI metrics (~94% token savings). Added `--full` flag for complete baseline isolation.
- `ods workspaces` — manage globally tracked ODS workspace paths (`add`, `remove`, `list`, `path`) stored in human-readable TOML at `~/.ods/odsconfig.toml` (legacy `~/.ods/workspaces.toml` is auto-migrated on read).
- `ods share [path] --out DIR` — publish a `share`-filtered copy of a workspace or subtree as a real directory (`--include-org`, `--include-private`), ready to `git init`/push yourself. `share:` set on any `index.md` now also acts as a directory-level default that cascades to descendants (a document's own `share` still wins).

### Changed

- **Test coverage:** policy in `docs/maintainer/coverage.md`; scripts `coverage.sh` + `coverage-100-check.sh`; CI floor **75%** lines. New tests: OKF audit/model/init/parse (full surface), pipeline apply, model `CodeRole`/`odc` pin, CLI okf/upgrade/share/export/help. Engine (`odc-core`) ~**84%** lines; first modules at **100%**: `okf/audit`, `okf/model`, `pipeline/apply`.
- **10K-oriented performance (no disk cache):**
  - Functional pipeline modules (`odc-core/src/pipeline/`): discover → parallel parse (`rayon`) → index.
  - Graph commands (`lint`, `doctor`, `index`, `context`, `graph`, `find`, `tags`, `profiles`) load with `include_body: false` (note bodies dropped; **`index.md` bodies kept** for child-list rules).
  - `odc watch` / `serve` keep a long-lived workspace and reparse **dirty paths only** (full reload only on first tick or large dirty sets). `ODC_JOBS` caps parse threads.
  - Lint report `.odc/odc-errors.md` capped at 500 diagnostics + summary.
- **Strict Workspace Targeting**: `find_workspace_root` no longer falls back to arbitrary directories. ODS CLI commands now verify local `ods:` root index markers or global registry tracking before executing, preventing accidental folder pollution.
- Auto-update respects `ODC_AUTO_UPDATE` as well as `ODS_AUTO_UPDATE`.

## [0.0.1] - 2026-07-19

### Added

- `ods setup` — first-run/update/service/doctor workflow for end users.
- Private-release-safe install scripts for macOS/Linux and Windows.
- Release verification for all six platform assets plus `SHA256SUMS`.
- End-user quickstart and tooling docs for CLI-only install, setup, service, CI, and AI context.

### Changed

- Root workspace ignores internal skill artifacts and removed extension source.
- Installer scripts skip downloads when the installed `ods` version is already current.
- `scripts/install-from-release.sh` and the skill installer now share the same GitHub API asset flow.

### Removed

- Tracked `ods-lsp`, Zed extension, and `install-zed-ods-lsp.sh` remnants.
- Obsolete cleanup script and removed-component placeholder document.

## [0.1.15] - 2026-07-17

### Added

- `ods export` — write full workspace graph to `graph.md` for AI review
- `ods start` / `ods stop` / `ods serve` — background user service (Linux/macOS/Windows)
- Watch rename pairing with pending removals (split delete/create batches)
- Path-shaped `id:` orphan heal when filename and id drift

### Changed

- Single product binary **`ods` only** (version `0.1.15`)
- Workspace members: `ods`, `ods-core`, `ods-test-support`
- Self-update installs `ods` only

### Removed

- `ods-lsp` language server from product/CI/release path
- Zed extension packaging from release workflow

### Documentation

- README end-to-end install → init → start → lint → export
- `docs/FEATURE_MATRIX.md` keys × auto-update matrix
- Quickstart aligned with CLI-only product

## [0.1.12] - 2026-07-16

### Notes

- Prior workspace package version line; see git history and GitHub tags for details.
