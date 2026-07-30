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
ODC=""
for candidate in \
  "${ROOT}/.artifacts/target/release/odc" \
  "${ROOT}/target/release/odc" \
  "${ROOT}/.artifacts/target/debug/odc" \
  "${ROOT}/target/debug/odc" \
  "${ROOT}/.artifacts/target/release/ods" \
  "${ROOT}/target/release/ods" \
  "${ROOT}/.artifacts/target/debug/ods" \
  "${ROOT}/target/debug/ods"; do
  if [ -x "${candidate}" ]; then
    ODC="${candidate}"
    break
  fi
done

if [ -z "${ODC}" ]; then
  echo "error: odc/ods binary not found" >&2
  find "${ROOT}" -name odc -o -name ods -type f 2>/dev/null | head >&2
  exit 1
fi

# Use ODS namespace when binary is odc
if [[ "$(basename "${ODC}")" == "odc" ]]; then
  ODS_CMD=("${ODC}" ods)
else
  ODS_CMD=("${ODC}")
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

# OKF smoke when binary is odc
if [[ "$(basename "${ODC}")" == "odc" ]]; then
  OKF_TMP=$(mktemp -d)
  run "${ODC}" okf init "${OKF_TMP}"
  run "${ODC}" okf lint "${OKF_TMP}"
  rm -rf "${OKF_TMP}"
fi

run "${ROOT}/src/action/scripts/test-action.sh"
echo "local checks passed"
