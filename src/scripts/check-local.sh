#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SAMPLE="${ROOT}/ods-test/ecommerce"
EXPORT_OUT="${TMPDIR:-/tmp}/ods-graph-local.md"

export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"
export ODS_AUTO_UPDATE=0
export ODC_AUTO_UPDATE=0

find_rustup() {
  if command -v rustup >/dev/null 2>&1; then
    command -v rustup
    return 0
  fi

  for candidate in "${HOME}/.cargo/bin/rustup" /opt/homebrew/bin/rustup /usr/local/bin/rustup; do
    if [ -x "${candidate}" ]; then
      echo "${candidate}"
      return 0
    fi
  done

  return 1
}

if RUSTUP="$(find_rustup 2>/dev/null)"; then
  TOOLCHAIN="${RUSTUP_TOOLCHAIN:-$("${RUSTUP}" show active-toolchain | awk '{print $1}')}"
  "${RUSTUP}" component add rustfmt clippy --toolchain "${TOOLCHAIN}" >/dev/null
  TOOLBIN="$(dirname "$("${RUSTUP}" which rustc --toolchain "${TOOLCHAIN}")")"
  export PATH="${TOOLBIN}:${PATH}"
fi

run() {
  echo "==> $*"
  "$@"
}

cd "${ROOT}"
run cargo fmt --all -- --check
run cargo clippy --workspace --all-targets --locked -- -D warnings
run cargo test --workspace --locked
if [ "${SKIP_RELEASE_BUILD:-}" != "true" ]; then
  run cargo build --workspace --release --locked
fi

cd "${ROOT}"
ODS=""
for candidate in \
  "${ROOT}/.artifacts/target/release/ods" \
  "${ROOT}/target/release/ods" \
  "${ROOT}/.artifacts/target/debug/ods" \
  "${ROOT}/target/debug/ods" \
  "${ROOT}/.artifacts/target/release/ods" \
  "${ROOT}/target/release/ods" \
  "${ROOT}/.artifacts/target/debug/ods" \
  "${ROOT}/target/debug/ods"; do
  if [ -x "${candidate}" ]; then
    ODS="${candidate}"
    break
  fi
done

if [ -z "${ODS}" ]; then
  echo "error: ods/ods binary not found" >&2
  find "${ROOT}" -name ods -o -name ods -type f 2>/dev/null | head >&2
  exit 1
fi

# Use ODS namespace when binary is ods
if [[ "$(basename "${ODS}")" == "ods" ]]; then
  ODS_CMD=("${ODS}" ods)
else
  ODS_CMD=("${ODS}")
fi

FIXTURES=(
  "${ROOT}/ods-test/ecommerce"
  "${ROOT}/ods-test/policy-handbook"
  "${ROOT}/ods-test/packs/engineering-pack"
)

for fixture in "${FIXTURES[@]}"; do
  if [ -d "${fixture}" ]; then
    run "${ODS_CMD[@]}" index --check "${fixture}"
    run "${ODS_CMD[@]}" lint "${fixture}"
  fi
done

run "${ODS_CMD[@]}" export "${SAMPLE}" --out "${EXPORT_OUT}"
test -s "${EXPORT_OUT}"
grep -q "ODS workspace graph" "${EXPORT_OUT}"

# OKF smoke when binary is ods
if [[ "$(basename "${ODS}")" == "ods" ]]; then
  OKF_TMP=$(mktemp -d)
  run "${ODS}" okf init "${OKF_TMP}"
  run "${ODS}" okf lint "${OKF_TMP}"
  rm -rf "${OKF_TMP}"
fi

run "${ROOT}/src/action/scripts/test-action.sh"
echo "local checks passed"

# Install script drift (src is source of truth for site copies)
if ! diff -q "${ROOT}/src/scripts/install.sh" "${ROOT}/app-web/public/install.sh" >/dev/null 2>&1; then
  echo "warning: app-web/public/install.sh differs from src/scripts/install.sh" >&2
fi
if ! diff -q "${ROOT}/src/scripts/install.ps1" "${ROOT}/app-web/public/install.ps1" >/dev/null 2>&1; then
  echo "warning: app-web/public/install.ps1 differs from src/scripts/install.ps1" >&2
fi
if ! diff -q "${ROOT}/src/scripts/install.sh" "${ROOT}/skills/ods/scripts/install-from-release.sh" >/dev/null 2>&1; then
  echo "warning: skills/ods/scripts/install-from-release.sh differs from src/scripts/install.sh" >&2
fi
