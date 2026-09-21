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
else echo "kineti: no build for $OS $ARCH yet (no linux-arm64/Windows). Use: npm i -g kineti"; exit 1; fi

if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
  URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

echo "kineti: download $URL"
TMP="$(mktemp)"
# Require HTTPS, retry once.
curl --proto '=https' --tlsv1.2 -fsSL --retry 2 "$URL" -o "$TMP"
# Optional checksum: set KINETI_SHA256 to the expected hex digest to verify.
if [ -n "${KINETI_SHA256:-}" ]; then
  if command -v shasum >/dev/null 2>&1; then GOT="$(shasum -a 256 "$TMP" | awk '{print $1}')";
  elif command -v sha256sum >/dev/null 2>&1; then GOT="$(sha256sum "$TMP" | awk '{print $1}')";
  else echo "kineti: cannot verify checksum (no shasum/sha256sum)"; exit 1; fi
  if [ "$GOT" != "$KINETI_SHA256" ]; then echo "kineti: checksum mismatch, refusing install"; rm -f "$TMP"; exit 1; fi
  echo "kineti: checksum ok"
else
  echo "kineti: no KINETI_SHA256 set, skipping checksum. Verify release notes over the same connection."
fi
chmod +x "$TMP"

# bun is required for bin/*.ts skills even with the binary install.
if ! command -v bun >/dev/null 2>&1; then
  echo "kineti: warning: 'bun' not found. Install from https://bun.sh for skills (spend/saga/evidence)."
fi
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) echo "kineti: warning: $INSTALL_DIR is not in PATH. Add it or set INSTALL_DIR=/opt/homebrew/bin." ;;
esac

if [ -w "$INSTALL_DIR" ]; then mv "$TMP" "$INSTALL_DIR/kineti";
else echo "kineti: need admin to write to $INSTALL_DIR"; sudo mv "$TMP" "$INSTALL_DIR/kineti"; fi

echo "kineti: installed. Run: kineti init"
