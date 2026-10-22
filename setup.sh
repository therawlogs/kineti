#!/usr/bin/env bash
# Kineti OS installer. Copies skills into every detected agent host.
# Safe to re-run: overwrites only kineti-* files it owns.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PREFIX="kineti-"
ONLY_HOST=""
INSTALL_ROOT=""
UNINSTALL=0
HOST_NAMES=()
HOST_DIRS=()
HOST_CONFS=()
MANIFEST=""

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then shasum -a 256 "$1" | awk '{print $1}'
  else echo "No SHA-256 utility found (install sha256sum or shasum)." >&2; return 1
  fi
}

manifest_hash() {
  local target="$1"
  [[ -f "$MANIFEST" ]] || return 0
  awk -F '\t' -v wanted="$target" '$1 == wanted { value=$2 } END { if (value != "") print value }' "$MANIFEST"
}

manifest_set() {
  local target="$1" digest="$2" tmp
  [[ "$target" != *$'\t'* && "$target" != *$'\n'* ]] || { echo "Unsupported tab or newline in install path: $target" >&2; return 1; }
  mkdir -p "$(dirname "$MANIFEST")"
  tmp="$MANIFEST.tmp.$$"
  if [[ -f "$MANIFEST" ]]; then awk -F '\t' -v wanted="$target" '$1 != wanted { print }' "$MANIFEST" > "$tmp"; else : > "$tmp"; fi
  printf '%s\t%s\n' "$target" "$digest" >> "$tmp"
  chmod 600 "$tmp"
  mv "$tmp" "$MANIFEST"
}

# Supported host names, read from hosts/*.conf so the list can never drift.
host_list() {
  local conf base out=""
  for conf in "$HERE"/hosts/*.conf; do
    [[ -e "$conf" ]] || continue
    base="$(basename "$conf" .conf)"
    out="${out:+$out|}$base"
  done
  echo "${out:-opencode|claude|gemini|codex|cursor}"
}

usage() {
  echo "Usage: ./setup.sh [--host $(host_list)] [--install-root [DIR]] [--uninstall]"
  exit 1
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --host) [[ $# -lt 2 ]] && { echo "Missing value for --host"; usage; }; ONLY_HOST="${2:-}"; shift 2 ;;
    --install-root) if [[ $# -ge 2 && "$2" != --* ]]; then INSTALL_ROOT="$2"; shift 2; else INSTALL_ROOT="$(pwd)"; shift 1; fi ;;
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
  local tmp dest digest current dir selected removed=0 preserved=0
  [[ -f "$MANIFEST" ]] || { echo "Kineti has no owned skill files recorded; nothing was removed."; return; }
  tmp="$MANIFEST.tmp.$$"
  : > "$tmp"
  while IFS=$'\t' read -r dest digest; do
    [[ -n "$dest" && -n "$digest" ]] || continue
    selected=0
    for dir in "${HOST_DIRS[@]}"; do
      case "$dest" in "$dir/${PREFIX}"*/SKILL.md) selected=1 ;; esac
      if [[ "$dir" == "$HOME/.gemini/config/plugins/kineti/skills" ]]; then
        case "$dest" in "$HOME/.gemini/config/plugins/kineti/plugin.json"|"$HOME/.gemini/config/plugins/kineti/rules/AGENTS.md") selected=1 ;; esac
      fi
    done
    if [[ $selected -eq 0 ]]; then printf '%s\t%s\n' "$dest" "$digest" >> "$tmp"; continue; fi
    # The manifest owns one file, never every path whose name starts kineti-.
    case "$dest" in "$HOME"/*/"${PREFIX}"*/SKILL.md|"$HOME/.gemini/config/plugins/kineti/plugin.json"|"$HOME/.gemini/config/plugins/kineti/rules/AGENTS.md") ;; *) echo "skipped unsafe manifest path: $dest"; preserved=$((preserved+1)); printf '%s\t%s\n' "$dest" "$digest" >> "$tmp"; continue ;; esac
    if [[ ! -f "$dest" ]]; then echo "already absent: $dest"; removed=$((removed+1)); continue; fi
    current="$(sha256_file "$dest")"
    if [[ "$current" != "$digest" ]]; then
      echo "preserved modified skill: $dest"
      preserved=$((preserved+1)); printf '%s\t%s\n' "$dest" "$digest" >> "$tmp"; continue
    fi
    rm -- "$dest"
    rmdir "$(dirname "$dest")" 2>/dev/null || true
    echo "removed owned skill: $dest"
    removed=$((removed+1))
  done < "$MANIFEST"
  if [[ -s "$tmp" ]]; then chmod 600 "$tmp"; mv "$tmp" "$MANIFEST"; else rm -f "$tmp" "$MANIFEST"; fi
  echo "Kineti uninstalled. Removed $removed owned skill files; preserved $preserved customized or unsafe entries."
  echo "Hook text blocks and the repository pointer are kept."
}

