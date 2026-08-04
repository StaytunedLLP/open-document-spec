#!/usr/bin/env bash
# Cheap always-on checks before claiming work complete.
# Usage: .agents/hooks/scripts/pre-handoff.sh [--full]
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
cd "$ROOT"

echo "==> pre-handoff: naming + odc residue"
./src/scripts/check-naming.sh
./src/scripts/check-odc-residue.sh

if [[ "${1:-}" == "--full" ]]; then
  echo "==> pre-handoff: full local gate"
  SKIP_RELEASE_BUILD="${SKIP_RELEASE_BUILD:-true}" ./src/scripts/check-local.sh
  if [[ "${ODS_COVERAGE_HANDOFF:-}" == "1" ]]; then
    echo "==> pre-handoff: coverage ≥90%"
    ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh
  fi
fi

echo "pre-handoff: OK"
