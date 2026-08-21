#!/usr/bin/env bash
# Kineti weekly memory job.
# Scans KINETI_PROJECTS (space-separated) for .kineti/journal.jsonl and runs
# the four maintenance commands on each. Safe on empty projects: skipped.
#
# Cron example (Monday 09:00):
#   0 9 * * 1 KINETI_PROJECTS="$HOME/Documents/Products" $HOME/Documents/Products/Kineti/scripts/weekly.sh >> $HOME/.kineti/weekly.log 2>&1
set -euo pipefail

KIN="$(cat "$HOME/.kineti/repo")"
BUN="${KINETI_BUN:-$HOME/.bun/bin/bun}"
command -v "$BUN" >/dev/null 2>&1 || BUN="$(command -v bun)"
PROJECTS="${KINETI_PROJECTS:-$PWD}"

echo "=== kineti weekly job $(date '+%Y-%m-%d %H:%M') ==="
for p in $PROJECTS; do
  [[ -f "$p/.kineti/journal.jsonl" ]] || continue
  echo "--- $p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" sweep        --dir "$p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" verify-chain --dir "$p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" time-order   --dir "$p"
  "$BUN" "$KIN/bin/kineti-memory-job.ts" promote      --dir "$p" || true
done
echo "=== done ==="
