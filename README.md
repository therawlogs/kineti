# Kineti OS

**The Context Integrity Layer (Context Integrity Protocol / CIP) & Autonomous Nervous System**

Kineti OS is a dual-stack agent verification harness and **Context Integrity Layer** (Context Integrity Protocol / CIP) for AI coding agents. Its wedge is spend ceilings, transactional undo (SAGA), project-scoped memory, and cryptographic evidence receipts. It combines a **Native Rust Nervous System** (`core-native/`) with a deterministic **TypeScript Governance Control Plane** (`bin/`, `src/`). A 5-part foundational research series is forthcoming at therawlogs.com. Consumer messaging connectors (WhatsApp/iMessage) and payment tooling in this repo are experimental.

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
- **Double-Buffered Atomic Snapshots**: Two-slot `RwLock` snapshot design with thread-safe read paths and deferred epoch reclamation of retired instances. Latency figures are withheld until reproducible benchmark scripts land in the repo.
- **Universal 20-Entity Provenance Kernel**: Content-addressed RFC 8785 JSON canonicalization with BLAKE3 and SHA-256 digests (hand-rolled under a zero-external-dependency constraint; see `docs/PLAN.md` for rationale and test vectors).
- **Monotonic Hybrid Logical Clock (HLC)**: Physical and logical causality tracking under clock skew.
- **3-Way Graph Commit Gate**: Rejects causal inversions, topological DAG cycles, and single-byte state tampering.
- **Sensory Reflex Triage**: Fast sensory classification dispatching zero-token emoji reactions for low-information conversational stimuli.
- **Protocolized Connectors**: Standard `KinetiConnectorProtocol` trait with consequence level gating (`Trivial`, `Operational`, `HighConsequence`) and single-use SHA-256 payload authorization tokens.
- **Spend Circuit Breaker**: Deterministic trip at 95% of limit ($47.50 of $50.00 ceiling) with OS exit code 3 halt.

### 3. TypeScript Governance Control Plane (`bin/`, `src/`)
- **13-Stage Software Factory**: Strict stage-gated lifecycle ensuring specifications, implementations, and test proofs precede release.
- **Transactional SAGA Undo Stack**: Guarantees LIFO file reversibility before every mutation.
- **Cryptographic Evidence Binding**: Cryptographic SHA-256 receipts bound to exact git tree hashes via `kineti-evidence.ts`.
- **Apple HIG Visual Companion**: Local web dashboard built with Apple Human Interface Guidelines (<12 KB payload, zero runtime JS frameworks).
- **Universal Model Context Protocol (MCP)**: 12 native governance tools exposed to Cursor, Claude Code, Antigravity, and Codex.

### 4. Verified Test Counts & Evaluation Roadmap
- **What is verified today**: 168 TypeScript governance tests and 251 native Rust tests (unit plus integration suites), 0 failures, bound to git tree hashes through delimited SHA-256 evidence receipts (`bin/kineti-evidence.ts`).
- **Frontier figures in `src/harness/benchmark.ts` are design targets, not measured results**: the ALE 76.4% pass rate, SWE-bench 4.2 min MTTR, and $0.31 per-outcome numbers are goal constants for the evaluation program. They have not been produced by empirical runs.
- **Kineti Hostile 100 (in development)**: a public suite of 100 hostile tool calls against the gate with published method and published failures. This is the benchmark the project intends to be judged by.
- **Directional Normalized Trust-Weighted Impact (DNTI)**: three-factor loss-averse outcome verification ($\Phi \times \sigma_\tau(SE) \times \Psi(\mathcal{T})$) designed to resist Goodhart-style metric gaming.
- **Cost Per Verified Outcome ($/Outcome)**: design goal of pricing work in verified business outcomes instead of raw token consumption.

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

# Run the 5 end-to-end verification flows (full workspace: 251 Rust tests via cargo test)
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
├── tests/                                   # Governance, Frontier Benchmarks & Causal Test Suites (168 tests)
├── GOOD_ROADMAP.md                          # Current plan, decisions, ship log (local only, not committed)
├── ROADMAP.md                               # Original phase spec, frozen (local only, not committed)
├── AGENTS.md                                # All agent rules in one file
└── kineti.config.json                       # Core System Configuration
```

---

## Verification & Safety Properties

Today Kineti provides local ledger tooling, spend circuit breakers, and evidence verification that agents invoke during their lifecycle (conventions in `hooks/` plus self-reported proofs via `kineti-spend.ts` and `kineti-evidence.ts`). An in-line MCP proxy gate with deterministic allow/ask/deny interception is in active sprint.

1. **Safe Rust**: `#![forbid(unsafe_code)]` enforced across all critical crates.
2. **Spending Ceiling**: `$50.00` default cap per project, settable at mirror time ($1-$1000); tooling exits with code 3 at 95%.
3. **Cryptographic Proof Binding**: Test runs recorded as SHA-256 evidence receipts bound to the exact code fingerprint.
4. **Clean No over Dirty Yes**: Agents report genuine impossibilities clearly rather than silently violating budget or counterparty boundaries.
5. **Outcome Verification**: DNTI loss-averse scoring plus dollars-per-verified-outcome, targeting the Hostile 100 suite as the public bar.

---

## Documentation

For full architecture deep-dives and research treatises, refer to [**docs/README.md**](docs/README.md).

## Agent rules

If you are an AI agent working in this repo, read [**AGENTS.md**](AGENTS.md) first. It holds all rules, workflows, memory spec, and program references in one file.

## Contributing

Humans and agents follow the same steps in [**CONTRIBUTING.md**](CONTRIBUTING.md): setup, tests with proof, money rules, and the ship checklist.
