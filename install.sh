#!/usr/bin/env bash
# Install claude-model-switch binary
# Usage: curl -fsSL https://raw.githubusercontent.com/codejunkie99/claude-model-switch/main/install.sh | sh
set -euo pipefail

REPO="codejunkie99/claude-model-switch"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64)        ARCH="x86_64" ;;
  arm64|aarch64) ARCH="aarch64" ;;
  *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

case "$OS" in
  darwin) TARGET="${ARCH}-apple-darwin" ;;
  linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
  *) echo "Unsupported OS: $OS" >&2; exit 1 ;;
esac

URL="https://github.com/${REPO}/releases/latest/download/claude-model-switch-${TARGET}.tar.gz"

echo "Installing claude-model-switch for ${TARGET}..."
mkdir -p "$INSTALL_DIR"

if command -v curl &>/dev/null; then
  curl -fsSL "$URL" | tar xz -C "$INSTALL_DIR"
elif command -v wget &>/dev/null; then
  wget -qO- "$URL" | tar xz -C "$INSTALL_DIR"
else
  echo "Error: curl or wget required" >&2
  exit 1
fi

chmod +x "$INSTALL_DIR/claude-model-switch"

# Check if INSTALL_DIR is on PATH
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
  echo ""
  echo "Add this to your shell profile:"
  echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
  echo ""
fi

echo "Installed claude-model-switch to $INSTALL_DIR/claude-model-switch"
echo ""
echo "Quick start:"
echo "  claude-model-switch init"
echo "  claude-model-switch setup glm --api-key sk-your-key"
echo "  claude-model-switch start"
echo "  claude-model-switch use glm"
