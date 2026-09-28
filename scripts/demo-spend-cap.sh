#!/usr/bin/env bash
# Kineti 30-60s Spend Circuit Breaker Terminal Demo
# Demonstrates: Clean spend logging, 95% stage ceiling trip (exit 3), fail-closed pre-flight,
# human-only override gating (exit 2 -> exit 0).

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="bun $REPO_ROOT/bin/kineti-spend.ts"

DEMO_DIR="${KINETI_DEMO_DIR:-$(mktemp -d 2>/dev/null || mktemp -d -t 'kineti-demo')}"
export KINETI_MACHINE_DIR="$DEMO_DIR/.machine"
mkdir -p "$KINETI_MACHINE_DIR"
cd "$DEMO_DIR"

# Initialize local kineti directory
mkdir -p .kineti
cat << 'EOF' > kineti.config.json
{
  "version": "0.3.6",
  "project": "demo",
  "spend_limit_usd": {
    "global": 50.0,
    "per_stage_default": 10.0
  }
}
EOF

echo "=========================================================="
echo "  KINETI OS — SPEND CIRCUIT BREAKER 30-60s TERMINAL DEMO"
echo "=========================================================="

echo -e "\n[Step 1] Baseline status (\$50 global ceiling, \$10 stage ceiling):"
echo "$ kineti spend status"
$CLI status

echo -e "\n[Step 2] Legitimate agent inference task in stage 'build':"
echo "$ kineti spend log --stage build --model sonnet --tokens-in 400000 --tokens-out 200000"
$CLI log --stage build --model sonnet --tokens-in 400000 --tokens-out 200000

echo -e "\n[Step 3] Status updates cleanly:"
echo "$ kineti spend status"
$CLI status

echo -e "\n[Step 4] Runaway task attempts heavy call (\$6.00), crossing 95% ceiling (\$9.50):"
echo "$ kineti spend log --stage build --model opus --tokens-in 100000 --tokens-out 60000"
set +e
$CLI log --stage build --model opus --tokens-in 100000 --tokens-out 60000
TRIP_EXIT=$?
set -e
echo "Exit code: $TRIP_EXIT (Expected: 3 - SPEND BREAKER TRIPPED)"

echo -e "\n[Step 5] Subsequent pre-flight check fails closed:"
echo "$ kineti spend check"
set +e
$CLI check
CHECK_EXIT=$?
set -e
echo "Exit code: $CHECK_EXIT (Expected: 3 - FAIL CLOSED)"

echo -e "\n[Step 6] Automated agent attempts unauthorized reset:"
echo "$ kineti spend reset"
set +e
$CLI reset
RESET_EXIT=$?
set -e
echo "Exit code: $RESET_EXIT (Expected: 2 - HUMAN ONLY)"

echo -e "\n[Step 7] Human developer confirms and resets circuit breaker:"
echo "$ kineti spend reset --i-am-human"
$CLI reset --i-am-human

echo -e "\n[Step 8] System restored to normal operational status:"
echo "$ kineti spend status"
$CLI status

# Cleanup demo sandbox
rm -rf "$DEMO_DIR"
echo -e "\n✅ Demo completed successfully across all 8 states with zero side-effects."
