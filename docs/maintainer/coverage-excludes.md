# Coverage excludes / T3 effect edges

These areas need mocks or platform CI; incomplete line coverage here does **not** block “100% product logic.”

| Path / region | Why hard to hit at 100% |
|---|---|
| `ods/src/update/asset_downloader.rs` | Live HTTP to GitHub |
| `ods/src/update/installer.rs` (network branches) | Release download |
| `ods/src/service/launchers.rs` (enable systemctl/launchctl) | Real OS privileges |
| `ods/src/main/watch_and_serve_runner.rs` (`ctrlc`, debounce loop) | Long-running / signal |
| `#[cfg(windows)]` / `#[cfg(target_os = "macos")]` alternate arms | Single OS CI job |

Prefer: unit tests for pure render helpers; integration tests with temp dirs for commands; inject seams rather than live network in CI.
