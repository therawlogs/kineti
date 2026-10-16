#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$ROOT/.kineti/acceptance/$(date +%s)-$$"
KINETI_VERSION="$(bun -p 'require("./package.json").version')"
KINETI_NPM_TARBALL="${KINETI_NPM_TARBALL:-$ROOT/.kineti/acceptance/candidate/kineti-$KINETI_VERSION.tgz}"
KINETI_CRATE_PATH="${KINETI_CRATE_PATH:-$ROOT/core-native/crates/kineti-cli}"
mkdir -p "$WORK/npm-baseline" "$WORK/npm-candidate" "$WORK/crate-baseline" "$WORK/crate-candidate"
export npm_config_cache="$WORK/npm-cache"
export CARGO_HOME="$WORK/cargo-home"

echo "Baseline: installing published npm kineti@0.4.0"
npm install --prefix "$WORK/npm-baseline" --no-audit --no-fund kineti@0.4.0
"$WORK/npm-baseline/node_modules/.bin/kineti" --help

if [ -z "$KINETI_NPM_TARBALL" ] || [ -z "$KINETI_CRATE_PATH" ]; then
  echo "Candidate check needs KINETI_NPM_TARBALL and KINETI_CRATE_PATH; baseline passed, candidate is unsupported." >&2
  exit 78
fi

echo "Candidate: installing local npm tarball $KINETI_NPM_TARBALL"
npm install --prefix "$WORK/npm-candidate" --no-audit --no-fund "$KINETI_NPM_TARBALL"
"$WORK/npm-candidate/node_modules/.bin/kineti" --help

echo "Baseline: installing published kineti-cli@0.4.0"
cargo install --root "$WORK/crate-baseline" --locked kineti-cli@0.4.0
"$WORK/crate-baseline/bin/kineti-cli" --version

echo "Candidate: installing local crate $KINETI_CRATE_PATH"
cargo install --root "$WORK/crate-candidate" --path "$KINETI_CRATE_PATH" --locked --force
"$WORK/crate-candidate/bin/kineti-cli" --version

echo "1 pass"
echo "0 fail"
echo "CLEAN INSTALL ACCEPTANCE PASSED"
