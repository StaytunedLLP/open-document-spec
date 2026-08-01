#!/usr/bin/env bash
# End-to-end smoke for Open Document Spec CLI (ods) + legacy ods argv0
set -euo pipefail
export PATH="${HOME}/.local/bin:${PATH}"
export ODS_AUTO_UPDATE=0
export ODC_AUTO_UPDATE=0

echo "==> version"
ods --version || ods --version

TMP="$(mktemp -d)"
trap 'rm -rf "${TMP}"' EXIT
cd "${TMP}"

echo "==> ods ods init + lint"
ods ods init .
ods ods index .
ods ods lint .
ods ods audit --write-report
test -f .ods/ods-errors.md

echo "==> ods okf init + lint"
OKF="${TMP}/okf"
ods okf init "${OKF}" --attested
ods okf lint "${OKF}"
ods okf doctor "${OKF}"
ods okf audit "${OKF}" --write-report

echo "==> bare ods lint must fail (namespace required)"
set +e
ods lint .
code=$?
set -e
test "${code}" -eq 2

echo "==> legacy ods argv0 still works"
if command -v ods >/dev/null 2>&1; then
  ods lint .
fi

echo "SMOKE OK at ${TMP}"
