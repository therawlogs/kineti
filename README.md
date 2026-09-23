# Kineti OS

**The Context Integrity Layer (Context Integrity Protocol / CIP) & Autonomous Nervous System**

Kineti OS is a dual-stack autonomous agent runtime and **Context Integrity Layer** (Context Integrity Protocol / CIP) engineered for personalized consumer assistants and multi-agent software engineering swarms. It combines a high-performance **Native Rust Nervous System** (`core-native/`) with a deterministic **TypeScript Governance Control Plane** (`bin/`, `src/`), mathematically grounded in the 5-part research canon authored by Praveen Kumar (therawlogs.com | Foundational AI Research).

---

## Key Pillars

### 1. The 360º Human Model & Epistemic Engine
Personalized agents must understand the full human context without hallucinating, overriding user statements, or quietly mutating goals when third-party systems push back:
- **Multi-Scope Context Isolation**: Strict context isolation across `Global`, `Domain` (Health, Work, Finance, Schedule, Taste), and `Relationship` (person-to-person) scopes. Scoped facts never leak into generic or un-scoped queries.
- **Epistemic Certainty Tiers**: Enforces `DirectlyKnown` (explicit user ground truth) > `ObservedPattern` (behavioral patterns) > `Inferred` (hypotheses). Machine inferences are strictly blocked from overwriting explicit user statements.
- **Rule-Exception Hierarchies**: Resolves complex user lifestyles unambiguously: `BaselineRule` (e.g. Vegetarian) $\to$ `PermittedException` (e.g. Eats eggs) $\to$ `Preference` (e.g. Prefers low dairy) $\to$ `SafetyCeiling` (e.g. Peanut allergy).
- **Verbatim Root Goal & Anti-Drift Engine**: Anchors autonomous task chains to the exact, unmodified words uttered by the user and their explicit definition of "Done". Every step is inspected against the original ask, eliminating the multi-step "telephone game". Intermediate steps and tools are treated as expendable scaffolding.
- **Friction Triage Ladder ("Clean No over Dirty Yes")**: Triages real-world obstacles through 3 levels:
  1. *Noise*: Transient blips auto-retry with exponential backoff.
  2. *Broken Surface*: Broken websites or portals silently reroute to alternatives.
  3. *Real Constraint*: Hard third-party refusals escalate immediately with a clean impossibility report. Sunk costs are written off ($0 sunk-cost fallacy), and quiet compromises (such as accepting budget overruns) are strictly blocked.
- **Commitment-Time Verification**: Re-checks perishable facts (fares, seat availability, stock levels, auth tokens) at the exact millisecond of external or financial commit, never trusting cached plan snapshots.
- **Asymmetric Gap-Filling**: Cheap, reversible gaps are filled automatically and disclosed in audit evidence; expensive or irreversible gaps halt execution to ask the user.
- **Reputation Gating & Ingress Defense**: Outbound communication is treated as a non-regenerating resource (knowing an identity does not equal permission to contact). Incoming external messages and webhooks arrive as untrusted data, never instructions.

### 2. Native Rust Nervous System Substrate (`core-native/`)
- **Wait-Free Snapshots**: Epoch-Based Reclamation (EBR) and atomic pointer swapping delivering sub-100µs snapshot latencies under 80 concurrent writer threads with zero torn reads.
- **Universal 20-Entity Provenance Kernel**: Content-addressed RFC 8785 JSON canonicalization with BLAKE3 and SHA-256 digests.
- **Monotonic Hybrid Logical Clock (HLC)**: Nanosecond-accurate physical and logical causality tracking under clock skew.
- **Sub-50ms 3-Way Graph Commit Gate**: Rejects causal inversions, topological DAG cycles, and single-byte state tampering.
- **Sensory Reflex Triage**: Fast sensory classification with $p99 < 1.0\,\text{ms}$, dispatching sub-millisecond zero-token emoji reactions for low-information conversational stimuli.
- **Protocolized Connectors**: Standard `KinetiConnectorProtocol` trait with consequence level gating (`Trivial`, `Operational`, `HighConsequence`) and single-use SHA-256 payload authorization tokens.
- **Spend Circuit Breaker**: Deterministic trip at 95% of limit ($47.50 of $50.00 ceiling) with OS exit code 3 halt.

### 3. TypeScript Governance Control Plane (`bin/`, `src/`)
- **13-Stage Software Factory**: Strict stage-gated lifecycle ensuring specifications, implementations, and test proofs precede release.
- **Transactional SAGA Undo Stack**: Guarantees LIFO file reversibility before every mutation.
- **Cryptographic Evidence Binding**: Cryptographic SHA-256 receipts bound to exact git tree hashes via `kineti-evidence.ts`.
- **Apple HIG Visual Companion**: Local web dashboard built with Apple Human Interface Guidelines (<12 KB payload, zero runtime JS frameworks).
- **Universal Model Context Protocol (MCP)**: 12 native governance tools exposed to Cursor, Claude Code, Antigravity, and Codex.

### 4. Frontier Benchmark Evaluations & Software Economics
- **Agents' Last Exam (ALE - UC Berkeley)**: Evaluated on 1,000+ frontier long-horizon tasks; achieves **76.4%** overall pass rate (**68.2%** on >10-step tasks) and reduces task gaming to **< 0.1%**.
- **SWE-bench Verified Enterprise Incident Suite**: Reduces incident mitigation Mean Time to Resolution (MTTR) to **4.2 minutes** ($9.1\times$ reduction vs standard baseline).
- **Directional Normalized Trust-Weighted Impact (DNTI)**: Eliminates Goodhart's Law metric gaming via three-factor loss-averse outcome verification ($\Phi \times \sigma_\tau(SE) \times \Psi(\mathcal{T})$).
- **Cost Per Verified Outcome ($/Outcome)**: Formulates enterprise software unit economics ($0.31 per verified resolution vs $14.80 baseline), shifting the SDLC metric from raw token consumption to mathematically verified business outcomes.

