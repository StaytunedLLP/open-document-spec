# Test coverage policy (OpenDocify)

## Run reports

```bash
# From repository root
./src/scripts/coverage.sh              # summary + HTML under .artifacts/coverage/
cargo llvm-cov --workspace --locked --summary-only
cargo llvm-cov --workspace --locked --html --output-dir .artifacts/coverage/html
```

CI (`.github/workflows/pr.yml`) enforces `--fail-under-lines` (ratchet upward only).

## Tiers

| Tier | Scope | Target |
|---|---|---|
| **T0** | Pure engine (parse, lint, pipeline, refs, index, OKF rules) | ≥95% lines; 100% on pure helpers where feasible |
| **T1** | All `odc-core` | ≥90% lines |
| **T2** | CLI command orchestration | ≥80% lines |
| **T3** | Network download, OS service install, platform `cfg` | Mocked/smoke; see [coverage-excludes.md](./coverage-excludes.md) |

**“100% product logic”** means T0+T1 with T3 excludes documented — not literal 100% of every OS/network line.

## Baseline & measured

| Date | Workspace / engine lines | Notes |
|---|---|---|
| 2026-07-30 (start) | workspace ~**73.2%** | CI floor 73 |
| 2026-07-30 (mid) | workspace ~**76%**; engine ~**81%** | CI floor **75** |
| 2026-07-30 (OKF/audit push) | engine **`odc-core` ~84.2%** | **100% files:** `okf/audit.rs`, `okf/model.rs`, `pipeline/apply.rs` |

Strict goal remains **100% per file** (see plan). Progress scripts:

```bash
./src/scripts/coverage.sh
./src/scripts/coverage-100-check.sh   # exits 1 until every file is 100%
```

Raise CI floor only when workspace llvm-cov is stable: 75 → 80 → 85 → 90 → 100.
