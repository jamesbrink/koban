#!/bin/sh
# Install koban - Invoice Ninja from the terminal
# https://github.com/jamesbrink/koban
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/jamesbrink/koban/main/install.sh | sh
#
# Options:
#   KOBAN_INSTALL_DIR  install directory (default: ~/.local/bin)
#   KOBAN_VERSION      release tag, such as v0.1.0 (default: latest)

set -e

REPO="jamesbrink/koban"
VERSION="${KOBAN_VERSION:-latest}"
INSTALL_DIR="${KOBAN_INSTALL_DIR:-$HOME/.local/bin}"
BIN="koban"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
    Linux)
        case "${ARCH}" in
            x86_64)        ASSET="koban-x86_64-unknown-linux-gnu.tar.gz" ;;
            aarch64|arm64) ASSET="koban-aarch64-unknown-linux-gnu.tar.gz" ;;
            *)
                echo "Error: unsupported Linux architecture: ${ARCH}" >&2
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "${ARCH}" in
            arm64|aarch64) ASSET="koban-aarch64-apple-darwin.tar.gz" ;;
            x86_64)        ASSET="koban-x86_64-apple-darwin.tar.gz" ;;
            *)
                echo "Error: unsupported macOS architecture: ${ARCH}" >&2
                exit 1
                ;;
        esac
        ;;
    *)
        echo "Error: unsupported OS: ${OS}" >&2
        echo "  koban ships prebuilt binaries for Linux and macOS." >&2
        echo "  For other platforms, install via cargo:" >&2
        echo "    cargo install koban-cli" >&2
        exit 1
        ;;
esac

if [ "${VERSION}" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
    SUMS_URL="https://github.com/${REPO}/releases/latest/download/SHA256SUMS"
else
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
    SUMS_URL="https://github.com/${REPO}/releases/download/${VERSION}/SHA256SUMS"
fi

if ! command -v curl >/dev/null 2>&1; then
    echo "Error: curl is required but was not found in PATH." >&2
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    echo "Error: tar is required but was not found in PATH." >&2
    exit 1
fi

mkdir -p "${INSTALL_DIR}"

TMPDIR="$(mktemp -d)"
trap 'rm -rf "${TMPDIR}"' EXIT

echo "Installing koban (${VERSION}) for ${OS}/${ARCH}..."
echo "  from: ${URL}"
echo "  to:   ${INSTALL_DIR}/${BIN}"

if ! curl -fsSL "${URL}" -o "${TMPDIR}/${ASSET}"; then
    echo "Error: failed to download ${URL}" >&2
    echo "  Available releases: https://github.com/${REPO}/releases" >&2
    exit 1
fi

if curl -fsSL "${SUMS_URL}" -o "${TMPDIR}/SHA256SUMS" 2>/dev/null; then
    EXPECTED="$(grep " ${ASSET}\$" "${TMPDIR}/SHA256SUMS" | awk '{print $1}')"
    if [ -n "${EXPECTED}" ]; then
        if command -v sha256sum >/dev/null 2>&1; then
            ACTUAL="$(sha256sum "${TMPDIR}/${ASSET}" | awk '{print $1}')"
        elif command -v shasum >/dev/null 2>&1; then
            ACTUAL="$(shasum -a 256 "${TMPDIR}/${ASSET}" | awk '{print $1}')"
        else
            ACTUAL=""
        fi

        if [ -z "${ACTUAL}" ]; then
            echo "Warning: could not verify checksum because sha256sum/shasum was not found." >&2
        elif [ "${ACTUAL}" != "${EXPECTED}" ]; then
            echo "Error: checksum mismatch for ${ASSET}" >&2
            echo "  expected: ${EXPECTED}" >&2
            echo "  got:      ${ACTUAL}" >&2
            exit 1
        else
            echo "Checksum verified (SHA-256)."
        fi
    fi
else
    echo "Warning: SHA256SUMS was not available; skipping checksum verification." >&2
fi

tar -xzf "${TMPDIR}/${ASSET}" -C "${TMPDIR}"
install -m 755 "${TMPDIR}/${BIN}" "${INSTALL_DIR}/${BIN}"

if [ "${OS}" = "Darwin" ]; then
    xattr -d com.apple.quarantine "${INSTALL_DIR}/${BIN}" 2>/dev/null || true
fi

echo ""
echo "koban installed to ${INSTALL_DIR}/${BIN}"

case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo ""
        echo "Add ${INSTALL_DIR} to your PATH:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
        echo ""
        echo "Or add it to your shell profile."
        ;;
esac

"${INSTALL_DIR}/${BIN}" --version
