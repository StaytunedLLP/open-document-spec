#!/usr/bin/env bash
# OpenDocify (odc) installer — downloads prebuilt `odc` (+ `ods` symlink) binary from GitHub Releases.
#
# Supported platforms (auto-detected):
#   macOS  — Apple Silicon (arm64), Intel (x86_64)
#   Linux  — x86_64, arm64
#
# Windows: use install.ps1 instead.
#
# This repository is private. Export a GitHub token before running:
#   export GH_TOKEN="$(gh auth token)"   # or GITHUB_TOKEN with repo scope
#   curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" \
#     https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.sh | bash
#
# Note: the Authorization header on curl only fetches this script. The script
# itself also needs GH_TOKEN/GITHUB_TOKEN in the environment to download assets.
#
# Options via environment variables:
#   ODS_VERSION   — pin a release tag, e.g. "v0.1.0"  (default: latest stable)
#   ODS_PREFIX    — directory to install binaries into  (default: ~/.local/bin)
#   ODS_NO_VERIFY — set to "1" to skip SHA256 checksum verification
#   GH_TOKEN / GITHUB_TOKEN — required while the repo is private
#
set -euo pipefail

REPO="StaytunedLLP/open-document-spec"
API="https://api.github.com/repos/${REPO}"

# ── Helpers ───────────────────────────────────────────────────────────────────
info()  { echo "==> $*"; }
warn()  { echo "WARN: $*" >&2; }
fatal() { echo "error: $*" >&2; exit 1; }

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fatal "required command not found: $1 — please install it and retry"
}

strip_v() {
  printf '%s' "$1" | sed -E 's/^[vV]//'
}

version_key() {
  strip_v "$1" | awk -F. '{
    major=$1+0; minor=$2+0; patch=$3+0;
    printf "%010d.%010d.%010d\n", major, minor, patch
  }'
}

installed_ods_version() {
  if command -v odc >/dev/null 2>&1; then
    odc --version 2>/dev/null | awk '{print $2}' | head -1
  elif command -v ods >/dev/null 2>&1; then
    ods --version 2>/dev/null | awk '{print $2}' | head -1
  elif [ -x "${ODS_PREFIX:-${ODC_PREFIX:-${HOME}/.local/bin}}/odc" ]; then
    "${ODS_PREFIX:-${ODC_PREFIX:-${HOME}/.local/bin}}/odc" --version 2>/dev/null | awk '{print $2}' | head -1
  elif [ -x "${ODS_PREFIX:-${HOME}/.local/bin}/ods" ]; then
    "${ODS_PREFIX:-${HOME}/.local/bin}/ods" --version 2>/dev/null | awk '{print $2}' | head -1
  else
    printf ''
  fi
}

version_ge() {
  [ "$(version_key "$1")" \> "$(version_key "$2")" ] || [ "$(version_key "$1")" = "$(version_key "$2")" ]
}

github_token() {
  if [ -n "${GH_TOKEN:-}" ]; then
    printf '%s' "${GH_TOKEN}"
  elif [ -n "${GITHUB_TOKEN:-}" ]; then
    printf '%s' "${GITHUB_TOKEN}"
  else
    printf ''
  fi
}

private_repo_hint() {
  cat >&2 <<'EOF'
This repository is private. Unauthenticated downloads return HTTP 404.

  export GH_TOKEN="$(gh auth token)"   # or a PAT with repo scope
  curl -fsSL -H "Authorization: Bearer ${GH_TOKEN}" \
    https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.sh | bash

GH_TOKEN must be exported in the environment that runs this script (not only on curl).
EOF
}

# curl wrapper. Args: extra curl flags… then URL.
# Always sets User-Agent + Accept + optional Authorization.
api_curl() {
  local -a args=(
    -fsSL --tlsv1.2
    --connect-timeout 30 --max-time 300
    --retry 3 --retry-delay 2
    -H "User-Agent: ods-install"
  )
  local token
  token="$(github_token)"
  if [ -n "${token}" ]; then
    args+=(-H "Authorization: Bearer ${token}")
  fi
  curl "${args[@]}" "$@"
}

