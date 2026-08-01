#!/usr/bin/env bash
# Unit & integration test suite for ODS GitHub Action setup script & matcher
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

info()  { echo "==> [TEST] $*"; }
fatal() { echo "error: [TEST FAIL] $*" >&2; exit 1; }

# 1. Validate problem-matcher.json syntax
info "Test 1: Validate problem-matcher.json JSON syntax"
if command -v python3 >/dev/null 2>&1; then
  python3 -c "import json; json.load(open('${REPO_ROOT}/src/action/problem-matcher.json'))" \
    || fatal "problem-matcher.json is invalid JSON"
fi

# 2. Test problem-matcher regex pattern matching
info "Test 2: Validate problem-matcher regex against ODS lint output"
if command -v python3 >/dev/null 2>&1; then
  python3 -c '
import json, os, re
matcher = json.load(open("'"${REPO_ROOT}"'/src/action/problem-matcher.json"))
pattern = matcher["problemMatcher"][0]["pattern"][0]["regexp"]
regex = re.compile(pattern)
sample = "error: docs/setup.md: dangling reference to nonexistent"
m = regex.match(sample)
assert m, "Regex failed to match sample"
assert m.group(1) == "error"
assert m.group(2) == "docs/setup.md"
assert m.group(3) == "dangling reference to nonexistent"
' || fatal "problem-matcher regex pattern check failed"
fi

# 3. Test setup-only mode
info "Test 3: Execute setup.sh in setup-only mode (command: none)"
INPUT_COMMAND="none" \
INPUT_VERSION="v0.1.24" \
INPUT_PATH="${REPO_ROOT}/ods-test/ecommerce" \
"${SCRIPT_DIR}/setup.sh" || fatal "setup.sh failed in setup-only mode"

# 4. Test clean workspace linting
info "Test 4: Execute setup.sh lint command on clean workspace"
INPUT_COMMAND="lint" \
INPUT_VERSION="v0.1.24" \
INPUT_PATH="${REPO_ROOT}/ods-test/ecommerce" \
"${SCRIPT_DIR}/setup.sh" || fatal "setup.sh lint command failed on clean workspace"

# 5. Test clean workspace index check
info "Test 5: Execute setup.sh index-check command"
INPUT_COMMAND="index-check" \
INPUT_VERSION="v0.1.24" \
INPUT_PATH="${REPO_ROOT}/ods-test/ecommerce" \
"${SCRIPT_DIR}/setup.sh" || fatal "setup.sh index-check command failed"

# 6. Test failure detection on broken workspace
info "Test 6: Verify setup.sh detects lint errors and exits non-zero"
TMP_WORKSPACE=$(mktemp -d)
trap 'rm -rf "${TMP_WORKSPACE}"' EXIT

cat > "${TMP_WORKSPACE}/index.md" << 'EOF'
---
ods: 0.1
ods: ">=0.0.1"
---
# Root Index
EOF

cat > "${TMP_WORKSPACE}/doc.md" << 'EOF'
---
profile: note
status: draft
depends: [nonexistent-doc]
---
# Test Doc
EOF

if INPUT_COMMAND="lint" INPUT_VERSION="v0.1.24" INPUT_PATH="${TMP_WORKSPACE}" "${SCRIPT_DIR}/setup.sh" 2>/dev/null; then
  fatal "setup.sh should have failed on broken workspace with dangling reference"
else
  info "Correctly detected lint error and exited non-zero"
fi

echo ""
info "All 6 GitHub Action local tests passed successfully!"
