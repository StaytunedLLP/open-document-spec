#!/usr/bin/env bash
# Build release binaries and a local archive shaped like GitHub Releases (odc-*).
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
NAME="odc-${TAG}-${ASSET}"

echo "==> cargo build --release -p odc"
cargo build --release -p odc --bin odc --bin ods

BIN_DIR=""
for d in .artifacts/target/release target/release; do
  if [ -x "${d}/odc" ]; then BIN_DIR="${d}"; break; fi
done
[ -n "${BIN_DIR}" ] || { echo "error: odc binary not found" >&2; exit 1; }

rm -rf "${OUT_DIR}/${NAME}"
mkdir -p "${OUT_DIR}/${NAME}"
cp "${BIN_DIR}/odc" "${OUT_DIR}/${NAME}/odc"
if [ -x "${BIN_DIR}/ods" ]; then
  cp "${BIN_DIR}/ods" "${OUT_DIR}/${NAME}/ods"
else
  cp "${BIN_DIR}/odc" "${OUT_DIR}/${NAME}/ods"
fi

cat > "${OUT_DIR}/${NAME}/INSTALL.txt" <<EOF
OpenDocify CLI ${TAG} (${ASSET}) — local package (not a GitHub Release)

  install -m 755 odc ~/.local/bin/odc
  ln -sfn ~/.local/bin/odc ~/.local/bin/ods

  odc --version
  odc ods lint .
  odc okf init /tmp/okf-demo && odc okf lint /tmp/okf-demo
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
