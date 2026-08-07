#!/usr/bin/env bash
# ==============================================================================
# Kineti OS (v0.1) Master Multi-Platform Deployment Engine
# Deploys Kineti OS from /Users/praveen/Documents/Kineti_OS to ALL AI platforms:
#   - Gemini / Antigravity (~/.gemini/config/)
#   - Claude Code (~/.claude/)
#   - OpenAI Codex (~/.codex/)
#   - xAI Grok (~/.grok/)
#   - OpenCode (~/.opencode/)
#   - Cursor IDE (~/.cursor/)
# ==============================================================================

set -e

KINETI_DIR="/Users/praveen/Documents/Kineti_OS"

echo "================================================================="
echo "🚀 Deploying Kineti OS (v0.1) Across All AI Platforms & Agents"
echo "================================================================="

# 1. Deploy to Gemini / Antigravity
echo "[1/6] Deploying to Gemini / Antigravity..."
mkdir -p ~/.gemini/config/scripts ~/.gemini/config/skills
cp "$KINETI_DIR"/ETHOS.md ~/.gemini/config/
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.gemini/config/
cp "$KINETI_DIR"/master_fde_founder_specification.md ~/.gemini/config/
cp "$KINETI_DIR"/founder_sprint_framework.md ~/.gemini/config/
cp "$KINETI_DIR"/sdlc_hybrid_guide.md ~/.gemini/config/
cp "$KINETI_DIR"/README.md ~/.gemini/config/
cp -r "$KINETI_DIR"/scripts/* ~/.gemini/config/scripts/
cp -r "$KINETI_DIR"/skills/* ~/.gemini/config/skills/
echo "  └─ Done: Gemini (~/.gemini/config/)"

# 2. Deploy to Claude Code
echo "[2/6] Deploying to Claude Code CLI..."
mkdir -p ~/.claude/skills
cp "$KINETI_DIR"/ETHOS.md ~/.claude/CLAUDE.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.claude/
cp -r "$KINETI_DIR"/skills/* ~/.claude/skills/
echo "  └─ Done: Claude Code (~/.claude/)"

# 3. Deploy to OpenAI Codex
echo "[3/6] Deploying to OpenAI Codex CLI..."
mkdir -p ~/.codex/skills
cp "$KINETI_DIR"/ETHOS.md ~/.codex/CODEX.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.codex/
cp -r "$KINETI_DIR"/skills/* ~/.codex/skills/
echo "  └─ Done: OpenAI Codex (~/.codex/)"

# 4. Deploy to xAI Grok
echo "[4/6] Deploying to xAI Grok..."
mkdir -p ~/.grok/skills
cp "$KINETI_DIR"/ETHOS.md ~/.grok/GROK.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.grok/
cp -r "$KINETI_DIR"/skills/* ~/.grok/skills/
echo "  └─ Done: xAI Grok (~/.grok/)"

# 5. Deploy to OpenCode
echo "[5/6] Deploying to OpenCode..."
mkdir -p ~/.opencode/skills
cp "$KINETI_DIR"/ETHOS.md ~/.opencode/OPENCODE.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.opencode/
cp -r "$KINETI_DIR"/skills/* ~/.opencode/skills/
echo "  └─ Done: OpenCode (~/.opencode/)"

# 6. Deploy to Cursor IDE
echo "[6/6] Deploying to Cursor IDE..."
mkdir -p ~/.cursor/rules
cp "$KINETI_DIR"/ETHOS.md ~/.cursor/rules/ETHOS.md
cp "$KINETI_DIR"/guide_to_kinetios.md ~/.cursor/rules/
echo "  └─ Done: Cursor IDE (~/.cursor/rules/)"

echo ""
echo "================================================================="
echo "✅ Kineti OS (v0.1) Successfully Deployed Across ALL 6 Platforms!"
echo "================================================================="
