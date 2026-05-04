#!/bin/sh
# Install script for wado - WildFly container management CLI
# Usage: curl -fsSL https://raw.githubusercontent.com/hpehl/wado/main/install.sh | sh

set -eu

REPO="hpehl/wado"
BINARY="wado"
INSTALL_DIR="${WADO_INSTALL_DIR:-$HOME/.local/bin}"

main() {
    os=$(detect_os)
    arch=$(detect_arch)
    target=$(map_target "$os" "$arch")

    version=$(fetch_latest_version)
    echo "Installing ${BINARY} ${version} (${target})..."

    tmpdir=$(mktemp -d)
    trap 'rm -rf "$tmpdir"' EXIT

    url="https://github.com/${REPO}/releases/download/${version}/${BINARY}-${target}.tar.gz"
    echo "Downloading ${url}..."
    download "$url" "${tmpdir}/archive.tar.gz"

    tar -xzf "${tmpdir}/archive.tar.gz" -C "$tmpdir"

    mkdir -p "$INSTALL_DIR"
    mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"

    echo ""
    echo "${BINARY} ${version} installed to ${INSTALL_DIR}/${BINARY}"

    if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
        echo ""
        echo "Add ${INSTALL_DIR} to your PATH:"
        echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi
}

detect_os() {
    case "$(uname -s)" in
        Linux*)  echo "linux" ;;
        Darwin*) echo "darwin" ;;
        *)
            echo "Error: Unsupported operating system '$(uname -s)'." >&2
            echo "For Windows, download the .zip from https://github.com/${REPO}/releases" >&2
            exit 1
            ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)
            echo "Error: Unsupported architecture '$(uname -m)'." >&2
            exit 1
            ;;
    esac
}

map_target() {
    os=$1
    arch=$2
    case "${os}-${arch}" in
        linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
        darwin-x86_64)  echo "x86_64-apple-darwin" ;;
        darwin-aarch64) echo "aarch64-apple-darwin" ;;
        *)
            echo "Error: No prebuilt binary for ${os}/${arch}." >&2
            echo "You can install from source: cargo install wado" >&2
            exit 1
            ;;
    esac
}

fetch_latest_version() {
    url="https://api.github.com/repos/${REPO}/releases/latest"
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$url" | grep '"tag_name"' | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
    else
        echo "Error: curl or wget is required." >&2
        exit 1
    fi
}

download() {
    url=$1
    output=$2
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$url" -O "$output"
    fi
}

main
