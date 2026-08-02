#!/usr/bin/env bash
# End-to-end smoke for Open Document Spec CLI (ods)
set -euo pipefail
export PATH="${HOME}/.local/bin:${PATH}"
export ODS_AUTO_UPDATE=0

echo "==> version"
ods --version

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
cd "${TMP}"

echo "==> ODS: init + lint + audit"
ods init .
ods index .
ods lint .
ods audit --write-report
test -f .ods/ods-errors.md

echo "==> OKF via flags: init + lint + doctor + audit"
OKF="${TMP}/okf"
ods init --okf "${OKF}" --attested
ods lint --okf "${OKF}"
ods doctor --okf "${OKF}"
ods audit --okf "${OKF}" --write-report

echo "==> Skills via flags: init + lint"
SKILL="${TMP}/demo-skill"
ods init --skills "${SKILL}"
ods lint --skills "${SKILL}"

echo "==> namespaces / --ods must fail"
set +e
ods okf lint "${OKF}"
code_ns=$?
ods lint --ods .
code_ods=$?
set -e
test "${code_ns}" -ne 0
test "${code_ods}" -ne 0

echo "SMOKE OK at ${TMP}"
