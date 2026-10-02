#!/usr/bin/env bash
# No-Key Demo: Test -> Proof -> Edit -> Stale -> Repair -> Re-Prove
# Demonstrates Kineti's evidence system without API keys

set -euo pipefail

echo "=== Kineti No-Key Demo ==="
echo

# Step 1: Run tests and record proof
echo "Step 1: Run tests and record proof"
echo "-----------------------------------"
bun bin/kineti-evidence.ts run --label demo -- bun test tests/lib.test.ts
echo

# Step 2: Verify proof is fresh
echo "Step 2: Verify proof is fresh"
echo "------------------------------"
bun bin/kineti-evidence.ts check --label demo
echo

# Step 3: Show fingerprint
echo "Step 3: Current code fingerprint"
echo "--------------------------------"
bun bin/kineti-evidence.ts fingerprint
echo

# Step 4: Edit a file (make proof stale)
echo "Step 4: Edit a source file (making proof stale)"
echo "------------------------------------------------"
echo "// Demo edit: $(date)" >> src/lib.ts
echo "Edited src/lib.ts"
echo

# Step 5: Check proof is now stale
echo "Step 5: Check proof (should be STALE)"
echo "--------------------------------------"
if bun bin/kineti-evidence.ts check --label demo 2>&1; then
  echo "UNEXPECTED: proof still fresh"
else
  echo "EXPECTED: proof is STALE (code changed after run)"
fi
echo

# Step 6: Show new fingerprint
echo "Step 6: New code fingerprint"
echo "----------------------------"
bun bin/kineti-evidence.ts fingerprint
echo

# Step 7: Repair - re-run tests
echo "Step 7: Repair - re-run tests and re-prove"
echo "-------------------------------------------"
bun bin/kineti-evidence.ts run --label demo -- bun test tests/lib.test.ts
echo

# Step 8: Verify proof is fresh again
echo "Step 8: Verify proof is fresh again"
echo "------------------------------------"
bun bin/kineti-evidence.ts check --label demo
echo

# Cleanup: revert the edit
echo "Step 9: Cleanup - revert edit"
echo "------------------------------"
sed -i '' '$d' src/lib.ts
echo "Reverted src/lib.ts"
echo

echo "=== Demo Complete ==="