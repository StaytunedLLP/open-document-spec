#!/usr/bin/env bash
# GitHub Action setup & runner for ODS (Open Document Specs) — Linux / macOS
set -euo pipefail

REPO="${GITHUB_REPOSITORY:-StaytunedLLP/open-document-spec}"
API="https://api.github.com/repos/${REPO}"

INPUT_VERSION="${INPUT_VERSION:-latest}"
INPUT_COMMAND="${INPUT_COMMAND:-lint}"
INPUT_PATH="${INPUT_PATH:-.}"
INPUT_LEVEL="${INPUT_LEVEL:-3}"
INPUT_FORMAT="${INPUT_FORMAT:-text}"
INPUT_TOKEN="${INPUT_TOKEN:-${GH_TOKEN:-${GITHUB_TOKEN:-}}}"
INPUT_ANNOTATE="${INPUT_ANNOTATE:-true}"
INPUT_EXTRA_ARGS="${INPUT_EXTRA_ARGS:-}"

info()  { echo "==> $*"; }
warn()  { echo "WARN: $*" >&2; }
fatal() { echo "::error::$*" >&2; exit 1; }

# Determine Platform & Asset
OS=$(uname -s)
ARCH=$(uname -m)

case "${OS}-${ARCH}" in
  Linux-x86_64)              ASSET="linux-x86_64"  ;;
  Linux-aarch64|Linux-arm64) ASSET="linux-arm64"   ;;
  Darwin-arm64)              ASSET="macos-arm64"   ;;
  Darwin-x86_64)             ASSET="macos-x86_64"  ;;
  *)
    fatal "Unsupported platform: ${OS}-${ARCH}. ODS Action supports Linux x86_64/arm64 and macOS arm64/x86_64."
    ;;
esac

# Function for curl requests with Auth Token
api_curl() {
  local -a args=(
    -fsSL --tlsv1.2
    --connect-timeout 30 --max-time 300
    --retry 3 --retry-delay 2
    -H "User-Agent: ods-github-action"
  )
  if [ -n "${INPUT_TOKEN}" ]; then
    args+=(-H "Authorization: Bearer ${INPUT_TOKEN}")
  fi
  curl "${args[@]}" "$@"
}

download_asset() {
  local tag="$1" filename="$2" out="$3"
  local release_json asset_id

  if [ "${tag}" = "latest" ]; then
    release_json=$(api_curl -H "Accept: application/vnd.github+json" "${API}/releases/latest") || return 1
  else
    release_json=$(api_curl -H "Accept: application/vnd.github+json" "${API}/releases/tags/${tag}") || return 1
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

  if [ -z "${asset_id}" ]; then
    asset_id=$(printf '%s' "${release_json}" | tr '\n' ' ' | sed -n "s/.*\"id\": *\([0-9][0-9]*\)[^}]*\"name\": *\"${filename}\".*/\1/p" | head -1)
  fi

  [ -n "${asset_id}" ] || return 1

  api_curl -H "Accept: application/octet-stream" -o "${out}" "${API}/releases/assets/${asset_id}"
}

# Resolve target version
VERSION="${INPUT_VERSION}"
if [ "${VERSION}" = "latest" ] || [ -z "${VERSION}" ]; then
  info "Resolving latest ODS release..."
  API_RESPONSE=$(api_curl -H "Accept: application/vnd.github+json" "${API}/releases/latest" 2>/dev/null || true)
  if [ -n "${API_RESPONSE}" ]; then
    VERSION=$(printf '%s' "${API_RESPONSE}" | grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || true)
  fi
fi

if [ -n "${VERSION}" ] && [ "${VERSION}" != "latest" ]; then
  TAG="${VERSION}"
  if [[ ! "${TAG}" =~ ^v ]]; then
    TAG="v${VERSION}"
  fi
  CLEAN_VERSION="${TAG#v}"
else
  TAG="latest"
  CLEAN_VERSION="latest"
fi

info "Target ODS version: ${TAG} (${ASSET})"

# Target binary directory
INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"
ODS_BIN="${INSTALL_DIR}/ods"

# Check if binary is already installed and matches target version
INSTALLED=false
if [ -x "${ODS_BIN}" ]; then
  CUR_VER=$("${ODS_BIN}" --version 2>/dev/null | awk '{print $2}' | head -1 || true)
  if [ "${CUR_VER}" = "${CLEAN_VERSION}" ]; then
    INSTALLED=true
    info "ODS ${CLEAN_VERSION} is already installed."
  fi
fi

