#!/usr/bin/env bash
# Deploy Kineti OS (v0.1) to Gemini / Antigravity (~/.gemini/config/)
set -e
KINETI_DIR="/Users/praveen/Documents/Kineti_OS"
mkdir -p ~/.gemini/config/scripts ~/.gemini/config/skills
cp "$KINETI_DIR"/ETHOS.md ~/.gemini/config/
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.gemini/config/
cp "$KINETI_DIR"/master_fde_founder_specification.md ~/.gemini/config/
cp "$KINETI_DIR"/founder_sprint_framework.md ~/.gemini/config/
cp "$KINETI_DIR"/sdlc_hybrid_guide.md ~/.gemini/config/
cp "$KINETI_DIR"/README.md ~/.gemini/config/
cp -r "$KINETI_DIR"/scripts/* ~/.gemini/config/scripts/
cp -r "$KINETI_DIR"/skills/* ~/.gemini/config/skills/
echo "✅ Kineti OS successfully deployed to Gemini / Antigravity (~/.gemini/config/)"
