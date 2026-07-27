#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SAMPLE="${ROOT}/ods-test/ecommerce"
EXPORT_OUT="${TMPDIR:-/tmp}/ods-graph-local.md"

export CARGO_TERM_COLOR="${CARGO_TERM_COLOR:-always}"
export RUST_BACKTRACE="${RUST_BACKTRACE:-1}"

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
  "${ROOT}/target/debug/ods"; do
  if [ -x "${candidate}" ]; then
    ODS="${candidate}"
    break
  fi
done

if [ -z "${ODS}" ]; then
  echo "error: ods binary not found" >&2
  find "${ROOT}" -name ods -type f 2>/dev/null | head >&2
  exit 1
fi

FIXTURES=(
  "${ROOT}/ods-test/ecommerce"
  "${ROOT}/ods-test/policy-handbook"
  "${ROOT}/ods-test/packs/engineering-pack"
)

for fixture in "${FIXTURES[@]}"; do
  if [ -d "${fixture}" ]; then
    run "${ODS}" index --check "${fixture}"
    run "${ODS}" lint "${fixture}"
  fi
done

run "${ODS}" export "${SAMPLE}" --out "${EXPORT_OUT}"
test -s "${EXPORT_OUT}"
grep -q "ODS workspace graph" "${EXPORT_OUT}"
run "${ROOT}/src/action/scripts/test-action.sh"
echo "local checks passed"
