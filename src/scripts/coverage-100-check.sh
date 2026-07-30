#!/usr/bin/env bash
# Fail if any instrumented .rs file has line coverage < 100%.
# Usage: ./src/scripts/coverage-100-check.sh
# Requires: cargo-llvm-cov, python3
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="${ODC_COVERAGE_DIR:-$ROOT/.artifacts/coverage}"
mkdir -p "$OUT"

echo "==> llvm-cov summary"
cargo llvm-cov --workspace --locked --summary-only "$@" | tee "$OUT/summary.txt"

python3 - <<'PY'
import re, sys
from pathlib import Path
text = Path(".artifacts/coverage/summary.txt").read_text(errors="replace")
# also accept OUT path relative
if not text.strip():
    text = open("summary.txt").read()
below = []
for line in text.splitlines():
    parts = line.split()
    if not parts or not parts[0].endswith(".rs"):
        continue
    pct_idxs = [i for i, p in enumerate(parts) if p.endswith("%") and p[0].isdigit()]
    if len(pct_idxs) < 3:
        continue
    li = pct_idxs[2]
    pct = float(parts[li].rstrip("%"))
    miss = int(parts[li - 1])
    tot = int(parts[li - 2])
    if pct < 100.0 - 1e-9:
        below.append((pct, parts[0], miss, tot))
below.sort()
if below:
    print(f"\n{len(below)} file(s) below 100% line coverage:\n", file=sys.stderr)
    for pct, name, miss, tot in below:
        print(f"  {pct:6.2f}%  miss {miss}/{tot}  {name}", file=sys.stderr)
    sys.exit(1)
print("All listed .rs files are at 100% line coverage.")
PY
