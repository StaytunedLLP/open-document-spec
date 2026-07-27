#!/usr/bin/env bash
# Open Document Spec (ODS) — Universal OS-Agnostic Installer
# Supported OS: macOS (Intel/Apple Silicon), Linux (x86_64/aarch64), BSD, Windows (WSL/Git Bash/MSYS2)
# Site: https://opendocify.com / https://prod-ods-260726.web.app

set -e

BOLD="\031[1m"
GREEN="\033[32m"
CYAN="\033[36m"
YELLOW="\033[33m"
RESET="\033[0m"

echo -e "${CYAN}--------------------------------------------------------${RESET}"
echo -e "${BOLD}  Installing Open Document Spec CLI (ods)...${RESET}"
echo -e "${CYAN}--------------------------------------------------------${RESET}"

# 1. Normalize Operating System
RAW_OS="$(uname -s)"
case "${RAW_OS}" in
    Darwin*)            OS="darwin" ;;
    Linux*)             OS="linux" ;;
    FreeBSD*|OpenBSD*)  OS="freebsd" ;;
    MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
    *)                  OS="unknown" ;;
esac

# 2. Normalize System Architecture
RAW_ARCH="$(uname -m)"
case "${RAW_ARCH}" in
    x86_64|amd64)       ARCH="x86_64" ;;
    arm64|aarch64)      ARCH="aarch64" ;;
    armv7l|armv6l)      ARCH="armhf" ;;
    i386|i686)          ARCH="x86" ;;
    *)                  ARCH="${RAW_ARCH}" ;;
esac

echo -e "Detected OS: ${GREEN}${OS}${RESET} | Arch: ${GREEN}${ARCH}${RESET}"

# 3. Determine Non-Privileged Destination Directory ($HOME/.ods/bin)
TARGET_DIR="${ODS_INSTALL_DIR:-$HOME/.ods/bin}"
mkdir -p "${TARGET_DIR}"

BINARY_NAME="ods"
if [ "${OS}" = "windows" ]; then
    BINARY_NAME="ods.exe"
fi

INSTALLED=0

# Method A: Try Pre-compiled Binary Release from GitHub Releases
VERSION="${ODS_VERSION:-latest}"
if [ "${VERSION}" = "latest" ]; then
    DOWNLOAD_URL="https://github.com/StaytunedLLP/open-document-specs/releases/latest/download/ods-${OS}-${ARCH}.tar.gz"
else
    DOWNLOAD_URL="https://github.com/StaytunedLLP/open-document-specs/releases/download/${VERSION}/ods-${OS}-${ARCH}.tar.gz"
fi

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t 'ods')"

echo -e "Fetching ${VERSION} binary release for ${OS}-${ARCH}..."
if curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/ods.tar.gz" 2>/dev/null; then
    tar -xzf "${TMP_DIR}/ods.tar.gz" -C "${TMP_DIR}"
    if [ -f "${TMP_DIR}/${BINARY_NAME}" ]; then
        mv "${TMP_DIR}/${BINARY_NAME}" "${TARGET_DIR}/${BINARY_NAME}"
        chmod +x "${TARGET_DIR}/${BINARY_NAME}"
        INSTALLED=1
    fi
fi
rm -rf "${TMP_DIR}"

# Method B: Fallback to Rust Cargo Installation if Binary Release is unavailable
if [ "${INSTALLED}" -eq 0 ] && command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}Pre-compiled binary release not available yet. Building via Cargo...${RESET}"
    if cargo install --git https://github.com/StaytunedLLP/open-document-specs ods || cargo install ods; then
        INSTALLED=1
        TARGET_DIR="$HOME/.cargo/bin"
    fi
fi

# Method C: Final Fallback Guidance
if [ "${INSTALLED}" -eq 0 ]; then
    echo -e "${YELLOW}Could not download pre-compiled binary for ${OS}-${ARCH}.${RESET}"
    echo "To install ODS on your system:"
    echo "1. Install Rust toolchain from https://rustup.rs"
    echo "2. Run: cargo install ods-cli"
    exit 1
fi

# 4. Automatic Shell PATH Environment Configuration
SHELL_PROFILE=""
if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
    SHELL_PROFILE="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
    SHELL_PROFILE="$HOME/.bashrc"
elif [ -f "$HOME/.profile" ]; then
    SHELL_PROFILE="$HOME/.profile"
fi

if [[ ":$PATH:" != *":${TARGET_DIR}:"* ]]; then
    export PATH="${TARGET_DIR}:$PATH"
    if [ -n "${SHELL_PROFILE}" ] && [ -w "${SHELL_PROFILE}" ]; then
        if ! grep -q "ods/bin" "${SHELL_PROFILE}" 2>/dev/null; then
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
