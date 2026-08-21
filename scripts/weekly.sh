#!/usr/bin/env bash
# Kineti weekly memory job. Point PROJECTS at space-separated project roots.
set -euo pipefail
KIN="$(cat "$HOME/.kineti/repo")"
PROJECTS="${KINETI_PROJECTS:-$PWD}"
for p in $PROJECTS; do
  [[ -f "$p/.kineti/journal.jsonl" ]] || continue
  bun "$KIN/bin/kineti-memory-job.ts" sweep        --dir "$p"
  bun "$KIN/bin/kineti-memory-job.ts" verify-chain --dir "$p"
  bun "$KIN/bin/kineti-memory-job.ts" time-order   --dir "$p"
  bun "$KIN/bin/kineti-memory-job.ts" promote      --dir "$p" || true
done
