# Project Roadmap

This document lists future planned improvements and completed capabilities.

## Planned Updates

| Feature | Requirement | First Step |
|---|---|---|
| **Similarity Search** | Requires an embedding API key. Keyword search works today. | Configure an embedding provider in `gbrain`. |
| **Second Opinion Skill** | Requires a second command-line tool installed (such as `codex` or `gemini`). | Install a second CLI and test `kineti-second-opinion`. |
| **Gemini Integration Test** | Confirm tool detection in Gemini. | Run an interactive session in Gemini and verify memory commands. |

## Future Considerations

| Improvement | Purpose | Next Step |
|---|---|---|
| **Automated Output Quality Evaluation** | Evaluate generated responses against verified human examples. | Collect 50 example pairs in `.kineti/golden.jsonl` and create `bin/kineti-eval.ts`. |
| **Native Graph Storage** | Store cause-and-effect records directly in a graph database. | Add link scoring to `kineti-memory-job` before changing databases. |
| **Team Mode** | Allow multiple team members to share memory securely. | Add workspace permissions to memory storage. |

## Completed in Version 3.1

- **Multi-Repository Fleet Governance**: Multi-repo grid (`Fleet View`), repository switcher dropdown, and unified spending analytics.
- **Apple HIG Web Design**: Full adoption of Apple Human Interface Guidelines, material library (vibrancy, blurs, hairlines), continuous squircles, and removal of mock window chrome.
- **Settings & Integrations Drawer**: Slide-out panel for GitHub App connection, one-click agent IDE auto-latching (Cursor, Claude Code, Antigravity, Codex), and repository owner budget allocation.
- **Agent Swarm Coordination & Cryptographic Identity**: Ed25519 keypairs per agent, anti-drift envelopes, dual-signed Outcome Verification Tickets (OVT), and role-gated approvals.

## Completed in Version 3.0

- Direct task entry (`bugfix`, `feature`, `refactor`, `audit`) and full project workflow
- Spend limit protection ($50 USD max)
- Step-by-step undo tracking
- Test verification with file hashes
- Web companion dashboard with plain English summaries
- Automated weekly integrity checks
