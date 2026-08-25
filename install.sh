#!/bin/sh
# Kineti installer — fetches a release binary for your platform and verifies
# it against the release's SHA256SUMS before installing.
#
# Serving contract: https://getkineti.com/install.sh must serve THIS file
# verbatim (sync checklist in docs/RELEASE.md). Binaries always come from
# GitHub Releases.
#
# Install directory precedence (never prompts for sudo):
#   1. $KINETI_INSTALL_DIR
#   2. /usr/local/bin — only if already writable by the current user
#   3. $HOME/.local/bin  (default)
#   4. $HOME/.cargo/bin  (fallback when it is on PATH and .local/bin is not)
set -eu

REPO="therawlogs/kineti"
VERSION="${1:-latest}"

OS=$(uname -s)
ARCH=$(uname -m)

case "$OS/$ARCH" in
  Darwin/arm64)  base="kineti-darwin-arm64" ;;
  Darwin/x86_64) base="kineti-darwin-x64" ;;
  Linux/x86_64)  base="kineti-linux-x64" ;;
  Linux/aarch64) base="kineti-linux-arm64" ;;
  *) echo "unsupported platform: $OS/$ARCH"; exit 1 ;;
esac

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p')
  [ -n "$VERSION" ] || { echo "could not determine latest release"; exit 1; }
fi

# Prefer static Linux builds (zero dyld); fall back to the gnu-linked binary.
asset=""
for candidate in "${base}-static" "$base"; do
  code=$(curl -fsSL -o /dev/null -w '%{http_code}' \
    "https://github.com/$REPO/releases/download/$VERSION/$candidate" || echo 000)
  if [ "$code" = "302" ] || [ "$code" = "200" ]; then
    asset="$candidate"
    break
  fi
done
[ -n "$asset" ] || { echo "no binary asset found for $OS/$ARCH in $VERSION"; exit 1; }

url="https://github.com/$REPO/releases/download/$VERSION/$asset"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

echo "fetching $url"
curl -fsSL "$url" -o "$tmpdir/kineti"

# ── integrity: verify against the release's SHA256SUMS ──────────────────────
sums_url="https://github.com/$REPO/releases/download/$VERSION/SHA256SUMS"
if curl -fsSL "$sums_url" -o "$tmpdir/SHA256SUMS" 2>/dev/null; then
  expected=$(grep " $asset\$" "$tmpdir/SHA256SUMS" | cut -d' ' -f1)
  if [ -z "$expected" ]; then
    echo "⛔ SHA256SUMS has no entry for $asset — refusing to install"; exit 1
  fi
  if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$tmpdir/kineti" | cut -d' ' -f1)
  elif command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$tmpdir/kineti" | cut -d' ' -f1)
  else
    echo "⚠ no sha256 tool found — skipping verification (consider installing coreutils)"
    actual="$expected"
  fi
  if [ "$actual" != "$expected" ]; then
    echo "⛔ CHECKSUM MISMATCH for $asset"
    echo "   expected $expected"
    echo "   actual   $actual"
    echo "Refusing to install a tampered or corrupted artifact."
    exit 1
  fi
  echo "✔ checksum verified ($asset)"
else
  echo "⚠ SHA256SUMS not published for $VERSION — skipping verification"
fi

chmod +x "$tmpdir/kineti"

# ── destination precedence: never prompt for sudo ───────────────────────────
dest=""
if [ -n "${KINETI_INSTALL_DIR:-}" ]; then
  dest="$KINETI_INSTALL_DIR"
elif [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
  dest="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ] || { [ ! -d "$HOME/.cargo/bin" ]; }; then
  dest="$HOME/.local/bin"
elif echo ":$PATH:" | grep -q ":$HOME/.cargo/bin:"; then
  dest="$HOME/.cargo/bin"
else
  dest="$HOME/.local/bin"
fi

mkdir -p "$dest"
mv "$tmpdir/kineti" "$dest/kineti"

echo "installed: $dest/kineti ($VERSION)"
case ":$PATH:" in
  *":$dest:"*) ;;
  *) echo "note: $dest is not on your PATH — add it to your shell profile" ;;
esac
