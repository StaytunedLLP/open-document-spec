#!/usr/bin/env bash
# Measure ods serve RSS against service.max_rss_mb (default 10).
# Usage: ./src/scripts/mem-serve.sh [workspace] [budget_mb]
set -euo pipefail
ROOT="${1:-.}"
BUDGET_MB="${2:-}"
# Default 10MB for release; debug binaries typically need a higher canary.
if [[ -z "$BUDGET_MB" ]]; then
  if [[ "${ODS_BIN:-}" == *"/debug/"* ]] || [[ "${ODS_MEM_TEST_RELAXED:-}" == "1" ]]; then
    BUDGET_MB=32
  else
    BUDGET_MB=10
  fi
fi
LIMIT_KB=$((BUDGET_MB * 1024))

ODS_BIN="${ODS_BIN:-ods}"
if ! command -v "$ODS_BIN" >/dev/null 2>&1; then
  if [[ -x .artifacts/target/release/ods ]]; then
    ODS_BIN=.artifacts/target/release/ods
  elif [[ -x .artifacts/target/debug/ods ]]; then
    ODS_BIN=.artifacts/target/debug/ods
  else
    echo "error: ods binary not found (set ODS_BIN or build with cargo build -p ods-cli)"
    exit 2
  fi
fi

TMP=$(mktemp -d)
trap 'kill "$PID" 2>/dev/null || true; rm -rf "$TMP"' EXIT

"$ODS_BIN" init "$TMP/ws" >/dev/null
# Seed a few docs for a non-empty workspace
for i in 1 2 3 4 5; do
  cat >"$TMP/ws/doc$i.md" <<EOF
---
profile: note
status: draft
description: seed doc $i
---

# Doc $i
EOF
done

"$ODS_BIN" serve --mode poll --memory-report --poll-secs 60 --root "$TMP/ws" \
  2>"$TMP/serve.err" >/dev/null &
PID=$!
sleep 2
kill -TERM "$PID" 2>/dev/null || true
wait "$PID" 2>/dev/null || true

REPORT=$(grep -E 'rss_kb=' "$TMP/serve.err" | tail -1 || true)
echo "$REPORT"
RSS=$(echo "$REPORT" | sed -n 's/.*rss_kb=\([0-9][0-9]*\).*/\1/p')
if [[ -z "$RSS" ]]; then
  echo "error: could not parse rss_kb from serve output"
  cat "$TMP/serve.err" || true
  exit 1
fi
echo "budget_mb=$BUDGET_MB limit_kb=$LIMIT_KB rss_kb=$RSS"
if (( RSS > LIMIT_KB )); then
  echo "FAIL: rss_kb $RSS exceeds budget $LIMIT_KB KB"
  exit 1
fi
echo "OK: within ${BUDGET_MB}MB budget"
exit 0
