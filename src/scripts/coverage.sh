#!/usr/bin/env bash
# Generate OpenDocify workspace coverage summary + HTML.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="${ODC_COVERAGE_DIR:-$ROOT/.artifacts/coverage}"
mkdir -p "$OUT"

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "error: cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov --locked" >&2
  exit 1
fi

echo "==> summary"
cargo llvm-cov --workspace --locked --summary-only "$@" | tee "$OUT/summary.txt"

echo "==> HTML → $OUT/html"
cargo llvm-cov --workspace --locked --html --output-dir "$OUT/html" "$@"

echo "==> done. Open $OUT/html/index.html"