# Download a release asset by name via the GitHub API (required for private repos).
# Browser-style /releases/download/ URLs return 404 for private assets even with Bearer.
# Usage: download_asset <tag|latest> <filename> <output-path>
download_asset() {
  local tag="$1" filename="$2" out="$3"
  local release_json asset_id

  if [ "${tag}" = "latest" ]; then
    release_json=$(api_curl -H "Accept: application/vnd.github+json" \
      "${API}/releases/latest") \
      || return 1
  else
    release_json=$(api_curl -H "Accept: application/vnd.github+json" \
      "${API}/releases/tags/${tag}") \
      || return 1
  fi

  if command -v python3 >/dev/null 2>&1; then
    asset_id=$(printf '%s' "${release_json}" | FILENAME="${filename}" python3 -c '
import json, os, sys
name = os.environ["FILENAME"]
data = json.load(sys.stdin)
for a in data.get("assets", []):
    if a.get("name") == name:
        print(a["id"])
        raise SystemExit(0)
raise SystemExit(1)
' 2>/dev/null) || asset_id=""
  else
    asset_id=""
  fi

  # Fallback without python: id usually appears before name in GitHub JSON
  if [ -z "${asset_id}" ]; then
    asset_id=$(printf '%s' "${release_json}" \
      | tr '\n' ' ' \
      | sed -n "s/.*\"id\": *\([0-9][0-9]*\)[^}]*\"name\": *\"${filename}\".*/\1/p" \
      | head -1)
  fi
  [ -n "${asset_id}" ] || {
    warn "asset '${filename}' not found on release ${tag}"
    return 1
  }

  api_curl -H "Accept: application/octet-stream" \
    -o "${out}" \
    "${API}/releases/assets/${asset_id}"
}

# ── Dependency check ──────────────────────────────────────────────────────────
need_cmd curl
need_cmd tar

TOKEN="$(github_token)"
if [ -z "${TOKEN}" ]; then
  warn "GH_TOKEN / GITHUB_TOKEN not set — downloads will fail if the repo is private"
fi

# ── Platform detection → short asset id (os-arch) ─────────────────────────────
OS=$(uname -s)
ARCH=$(uname -m)

case "${OS}-${ARCH}" in
  Linux-x86_64)              ASSET="linux-x86_64"  ;;
  Linux-aarch64|Linux-arm64) ASSET="linux-arm64"   ;;
  Darwin-arm64)              ASSET="macos-arm64"   ;;
  Darwin-x86_64)             ASSET="macos-x86_64"  ;;
  *)
    fatal "unsupported platform: ${OS}-${ARCH}
  Supported: Linux x86_64/arm64, macOS arm64/x86_64
  Windows: use src/scripts/install.ps1 (PowerShell)
  Build from source: cargo install --path src/crates/odc --bin odc --bin ods"
    ;;
esac

# ── Version resolution ────────────────────────────────────────────────────────
VERSION="${1:-${ODS_VERSION:-}}"
if [ -z "${VERSION}" ]; then
  info "Resolving latest ODS release..."
  if ! API_RESPONSE=$(api_curl -H "Accept: application/vnd.github+json" \
      "${API}/releases/latest" 2>/dev/null); then
    if [ -z "${TOKEN}" ]; then
      private_repo_hint
    fi
    fatal "could not reach GitHub API — check network and token"
  fi
  VERSION=$(printf '%s' "${API_RESPONSE}" \
    | grep '"tag_name"' | head -1 \
    | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')
  [ -n "${VERSION}" ] || fatal "could not resolve latest release tag — is the token valid for ${REPO}?"
fi
info "Installing ODS ${VERSION} for ${ASSET}"

INSTALLED_VERSION="$(installed_ods_version)"
if [ -n "${INSTALLED_VERSION}" ] && version_ge "${INSTALLED_VERSION}" "${VERSION}"; then
  info "ods ${INSTALLED_VERSION} is up to date (latest $(strip_v "${VERSION}"))"
  command -v ods >/dev/null 2>&1 && ods --version || "${ODS_PREFIX:-${HOME}/.local/bin}/ods" --version
  exit 0
fi

# ── Filenames (prefer odc-*, fall back to legacy ods-*) ───────────────────────
FILENAME="odc-${VERSION}-${ASSET}.tar.gz"
FALLBACK_FILENAME="ods-${VERSION}-${ASSET}.tar.gz"

# ── Temp workspace ────────────────────────────────────────────────────────────
TMPDIR_ODS=$(mktemp -d)
trap 'rm -rf "${TMPDIR_ODS}"' EXIT

# ── Download archive ──────────────────────────────────────────────────────────
info "Downloading ${FILENAME}..."
if ! download_asset "${VERSION}" "${FILENAME}" "${TMPDIR_ODS}/${FILENAME}"; then
  info "Trying legacy archive ${FALLBACK_FILENAME}..."
  FILENAME="${FALLBACK_FILENAME}"
  if ! download_asset "${VERSION}" "${FILENAME}" "${TMPDIR_ODS}/${FILENAME}"; then
    if [ -z "${TOKEN}" ]; then
      private_repo_hint
    fi
    fatal "download failed for odc-/ods- archive on ${VERSION}
  Check that version exists at: https://github.com/${REPO}/releases"
  fi
fi

# ── Checksum verification ─────────────────────────────────────────────────────
if [ "${ODS_NO_VERIFY:-0}" != "1" ]; then
  info "Verifying checksum..."
  if ! download_asset "${VERSION}" "SHA256SUMS" "${TMPDIR_ODS}/SHA256SUMS"; then
    if [ -z "${TOKEN}" ]; then
      private_repo_hint
    fi
    fatal "could not download SHA256SUMS for ${VERSION}"
  fi

  EXPECTED=$(grep " ${FILENAME}$" "${TMPDIR_ODS}/SHA256SUMS" | awk '{print $1}')
  [ -n "${EXPECTED}" ] || fatal "no checksum found for '${FILENAME}' in SHA256SUMS"

  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "${TMPDIR_ODS}/${FILENAME}" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "${TMPDIR_ODS}/${FILENAME}" | awk '{print $1}')
  else
    warn "no sha256sum or shasum found — skipping checksum verification"
    ACTUAL="${EXPECTED}"
  fi

  [ "${EXPECTED}" = "${ACTUAL}" ] \
    || fatal "checksum mismatch!
  Expected: ${EXPECTED}
  Got:      ${ACTUAL}
  The downloaded file may be corrupt or tampered with."
  info "Checksum OK"
