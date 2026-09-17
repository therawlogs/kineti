# Kineti OS

**The Autonomous Nervous System & 360º Human Context Integrity Harness**

Kineti OS is a dual-stack autonomous agent runtime engineered for personalized consumer assistants and multi-agent software engineering swarms. It combines a high-performance **Native Rust Nervous System** (`core-native/`) with a deterministic **TypeScript Governance Control Plane** (`bin/`, `src/`).

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
│   ├── Cargo.toml                           # Workspace manifest (7 active crates)
│   └── crates/
│       ├── kineti-core/                     # EBR Snapshots, HLC, Kernel, Gate, Root Goal
│       ├── kineti-memory/                   # Epistemic Engine, Causal Graph, Vector Index
│       ├── kineti-reflex/                   # Sensory Triage, Emoji Reflexes, Style Profiler
│       ├── kineti-connectors/               # Protocolized Connectors & Permission Gating
│       ├── kineti-actions/                  # Action Execution & Confirmation Gates
│       ├── kineti-gateway/                  # WhatsApp Webhooks & Apple iMessage Bridge
│       ├── kineti-harness/                  # Outcome Verification Tickets (OVT) & Shadow Workspaces
│       └── kineti-cli/                      # Native Binary CLI Entrypoint & Daemon
├── bin/                                     # TypeScript Governance Control Plane
│   ├── kineti.ts                            # Main TypeScript CLI router
│   ├── kineti-spend.ts                      # Hardware spend circuit breaker ($50 ceiling)
│   ├── kineti-saga.ts                       # LIFO undo stack & transactional rollback
│   ├── kineti-evidence.ts                   # Delimited SHA-256 test proofs
│   ├── kineti-companion.ts                  # Apple HIG visual companion server
│   ├── kineti-mcp.ts                        # Model Context Protocol (MCP) server
│   └── kineti-ci.ts                         # Stage-agnostic CI verification
├── docs/                                    # Technical Documentation & Specifications
│   ├── README.md                            # Documentation Index
│   ├── CANONICAL_ARCHITECTURE_PLAN.md       # Canonical Production Architecture Spec
│   ├── ARCHITECTURE_AUDIT_AND_BENCHMARK_REPORT.md # Concurrency & Performance Benchmarks
│   ├── APPLE_DESIGN_GUIDE.md                # Apple HIG UI & Design Standards
│   ├── SECURITY_REPORT.md                   # Security Audit & Origin Gating Analysis
│   ├── SWARM_COORDINATION_AND_IDENTITY.md   # Multi-Agent Swarm Topology & OVTs
│   └── archive/                             # Archived historical notes
├── research/                                # Foundational Research Treatise Series
│   ├── paper_1_autonomous_nervous_system.md # 13-Stage Software Factory & SAGA Undo
│   ├── paper_2_physics_of_context.md        # The Physics of Context & EBR Atomics
│   ├── paper_3_beyond_vector_search.md      # 20-Entity Active Kernel & Causal Graphs
│   ├── paper_4_sensory_reflex_and_style.md  # Sub-1ms Sensory Triage & Style Profiling
│   └── paper_5_outcome_engineering.md       # Outcome Engineering, OVTs & Spend Breaker
├── tests/                                   # Governance & Benchmark Test Suites (83 tests)
└── kineti.config.json                       # Core System Configuration
```

---

## Verification & Safety Guarantees

1. **100% Safe Rust**: `#![forbid(unsafe_code)]` enforced across all critical crates.
2. **Deterministic Spending Ceiling**: `$50.00` total spending cap; automatically halts with exit code 3 at 95% ($47.50).
3. **Cryptographic Proof Binding**: All commits backed by git-tree SHA-256 evidence receipts.
4. **Clean No over Dirty Yes**: Agents report genuine impossibilities clearly rather than silently violating budget or counterparty boundaries.

---

## Documentation

For full architecture deep-dives and research treatises, refer to [**docs/README.md**](docs/README.md).