install_root_files() {
  local target="${1:-$(pwd)}"
  echo "Installing Kineti workflow guidance into: $target"

  # 1. CLAUDE.md
  if [[ ! -f "$target/CLAUDE.md" ]]; then
    cat << 'EOF' > "$target/CLAUDE.md"
# Kineti OS — Workflow Guidance for Claude Code

This repository uses Kineti workflow guidance. These instructions explain the process; they are not a security enforcement hook.

For a new project, ask the user to choose harness, gate, or both before creating project state. Continue only through relevant workflow steps; direct fixes and reviews may start where they fit. Keep the project goal and approved decisions visible. Before changes, register an undo step. Check spending before substantial work and save test evidence through Kineti. Preserve user files and configuration.
EOF
    echo "  + CLAUDE.md"
  fi

  # 2. AGENTS.md
  if [[ ! -f "$target/AGENTS.md" ]]; then
    cat << 'EOF' > "$target/AGENTS.md"
# Kineti OS — Universal Agent Standing Directives

These instructions provide workflow guidance. They do not enforce host behavior or intercept file writes.

## Rules
1. At new-project activation, ask the user to choose harness, gate, or both. Do not create state if the choice is canceled.
2. For a new project, use discovery, design and mock-data approval, approved data intake or no-data, then feature and architecture choices and an approved outcome contract.
3. Direct fixes, reviews, and cleanup may start at the relevant step. Do not require every step for a small task.
4. Preserve the locked goal and existing user work. Record approvals against the exact artifact version.
5. Save undo steps before changes, check spending, and record fresh test evidence.
EOF
    echo "  + AGENTS.md"
  fi

  # 3. .cursor/rules/kineti.mdc
  mkdir -p "$target/.cursor/rules"
  if [[ ! -f "$target/.cursor/rules/kineti.mdc" ]]; then
    cat << 'EOF' > "$target/.cursor/rules/kineti.mdc"
---
description: Kineti OS Workflow Guidance
globs: *
alwaysApply: true
---

# Kineti OS — Cursor Workflow Guidance

This file provides guidance. It is not an enforcement hook and does not intercept other tools.

For a new project, ask whether the user wants harness, gate, or both before creating state. Use the relevant workflow steps and allow direct fixes or reviews to start where they fit. Preserve existing files. Register undo steps, check spending, and save fresh test evidence before reporting results.
EOF
    echo "  + .cursor/rules/kineti.mdc"
  fi

  # 4. CODEX.md
  if [[ ! -f "$target/CODEX.md" ]]; then
    cat << 'EOF' > "$target/CODEX.md"
# Kineti OS — Workflow Guidance for OpenAI Codex

These instructions provide guidance; they do not enforce host behavior.

1. At new-project activation, ask the user to choose harness, gate, or both before creating state.
2. Use discovery, approved design and mock-data shape, approved data intake or no-data, architecture choices, and the outcome contract for new projects.
3. Direct fixes and reviews may start at the relevant step; do not require the whole workflow for a small task.
4. Preserve the locked goal and user files. Register undo steps, check spending, and save fresh test evidence.
EOF
    echo "  + CODEX.md"
  fi
}

