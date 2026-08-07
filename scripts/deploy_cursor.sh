#!/usr/bin/env bash
# Deploy Kineti OS (v0.1) to Cursor IDE (~/.cursor/)
set -e
KINETI_DIR="/Users/praveen/Documents/Kineti_OS"
mkdir -p ~/.cursor/rules
cp "$KINETI_DIR"/ETHOS.md ~/.cursor/rules/ETHOS.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.cursor/rules/
echo "✅ Kineti OS successfully deployed to Cursor IDE (~/.cursor/rules/)"
