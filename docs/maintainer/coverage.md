# Test coverage policy (Open Document Spec)

## Run reports

```bash
# From repository root (applies T3 excludes; see coverage-excludes.md)
./src/scripts/coverage.sh              # summary + HTML under .artifacts/coverage/
cargo llvm-cov --workspace --locked \
  --ignore-filename-regex '(asset_downloader\.rs|update/installer\.rs|update/binary_replacer\.rs|update/http_helpers\.rs|service/launchers\.rs|watch_and_serve_runner\.rs|okf_watch\.rs|github_release\.rs|setup_command\.rs|lsp_command\.rs|git_sync\.rs)' \
  --summary-only
```

Optional local gate:

```bash
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
```

CI (`.github/workflows/pr.yml`) enforces `--fail-under-lines` with the same T3 ignore list (ratchet upward only).

## Tiers

| Tier | Scope | Target |
|---|---|---|
| **Workspace bar** | All crates, **minus T3 excludes** | **≥90% lines** (production readiness) |
| **T0** | Pure engine (parse, lint, pipeline, refs, index, OKF rules) | ≥95% lines; 100% on pure helpers where feasible |
| **T1** | All `ods-core` | ≥90% lines |
| **T2** | CLI command orchestration | ≥80% lines |
| **T3** | Network download, OS service install, long-running watch | **Excluded** from workspace bar; see [coverage-excludes.md](./coverage-excludes.md) |

**“100% product logic”** means T0+T1 with T3 excludes documented — not literal 100% of every OS/network line.

## Baseline & measured

| Date | Workspace / engine lines | Notes |
|---|---|---|
| 2026-07-30 (start) | workspace ~**73.2%** | CI floor 73 |
| 2026-07-30 (mid) | workspace ~**76%**; engine ~**81%** | CI floor **75** |
| 2026-07-30 (OKF/audit push) | engine `ods-core` ~**84.2%** | **100% files:** `okf/audit.rs`, `okf/model.rs`, `pipeline/apply.rs` |
| 2026-07-30 (100% coverage PR) | workspace **82.31%** lines | prior peak before multi-spec |
| 2026-08-02 (multi-spec flags) | workspace **~74.65%** / later **76.79%** raw | CI floor **73**; multi_spec + skills engine |
| 2026-08-02 (coverage elevation) | workspace **~88.9%** with T3 ignore; **`ods-core` ~92%**; CLI ~**84%** | CI floor **73 → 88**; stretch target remains **≥90%** workspace |
| 2026-08-04 (schema + edge push) | target workspace **≥90%** with T3 ignore; non-T3 files **≥90%** | CI floor **88 → 90**; schema-driven keys + residue gates |

Strict aspirational goal remains **100% per file** (see `coverage-100-check.sh`). Progress:

```bash
./src/scripts/coverage.sh
ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
./src/scripts/coverage-100-check.sh   # exits 1 until every file is 100% (no T3 ignore)
```

Raise CI floor only when workspace llvm-cov is stable: **90** → 95 → 100.
