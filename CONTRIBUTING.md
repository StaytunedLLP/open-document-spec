# Contributing to Open Document Spec (ODS)

Thank you for improving ODS. This repository holds the **spec**, **guide**, and **Rust reference implementation** (`ods` CLI).

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
cargo install --path crates/ods --bin ods --locked --force
```

## Layout

| Path | Role |
| --- | --- |
| `specs/` | Normative specification |
| `docs/guide/` | End-user guide |
| `docs/FEATURE_MATRIX.md` | Keys × auto-update matrix |
| `CHANGELOG.md` | Optional manual release history |
| `src/crates/ods-core` | Shared library |
| `src/crates/ods` | `ods` CLI (only shipped binary) |

## Tests & coverage

- Prefer unit tests next to pure logic and integration tests under `crates/*/tests/`.
- Production bar: high line coverage on `ods-core` (aim ≥85% workspace over time).
- CI enforces a coverage floor (`--fail-under-lines`, currently 73%, see `.github/workflows/ci.yml`) that ratchets upward as coverage improves — never lower it to make a PR pass; add tests instead.
- Always keep `ods index --check .` and `ods lint .` green at repo root.

```bash
cargo install cargo-llvm-cov --locked
cd src && cargo llvm-cov --workspace --locked --fail-under-lines 73
```

## CI

Workflows in `.github/workflows/` consist of **two workflows** total:

| Workflow | When | Notes |
| --- | --- | --- |
| `pr.yml` | PRs (open/sync/labels), push to `main`, manual | Unified quality gate: PR version bump + Linux & Windows tests (`fmt`/`clippy`/`test`/`ods`/`cov`) |
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
3. Update `docs/FEATURE_MATRIX.md` / guide if you add a user-visible feature.
4. Do not commit `.artifacts/` or `target/`.
5. Ensure CI is green on the self-hosted runner.
6. **Reuse the same PR branch** — keep pushing commits to the existing head branch; do not open a new branch/PR for follow-up fixes on the same change set.

## Versioning & release workflow (Maintainers)

End users install **only** from [GitHub Releases](https://github.com/StaytunedLLP/open-document-spec/releases) via `src/scripts/install.sh` / `src/scripts/install.ps1` or manual archive download; then use `ods setup` / `ods update`. Do **not** cut releases by pushing a tag by hand.

### Where the version lives

**One line** controls the published version for `ods` and `ods-core` (workspace packages):

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
   - builds multi-OS **`ods`** archives
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
- `ods --version` matches that tag.

## License

This project is **proprietary** software owned by **Staytuned LLP**. See `LICENSE`.
Contributions, if accepted, are assigned to or licensed exclusively to Staytuned LLP
under terms agreed with the maintainers — they are not MIT or open-source licensed.
