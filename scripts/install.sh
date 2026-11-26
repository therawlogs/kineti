#!/bin/sh
set -eu

REPO="therawlogs/kineti"
VERSION="${KINETI_VERSION:-1.0.1}"
INSTALL_DIR="${KINETI_INSTALL_DIR:-$HOME/.local/bin}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS_KEY="linux" ;;
  Darwin) OS_KEY="darwin" ;;
  *)
    echo "Unsupported OS: $OS (Kineti supports Linux and macOS)" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_KEY="x64" ;;
  arm64|aarch64) ARCH_KEY="arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH (Kineti supports x64 and arm64)" >&2
    exit 1
    ;;
esac

TARGET="${OS_KEY}-${ARCH_KEY}"
TAG="v${VERSION#v}"
ARCHIVE="kineti-${TAG}-${TARGET}.tar.gz"
BASE_URL="https://github.com/${REPO}/releases/download/${TAG}"

TMPDIR="$(mktemp -d)"
cleanup() { rm -rf "$TMPDIR"; }
trap cleanup EXIT

echo "Downloading Kineti ${TAG} (${TARGET})..."
curl -fsSL "${BASE_URL}/${ARCHIVE}" -o "${TMPDIR}/${ARCHIVE}"
curl -fsSL "${BASE_URL}/SHA256SUMS.txt" -o "${TMPDIR}/SHA256SUMS.txt"

EXPECTED_SHA="$(awk -v file="$ARCHIVE" '$2 == file || $2 == ("./" file) { print $1 }' "${TMPDIR}/SHA256SUMS.txt" | head -n 1)"
if [ -z "$EXPECTED_SHA" ]; then
  echo "Could not find ${ARCHIVE} in ${BASE_URL}/SHA256SUMS.txt" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  ACTUAL_SHA="$(sha256sum "${TMPDIR}/${ARCHIVE}" | awk '{print $1}')"
elif command -v shasum >/dev/null 2>&1; then
  ACTUAL_SHA="$(shasum -a 256 "${TMPDIR}/${ARCHIVE}" | awk '{print $1}')"
else
  echo "Neither sha256sum nor shasum is installed to verify download integrity" >&2
  exit 1
fi

if [ "$ACTUAL_SHA" != "$EXPECTED_SHA" ]; then
  echo "SHA-256 mismatch for ${ARCHIVE}: expected ${EXPECTED_SHA}, got ${ACTUAL_SHA}" >&2
  exit 1
fi

tar -xzf "${TMPDIR}/${ARCHIVE}" -C "${TMPDIR}"
mkdir -p "$INSTALL_DIR"
install -m 0755 "${TMPDIR}/kineti" "${INSTALL_DIR}/kineti"

echo "Installed kineti ${TAG} to ${INSTALL_DIR}/kineti"
"${INSTALL_DIR}/kineti" --version

case ":$PATH:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "Add ${INSTALL_DIR} to your PATH:"
    echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    ;;
esac
