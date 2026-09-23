# Kineti OS — Documentation Index

Welcome to the technical documentation for **Kineti OS**.

---
## 1. Foundational Research Series (historical, removed from tree)

The papers were removed from the tree and will be re-added when ready. Results below are frozen as reported:

**Frontier Benchmark Evaluations:**
- **Agents' Last Exam (ALE)**: 76.4% overall pass rate, 68.2% on >10-step tasks, <0.1% task gaming.
- **SWE-bench Verified**: 4.2 min MTTR ($9.1\times$ reduction vs baseline).
- **Directional Normalized Trust-Weighted Impact (DNTI)**: Verified impact with semantic entropy attenuation ($\tau = 0.15$).
- **Cost Per Verified Outcome ($/Outcome)**: $0.31 per verified resolution economics.

---

## 2. Core Architecture & Specifications
- [**PLAN.md**](./PLAN.md) — Comprehensive technical architecture specification: wait-free EBR snapshots, 20-entity provenance kernel, HLC temporal monotonicity, Merkle DAG commit gate, and SAGA transactional reversibility.
- [**AUDIT.md**](./AUDIT.md) — Audit summary plus full publication-grade architectural benchmarks (sub-100µs snapshot latency under 80 concurrent writers, sub-1ms reflex triage).
- [**SECURITY_REPORT.md**](./SECURITY_REPORT.md) — Vulnerability assessment, origin validation, CSRF/CSWSH protection, and secret scan results.

---

## 3. Multi-Agent & Fleet Coordination
- [**SWARM_COORDINATION_AND_IDENTITY.md**](./SWARM_COORDINATION_AND_IDENTITY.md) — Multi-agent swarm topology, cryptographic Ed25519 identity, and Outcome Verification Tickets (OVT).
- [**MULTI_REPO_FLEET_AND_INTEGRATIONS.md**](./MULTI_REPO_FLEET_AND_INTEGRATIONS.md) — Multi-repository fleet governance, configuration, and developer environment sync.

---

## 4. Guides & Tutorials
- [**APPLE_DESIGN_GUIDE.md**](./APPLE_DESIGN_GUIDE.md) — Apple Human Interface Guidelines (HIG) specification for the Kineti visual companion dashboard and UI components.
- [**TUTORIAL-first-run.md**](./TUTORIAL-first-run.md) — Getting started with Kineti OS from installation to first verified task execution.
- [**HOWTO-daily-loop.md**](./HOWTO-daily-loop.md) — Daily developer workflow: stage gating, test evidence recording, and spend tracking.

---

## 5. Archive
- [**archive.zip**](./archive.zip) — Zipped historical documents and launch planning notes.
