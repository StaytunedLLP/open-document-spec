# Contributing to Open Document Spec (ODS)

Thank you for improving ODS. This repository holds the **spec**, **guide**, and **Rust reference implementation** (`odc` (legacy `ods`) CLI).

## Development setup

```bash
# From repository root
cd src
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D clippy::correctness -D clippy::suspicious
```

Binaries land under `.artifacts/target/release/` (see `.cargo/config.toml`).

Install locally:

```bash
cargo install --path src/crates/odc --bin odc --bin ods --locked --force
```

## Layout

| Path | Role |
| --- | --- |
| `specs/` | Normative specification (ODS keys stay `ods:`) |
| `docs/guide/` | End-user guide (keep in sync with `app-web/src/content/docs/`) |
| `docs/plan/` | Cutover / architecture plans (`odc_tool_keys_legacy_cleanup.md` is product SoT) |
| `docs/specs/` | Author key maps (ODS vs OKF) |
| `docs/maintainer/` | Coverage, functional style |
| `CHANGELOG.md` | Optional manual release history |
| `skills/ods`, `skills/okf` | AI assistant skills |
| `src/crates/odc-core` | Shared library (ODS + native OKF engines) |
| `src/crates/odc` | Primary binary `odc` + legacy argv0 `ods` |
| `src/scripts/` | install, check-local, coverage, smoke-odc |
| `src/action/` | GitHub composite action |

### CLI crate layout (`src/crates/odc/src/`)