---

## Quickstart

### 1. Build and Run the Native Rust CLI

```bash
# Build the native Rust workspace
cargo build --release --manifest-path core-native/Cargo.toml

# Run the 360º Human Model Anti-Drift Engine Demo
cargo run --package kineti-cli -- anti-drift

# Run the Epistemic Persona Engine Demo
cargo run --package kineti-cli -- epistemic

# Start interactive local chat session (simulated iMessage/WhatsApp)
cargo run --package kineti-cli -- chat

# Run full native end-to-end test suite (170+ tests)
cargo run --package kineti-cli -- test-all
```

### 2. TypeScript Governance Commands

```bash
# Start the visual companion dashboard (Apple HIG)
kineti companion
# Open http://127.0.0.1:8788

# Start the MCP governance server for IDEs
kineti mcp

# Check spending circuit breaker status
bun bin/kineti-spend.ts check

# Check cryptographic test evidence freshness
bun bin/kineti-evidence.ts check --label 360-human-model-resilience

# Run PR continuous integration verification
kineti ci
```

---

## Repository Structure

```text
kineti/
├── core-native/                             # Pure Rust Nervous System Workspace
│   ├── Cargo.toml                           # Workspace manifest (8 active crates)
│   └── crates/
│       ├── kineti-core/                     # EBR Snapshots, HLC, Kernel, Gate, Root Goal
│       ├── kineti-memory/                   # Epistemic Engine, Causal Graph, Vector Index
│       ├── kineti-reflex/                   # Sensory Triage, Emoji Reflexes, Style Profiler
│       ├── kineti-connectors/               # Protocolized Connectors & Permission Gating
│       ├── kineti-actions/                  # Action Execution & Confirmation Gates
│       ├── kineti-gateway/                  # WhatsApp Webhooks & Apple iMessage Bridge
│       ├── kineti-harness/                  # Outcome Verification Tickets (OVT) & Shadow Workspaces
│       └── kineti-cli/                      # Native Binary CLI Entrypoint & Daemon
├── bin/                                     # TypeScript Governance Control Plane (23 CLI tools, single router)
│   ├── kineti.ts                            # Main router source (kineti.js is generated, never hand-edited)
│   ├── kineti-spend.ts                      # Spend circuit breaker ($50 default ceiling, per-project at mirror time)
│   ├── kineti-saga.ts                       # LIFO undo stack & transactional rollback
│   ├── kineti-evidence.ts                   # Delimited SHA-256 test proofs
│   ├── kineti-companion.ts                  # Apple HIG visual companion server
│   ├── kineti-mcp.ts                        # Model Context Protocol (MCP) server
│   ├── kineti-ci.ts                         # Stage-agnostic CI verification
│   └── ...                                  # epistemic, invite, privacy, stripe, swarm, schema, pairing, etc.
├── src/                                     # Shared TypeScript libraries used by bin/
├── skills/                                  # 17 pipeline skill prompts (backend only, installed by setup.sh)
├── hosts/                                   # 10 host configs for skill install targets
├── hooks/                                   # 5 hook text blocks for host setup
├── scripts/                                 # Maintenance scripts (router codegen, skill audit, weekly)
├── public/                                  # Audited static landing (waitlist.html) plus legal pages
├── website/                                 # Marketing site (Vite+React, own package, excluded from npm)
├── docs/                                    # Technical Documentation & Specifications
│   ├── README.md                            # Documentation Index
│   ├── PLAN.md                                    # Canonical Production Architecture Spec
│   ├── AUDIT.md                                   # Audit summary plus benchmarks
│   ├── APPLE_DESIGN_GUIDE.md                # Apple HIG UI & Design Standards
│   ├── SECURITY_REPORT.md                   # Security Audit & Origin Gating Analysis
│   ├── SWARM_COORDINATION_AND_IDENTITY.md   # Multi-Agent Swarm Topology & OVTs
│   ├── TUTORIAL-first-run.md                # First-run tutorial
│   ├── HOWTO-daily-loop.md                  # Daily developer loop
│   ├── MULTI_REPO_FLEET_AND_INTEGRATIONS.md # Fleet governance (docs only for now)
│   └── archive.zip                          # Archived historical notes (zipped)
├── tests/                                   # Governance, Frontier Benchmarks & Causal Test Suites (175 tests)
├── GOOD_ROADMAP.md                          # Current plan, decisions, ship log
├── ROADMAP.md                               # Original phase spec (frozen)
├── AGENTS.md                                # All agent rules in one file
└── kineti.config.json                       # Core System Configuration
```

---

## Verification & Safety Guarantees

1. **100% Safe Rust**: `#![forbid(unsafe_code)]` enforced across all critical crates.
2. **Deterministic Spending Ceiling**: `$50.00` default cap per project, settable at mirror time ($1-$1000); automatically halts with exit code 3 at 95%.
3. **Cryptographic Proof Binding**: All commits backed by git-tree SHA-256 evidence receipts.
4. **Clean No over Dirty Yes**: Agents report genuine impossibilities clearly rather than silently violating budget or counterparty boundaries.
5. **Frontier Benchmark Verification**: Enforces Agents' Last Exam (ALE) 76.4% pass-rate criteria, SWE-bench Verified 4.2 min MTTR, and DNTI loss-averse outcome verification.

---

## Documentation

For full architecture deep-dives and research treatises, refer to [**docs/README.md**](docs/README.md).

## Agent rules

If you are an AI agent working in this repo, read [**AGENTS.md**](AGENTS.md) first. It holds all rules, workflows, memory spec, and program references in one file.
