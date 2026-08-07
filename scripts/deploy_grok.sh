#!/usr/bin/env bash
# Deploy Kineti OS (v0.1) to xAI Grok (~/.grok/)
set -e
KINETI_DIR="/Users/praveen/Documents/Kineti_OS"
mkdir -p ~/.grok/skills
cp "$KINETI_DIR"/ETHOS.md ~/.grok/GROK.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.grok/
cp -r "$KINETI_DIR"/skills/* ~/.grok/skills/
echo "✅ Kineti OS successfully deployed to xAI Grok (~/.grok/)"
