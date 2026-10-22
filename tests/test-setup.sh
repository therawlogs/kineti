#!/usr/bin/env bash
# Smoke test for setup.sh using a fake HOME. Touches nothing real.
# Run by hand: ./tests/test-setup.sh (not part of `bun test`).
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# Keep the isolated HOME inside the project, as required by AGENTS.md.
mkdir -p "$HERE/.kineti/acceptance"
TMP="$(mktemp -d "$HERE/.kineti/acceptance/setup-smoke.XXXXXX")"
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
    antigravity) skill_root="$TMP/home-$host/.gemini/config/plugins/kineti/skills" ;;
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

# Antigravity plugin metadata follows the same ownership rules as skills.
AGY_HOME="$TMP/home-antigravity"
AGY_PLUGIN="$AGY_HOME/.gemini/config/plugins/kineti"
[[ -f "$AGY_PLUGIN/plugin.json" && -f "$AGY_PLUGIN/rules/AGENTS.md" ]] || { echo "FAIL: Antigravity plugin metadata missing"; exit 1; }
HOME="$AGY_HOME" bash "$HERE/setup.sh" --host antigravity >/dev/null
printf 'custom owner guidance\n' > "$AGY_PLUGIN/rules/AGENTS.md"
HOME="$AGY_HOME" bash "$HERE/setup.sh" --host antigravity --uninstall >/dev/null
[[ ! -e "$AGY_PLUGIN/plugin.json" && ! -e "$AGY_PLUGIN/skills/kineti-officehours/SKILL.md" ]] || { echo "FAIL: owned Antigravity files survived uninstall"; exit 1; }
[[ "$(cat "$AGY_PLUGIN/rules/AGENTS.md")" == "custom owner guidance" ]] || { echo "FAIL: customized Antigravity guidance removed"; exit 1; }

# 7. Paths with spaces, unmanaged conflicts, and customized owned files survive.
SPACE_HOME="$TMP/home with spaces"
mkdir -p "$SPACE_HOME/.cursor/skills/kineti-officehours"
printf 'personal conflict\n' > "$SPACE_HOME/.cursor/skills/kineti-officehours/SKILL.md"
out="$(HOME="$SPACE_HOME" bash "$HERE/setup.sh" --host cursor)"
[[ "$(cat "$SPACE_HOME/.cursor/skills/kineti-officehours/SKILL.md")" == "personal conflict" ]] || { echo "FAIL: unmanaged conflict was overwritten"; exit 1; }
echo "$out" | grep -q "conflict preserved:.*kineti-officehours" || { echo "FAIL: unmanaged conflict not reported"; exit 1; }

# A clean owned file is tracked, but later user edits are preserved on uninstall.
rm -rf "$SPACE_HOME/.cursor/skills/kineti-officehours"
HOME="$SPACE_HOME" bash "$HERE/setup.sh" --host cursor >/dev/null
printf 'user customization\n' > "$SPACE_HOME/.cursor/skills/kineti-officehours/SKILL.md"
out="$(HOME="$SPACE_HOME" bash "$HERE/setup.sh" --host cursor --uninstall)"
[[ "$(cat "$SPACE_HOME/.cursor/skills/kineti-officehours/SKILL.md")" == "user customization" ]] || { echo "FAIL: customized owned file was deleted"; exit 1; }
echo "$out" | grep -q "preserved modified skill" || { echo "FAIL: customized file was not reported"; exit 1; }
[[ -f "$SPACE_HOME/.kineti/install-manifest.tsv" ]] || { echo "FAIL: preserved customization lost its ownership record"; exit 1; }

# 8. Optional project guidance describes capabilities without claiming enforcement.
GUIDANCE_HOME="$TMP/guidance home"
GUIDANCE_ROOT="$TMP/guidance-project"
mkdir -p "$GUIDANCE_ROOT"
printf 'keep my rules\n' > "$GUIDANCE_ROOT/AGENTS.md"
HOME="$GUIDANCE_HOME" bash "$HERE/setup.sh" --install-root "$GUIDANCE_ROOT" >/dev/null
[[ "$(cat "$GUIDANCE_ROOT/AGENTS.md")" == "keep my rules" ]] || { echo "FAIL: existing project guidance was overwritten"; exit 1; }
grep -q "not a security enforcement hook" "$GUIDANCE_ROOT/CLAUDE.md" || { echo "FAIL: generated guidance overclaims enforcement"; exit 1; }
if grep -Eq "Stage 6|Do NOT skip stages|manual slash commands" "$GUIDANCE_ROOT"/{CLAUDE.md,CODEX.md,.cursor/rules/kineti.mdc}; then
  echo "FAIL: generated guidance contains a retired workflow restriction"; exit 1
fi

echo "PASS: installer smoke test (clean setup, preservation, repeat safety, uninstall, 7 host installs)"
