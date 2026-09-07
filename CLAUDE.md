# Kineti OS — Autonomous Runtime Directives for Claude Code

This repository is governed by **Kineti OS v3**. You operate under standing governance rules without requiring the user to manually type slash commands.

## Automatic Operational Protocol
1. **Plain English & Numbered Choices**: Communicate in clear, jargon-free plain English. Present decisions as numbered options (1, 2, 3) for 1-click selection.
2. **Sequential State Awareness**: On every task, inspect `.kineti/state.json` to identify the active pipeline stage (1 to 13). Do NOT skip stages.
3. **Hard Gate: No Code Before Spec Approval**:
   - Stages 1–6 are discovery, architecture, and specifications.
   - You MUST NOT write production code or create application source files until Stage 6 (Spec) is explicitly approved by the human operator.
4. **Automatic Evidence Recording**:
   - When running tests or verifications, record cryptographic proofs using `bun bin/kineti-evidence.ts run --label <label> -- <command>` or via the Kineti MCP server.
5. **Spend Circuit Breaker**:
   - Continuously monitor spend. If task spend exceeds $50.00 USD, pause execution immediately and report to the operator.
6. **Undo & Saga Safety**:
   - Register reverse rollback commands before major file mutations via `bun bin/kineti-saga.ts push "<undo-command>"`.

## Available Kineti Runtime Commands (Run transparently in background)
- State & Gates: `bun bin/kineti-state.ts status` / `bun bin/kineti-state.ts set gate.<name> pass`
- Spend Accounting: `bun bin/kineti-spend.ts status` / `bun bin/kineti-spend.ts log`
- Evidence Proofs: `bun bin/kineti-evidence.ts check --label <name>`
- Visual Companion: Active at `http://127.0.0.1:8788` (`bun run companion`)
- CI Verification: `bun run ci`
