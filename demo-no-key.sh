#!/usr/bin/env bash
# No-Key Demo: Test -> Proof -> Edit -> Stale -> Repair -> Re-Prove
# Demonstrates Kineti's evidence system without API keys.

set -euo pipefail

DEMO_EDITED=0
DEMO_BACKUP=""
DEMO_PAUSE_SECONDS="${DEMO_PAUSE_SECONDS:-0}"
case "$DEMO_PAUSE_SECONDS" in
  ''|*[!0-9]*) echo "DEMO_PAUSE_SECONDS must be a whole number from 0 to 15" >&2; exit 2 ;;
esac
if [ "$DEMO_PAUSE_SECONDS" -gt 15 ]; then
  echo "DEMO_PAUSE_SECONDS must be a whole number from 0 to 15" >&2
  exit 2
fi
pause_for_recording() {
  if [ "$DEMO_PAUSE_SECONDS" -gt 0 ]; then sleep "$DEMO_PAUSE_SECONDS"; fi
}
cleanup_demo_edit() {
  if [ "$DEMO_EDITED" -eq 1 ] && [ -n "$DEMO_BACKUP" ] && [ -f "$DEMO_BACKUP" ]; then
    cp -p "$DEMO_BACKUP" bin/lib.ts
  fi
  if [ -n "$DEMO_BACKUP" ]; then rm -f "$DEMO_BACKUP"; fi
}
trap cleanup_demo_edit EXIT

mkdir -p .kineti/demos
DEMO_BACKUP="$(mktemp .kineti/demos/no-key.XXXXXX)"
cp -p bin/lib.ts "$DEMO_BACKUP"

echo "=== Kineti No-Key Demo ==="
echo

echo "Step 1: Run tests and record proof"
echo "-----------------------------------"
bun bin/kineti-evidence.ts run --label demo --required-test tests/lib.test.ts -- bun test tests/lib.test.ts
echo
pause_for_recording

echo "Step 2: Verify proof is fresh"
echo "------------------------------"
bun bin/kineti-evidence.ts check --label demo
echo
pause_for_recording

echo "Step 3: Current code fingerprint"
echo "--------------------------------"
bun bin/kineti-evidence.ts fingerprint
echo
pause_for_recording

echo "Step 4: Edit a source file (making proof stale)"
echo "------------------------------------------------"
DEMO_EDITED=1
echo "// Demo edit: $(date)" >> bin/lib.ts
echo "Edited bin/lib.ts"
echo
pause_for_recording

echo "Step 5: Check proof (should be STALE)"
echo "--------------------------------------"
set +e
STALE_OUTPUT="$(bun bin/kineti-evidence.ts check --label demo 2>&1)"
STALE_STATUS=$?
set -e
printf '%s\n' "$STALE_OUTPUT"
if [ "$STALE_STATUS" -eq 0 ] || ! printf '%s\n' "$STALE_OUTPUT" | rg -q 'STALE'; then
  echo "ERROR: expected a STALE receipt result" >&2
  exit 1
fi
echo "EXPECTED: proof is stale after the edit"
echo
pause_for_recording

echo "Step 6: New code fingerprint"
echo "----------------------------"
bun bin/kineti-evidence.ts fingerprint
echo
pause_for_recording

echo "Step 7: Repair and re-run tests"
echo "--------------------------------"
cp -p "$DEMO_BACKUP" bin/lib.ts
DEMO_EDITED=0
echo "Restored bin/lib.ts"
bun bin/kineti-evidence.ts run --label demo --required-test tests/lib.test.ts -- bun test tests/lib.test.ts
echo
pause_for_recording

echo "Step 8: Verify proof is fresh again"
echo "------------------------------------"
bun bin/kineti-evidence.ts check --label demo
echo
pause_for_recording

echo "=== Demo Complete ==="
echo "1 pass"
echo "0 fail"
