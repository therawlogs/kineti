#!/usr/bin/env bash
# Kineti weekly memory job.
# Scans KINETI_PROJECTS (space-separated) for .kineti/journal.jsonl and runs
# the four maintenance commands on each. Safe on empty projects: skipped.
#
# Cron example (Monday 09:00):
#   0 9 * * 1 KINETI_PROJECTS="$HOME/Documents/Products" $HOME/Documents/Products/Kineti/scripts/weekly.sh >> $HOME/.kineti/weekly.log 2>&1
set -euo pipefail

REPO_FILE="$HOME/.kineti/repo"
if [[ ! -f "$REPO_FILE" ]]; then
  echo "kineti: error: repository pointer '$REPO_FILE' not found. Run ./setup.sh first." >&2
  exit 1
fi
KIN="$(head -n 1 "$REPO_FILE" | tr -d '\r\n')"
if [[ ! -d "$KIN" ]]; then
  echo "kineti: error: repository directory '$KIN' does not exist." >&2
  exit 1
fi

BUN="${KINETI_BUN:-$HOME/.bun/bin/bun}"
command -v "$BUN" >/dev/null 2>&1 || BUN="$(command -v bun)"

if [[ -n "${KINETI_PROJECTS:-}" ]]; then
  IFS=':' read -r -a project_list <<< "$KINETI_PROJECTS"
else
  project_list=("$PWD")
fi

echo "=== kineti weekly job $(date '+%Y-%m-%d %H:%M') ==="
for p in "${project_list[@]}"; do
  [[ -f "$p/.kineti/journal.jsonl" ]] || continue
  echo "--- $p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" sweep        --dir "$p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" verify-chain --dir "$p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" time-order   --dir "$p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" promote      --dir "$p" || true
done
echo "=== done ==="
