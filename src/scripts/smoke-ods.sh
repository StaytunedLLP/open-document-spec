#!/usr/bin/env bash
# End-to-end smoke after install-from-release.sh
set -euo pipefail
export PATH="${HOME}/.local/bin:${PATH}"

echo "==> version"
ods --version

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
cd "${TMP}"

echo "==> init"
ods init .

echo "==> index + lint"
ods index .
ods lint .

echo "==> export"
ods export . --out graph.md
test -s graph.md

echo "==> doctor"
ods doctor . | tee "${TMP}/doctor.txt"
grep -q "ods version:" "${TMP}/doctor.txt"
grep -q "documents:" "${TMP}/doctor.txt"
grep -q "indexes: current" "${TMP}/doctor.txt"
grep -q "profile conflicts: none" "${TMP}/doctor.txt"

echo "SMOKE OK at ${TMP}"
