#!/usr/bin/env bash
# Deploy Kineti OS (v0.1) to OpenAI Codex CLI (~/.codex/)
set -e
KINETI_DIR="/Users/praveen/Documents/Kineti_OS"
mkdir -p ~/.codex/skills
cp "$KINETI_DIR"/ETHOS.md ~/.codex/CODEX.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.codex/
cp -r "$KINETI_DIR"/skills/* ~/.codex/skills/
echo "✅ Kineti OS successfully deployed to OpenAI Codex (~/.codex/)"
