#!/bin/sh
# Kineti installer — fetches the latest release binary for your platform.
set -e

REPO="iwpraveen/kineti"
VERSION="${1:-latest}"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS/$ARCH" in
  Darwin/arm64)  asset="kineti-darwin-arm64" ;;
  Darwin/x86_64) asset="kineti-darwin-x64" ;;
  Linux/x86_64)  asset="kineti-linux-x64" ;;
  Linux/aarch64) asset="kineti-linux-arm64" ;;
  *) echo "unsupported platform: $OS/$ARCH"; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
  [ -n "$VERSION" ] || { echo "could not determine latest release"; exit 1; }
fi

url="https://github.com/$REPO/releases/download/$VERSION/$asset"
echo "fetching $url"
tmp=$(mktemp)
curl -fsSL "$url" -o "$tmp"
chmod +x "$tmp"

dest="$HOME/.local/bin"
mkdir -p "$dest"
mv "$tmp" "$dest/kineti"

echo "installed: $dest/kineti ($VERSION)"
case ":$PATH:" in
  *":$dest:"*) ;;
  *) echo "note: $dest is not on your PATH — add it to your shell profile" ;;
esac
