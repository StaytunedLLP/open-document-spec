#!/usr/bin/env bash
# Open Document Spec (ODS) — Universal OS-Agnostic Installer
# Supported OS: macOS (Intel/Apple Silicon), Linux (x86_64/aarch64), BSD, Windows (WSL/Git Bash/MSYS2)
# Site: https://opendocify.com / https://prod-ods-260726.web.app

set -euo pipefail

BOLD="\033[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${CYAN}--------------------------------------------------------${RESET}"
echo -e "${BOLD}  Installing Open Document Spec CLI (ods)...${RESET}"
echo -e "${CYAN}--------------------------------------------------------${RESET}"

REPO="StaytunedLLP/open-document-spec"
API="https://api.github.com/repos/${REPO}"

# 1. Normalize Operating System
RAW_OS="$(uname -s)"
case "${RAW_OS}" in
    Darwin*)            OS="macos" ;;
    Linux*)             OS="linux" ;;
    MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
    *)                  OS="linux" ;;
esac

# 2. Normalize System Architecture
RAW_ARCH="$(uname -m)"
case "${RAW_ARCH}" in
    x86_64|amd64)       ARCH="x86_64" ;;
    arm64|aarch64)      ARCH="arm64" ;;
    *)                  ARCH="${RAW_ARCH}" ;;
esac

ASSET="${OS}-${ARCH}"
echo -e "Detected OS: ${GREEN}${OS}${RESET} | Arch: ${GREEN}${ARCH}${RESET}"

# 3. Determine Destination Directory ($HOME/.local/bin)
TARGET_DIR="${ODS_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "${TARGET_DIR}"

BINARY_NAME="ods"
ARCHIVE_EXT="tar.gz"
if [ "${OS}" = "windows" ]; then
    BINARY_NAME="ods.exe"
    ARCHIVE_EXT="zip"
fi

INSTALLED=0

# Determine Version & Release Asset
VERSION="${ODS_VERSION:-}"
if [ -z "${VERSION}" ] || [ "${VERSION}" = "latest" ]; then
    RELEASE_JSON=$(curl -sSfL -H "User-Agent: ods-installer" "${API}/releases/latest" 2>/dev/null || true)
    if [ -n "${RELEASE_JSON}" ]; then
        VERSION=$(echo "${RELEASE_JSON}" | grep '"tag_name"' | head -1 | sed -E 's/.*"tag_name": *"(v?[^"]+)".*/\1/' || true)
    fi
fi

if [ -z "${VERSION}" ]; then
    VERSION="v0.0.1"
fi

if [[ ! "${VERSION}" =~ ^v ]]; then
    TAG="v${VERSION}"
else
    TAG="${VERSION}"
fi

FILENAME="ods-${TAG}-${ASSET}.${ARCHIVE_EXT}"
TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'ods')"
trap 'rm -rf "${TMP_DIR}"' EXIT

echo -e "Fetching ${TAG} binary release for ${ASSET}..."

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${FILENAME}"
TOKEN="${GH_TOKEN:-${GITHUB_TOKEN:-}}"

DOWNLOADED=0
if [ -n "${TOKEN}" ]; then
    ASSET_ID=$(curl -sSfL -H "Authorization: Bearer ${TOKEN}" -H "Accept: application/vnd.github+json" "${API}/releases/tags/${TAG}" 2>/dev/null | grep -B 2 "\"name\": \"${FILENAME}\"" | grep '"id"' | head -1 | sed -E 's/.*"id": ([0-9]+).*/\1/' || true)
    if [ -n "${ASSET_ID}" ]; then
        if curl -sSfL -H "Authorization: Bearer ${TOKEN}" -H "Accept: application/octet-stream" "${API}/releases/assets/${ASSET_ID}" -o "${TMP_DIR}/${FILENAME}" 2>/dev/null; then
            DOWNLOADED=1
        fi
    fi
fi

if [ "${DOWNLOADED}" -eq 0 ]; then
    if curl -sSfL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${FILENAME}" 2>/dev/null; then
        DOWNLOADED=1
    fi
fi

if [ "${DOWNLOADED}" -eq 1 ]; then
    if [ "${ARCHIVE_EXT}" = "zip" ]; then
        unzip -q "${TMP_DIR}/${FILENAME}" -d "${TMP_DIR}" 2>/dev/null || true
    else
        tar -xzf "${TMP_DIR}/${FILENAME}" -C "${TMP_DIR}" 2>/dev/null || true
    fi

    FOUND_BIN=$(find "${TMP_DIR}" -type f -name "${BINARY_NAME}" 2>/dev/null | head -1 || true)
    if [ -n "${FOUND_BIN}" ]; then
        cp "${FOUND_BIN}" "${TARGET_DIR}/${BINARY_NAME}"
        chmod +x "${TARGET_DIR}/${BINARY_NAME}"
        INSTALLED=1
    fi
fi

# Fallback to Rust Cargo Installation if Binary Release is unavailable
if [ "${INSTALLED}" -eq 0 ]; then
    echo -e "${YELLOW}Could not download pre-compiled binary for ${ASSET}.${RESET}"
    if command -v cargo >/dev/null 2>&1; then
        echo -e "${YELLOW}Rust/Cargo is available. Falling back to building via Cargo...${RESET}"
        if cargo install --git "https://github.com/${REPO}" ods || cargo install ods-cli; then
            INSTALLED=1
            TARGET_DIR="$HOME/.cargo/bin"
        fi
    fi
fi

if [ "${INSTALLED}" -eq 0 ]; then
    echo "To install ODS on your system:"
    echo "1. Install Rust toolchain from https://rustup.rs"
    echo "2. Run: cargo install ods-cli"
    exit 1
fi

# 4. Automatic Shell PATH Environment Configuration
SHELL_PROFILE=""
if [ -n "${ZSH_VERSION:-}" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -n "${BASH_VERSION:-}" ] || [ -f "$HOME/.bashrc" ]; then
    SHELL_PROFILE="$HOME/.bashrc"
elif [ -f "$HOME/.profile" ]; then
    SHELL_PROFILE="$HOME/.profile"
fi

if [[ ":$PATH:" != *":${TARGET_DIR}:"* ]]; then
    export PATH="${TARGET_DIR}:$PATH"
    if [ -n "${SHELL_PROFILE}" ] && [ -w "${SHELL_PROFILE}" ]; then
        if ! grep -q "${TARGET_DIR}" "${SHELL_PROFILE}" 2>/dev/null; then
            echo -e "\n# Open Document Spec CLI" >> "${SHELL_PROFILE}"
            echo "export PATH=\"${TARGET_DIR}:\$PATH\"" >> "${SHELL_PROFILE}"
            echo -e "${CYAN}Added ${TARGET_DIR} to your PATH in ${SHELL_PROFILE}${RESET}"
        fi
    fi
fi

echo -e ""
echo -e "${GREEN}${BOLD}✓ Open Document Spec CLI (ods) installed successfully!${RESET}"
echo -e "Binary Location: ${CYAN}${TARGET_DIR}/${BINARY_NAME}${RESET}"
echo -e "Get Started: Run '${BOLD}ods --help${RESET}' or '${BOLD}ods setup .${RESET}'"
