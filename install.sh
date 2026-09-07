#!/usr/bin/env bash
# Kineti install: downloads the right binary for your computer.
set -euo pipefail

REPO="${KINETI_REPO:-therawlogs/kineti}"
VERSION="${KINETI_VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$OS" = "darwin" ] && [ "$ARCH" = "arm64" ]; then ASSET="kineti-darwin-arm64";
elif [ "$OS" = "darwin" ] && [ "$ARCH" = "x86_64" ]; then ASSET="kineti-darwin-x64";
elif [ "$OS" = "linux" ] && [ "$ARCH" = "x86_64" ]; then ASSET="kineti-linux-x64";
else echo "kineti: no build for $OS $ARCH yet. Use: npm i -g kineti"; exit 1; fi

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

echo "kineti: download $URL"
TMP="$(mktemp)"
curl -fsSL "$URL" -o "$TMP"
chmod +x "$TMP"

if [ -w "$INSTALL_DIR" ]; then mv "$TMP" "$INSTALL_DIR/kineti";
else echo "kineti: need admin to write to $INSTALL_DIR"; sudo mv "$TMP" "$INSTALL_DIR/kineti"; fi

echo "kineti: installed. Run: kineti init"
