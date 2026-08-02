#!/usr/bin/env bash
# Build release binaries and a local archive shaped like GitHub Releases (ods-*).
# Does NOT publish. Use to validate installers offline.
#
#   ./src/scripts/package-local-release.sh
#   OUT=/tmp/my-out ./src/scripts/package-local-release.sh
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${ROOT}"

VERSION="$(grep -E '^version\s*=' Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
TAG="v${VERSION}"
ASSET="${ASSET:-linux-x86_64}"
OUT_DIR="${OUT:-${ROOT}/dist-local}"
NAME="ods-${TAG}-${ASSET}"

echo "==> cargo build --release -p ods"
cargo build --release -p ods --bin ods --bin ods

BIN_DIR=""
for d in .artifacts/target/release target/release; do
  if [ -x "${d}/ods" ]; then BIN_DIR="${d}"; break; fi
done
[ -n "${BIN_DIR}" ] || { echo "error: ods binary not found" >&2; exit 1; }

rm -rf "${OUT_DIR}/${NAME}"
mkdir -p "${OUT_DIR}/${NAME}"
cp "${BIN_DIR}/ods" "${OUT_DIR}/${NAME}/ods"
if [ -x "${BIN_DIR}/ods" ]; then
  cp "${BIN_DIR}/ods" "${OUT_DIR}/${NAME}/ods"
else
  cp "${BIN_DIR}/ods" "${OUT_DIR}/${NAME}/ods"
fi

cat > "${OUT_DIR}/${NAME}/INSTALL.txt" <<EOF
Open Document Spec CLI ${TAG} (${ASSET}) — local package (not a GitHub Release)

  install -m 755 ods ~/.local/bin/ods
  ln -sfn ~/.local/bin/ods ~/.local/bin/ods

  ods --version
  ods lint .
  ods init --okf /tmp/okf-demo && ods lint --okf /tmp/okf-demo
EOF

(
  cd "${OUT_DIR}"
  tar czf "${NAME}.tar.gz" "${NAME}"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${NAME}.tar.gz" > "SHA256SUMS.${ASSET}"
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${NAME}.tar.gz" > "SHA256SUMS.${ASSET}"
  fi
)

echo "==> wrote ${OUT_DIR}/${NAME}.tar.gz"
ls -lh "${OUT_DIR}/${NAME}.tar.gz" "${OUT_DIR}/SHA256SUMS.${ASSET}" 2>/dev/null || true
