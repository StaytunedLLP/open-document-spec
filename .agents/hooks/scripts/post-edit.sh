#!/usr/bin/env bash
# Called from post-edit hook with the edited path as $1 (optional).
set -euo pipefail
p="${1:-${TOOL_INPUT_PATH:-${TOOL_INPUT_file_path:-}}}"
[ -z "$p" ] && exit 0

case "$p" in
  */specs/*|*skills/ods/references*)
    echo "[.agents] Spec/skill refs edited — run: ods lint (add --okf/--skills if those trees changed); sync skills/ods/references if ODS keys changed"
    ;;
  */src/ods-core/src/spec/*|*/specs/*/keys.md)
    echo "[.agents] Keys/schema path — keep SpecSchemaRegistry + specs/*/keys.md in the same change; test: cargo test -p ods-core --lib spec::schema"
    ;;
  */src/ods-*/*|*/src/scripts/*)
    echo "[.agents] Engine/scripts edited — before handoff: SKIP_RELEASE_BUILD=true ./src/scripts/check-local.sh"
    echo "[.agents] If tests/coverage-sensitive: ODS_COVERAGE_FAIL_UNDER_LINES=90 ./src/scripts/coverage.sh"
    ;;
  */src/scripts/install.*)
    echo "[.agents] Install script is SoT — sync app-web/public/ and skills/ods/scripts/ copies"
    ;;
  */app-web/src/*)
    echo "[.agents] Site edit — check nav, sitemap, redirects, llms.txt if routes/keys changed"
    ;;
  */docs/**)
    echo "[.agents] Docs edit — prefer links to docs/plan/archive/ for historical plans; avoid dead paths"
    ;;
esac
