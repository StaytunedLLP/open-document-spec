#!/usr/bin/env bash
# End-to-end smoke for OpenDocify CLI (odc) + legacy ods argv0
set -euo pipefail
export PATH="${HOME}/.local/bin:${PATH}"
export ODS_AUTO_UPDATE=0
export ODC_AUTO_UPDATE=0

echo "==> version"
odc --version || ods --version

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
cd "${TMP}"

echo "==> odc ods init + lint"
odc ods init .
odc ods index .
odc ods lint .
odc ods audit --write-report
test -f .odc/odc-errors.md

echo "==> odc okf init + lint"
OKF="${TMP}/okf"
odc okf init "${OKF}" --attested
odc okf lint "${OKF}"
odc okf doctor "${OKF}"
odc okf audit "${OKF}" --write-report

echo "==> bare odc lint must fail (namespace required)"
set +e
odc lint .
code=$?
set -e
test "${code}" -eq 2

echo "==> legacy ods argv0 still works"
if command -v ods >/dev/null 2>&1; then
  ods lint .
fi

echo "SMOKE OK at ${TMP}"
