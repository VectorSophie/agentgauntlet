#!/usr/bin/env bash
set -euo pipefail

REPO="VectorSophie/agentgauntlet"
BIN="agentgauntlet"

# Detect OS + arch
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS/$ARCH" in
  linux/x86_64)  ARTIFACT="agentgauntlet-linux-x86_64" ;;
  darwin/x86_64) ARTIFACT="agentgauntlet-macos-x86_64" ;;
  darwin/arm64)  ARTIFACT="agentgauntlet-macos-aarch64" ;;
  *)
    echo "Unsupported platform: $OS/$ARCH"
    echo "Try: cargo install agentgauntlet"
    exit 1
    ;;
esac

LATEST=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')
URL="https://github.com/$REPO/releases/download/$LATEST/$ARTIFACT"

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"

echo "Installing agentgauntlet $LATEST → $INSTALL_DIR/$BIN"
curl -fsSL "$URL" -o "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN"

echo ""
echo "✅ Done! Run: agentgauntlet scan"
echo ""

# PATH hint
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo "⚠️  Add to PATH: export PATH=\"\$PATH:$INSTALL_DIR\""
fi
