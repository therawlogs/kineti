# Project Roadmap

This document lists future planned improvements and completed capabilities of the Kineti platform.

---

## Production Roadmap

| Capability | Purpose | Target Milestone |
|---|---|---|
| **Native macOS Menu Bar Companion** | Lightweight background companion tray item displaying spend, active goals, and one-click undo | v4.1 |
| **Distributed Peer Mesh Protocol** | Agent-to-agent negotiation protocol emitting minimum-necessary artifacts (e.g. disjoint availability windows rather than whole calendars) | v4.2 |
| **Hardware Enclave Signing (Secure Enclave / TPM)** | Hardware-backed Ed25519 signing for high-consequence financial and contract execution tokens | v4.3 |
| **Encrypted Multi-Device Persona Sync** | End-to-end encrypted epistemic memory synchronization across multiple authorized consumer devices | v4.4 |

---

## Completed in Version 4.0 (Current Production Release)

- **The 360º Human Model**:
  - Multi-Scope Context Isolation (`Global`, `Domain`, `Relationship`) preventing cross-scope data leakage.
  - Epistemic Certainty Tiers (`DirectlyKnown` > `ObservedPattern` > `Inferred`) ensuring inferences never override explicit user statements.
  - Rule-Exception-Allergy Hierarchies (`BaselineRule` $\to$ `PermittedException` $\to$ `Preference` $\to$ `SafetyCeiling`).
  - Verbatim Root Goal Invariance & Anti-Drift Engine preventing the multi-step "telephone game".
  - Friction Triage Ladder delivering "Clean No over Dirty Yes" with zero sunk-cost fallacy.
  - Commitment-Time Verification re-checking perishable facts at the exact millisecond of commit.
  - Asymmetric Gap-Filling (cheap/reversible filled with disclosed assumptions; expensive/irreversible held for user approval).
  - Outbound Reputation Gating and Ingress Threat Defense.
- **Pure Native Rust Nervous System (`core-native/`)**:
  - 100% Safe Rust (`#![forbid(unsafe_code)]`) across all crates.
  - Lock-free atomic state snapshots with EBR (<100µs latency under 80 concurrent writer threads).
  - Universal 20-Entity Provenance Kernel with RFC 8785 JSON canonicalization and BLAKE3/SHA-256 digests.
  - Sub-50ms 3-way graph commit gate enforcing topological rank acyclicity, HLC temporal monotonicity, and Merkle DAG integrity.
  - Fast-path spend circuit breaker ($50.00 ceiling, $47.50 trip at 95%).
  - Sensory reflex triage ($p99 < 1.0\,\text{ms}$) with zero-token emoji reactions.
  - Protocolized Connectors with single-use SHA-256 payload authorization tokens.
- **Native CLI & MCP Integration**:
  - `kineti-cli`: Interactive CLI (`chat`, `anti-drift`, `epistemic`, `status`, `test-all`).
  - `kineti-mcp`: Integrated MCP server exposing epistemic persona evaluation to IDEs.
- **Test Suite & Cryptographic Evidence**:
  - 170+ native Rust tests passing with 0 warnings.
  - 83 TypeScript governance tests passing.
  - Cryptographic evidence verified fresh (`360-human-model-resilience`).

---

## Completed in Version 3.1

- **Multi-Repository Fleet Governance**: Multi-repo grid (`Fleet View`), repository switcher dropdown, and unified spending analytics.
- **Apple HIG Web Design**: Full adoption of Apple Human Interface Guidelines, material library (vibrancy, blurs, hairlines), continuous squircles, and removal of mock window chrome.
- **Settings & Integrations Drawer**: Slide-out panel for GitHub App connection, one-click agent IDE auto-latching (Cursor, Claude Code, Antigravity, Codex), and repository owner budget allocation.
- **Agent Swarm Coordination & Cryptographic Identity**: Ed25519 keypairs per agent, anti-drift envelopes, dual-signed Outcome Verification Tickets (OVT), and role-gated approvals.

---

## Completed in Version 3.0

- Direct task entry (`bugfix`, `feature`, `refactor`, `audit`) and full project workflow
- Spend limit protection ($50 USD max)
- Step-by-step undo tracking via SAGA LIFO stack
- Test verification with file hashes
- Web companion dashboard with plain English summaries
- Automated weekly integrity checks
