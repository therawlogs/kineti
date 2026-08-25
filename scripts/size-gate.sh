#!/bin/sh
# Size gate (ETHOS budget): release binary must stay under 10 MB.
# Fails the build at >= 10 MB, warns at >= 8 MB.
set -eu

bin="${1:-target/release/kineti}"
LIMIT_MB=10
WARN_MB=8

if [ ! -f "$bin" ]; then
    echo "size-gate: $bin not found — build release first" >&2
    exit 1
fi

bytes=$(wc -c < "$bin" | tr -d ' ')
limit=$((LIMIT_MB * 1024 * 1024))
warn=$((WARN_MB * 1024 * 1024))

printf 'size-gate: %s bytes (%s MiB / %s MiB budget)\n' \
    "$bytes" "$((bytes / 1048576))" "$LIMIT_MB"

if [ "$bytes" -ge "$limit" ]; then
    echo "⛔ size-gate FAILED: binary exceeds ${LIMIT_MB} MB budget" >&2
    exit 1
fi
if [ "$bytes" -ge "$warn" ]; then
    echo "⚠ size-gate: approaching ${LIMIT_MB} MB budget"
fi
echo "✔ size-gate passed"
