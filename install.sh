#!/usr/bin/env bash
set -euo pipefail

REPO="youngyunjeong/mcps"
SERVER="${1:-fast-file-editor}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "==> Installing ${SERVER} from ${REPO}..."

OS="$(uname -s)"
case "$OS" in
  Darwin) OS_TAG="apple-darwin" ;;
  Linux)  OS_TAG="unknown-linux-gnu" ;;
  *)
    echo "Error: Unsupported operating system $OS. Please build from source using cargo." >&2
    exit 1
    ;;
esac

ARCH="$(uname -m)"
case "$ARCH" in
  arm64|aarch64) ARCH_TAG="aarch64" ;;
  x86_64|amd64)  ARCH_TAG="x86_64" ;;
  *)
    echo "Error: Unsupported architecture $ARCH. Please build from source using cargo." >&2
    exit 1
    ;;
esac

TARGET="${ARCH_TAG}-${OS_TAG}"
ARCHIVE_NAME="${SERVER}-v0.1.0-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v0.1.0/${ARCHIVE_NAME}"

echo "--> Detected platform: ${TARGET}"
echo "--> Downloading ${DOWNLOAD_URL}..."

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/archive.tar.gz"

mkdir -p "$INSTALL_DIR"
tar -xzf "$TMP_DIR/archive.tar.gz" -C "$TMP_DIR"
mv "$TMP_DIR/${SERVER}" "$INSTALL_DIR/${SERVER}"
chmod +x "$INSTALL_DIR/${SERVER}"

echo "==> Successfully installed ${SERVER} to ${INSTALL_DIR}/${SERVER}!"
echo ""
echo "To configure in Antigravity or Claude Desktop, point command to:"
echo "  ${INSTALL_DIR}/${SERVER}"
