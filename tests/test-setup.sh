#!/usr/bin/env bash
# Smoke test for setup.sh using a fake HOME. Touches nothing real.
# Run by hand: ./tests/test-setup.sh (not part of `bun test`).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Keep the isolated HOME inside the project, as required by AGENTS.md.
TMP="$HERE/.kineti/acceptance/setup-smoke"
rm -rf "$TMP"
mkdir -p "$TMP"
trap 'rm -rf "$TMP"' EXIT

run() { HOME="$TMP/home" bash "$HERE/setup.sh" "$@"; }

# 1. No hosts present -> skip everything, exit 0
out="$(run)"
echo "$out" | grep -q "skipped: opencode" || { echo "FAIL: empty machine should skip opencode"; exit 1; }
echo "$out" | grep -q "No agent host folders were found" || { echo "FAIL: empty machine should report zero hosts"; exit 1; }

# 2. Create two fake hosts -> both install; a user-owned skill is preserved.
mkdir -p "$TMP/home/.opencode/skills" "$TMP/home/.gemini/config/skills"
mkdir -p "$TMP/home/.opencode/skills/my-skill"
printf 'user content\n' > "$TMP/home/.opencode/skills/my-skill/SKILL.md"
out="$(run)"
skill_count="$(find "$HERE/skills" -mindepth 1 -maxdepth 1 -type d | wc -l | tr -d ' ')"
echo "$out" | grep -q "installed: $skill_count skills.*opencode" || { echo "FAIL: opencode install"; exit 1; }
echo "$out" | grep -q "installed: $skill_count skills.*gemini" || { echo "FAIL: gemini install"; exit 1; }
[[ -f "$TMP/home/.opencode/skills/kineti-qa/SKILL.md" ]] || { echo "FAIL: file missing"; exit 1; }
[[ "$(cat "$TMP/home/.opencode/skills/my-skill/SKILL.md")" == "user content" ]] || { echo "FAIL: user skill changed"; exit 1; }

# 3. Idempotent second run
out2="$(run)"
[[ "$(echo "$out2" | grep -c "installed: $skill_count")" == "2" ]] || { echo "FAIL: not idempotent"; exit 1; }

# 4. Uninstall removes only kineti-* files
run --uninstall >/dev/null
[[ ! -d "$TMP/home/.opencode/skills/kineti-qa" ]] || { echo "FAIL: uninstall left files"; exit 1; }
[[ -d "$TMP/home/.gemini/config/skills" ]] || { echo "FAIL: uninstalled too much"; exit 1; }

# 5. Force flag creates a missing host (including cursor)
run --host claude >/dev/null
[[ -f "$TMP/home/.claude/skills/kineti-spec/SKILL.md" ]] || { echo "FAIL: force install"; exit 1; }
run --host cursor >/dev/null
[[ -f "$TMP/home/.cursor/skills/kineti-spec/SKILL.md" ]] || { echo "FAIL: cursor force install"; exit 1; }
run --uninstall >/dev/null

# 6. Every configured host installs in a clean home. Model-side routing needs
# live host/model runners, which this deterministic acceptance environment lacks.
for host in antigravity claude codex cursor fx gemini opencode; do
  HOME="$TMP/home-$host" CODEX_HOME="$TMP/home-$host/.codex" bash "$HERE/setup.sh" --host "$host" >/dev/null
  [[ -f "$TMP/home-$host/.kineti/repo" ]] || { echo "FAIL: $host setup pointer missing"; exit 1; }
  case "$host" in
    antigravity) skill_root="$TMP/home-$host/.antigravity/skills" ;;
    claude) skill_root="$TMP/home-$host/.claude/skills" ;;
    codex) skill_root="$TMP/home-$host/.codex/skills" ;;
    cursor) skill_root="$TMP/home-$host/.cursor/skills" ;;
    fx) skill_root="$TMP/home-$host/.fx/skills" ;;
    gemini) skill_root="$TMP/home-$host/.gemini/config/skills" ;;
    opencode) skill_root="$TMP/home-$host/.opencode/skills" ;;
  esac
  [[ -f "$skill_root/kineti-officehours/SKILL.md" ]] || { echo "FAIL: $host skill missing"; exit 1; }
  echo "ROUTING_UNSUPPORTED host=$host reason=no-isolated-host-model-runner"
done

echo "PASS: installer smoke test (clean setup, preservation, repeat safety, uninstall, 7 host installs)"
