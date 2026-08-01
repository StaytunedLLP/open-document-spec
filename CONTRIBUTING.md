# Contributing to Open Document Spec (ODS)

Thank you for improving ODS. This repository holds the **spec**, **guide**, and **Rust reference implementation** (`ods` CLI).

## Development setup

```bash
# From repository root
cargo test --workspace --locked
cargo build --workspace --release --locked
cargo fmt --all
cargo clippy --workspace --all-targets --locked -- -D clippy::correctness -D clippy::suspicious
```

Binaries land under `.artifacts/target/release/` (see `.cargo/config.toml`).

Install locally:

```bash
cargo install --path src/crates/ods-cli --bin ods --locked --force
```

## Layout

| Path | Role |
| --- | --- |
| `specs/` | Normative specification (ODS keys stay `ods:`) |
| `docs/guide/` | End-user guide (keep in sync with `app-web/src/content/docs/`) |
| `docs/plan/` | Cutover & architecture plans (`docs/plan/aug-01/` master plan) |
| `docs/specs/` | Author key maps (ODS vs OKF) |
| `docs/maintainer/` | Coverage, functional style |
| `CHANGELOG.md` | Optional manual release history |
| `skills/ods` | AI assistant single skill |
| `src/crates/ods-core` | Core library (ODS + native OKF engines) |
| `src/crates/ods-cli` | Primary binary `ods` |
| `src/crates/ods-test-support` | Test workspace support library |
| `src/scripts/` | install, check-local, coverage, smoke-ods |
| `src/action/` | GitHub composite action |

### CLI crate layout (`src/crates/ods-cli/src/`)

| Path | Role |
| --- | --- |
| `main.rs` | Thin bin entrypoint (`include!` of main/*) |
| `main/cli/` | Entry, dispatch, argv parser, exit codes |
| `main/commands/` | Reorganized commands (`document/`, `profile/`, `workspace/`, `lifecycle/`, `service/`) |
| `main/support/` | Formatters, loaders, git helpers |
| `service/`, `update/` | OS service + self-update implementation |
| `tests/*.test.rs` | Integration tests (declared in `Cargo.toml`) |

### Core crate layout (`src/crates/ods-core/src/`)

Semantic domain subfolders (`model/`, `parse/`, `profiles/`, `lint/`, `graph/`, `lifecycle/`, `fs/`, `bench/`, `export/`).

New engine code should follow [functional style](docs/maintainer/functional-style.md): data + free functions, no `*Manager` types.

**Product naming:** **`ods`** is the tool; bare commands run ODS natively; OKF runs via `--okf` flag.

Coverage: see [docs/maintainer/coverage.md](docs/maintainer/coverage.md). Run `./src/scripts/coverage.sh`. CI enforces a line floor (currently 75%).

## Tests & coverage

- Prefer unit tests next to pure logic and integration tests under `src/crates/*/tests/`.
- Production bar: high line coverage on `ods-core` (aim ≥85% workspace over time).
- CI enforces a coverage floor (`--fail-under-lines`, currently 75%).
- Always keep `ods index --check .` and `ods lint .` green at repo root.

```bash
cargo install cargo-llvm-cov --locked
cargo llvm-cov --workspace --locked --fail-under-lines 75
```

## CI

Workflows in `.github/workflows/` consist of **two workflows** total:

| Workflow | When | Notes |
| --- | --- | --- |
| `pr.yml` | PRs (open/sync/labels), push to `main`, manual | Unified quality gate: Linux & Windows tests (`fmt`/`clippy`/`test`/`ods`/`cov`) |
| `release.yml` | After `pr.yml` succeeds on main | Multi-OS binary builds, GitHub Release tag `vX.Y.Z` publish |

**Do not create release tags by hand.** Releases are cut automatically when CI is green on main.

## Pull requests

1. Keep changes focused (spec vs guide vs code).
2. Prefer [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat(ods): …`, `fix(watch): …`, `docs: …`, `ci: …`, `chore: …`
3. Update `docs/guide/` if you add a user-visible feature.
