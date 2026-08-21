#!/usr/bin/env bash
# Smoke test for setup.sh using a fake HOME. Touches nothing real.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

run() { HOME="$TMP/home" bash "$HERE/setup.sh" "$@"; }

# 1. No hosts present -> skip everything, exit 0
out="$(run)"
echo "$out" | grep -q "skipped: opencode" || { echo "FAIL: empty machine should skip opencode"; exit 1; }
echo "$out" | grep -q "No agent host folders were found" || { echo "FAIL: empty machine should report zero hosts"; exit 1; }

# 2. Create two fake hosts -> both install
mkdir -p "$TMP/home/.opencode/skills" "$TMP/home/.gemini/config/skills"
out="$(run)"
echo "$out" | grep -q "installed: 16 skills.*opencode" || { echo "FAIL: opencode install"; exit 1; }
echo "$out" | grep -q "installed: 16 skills.*gemini" || { echo "FAIL: gemini install"; exit 1; }
[[ -f "$TMP/home/.opencode/skills/kineti-qa/SKILL.md" ]] || { echo "FAIL: file missing"; exit 1; }

# 3. Idempotent second run
out2="$(run)"
[[ "$(echo "$out2" | grep -c 'installed: 16')" == "2" ]] || { echo "FAIL: not idempotent"; exit 1; }

# 4. Uninstall removes only kineti-* files
run --uninstall >/dev/null
[[ ! -d "$TMP/home/.opencode/skills/kineti-qa" ]] || { echo "FAIL: uninstall left files"; exit 1; }
[[ -d "$TMP/home/.gemini/config/skills" ]] || { echo "FAIL: uninstalled too much"; exit 1; }

# 5. Force flag creates a missing host
run --host claude >/dev/null
[[ -f "$TMP/home/.claude/skills/kineti-spec/SKILL.md" ]] || { echo "FAIL: force install"; exit 1; }
run --uninstall >/dev/null

echo "PASS: installer smoke test (5 checks)"
