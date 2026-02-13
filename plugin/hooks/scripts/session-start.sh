#!/usr/bin/env bash
# SessionStart hook: auto-install claude-model-switch binary and start proxy
set -euo pipefail

CMS_BIN="claude-model-switch"
INSTALL_DIR="$HOME/.local/bin"
REPO="codejunkie99/claude-model-switch"
CONFIG="$HOME/.claude/model-profiles.json"
PID_FILE="$HOME/.claude/model-switch-proxy.pid"

# Ensure install dir exists
mkdir -p "$INSTALL_DIR"

# Also check INSTALL_DIR directly in case it's not on PATH
if command -v "$CMS_BIN" &>/dev/null; then
  CMS_BIN="$(command -v "$CMS_BIN")"
elif [ -x "$INSTALL_DIR/$CMS_BIN" ]; then
  CMS_BIN="$INSTALL_DIR/$CMS_BIN"
fi

# Check if binary is installed
if [ ! -x "$CMS_BIN" ]; then
  CMS_BIN="$INSTALL_DIR/claude-model-switch"
  echo "[cms] claude-model-switch not found. Installing..."

  OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
  ARCH="$(uname -m)"
  case "$ARCH" in
    x86_64)  ARCH="x86_64" ;;
    arm64|aarch64) ARCH="aarch64" ;;
    *) echo "[cms] Unsupported architecture: $ARCH"; exit 1 ;;
  esac

  case "$OS" in
    darwin) TARGET="${ARCH}-apple-darwin" ;;
    linux)  TARGET="${ARCH}-unknown-linux-gnu" ;;
    *) echo "[cms] Unsupported OS: $OS"; exit 1 ;;
  esac

  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/claude-model-switch-${TARGET}.tar.gz"

  if command -v curl &>/dev/null; then
    curl -fsSL "$DOWNLOAD_URL" | tar xz -C "$INSTALL_DIR"
  elif command -v wget &>/dev/null; then
    wget -qO- "$DOWNLOAD_URL" | tar xz -C "$INSTALL_DIR"
  else
    # Fallback: try cargo install
    if command -v cargo &>/dev/null; then
      echo "[cms] No curl/wget found. Trying cargo install..."
      cargo install claude-model-switch
    else
      echo "[cms] Cannot install: need curl, wget, or cargo"
      exit 1
    fi
  fi

  chmod +x "$INSTALL_DIR/$CMS_BIN" 2>/dev/null || true
  echo "[cms] Installed to $INSTALL_DIR/$CMS_BIN"
fi

# First-time init if no config exists
if [ ! -f "$CONFIG" ]; then
  "$CMS_BIN" init 2>/dev/null || true
fi

# Start proxy if not running
if [ -f "$PID_FILE" ]; then
  PID=$(cat "$PID_FILE")
  if ! kill -0 "$PID" 2>/dev/null; then
    rm -f "$PID_FILE"
    "$CMS_BIN" start 2>/dev/null || true
  fi
else
  "$CMS_BIN" start 2>/dev/null || true
fi

echo "[cms] Ready"
