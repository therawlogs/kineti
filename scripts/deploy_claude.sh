#!/usr/bin/env bash
# Deploy Kineti OS (v0.1) to Claude Code CLI (~/.claude/)
set -e
KINETI_DIR="/Users/praveen/Documents/Kineti_OS"
mkdir -p ~/.claude/skills
cp "$KINETI_DIR"/ETHOS.md ~/.claude/CLAUDE.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.claude/
cp -r "$KINETI_DIR"/skills/* ~/.claude/skills/
echo "✅ Kineti OS successfully deployed to Claude Code (~/.claude/)"
