# Kineti OS — Documentation Index

Welcome to the technical documentation for **Kineti OS**.

---
## 1. Foundational Research Series (historical, removed from tree)

The papers were removed from the tree and will be re-added when ready. The figures below are design targets for the evaluation program (unmeasured in production):

**Evaluation Program Design Targets:**
- **Agents' Last Exam (ALE)**: 76.4% overall pass rate target, 68.2% on >10-step tasks.
- **SWE-bench Target**: 4.2 min MTTR target ($9.1\times$ reduction vs baseline).
- **Directional Normalized Trust-Weighted Impact (DNTI)**: Verification metric framework with semantic entropy attenuation ($\tau = 0.15$).
- **Cost Per Verified Outcome ($/Outcome)**: $0.31 per verified resolution target.

---

## 2. Core Architecture & Specifications
- [**PLAN.md**](./PLAN.md) — Comprehensive technical architecture specification: wait-free EBR snapshots, 20-entity provenance kernel, HLC temporal monotonicity, Merkle DAG commit gate, and SAGA transactional reversibility.
- [**SECURITY_REPORT.md**](./SECURITY_REPORT.md) — Vulnerability assessment, origin validation, CSRF/CSWSH protection, and secret scan results.

---

## 3. Multi-Agent & Fleet Coordination
- [**SWARM_COORDINATION_AND_IDENTITY.md**](./SWARM_COORDINATION_AND_IDENTITY.md) — Multi-agent swarm topology, cryptographic Ed25519 identity, and Outcome Verification Tickets (OVT).
- [**MULTI_REPO_FLEET_AND_INTEGRATIONS.md**](./MULTI_REPO_FLEET_AND_INTEGRATIONS.md) — Multi-repository fleet governance, configuration, and developer environment sync.

---

## 4. Guides & Tutorials
- [**TUTORIAL-first-run.md**](./TUTORIAL-first-run.md) — Getting started with Kineti OS from installation to first verified task execution.
- [**HOWTO-daily-loop.md**](./HOWTO-daily-loop.md) — Daily developer workflow: stage gating, test evidence recording, and spend tracking.