| Path | Role |
| --- | --- |
| `main.rs` / `ods_main.rs` | Thin bin entrypoints (`include!` of main/*) |
| `main/cli/` | Entry, dispatch, argv parser, exit codes |
| `main/commands/` | User-facing commands (`okf/`, `pack/`, `workspaces/`, …) |
| `main/support/` | Formatters, loaders, git helpers |
| `service/`, `update/` | OS service + self-update implementation |
| `tests/*.test.rs` | Integration tests (declared in `Cargo.toml`) |

### Core crate layout (`src/crates/odc-core/src/`)

Domain folders (`lint/`, `mv/`, `parse/`, `okf/`, `bench/`, `share/`, …). Prefer unit tests inside the domain folder; cross-module integration tests under `tests/`.

New engine code should follow [functional style](docs/maintainer/functional-style.md): data + free functions, no `*Manager` types.

**Product naming:** teach **`odc`** as the tool; bare commands auto-detect ODS/OKF; nested document key **`ods:`** is intentional and must not be renamed.

Coverage: see [docs/maintainer/coverage.md](docs/maintainer/coverage.md). Run `./src/scripts/coverage.sh`. CI enforces a line floor (currently 75%).

## Tests & coverage

- Prefer unit tests next to pure logic and integration tests under `crates/*/tests/`.
- Production bar: high line coverage on `odc-core` (aim ≥85% workspace over time).
- CI enforces a coverage floor (`--fail-under-lines`, currently 75%, see `.github/workflows/pr.yml`) that ratchets upward as coverage improves — never lower it to make a PR pass; add tests instead.
- Always keep `odc ods index --check .` and `odc ods lint .` green at repo root.

```bash
cargo install cargo-llvm-cov --locked
cd src && cargo llvm-cov --workspace --locked --fail-under-lines 75
```

## CI

Workflows in `.github/workflows/` consist of **two workflows** total:

| Workflow | When | Notes |
| --- | --- | --- |
| `pr.yml` | PRs (open/sync/labels), push to `main`, manual | Unified quality gate: PR version bump + Linux & Windows tests (`fmt`/`clippy`/`test`/`odc` (legacy `ods`)/`cov`) |
| `release.yml` | After `pr.yml` succeeds on main/master | Multi-OS binary builds, GitHub Release tag `vX.Y.Z` publish |

**Do not create release tags by hand.** Releases are cut automatically when CI is green on main.

Runner requirements:

- Linux x86_64 (or matching labels you configure)
- `git` and a C linker (`cc` / `gcc` / `clang`) — install with `sudo apt-get install -y build-essential` if needed
- Network access for first-time Rust install if the runner has no toolchain
- CI normalizes `PATH` (includes `/usr/bin`), checks for a working `rustc`/`cargo`, and installs latest
  stable via rustup **only if missing/broken** (then ensures `rustfmt` + `clippy`)

Jobs: **fmt → clippy → test → release build → ods index/lint/export smoke → optional coverage**.

## Pull requests

1. Keep changes focused (spec vs guide vs code).
2. Prefer [Conventional Commits](https://www.conventionalcommits.org/) so release notes group well:
   - `feat(ods): …`, `fix(watch): …`, `docs: …`, `ci: …`, `chore: …`
   - Breaking: `feat!: …` or body containing `BREAKING CHANGE`
3. Update `docs/guide/` (and `app-web/src/content/docs/`) if you add a user-visible feature.
4. Do not commit `.artifacts/` or `target/`.
5. Ensure CI is green on the self-hosted runner.
6. **Reuse the same PR branch** — keep pushing commits to the existing head branch; do not open a new branch/PR for follow-up fixes on the same change set.

## Versioning & release workflow (Maintainers)

End users install **only** from [GitHub Releases](https://github.com/StaytunedLLP/open-document-spec/releases) via `src/scripts/install.sh` / `src/scripts/install.ps1` or manual archive download; then use `odc setup` / `odc update`. Do **not** cut releases by pushing a tag by hand.

### Where the version lives

**One line** controls the published version for `odc` (legacy `ods`) and `odc-core` (workspace packages):

| File | Field |
|---|---|
| [`Cargo.toml`](Cargo.toml) | `version = "X.Y.Z"` under `[workspace.package]` |

Crates under `src/crates/*/Cargo.toml` use `version.workspace = true` — **do not** set a separate version there.
GitHub Release tag form: `v` + that string (e.g. `0.1.23` → tag **`v0.1.23`**).

### What you usually do (no hand-edit required)

1. Open a PR into `main`.
2. Workflow **`pr-version`** (`.github/workflows/pr-version.yml`) compares PR vs main and **auto-commits** the next version into `Cargo.toml`.
3. Merge the PR when CI is green.
4. Workflow **`ci`** runs on `main`; when it **succeeds**, workflow **`release`**:
   - tags `vX.Y.Z` from `Cargo.toml`
   - builds multi-OS **`odc` (legacy `ods`)** archives
   - publishes the GitHub Release (auto-generated notes) + `SHA256SUMS`
   - verifies all six published assets against `SHA256SUMS`

### Choose patch / minor / major on a PR

Add a label **before** or **after** open (bot re-runs on label changes):

| Label | Effect on next version (from main) |
|---|---|
| *(none)* or `release:patch` | `0.1.0` → **`0.1.1`** |
| `release:minor` | `0.1.0` → **`0.2.0`** |
| `release:major` | `0.1.0` → **`1.0.0`** |

### Manual version edit (only if needed)

If the bot cannot push (e.g. fork PR), edit **only** `Cargo.toml` (`[workspace.package] version = "X.Y.Z"`). Use the version **ahead of main**.

### Workflows

| Workflow | Path | Role |
|---|---|---|
| `pr` | `.github/workflows/pr.yml` | Unified PR & main quality gate (semver bump + multi-OS test) |
| `release` | `.github/workflows/release.yml` | After PR workflow green on main → tag + multi-OS release publish |

### After a release

Confirm:
- [Releases](https://github.com/StaytunedLLP/open-document-spec/releases) shows `vX.Y.Z` (not a pre-release)
- Assets for Linux / macOS / Windows + `SHA256SUMS` present.
- `odc --version` matches that tag.

## License

This project is **proprietary** software owned by **Staytuned LLP**. See `LICENSE`.
Contributions, if accepted, are assigned to or licensed exclusively to Staytuned LLP
under terms agreed with the maintainers — they are not MIT or open-source licensed.
