#!/usr/bin/env bash
# Kineti OS installer. Copies skills into every detected agent host.
# Safe to re-run: overwrites only kineti-* files it owns.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="kineti-"
ONLY_HOST=""
UNINSTALL=0

usage() {
  echo "Usage: ./setup.sh [--host opencode|claude|gemini|codex] [--uninstall]"
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host) ONLY_HOST="${2:-}"; shift 2 ;;
    --uninstall) UNINSTALL=1; shift ;;
    -h|--help) usage ;;
    *) echo "Unknown option: $1"; usage ;;
  esac
done

read_conf() { # $1 = conf file, $2 = key -> prints value (quotes stripped)
  grep -E "^$2=" "$1" | head -1 | cut -d= -f2- | tr -d '"'
}

expand_path() { # expand $HOME and ${CODEX_HOME:-...} without executing arbitrary code
  local p="$1"
  local codex_base="${CODEX_HOME:-$HOME/.codex}"
  p="${p//\$\{CODEX_HOME:-\$HOME\/.codex\}/$codex_base}"
  p="${p//\$HOME/$HOME}"
  echo "$p"
}

host_dirs() { # $1 = conf -> prints existing skills dirs (primary + fallback)
  local conf="$1" primary fallback
  primary="$(expand_path "$(read_conf "$conf" skills_dir_primary)")"
  fallback="$(expand_path "$(read_conf "$conf" skills_dir_fallback)")"
  [[ -n "$fallback" && -d "$fallback" && ! -d "$primary" ]] && primary="$fallback"
  echo "$primary"
}

load_hosts() {
  local conf name dir
  for conf in "$HERE"/hosts/*.conf; do
    name="$(read_conf "$conf" name)"
    [[ -n "$ONLY_HOST" && "$name" != "$ONLY_HOST" ]] && continue
    dir="$(host_dirs "$conf")"
    HOST_NAMES+=("$name"); HOST_DIRS+=("$dir"); HOST_CONFS+=("$conf")
  done
}

uninstall() {
  load_hosts
  local i dir removed=0
  for i in "${!HOST_NAMES[@]}"; do
    dir="${HOST_DIRS[$i]}"
    [[ -d "$dir" ]] || continue
    for d in "$dir/${PREFIX}"*; do
      [[ -e "$d" ]] || continue
      rm -rf "$d"; removed=$((removed+1))
      echo "removed: $d"
    done
  done
  echo "Kineti uninstalled. Removed $removed skill folders."
  echo "Note: hook text blocks in host settings are comments; remove them by hand if desired."
}

install() {
  load_hosts
  if [[ ${#HOST_NAMES[@]} -eq 0 ]]; then
    echo "No matching host found. Installed hosts are auto-detected;"
    echo "force one with: ./setup.sh --host <opencode|claude|gemini|codex>"
    exit 1
  fi
  local i name dir skill dest count total=0 skipped=0
  for i in "${!HOST_NAMES[@]}"; do
    name="${HOST_NAMES[$i]}"; dir="${HOST_DIRS[$i]}"; count=0
    # Auto mode installs only into existing host folders. --host forces creation.
    if [[ ! -d "$dir" && -z "$ONLY_HOST" ]]; then
      echo "skipped: $name (not installed on this machine)"
      skipped=$((skipped+1))
      continue
    fi
    mkdir -p "$dir"
    for skill in "$HERE"/skills/*/; do
      skill="$(basename "$skill")"
      dest="$dir/${PREFIX}${skill}"
      mkdir -p "$dest"
      cp "$HERE/skills/$skill/SKILL.md" "$dest/SKILL.md"
      count=$((count+1))
    done
    echo "installed: $count skills -> $dir ($name)"
    total=$((total+count))
  done
  echo ""
  echo "== Hook blocks (paste into each host's instructions file) =="
  for i in "${!HOST_NAMES[@]}"; do
    name="${HOST_NAMES[$i]}"
    [[ -d "${HOST_DIRS[$i]}" ]] || continue
    echo ""
    cat "$HERE/hooks/$name.txt"
  done
  local installed_hosts=0
  for i in "${!HOST_DIRS[@]}"; do [[ -d "${HOST_DIRS[$i]}" ]] && installed_hosts=$((installed_hosts+1)); done
  echo ""
  echo "Done. $total skill copies across $installed_hosts host(s). Re-run any time."
  printf '%s\n' "$HERE" > "$HOME/.kineti/repo"
  echo "Repository pointer written: $HOME/.kineti/repo -> $HERE"
  if [[ $total -eq 0 ]]; then
    echo "No agent host folders were found on this machine."
    echo "Create one (for example install opencode) or force a target:"
    echo "  ./setup.sh --host <opencode|claude|gemini|codex>"
  fi
}

mkdir -p "$HOME/.kineti"

if [[ $UNINSTALL -eq 1 ]]; then uninstall; else install; fi
