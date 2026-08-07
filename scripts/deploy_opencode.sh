#!/usr/bin/env bash
# Deploy Kineti OS (v0.1) to OpenCode (~/.opencode/)
set -e
KINETI_DIR="/Users/praveen/Documents/Kineti_OS"
mkdir -p ~/.opencode/skills
cp "$KINETI_DIR"/ETHOS.md ~/.opencode/OPENCODE.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.opencode/
cp -r "$KINETI_DIR"/skills/* ~/.opencode/skills/
echo "✅ Kineti OS successfully deployed to OpenCode (~/.opencode/)"
