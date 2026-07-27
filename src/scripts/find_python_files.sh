#!/usr/bin/env bash
set -euo pipefail

# Script to find all Python files (.py) and inline Python usages in the codebase.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

cd "${REPO_ROOT}"

echo "========================================================================="
echo "                  PYTHON FILES & SNIPPETS FINDER                         "
echo "========================================================================="
echo ""

echo "--- 1. Python Files (.py) ---"
PYTHON_FILES=$(find . -type f -name "*.py" -not -path "*/target/*" -not -path "*/.git/*")

if [ -n "${PYTHON_FILES}" ]; then
  echo "${PYTHON_FILES}"
else
  echo "No .py files found."
fi

echo ""
echo "--- 2. Files containing inline Python commands or shebangs ---"
INLINE_PYTHON=$(grep -rnE "(python3? -c|python3? <<|#!/usr/bin/env python3?)" . \
  --exclude-dir={.git,target,node_modules} \
  --exclude="src/scripts/find_python_files.sh" || true)

if [ -n "${INLINE_PYTHON}" ]; then
  echo "${INLINE_PYTHON}"
else
  echo "No inline Python usage found."
fi

echo ""
echo "========================================================================="