fi

# ── Extract ───────────────────────────────────────────────────────────────────
info "Extracting..."
tar xzf "${TMPDIR_ODS}/${FILENAME}" -C "${TMPDIR_ODS}"
EXTRACTED=""
for try in \
  "${TMPDIR_ODS}/odc-${VERSION}-${ASSET}" \
  "${TMPDIR_ODS}/ods-${VERSION}-${ASSET}"; do
  if [ -f "${try}/odc" ] || [ -f "${try}/ods" ]; then
    EXTRACTED="${try}"
    break
  fi
done
if [ -z "${EXTRACTED}" ]; then
  FOUND_BIN=$(find "${TMPDIR_ODS}" -type f \( -name odc -o -name ods \) 2>/dev/null | head -1 || true)
  if [ -n "${FOUND_BIN}" ]; then
    EXTRACTED=$(dirname "${FOUND_BIN}")
  fi
fi
[ -n "${EXTRACTED:-}" ] || fatal "binary 'odc'/'ods' not found in archive"

# ── Install ───────────────────────────────────────────────────────────────────
PREFIX="${ODC_PREFIX:-${ODS_PREFIX:-${HOME}/.local/bin}}"
mkdir -p "${PREFIX}"
SRC=""
if [ -f "${EXTRACTED}/odc" ]; then SRC="${EXTRACTED}/odc"
elif [ -f "${EXTRACTED}/ods" ]; then SRC="${EXTRACTED}/ods"
fi
[ -n "$SRC" ] || fatal "odc/ods binary missing in archive"
install -m 755 "${SRC}" "${PREFIX}/odc"
ln -sfn "${PREFIX}/odc" "${PREFIX}/ods" 2>/dev/null || install -m 755 "${SRC}" "${PREFIX}/ods"

echo ""
info "Installed successfully:"
echo "    ${PREFIX}/odc  (primary)"
echo "    ${PREFIX}/ods  (symlink → odc)"

# ── PATH hint ─────────────────────────────────────────────────────────────────
case ":${PATH}:" in
  *":${PREFIX}:"*) ;;
  *)
    echo ""
    echo "  NOTE: '${PREFIX}' is not yet in your PATH."
    echo "  Add this to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
    echo ""
    echo "    export PATH=\"${PREFIX}:\$PATH\""
    echo ""
    echo "  Then run: source ~/.bashrc  (or open a new terminal)"
    ;;
esac

# ── Next steps ────────────────────────────────────────────────────────────────
echo ""
echo "  Verify installation:"
echo "    ${PREFIX}/ods --version"
echo ""
echo "  Get started:"
echo "    ods init .              # make project ODS-compliant (creates root index.md)"
echo "    ods setup               # set up machine background service & check workspace health"
echo "    ods lint"
echo "    ods export              # optional graph.md for AI"
echo ""
echo "  Keep tools current (auto-check ~daily; opt-out: ODS_AUTO_UPDATE=0):"
echo "    export GH_TOKEN=\"\$(gh auth token)\"   # needed for private releases"
echo "    ods update              # update binary & restart background service"
echo ""
echo "  Guide: https://github.com/${REPO}/blob/main/README.md"
echo "  Changelog: https://github.com/${REPO}/blob/main/CHANGELOG.md"
echo ""
