#!/usr/bin/env bash
# Context-cost audit: words per skill. Skills load into agent context.
set -euo pipefail
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
total=0; count=0
printf "%-18s %6s\n" SKILL WORDS
for f in "$HERE"/skills/*/SKILL.md; do
  name=$(basename "$(dirname "$f")")
  w=$(wc -w < "$f" | tr -d ' ')
  total=$((total+w)); count=$((count+1))
  flag=""
  [ "$w" -gt 1200 ] && flag="  <-- TRIM"
  printf "%-18s %6s%s\n" "$name" "$w" "$flag"
done
echo "----------------------"
printf "%-18s %6s\n" "TOTAL ($count)" "$total"
