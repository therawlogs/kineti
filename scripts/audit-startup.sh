#!/bin/sh
# Startup-I/O audit: prove the light command path performs ZERO file opens
# before dispatch. Linux uses strace; macOS lacks a non-sudo tracer, so it
# prints SKIP (run `sudo fs_usage -w -f filesys <bin> --version` manually).
# Usage: scripts/audit-startup.sh [path-to-kineti]
set -eu

BIN="${1:-target/release/kineti}"
[ -x "$BIN" ] || { echo "audit: $BIN not found/executable" >&2; exit 2; }

case "$(uname -s)" in
  Linux)
    if ! command -v strace >/dev/null 2>&1; then
        echo "⚠ audit: strace not installed — apt install strace"; exit 3
    fi
    # trace the whole process; every open*/stat* on the version path is a hit
    out=$(strace -f -qq -e trace=open,openat,openat2,stat,statx,lstat,newfstatat \
          "$BIN" --version 2>&1 >/dev/null) || { echo "⛔ strace failed"; exit 1; }
    # allow nothing: no config reads, no /proc scans, nothing.
    hits=$(printf '%s\n' "$out" | grep -cE '^\s*(open|openat|openat2|stat|statx|lstat|newfstatat)\(' || true)
    echo "startup file-system calls on '--version' path: $hits"
    if [ "$hits" -eq 0 ]; then
        echo "✔ zero startup I/O confirmed"
    else
        printf '%s\n' "$out" | grep -E '^\s*(open|openat|openat2|stat|statx|lstat|newfstatat)\(' | head -20
        echo "⛔ FAIL: unexpected startup I/O (target: zero)"
        exit 1
    fi
    ;;
  Darwin)
    echo "SKIP: macOS has no non-sudo syscall tracer for this audit."
    echo "Manual check:  sudo fs_usage -w -f filesys '$BIN' --version"
    echo "(Design guarantee: no work happens before clap/fast-path dispatch;"
    echo " config and .kineti reads are strictly per-command.)"
    ;;
  *)
    echo "SKIP: unsupported platform $(uname -s)"
    ;;
esac
