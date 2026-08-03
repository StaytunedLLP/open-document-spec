#!/usr/bin/env bash
# Generate Open Document Spec workspace coverage summary + HTML.
#
# T3 effect edges (network download, OS service install, long-running watch)
# are excluded from the denominator via --ignore-filename-regex so the
# production bar measures product logic. See docs/maintainer/coverage-excludes.md.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
OUT="${ODS_COVERAGE_DIR:-${ODC_COVERAGE_DIR:-$ROOT/.artifacts/coverage}}"
mkdir -p "$OUT"

# Shared T3 ignore list (keep in sync with CI and coverage-excludes.md)
IGNORE_T3="${ODS_COVERAGE_IGNORE_T3:-(asset_downloader\\.rs|update/installer\\.rs|service/launchers\\.rs|watch_and_serve_runner\\.rs|okf_watch\\.rs|github_release\\.rs|setup_command\\.rs)}"
FAIL_UNDER="${ODS_COVERAGE_FAIL_UNDER_LINES:-}"

if ! command -v cargo-llvm-cov >/dev/null 2>&1 && ! cargo llvm-cov --version >/dev/null 2>&1; then
  echo "error: cargo-llvm-cov not installed. Run: cargo install cargo-llvm-cov --locked" >&2
  exit 1
fi

EXTRA=()
if [[ -n "$FAIL_UNDER" ]]; then
  EXTRA+=(--fail-under-lines "$FAIL_UNDER")
fi

echo "==> summary (T3 excludes applied)"
cargo llvm-cov --workspace --locked \
  --ignore-filename-regex "$IGNORE_T3" \
  --summary-only \
  "${EXTRA[@]}" \
  "$@" | tee "$OUT/summary.txt"

echo "==> HTML → $OUT (index at $OUT/html/index.html)"
cargo llvm-cov --workspace --locked \
  --ignore-filename-regex "$IGNORE_T3" \
  --html --output-dir "$OUT" \
  "$@"
# Rank files by missed lines (below 90%)
python3 - <<'PY' || true
from pathlib import Path
text = Path(".artifacts/coverage/summary.txt").read_text(errors="replace")
rows = []
for line in text.splitlines():
    parts = line.split()
    if not parts or not parts[0].endswith(".rs"):
        continue
    pct_idxs = [i for i, p in enumerate(parts) if p.endswith("%") and (p[0].isdigit() or p.startswith("0"))]
    if len(pct_idxs) < 3:
        continue
    li = pct_idxs[2]
    try:
        tot = int(parts[li - 2])
        miss = int(parts[li - 1])
        pct = float(parts[li].rstrip("%"))
    except Exception:
        continue
    rows.append((miss, pct, tot, parts[0]))
rows.sort(reverse=True)
out = Path(".artifacts/coverage/missing_ranked.txt")
lines = [f"{'MISS':>5} {'PCT':>7} {'TOT':>5} FILE"]
for miss, pct, tot, name in rows:
    if pct < 90.0 - 1e-9:
        lines.append(f"{miss:5d} {pct:6.2f}% {tot:5d} {name}")
out.write_text("\n".join(lines) + "\n")
print(f"==> ranked misses (<90%) → {out} ({len(lines)-1} files)")
# Print TOTAL line if present
for line in text.splitlines():
    if line.startswith("TOTAL") or line.strip().startswith("TOTAL"):
        print(line)
        break
PY

echo "==> done. Open $OUT/html/index.html (summary: $OUT/summary.txt)"
