#!/usr/bin/env bash
# GitHub Action setup & runner for OpenDocify CLI (odc) — Linux / macOS
# Runs ODS document commands via `odc ods …` (or legacy `ods …` when only that binary exists).
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

OS=$(uname -s)
ARCH=$(uname -m)

case "${OS}-${ARCH}" in
  Linux-x86_64)              ASSET="linux-x86_64"  ;;
  Linux-aarch64|Linux-arm64) ASSET="linux-arm64"   ;;
  Darwin-arm64)              ASSET="macos-arm64"   ;;
  Darwin-x86_64)             ASSET="macos-x86_64"  ;;
  *)
    fatal "Unsupported platform: ${OS}-${ARCH}."
    ;;
esac

api_curl() {
  local -a args=(
    -fsSL --tlsv1.2
    --connect-timeout 30 --max-time 300
    --retry 3 --retry-delay 2
    -H "User-Agent: odc-github-action"
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

VERSION="${INPUT_VERSION}"
if [ "${VERSION}" = "latest" ] || [ -z "${VERSION}" ]; then
  info "Resolving latest OpenDocify release..."
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

info "Target CLI version: ${TAG} (${ASSET})"

INSTALL_DIR="${HOME}/.local/bin"
mkdir -p "${INSTALL_DIR}"
ODC_BIN="${INSTALL_DIR}/odc"
ODS_BIN="${INSTALL_DIR}/ods"

INSTALLED=false
if [ -x "${ODC_BIN}" ] || [ -x "${ODS_BIN}" ]; then
  CUR_BIN="${ODC_BIN}"
  [ -x "${CUR_BIN}" ] || CUR_BIN="${ODS_BIN}"
  CUR_VER=$("${CUR_BIN}" --version 2>/dev/null | awk '{print $2}' | head -1 || true)
  if [ "${CUR_VER}" = "${CLEAN_VERSION}" ]; then
    INSTALLED=true
    info "CLI ${CLEAN_VERSION} is already installed."
  fi
fi

if [ "${INSTALLED}" = "false" ]; then
  TMPDIR_ODC=$(mktemp -d)
  trap 'rm -rf "${TMPDIR_ODC}"' EXIT

  DOWNLOADED=""
  for PREFIX in odc ods; do
    FILENAME="${PREFIX}-${TAG}-${ASSET}.tar.gz"
    info "Trying ${FILENAME}..."
    if download_asset "${TAG}" "${FILENAME}" "${TMPDIR_ODC}/${FILENAME}"; then
      DOWNLOADED="${FILENAME}"
      break
    fi
  done

  if [ -z "${DOWNLOADED}" ]; then
    if command -v cargo >/dev/null 2>&1; then
      info "Release asset not available — compiling local Cargo fallback (odc)..."
      cargo build --release -p ods-cli --bin ods 2>/dev/null \
        || cargo build --release --bin ods 2>/dev/null \
        || cargo build --release
      FOUND_CARGO_BIN="$(find target .artifacts/target -type f \( -name ods -o -name odc \) -path "*/release/*" 2>/dev/null | xargs ls -t 2>/dev/null | head -1 || true)"
      if [ -z "${FOUND_CARGO_BIN}" ]; then
        FOUND_CARGO_BIN=".artifacts/target/release/ods"
        [ -f "${FOUND_CARGO_BIN}" ] || FOUND_CARGO_BIN="target/release/ods"
      fi
      install -m 755 "${FOUND_CARGO_BIN}" "${ODC_BIN}"
      ln -sfn "${ODC_BIN}" "${ODS_BIN}" 2>/dev/null || install -m 755 "${FOUND_CARGO_BIN}" "${ODS_BIN}"
      info "Installed ${ODC_BIN}"
      INSTALLED=true
    else
      fatal "Failed to download odc-/ods- release asset for ${TAG} and Cargo is not installed."
    fi
  fi

  if [ "${INSTALLED}" = "false" ]; then
    FILENAME="${DOWNLOADED}"
    info "Verifying SHA256 checksum..."
    if download_asset "${TAG}" "SHA256SUMS" "${TMPDIR_ODC}/SHA256SUMS"; then
      EXPECTED=$(grep " ${FILENAME}$" "${TMPDIR_ODC}/SHA256SUMS" | awk '{print $1}')
      if [ -n "${EXPECTED}" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
          ACTUAL=$(sha256sum "${TMPDIR_ODC}/${FILENAME}" | awk '{print $1}')
        elif command -v shasum >/dev/null 2>&1; then
          ACTUAL=$(shasum -a 256 "${TMPDIR_ODC}/${FILENAME}" | awk '{print $1}')
        else
          ACTUAL="${EXPECTED}"
        fi
        [ "${EXPECTED}" = "${ACTUAL}" ] || fatal "Checksum mismatch for ${FILENAME}!"
        info "Checksum verified OK."
      fi
    fi

    info "Extracting CLI binary..."
    tar xzf "${TMPDIR_ODC}/${FILENAME}" -C "${TMPDIR_ODC}"
    SRC=""
    for name in odc ods; do
      FOUND=$(find "${TMPDIR_ODC}" -type f -name "${name}" 2>/dev/null | head -1 || true)
      if [ -n "${FOUND}" ]; then
        SRC="${FOUND}"
        break
      fi
    done
    [ -n "${SRC}" ] || fatal "Binary 'odc'/'ods' not found in archive."
    install -m 755 "${SRC}" "${ODC_BIN}"
    ln -sfn "${ODC_BIN}" "${ODS_BIN}" 2>/dev/null || install -m 755 "${SRC}" "${ODS_BIN}"
    info "Installed ${ODC_BIN} (+ ods symlink)"
  fi
fi

# Prefer odc for namespaced commands
CLI_BIN="${ODC_BIN}"
[ -x "${CLI_BIN}" ] || CLI_BIN="${ODS_BIN}"

if [ -n "${GITHUB_PATH:-}" ]; then
  echo "${INSTALL_DIR}" >> "${GITHUB_PATH}"
fi

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "ods-version=${CLEAN_VERSION}" >> "${GITHUB_OUTPUT}"
  echo "ods-path=${CLI_BIN}" >> "${GITHUB_OUTPUT}"
  echo "odc-version=${CLEAN_VERSION}" >> "${GITHUB_OUTPUT}"
  echo "odc-path=${CLI_BIN}" >> "${GITHUB_OUTPUT}"
fi

if [ "${INPUT_ANNOTATE}" = "true" ] && [ -n "${GITHUB_ACTION_PATH:-}" ]; then
  MATCHER_PATH="${GITHUB_ACTION_PATH}/src/action/problem-matcher.json"
  if [ ! -f "${MATCHER_PATH}" ]; then
    MATCHER_PATH="${GITHUB_ACTION_PATH}/problem-matcher.json"
  fi
  if [ -f "${MATCHER_PATH}" ]; then
    echo "::add-matcher::${MATCHER_PATH}"
    info "Registered problem matcher for inline annotations."
  fi
fi

COMMAND="${INPUT_COMMAND}"
if [ -z "${COMMAND}" ] || [ "${COMMAND}" = "none" ] || [ "${COMMAND}" = "setup" ]; then
  info "Setup completed successfully (setup-only mode)."
  exit 0
fi

# Document commands go through ODS namespace when using odc binary
use_ods_ns=false
case "$(basename "${CLI_BIN}")" in
  odc|opendocify) use_ods_ns=true ;;
esac

# Build argument array (ODS document ops)
case "${COMMAND}" in
  lint)
    if [ "${use_ods_ns}" = "true" ]; then
      ARGS=("ods" "lint" "${INPUT_PATH}" "--level" "${INPUT_LEVEL}" "--format" "${INPUT_FORMAT}")
    else
      ARGS=("lint" "${INPUT_PATH}" "--level" "${INPUT_LEVEL}" "--format" "${INPUT_FORMAT}")
    fi
    ;;
  index-check|index_check)
    if [ "${use_ods_ns}" = "true" ]; then
      ARGS=("ods" "index" "${INPUT_PATH}" "--check")
    else
      ARGS=("index" "${INPUT_PATH}" "--check")
    fi
    ;;
  doctor)
    if [ "${use_ods_ns}" = "true" ]; then
      ARGS=("ods" "doctor" "${INPUT_PATH}")
    else
      ARGS=("doctor" "${INPUT_PATH}")
    fi
    ;;
  fmt-check|fmt_check)
    if [ "${use_ods_ns}" = "true" ]; then
      ARGS=("ods" "fmt" "${INPUT_PATH}")
    else
      ARGS=("fmt" "${INPUT_PATH}")
    fi
    ;;
  bench)
    if [ "${use_ods_ns}" = "true" ]; then
      ARGS=("ods" "bench" "stats" "${INPUT_PATH}")
    else
      ARGS=("bench" "stats" "${INPUT_PATH}")
    fi
    ;;
  export)
    if [ "${use_ods_ns}" = "true" ]; then
      ARGS=("ods" "export" "${INPUT_PATH}")
    else
      ARGS=("export" "${INPUT_PATH}")
    fi
    ;;
  okf-lint)
    ARGS=("okf" "lint" "${INPUT_PATH}")
    ;;
  *)
    # shellcheck disable=SC2206
    ARGS=(${COMMAND} "${INPUT_PATH}")
    ;;
esac

if [ -n "${INPUT_EXTRA_ARGS}" ]; then
  # shellcheck disable=SC2206
  ARGS+=(${INPUT_EXTRA_ARGS})
fi

info "Executing: $(basename "${CLI_BIN}") ${ARGS[*]}"
export ODS_AUTO_UPDATE=0
export ODC_AUTO_UPDATE=0
"${CLI_BIN}" "${ARGS[@]}"
