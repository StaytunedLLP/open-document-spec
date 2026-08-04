#!/usr/bin/env bash
# Fail on product/code/fixture regressions that reintroduce legacy `odc` naming.
# Allowlisted: educational "do not invent", dual-read legacy env, CHANGELOG history,
# archive plans under docs/plan/archive/, and intentional preserve-unknown tests.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

fail=0

# Frontmatter fixture pin key (must not appear in tests/product code).
# Allowed only with nearby "legacy" or "preserve" commentary for intentional tests.
scan_frontmatter_odc() {
  local hits
  hits=$(grep -rnE 'odc:[[:space:]]*["'\'']?>=' \
    src/ods-core src/ods-cli src/ods-test-support src/fixtures \
    --include='*.rs' --include='*.md' 2>/dev/null || true)
  if [ -n "$hits" ]; then
    echo "ODC RESIDUE FAIL: frontmatter pin key odc: still present"
    echo "$hits"
    fail=1
  fi
}

# Primary product teaching of binary name `odc` (not dual-compat discovery).
scan_primary_binary_odc() {
  local hits
  hits=$(grep -rnE 'binary.*\bodc\b|primary.*\bodc\b|install.*\bodc\b' \
    src/scripts skills/ods/scripts docs/guide docs/other-specs docs/maintainer \
    README.md CONTRIBUTING.md \
    --include='*.sh' --include='*.ps1' --include='*.md' 2>/dev/null | \
    grep -vE 'legacy|historical|do not invent|ODC_AUTO|ODC_JOBS|odcconfig|archive|fallback|compat' || true)
  if [ -n "$hits" ]; then
    echo "ODC RESIDUE FAIL: product text still teaches odc as primary"
    echo "$hits"
    fail=1
  fi
}

# Hardcoded fixture strings that still embed odc: keys in Rust tests.
# Allow intentional preserve-unknown tests (filename or same-line allowlist).
scan_rust_fixture_odc() {
  local hits
  hits=$(grep -rn 'odc:' src/ods-core src/ods-cli --include='*.rs' 2>/dev/null | \
    grep -vE 'do not invent|legacy|preserve|unknown key|ODC_|legacy_odc_|multi_spec_and_skills\.test\.rs' || true)
  if [ -n "$hits" ]; then
    echo "ODC RESIDUE FAIL: rust still embeds odc: (non-allowlisted)"
    echo "$hits"
    fail=1
  fi
}

scan_frontmatter_odc
scan_primary_binary_odc
scan_rust_fixture_odc

if [ "$fail" -ne 0 ]; then
  exit 1
fi
echo "check-odc-residue: OK"
