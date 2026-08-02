# Coverage excludes / T3 effect edges

These areas need mocks or platform CI; incomplete line coverage here does **not** block the workspace production bar (**≥90% lines** with T3 excluded).

## llvm-cov `--ignore-filename-regex`

Used by `./src/scripts/coverage.sh` and CI:

```text
(asset_downloader\.rs|update/installer\.rs|service/launchers\.rs|watch_and_serve_runner\.rs|okf_watch\.rs|github_release\.rs|setup_command\.rs)
```

Override locally with `ODS_COVERAGE_IGNORE_T3='...'` if needed.

| Path / region | Why excluded |
|---|---|
| `ods-cli/.../update/asset_downloader.rs` | Live HTTP download of release assets |
| `ods-cli/.../update/installer.rs` | Network install + binary replace orchestration |
| `ods-cli/.../update/github_release.rs` | Live GitHub Releases API + download orchestration |
| `ods-cli/.../service/launchers.rs` | `systemctl` / `launchctl` / privileged service enable |
| `ods-cli/.../service/watch_and_serve_runner.rs` | Long-running watch/serve loops, `ctrlc`, debounce |
| `ods-cli/.../okf/okf_watch.rs` | Long-running OKF watch/serve poll loop (same class as serve runner) |
| `ods-cli/.../setup_command.rs` | Machine first-run: editor install, service enable, health checks |

## Still counted (prefer tests / seams)

| Path / region | Notes |
|---|---|
| `update/archive_extractor.rs` | Temp archives in tests |
| `#[cfg(windows)]` / macOS-only arms | Single-OS CI job; document residual |

Prefer: unit tests for pure render helpers; integration tests with temp dirs for commands; inject seams rather than live network in CI.