install() {
  load_hosts
  if [[ ${#HOST_NAMES[@]} -eq 0 ]]; then
    echo "No matching host found. Installed hosts are auto-detected;"
    echo "force one with: ./setup.sh --host <$(host_list)>"
    exit 1
  fi
  local i name dir skill dest count total=0 skipped=0 expected current tmp
  for i in "${!HOST_NAMES[@]}"; do
    name="${HOST_NAMES[$i]}"; dir="${HOST_DIRS[$i]}"; count=0
    # Auto mode installs only into existing host folders. --host forces creation.
    if [[ ! -d "$dir" && -z "$ONLY_HOST" ]]; then
      echo "skipped: $name (not installed on this machine)"
      skipped=$((skipped+1))
      continue
    fi
    mkdir -p "$dir"
    if [[ "$name" == "antigravity" ]]; then
      local plugin_root="$(dirname "$dir")" pair_source pair_target
      for pair_source in "$HERE/hosts/antigravity-plugin.json" "$HERE/hooks/antigravity.txt"; do
        if [[ "$pair_source" == *.json ]]; then pair_target="$plugin_root/plugin.json"; else pair_target="$plugin_root/rules/AGENTS.md"; fi
        expected="$(manifest_hash "$pair_target")"
        if [[ -e "$pair_target" ]]; then
          current="$(sha256_file "$pair_target")"
          if [[ -z "$expected" || "$current" != "$expected" ]]; then
            if ! cmp -s "$pair_source" "$pair_target"; then echo "conflict preserved: $pair_target"; skipped=$((skipped+1)); fi
            continue
          fi
        fi
        mkdir -p "$(dirname "$pair_target")"
        cp "$pair_source" "$pair_target.tmp.$$"; chmod 600 "$pair_target.tmp.$$"; mv "$pair_target.tmp.$$" "$pair_target"
        manifest_set "$pair_target" "$(sha256_file "$pair_target")"
      done
    fi
    for skill in "$HERE"/skills/*/; do
      skill="$(basename "$skill")"
      dest="$dir/${PREFIX}${skill}"
      mkdir -p "$dest"
      local source="$HERE/skills/$skill/SKILL.md" target="$dest/SKILL.md"
      expected="$(manifest_hash "$target")"
      if [[ -e "$target" ]]; then
        current="$(sha256_file "$target")"
        if [[ -z "$expected" ]]; then
          if cmp -s "$source" "$target"; then
            echo "unchanged unmanaged skill: $target"
          else
            echo "conflict preserved: $target (not owned by this install)"
            skipped=$((skipped+1))
          fi
          continue
        fi
        if [[ "$current" != "$expected" ]]; then
          echo "conflict preserved: $target (customized since Kineti installed it)"
          skipped=$((skipped+1))
          continue
        fi
      fi
      tmp="$target.tmp.$$"
      cp "$source" "$tmp"
      chmod 600 "$tmp"
      mv "$tmp" "$target"
      manifest_set "$target" "$(sha256_file "$target")"
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
  if [[ -f "$HOME/.kineti/repo" ]]; then
    cp "$HOME/.kineti/repo" "$HOME/.kineti/repo.bak" 2>/dev/null || true
  fi
  printf '%s\n' "$HERE" > "$HOME/.kineti/repo"
  echo "Repository pointer written: $HOME/.kineti/repo -> $HERE"
  if [[ $total -eq 0 ]]; then
    echo "No agent host folders were found on this machine."
    echo "Create one (for example install opencode) or force a target:"
    echo "  ./setup.sh --host <$(host_list)>"
  fi
  if [[ -n "$INSTALL_ROOT" ]]; then
    echo ""
    install_root_files "$INSTALL_ROOT"
  fi
}

mkdir -p "$HOME/.kineti"
MANIFEST="$HOME/.kineti/install-manifest.tsv"

if [[ $UNINSTALL -eq 1 ]]; then uninstall; else install; fi
