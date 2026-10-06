# Kineti OS — Documentation Index

Welcome to the technical documentation for **Kineti OS**. For current v0.4.0 behavior, start with the README and changelog. Historical plans and research notes describe goals, not guarantees.

---
## 1. Foundational Research Series (historical)

The papers were removed from the tree and will be re-added when ready. The figures below are design targets for the evaluation program (unmeasured in production):

**Evaluation Program Design Targets:**
- **Agents' Last Exam (ALE)**: 76.4% overall pass rate target, 68.2% on >10-step tasks.
- **SWE-bench Target**: 4.2 min MTTR target ($9.1\times$ reduction vs baseline).
- **Directional Normalized Trust-Weighted Impact (DNTI)**: Verification metric framework with semantic entropy attenuation ($\tau = 0.15$).
- **Cost Per Verified Outcome ($/Outcome)**: $0.31 per verified resolution target.

---

## Release notes
- [**CHANGELOG.md**](../CHANGELOG.md) — Current release contents and limits.

## 2. Core Architecture & Specifications
- [**PLAN.md**](./PLAN.md) — Historical architecture proposal. Its implementation targets are not a description of v0.4.0 behavior.
- [**ARCHITECTURE.md**](./ARCHITECTURE.md) — Rust core internals moved off the product README (hash-chained ledger, commit gate, snapshots, connectors).
- [**SECURITY_REPORT.md**](./SECURITY_REPORT.md) — Historical v3.1 security review. Read the dated v0.4.0 section for the current release review.

---

## 3. Multi-Agent & Fleet Coordination
- [**SWARM_COORDINATION_AND_IDENTITY.md**](./SWARM_COORDINATION_AND_IDENTITY.md) — Limits of the in-memory swarm demo and native OVT signature library.
- [**MULTI_REPO_FLEET_AND_INTEGRATIONS.md**](./MULTI_REPO_FLEET_AND_INTEGRATIONS.md) — Multi-repository fleet governance, configuration, and developer environment sync.
- [**AGENT_IDENTIFICATION_AND_WEBBOTAUTH.md**](./AGENT_IDENTIFICATION_AND_WEBBOTAUTH.md) — Automated agent identity, RFC 9421 HTTP Message Signatures, and the IETF Web Bot Auth draft standard.

---

## 4. Guides & Tutorials
- [**TUTORIAL-first-run.md**](./TUTORIAL-first-run.md) — Getting started with Kineti OS and recording a local test receipt.
- [**HOWTO-daily-loop.md**](./HOWTO-daily-loop.md) — Daily developer workflow: stage gating, test evidence recording, and spend tracking.
- [**UNDO_LIMITATIONS.md**](./UNDO_LIMITATIONS.md) — What undo can and cannot reverse (SAGA rollback boundaries).
- [**FINGERPRINT_LIMITATIONS.md**](./FINGERPRINT_LIMITATIONS.md) — What the test receipt fingerprint can and cannot see (false alarms and blind spots).

---

## 5. Research notes, not part of the CLI
- [**research/**](./research/) — Research notes, not part of the CLI.