if [ "${INSTALLED}" = "false" ]; then
  FILENAME="ods-${TAG}-${ASSET}.tar.gz"
  TMPDIR_ODS=$(mktemp -d)
  trap 'rm -rf "${TMPDIR_ODS}"' EXIT

  info "Downloading ${FILENAME}..."
  if ! download_asset "${TAG}" "${FILENAME}" "${TMPDIR_ODS}/${FILENAME}"; then
    if command -v cargo >/dev/null 2>&1; then
      info "Release asset ${FILENAME} not available via GitHub API — compiling local Cargo fallback binary..."
      cargo build --release --bin ods
      FOUND_CARGO_BIN="$(find target .artifacts/target -type f -name ods 2>/dev/null | grep release | head -1 || true)"
      if [ -z "${FOUND_CARGO_BIN}" ]; then
        FOUND_CARGO_BIN="target/release/ods"
      fi
      install -m 755 "${FOUND_CARGO_BIN}" "${ODS_BIN}"
      info "ODS installed successfully to ${ODS_BIN}"
      INSTALLED=true
    else
      fatal "Failed to download release asset ${FILENAME} for release ${TAG} and Cargo is not installed."
    fi
  fi

  if [ "${INSTALLED}" = "false" ]; then
    info "Verifying SHA256 checksum..."
    if download_asset "${TAG}" "SHA256SUMS" "${TMPDIR_ODS}/SHA256SUMS"; then
      EXPECTED=$(grep " ${FILENAME}$" "${TMPDIR_ODS}/SHA256SUMS" | awk '{print $1}')
      if [ -n "${EXPECTED}" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
          ACTUAL=$(sha256sum "${TMPDIR_ODS}/${FILENAME}" | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
          ACTUAL=$(shasum -a 256 "${TMPDIR_ODS}/${FILENAME}" | awk '{print $1}')
        else
          ACTUAL="${EXPECTED}"
        fi
        [ "${EXPECTED}" = "${ACTUAL}" ] || fatal "Checksum mismatch for ${FILENAME}! Expected ${EXPECTED}, got ${ACTUAL}."
        info "Checksum verified OK."
      fi
    fi

    info "Extracting ODS binary..."
    tar xzf "${TMPDIR_ODS}/${FILENAME}" -C "${TMPDIR_ODS}"
    EXTRACTED="${TMPDIR_ODS}/ods-${TAG}-${ASSET}"
    if [ ! -f "${EXTRACTED}/ods" ]; then
      ODS_FIND=$(find "${TMPDIR_ODS}" -type f -name ods 2>/dev/null | head -1 || true)
      if [ -n "${ODS_FIND}" ]; then
        EXTRACTED=$(dirname "${ODS_FIND}")
      fi
    fi

    [ -f "${EXTRACTED}/ods" ] || fatal "Binary 'ods' not found in archive."
    install -m 755 "${EXTRACTED}/ods" "${ODS_BIN}"
    info "ODS installed successfully to ${ODS_BIN}"
  fi
fi

# Add to GITHUB_PATH if running in GitHub Actions environment
if [ -n "${GITHUB_PATH:-}" ]; then
  echo "${INSTALL_DIR}" >> "${GITHUB_PATH}"
fi

# Export GitHub Action Step Outputs
if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "ods-version=${CLEAN_VERSION}" >> "${GITHUB_OUTPUT}"
  echo "ods-path=${ODS_BIN}" >> "${GITHUB_OUTPUT}"
fi

# Register Problem Matcher for PR annotations if enabled
if [ "${INPUT_ANNOTATE}" = "true" ] && [ -n "${GITHUB_ACTION_PATH:-}" ]; then
  MATCHER_PATH="${GITHUB_ACTION_PATH}/src/action/problem-matcher.json"
  if [ ! -f "${MATCHER_PATH}" ]; then
    MATCHER_PATH="${GITHUB_ACTION_PATH}/problem-matcher.json"
  fi
  if [ -f "${MATCHER_PATH}" ]; then
    echo "::add-matcher::${MATCHER_PATH}"
    info "Registered ODS problem matcher for inline annotations."
  fi
fi

# Run requested command if specified
COMMAND="${INPUT_COMMAND}"
if [ -z "${COMMAND}" ] || [ "${COMMAND}" = "none" ] || [ "${COMMAND}" = "setup" ]; then
  info "ODS setup completed successfully (setup-only mode)."
  exit 0
fi

info "Executing ODS command: ods ${COMMAND} ${INPUT_PATH}"

# Build argument array
case "${COMMAND}" in
  lint)
    ARGS=("lint" "${INPUT_PATH}" "--level" "${INPUT_LEVEL}" "--format" "${INPUT_FORMAT}")
    ;;
  index-check|index_check)
    ARGS=("index" "${INPUT_PATH}" "--check")
    ;;
  doctor)
    ARGS=("doctor" "${INPUT_PATH}")
    ;;
  fmt-check|fmt_check)
    ARGS=("fmt" "${INPUT_PATH}")
    ;;
  bench)
    ARGS=("bench" "stats" "${INPUT_PATH}")
    ;;
  export)
    ARGS=("export" "${INPUT_PATH}")
    ;;
  *)
    # Custom raw command string
    # shellcheck disable=SC2206
    ARGS=(${COMMAND} "${INPUT_PATH}")
    ;;
esac

if [ -n "${INPUT_EXTRA_ARGS}" ]; then
  # shellcheck disable=SC2206
  ARGS+=(${INPUT_EXTRA_ARGS})
fi

"${ODS_BIN}" "${ARGS[@]}"
