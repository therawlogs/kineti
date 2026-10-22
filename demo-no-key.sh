#!/usr/bin/env bash
# Run the test-to-package lifecycle in an isolated project copy.
set -euo pipefail

PROJECT_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEMO_ID="r25-$(date +%Y%m%d%H%M%S)-$$"
DEMO_ROOT="$PROJECT_ROOT/.kineti/demos/$DEMO_ID"
PACK_DIR="$DEMO_ROOT/.kineti/demos/packages"
SUCCESS_MARKER="$DEMO_ROOT/.kineti/demos/package-succeeded"
WORKING_LIB_SHA_BEFORE="$(shasum -a 256 "$PROJECT_ROOT/bin/lib.ts" | awk '{print $1}')"

if [ -e "$DEMO_ROOT" ]; then
  echo "Refusing to reuse existing demo fixture: $DEMO_ROOT" >&2
  exit 2
fi

mkdir -p "$DEMO_ROOT" "$DEMO_ROOT/tests" "$DEMO_ROOT/.kineti/tmp" "$PACK_DIR"
cp -R "$PROJECT_ROOT/bin" "$PROJECT_ROOT/hosts" "$PROJECT_ROOT/skills" "$DEMO_ROOT/"
cp "$PROJECT_ROOT/.npmignore" "$PROJECT_ROOT/package.json" "$PROJECT_ROOT/bun.lock" \
  "$PROJECT_ROOT/README.md" "$PROJECT_ROOT/LICENSE" "$PROJECT_ROOT/CHANGELOG.md" \
  "$PROJECT_ROOT/CONTRIBUTING.md" "$PROJECT_ROOT/CODE_OF_CONDUCT.md" \
  "$PROJECT_ROOT/SECURITY.md" "$PROJECT_ROOT/AGENTS.md" "$PROJECT_ROOT/kineti.config.json" \
  "$PROJECT_ROOT/setup.sh" "$PROJECT_ROOT/install.sh" "$PROJECT_ROOT/server.json" \
  "$PROJECT_ROOT/demo-no-key.sh" "$DEMO_ROOT/"
cp "$PROJECT_ROOT/tests/lib.test.ts" "$DEMO_ROOT/tests/lib.test.ts"
cp "$DEMO_ROOT/bin/lib.ts" "$DEMO_ROOT/.kineti/demos/original-lib.ts"
touch "$DEMO_ROOT/.kineti/npmrc"

export KINETI_MACHINE_DIR="$DEMO_ROOT/.kineti/machine"
export TMPDIR="$DEMO_ROOT/.kineti/tmp"
export NPM_CONFIG_USERCONFIG="$DEMO_ROOT/.kineti/npmrc"
export NPM_CONFIG_CACHE="$DEMO_ROOT/.kineti/npm-cache"
cd "$DEMO_ROOT"

echo "=== Kineti local test-to-package demonstration ==="
echo "Fixture: ${DEMO_ROOT#"$PROJECT_ROOT"/}"
echo

echo "Step 1: Run tests and record test evidence"
bun bin/kineti-evidence.ts run --label demo --required-test tests/lib.test.ts -- bun test tests/lib.test.ts
echo

echo "Step 2: Confirm the receipt is fresh"
bun bin/kineti-evidence.ts check --label demo
echo

echo "Step 3: Change only the isolated fixture"
printf '\n// temporary demonstration edit\n' >> bin/lib.ts
echo "Edited fixture bin/lib.ts; the working checkout was not edited."
echo

echo "Step 4: Try the packaging gate with stale evidence"
set +e
STALE_OUTPUT="$(bun bin/kineti-evidence.ts check --label demo 2>&1)"
STALE_STATUS=$?
set -e
printf '%s\n' "$STALE_OUTPUT"
if [ "$STALE_STATUS" -ne 4 ] || ! printf '%s\n' "$STALE_OUTPUT" | rg -q 'STALE'; then
  echo "ERROR: stale evidence did not block with the expected status" >&2
  exit 1
fi
shopt -s nullglob
BLOCKED_PACKAGES=("$PACK_DIR"/kineti-*.tgz)
if [ "${#BLOCKED_PACKAGES[@]}" -ne 0 ] || [ -e "$SUCCESS_MARKER" ]; then
  echo "ERROR: a package or success marker exists after the stale block" >&2
  exit 1
fi
echo "EXPECTED BLOCK: npm pack was not run; no package or success marker exists."
echo

echo "Step 5: Restore the fixture source and record new test evidence"
cp "$DEMO_ROOT/.kineti/demos/original-lib.ts" bin/lib.ts
bun bin/kineti-evidence.ts run --label demo --required-test tests/lib.test.ts -- bun test tests/lib.test.ts
bun bin/kineti-evidence.ts check --label demo
echo

echo "Step 6: Run the real local npm packaging command through the gate"
bun bin/kineti-evidence.ts run --label demo-package --kind package -- \
  npm pack --ignore-scripts --loglevel=error --pack-destination "$PACK_DIR"
bun bin/kineti-evidence.ts check --label demo-package
PACKAGES=("$PACK_DIR"/kineti-*.tgz)
if [ "${#PACKAGES[@]}" -ne 1 ]; then
  echo "ERROR: expected one package from the fresh control, found ${#PACKAGES[@]}" >&2
  exit 1
fi
PACKAGE_SHA="$(shasum -a 256 "${PACKAGES[0]}" | awk '{print $1}')"
WORKING_LIB_SHA_AFTER="$(shasum -a 256 "$PROJECT_ROOT/bin/lib.ts" | awk '{print $1}')"
if [ "$WORKING_LIB_SHA_BEFORE" != "$WORKING_LIB_SHA_AFTER" ]; then
  echo "ERROR: the working checkout changed during the fixture run" >&2
  exit 1
fi
touch "$SUCCESS_MARKER"
RESULT_FILE="$DEMO_ROOT/.kineti/demos/r25-result.json"
cat > "$RESULT_FILE" <<EOF
{
  "schema_version": 1,
  "test_receipt_label": "demo",
  "test_receipt_fresh_after_reproof": true,
  "stale_gate_exit_code": $STALE_STATUS,
  "blocked_package_count": ${#BLOCKED_PACKAGES[@]},
  "success_marker_present_after_stale_block": false,
  "package_receipt_label": "demo-package",
  "package_receipt_fresh": true,
  "package_file": "${PACKAGES[0]#"$DEMO_ROOT"/}",
  "package_sha256": "$PACKAGE_SHA"
}
EOF
echo "CONTROL PASS: ${PACKAGES[0]#"$DEMO_ROOT"/} sha256=$PACKAGE_SHA"
echo "WORKING CHECKOUT UNCHANGED: bin/lib.ts sha256=$WORKING_LIB_SHA_AFTER"
echo "Result record: ${RESULT_FILE#"$DEMO_ROOT"/}"
echo "Success marker: ${SUCCESS_MARKER#"$DEMO_ROOT"/}"
echo
echo "=== Demonstration complete ==="
