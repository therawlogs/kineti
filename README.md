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

### 1. Install via npm

```bash
# Install globally
npm install -g kineti

# Or run directly with npx
npx kineti --help
```

### 2. Connect to Your AI Editor (Cursor, Claude Code, Cline)

Run Kineti directly as an MCP governance server inside your AI editor to enforce spend caps ($50 ceiling), transactional SAGA undo, and cryptographic test verification.

#### Claude Code (CLI)
```bash
claude mcp add kineti npx -y kineti mcp
```

#### Claude Desktop
Add to `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS) or `%APPDATA%\Claude\claude_desktop_config.json` (Windows):
```json
{
  "mcpServers": {
    "kineti": {
      "command": "npx",
      "args": ["-y", "kineti", "mcp"]
    }
  }
}
```

#### Cursor
Add to your project's `.cursor/mcp.json` or Cursor Settings $\to$ Features $\to$ MCP:
```json
{
  "mcpServers": {
    "kineti": {
      "command": "npx",
      "args": ["-y", "kineti", "mcp"]
    }
  }
}
```

#### Cline / Roo Code / Windsurf
Add a new stdio MCP server:
- **Server Name**: `kineti`
- **Command**: `npx`
- **Args**: `["-y", "kineti", "mcp"]`

---

### 3. Core Governance Commands

```bash
# Start the visual companion dashboard (Apple HIG)
kineti companion
# Open http://127.0.0.1:8788

# Start the MCP governance server directly
kineti mcp

# Check spend circuit breaker status ($50 default ceiling)
kineti spend check

# Verify cryptographic test evidence freshness
kineti evidence check --label 360-human-model-resilience

# Run stage-agnostic CI verification
kineti ci
```

### 4. Native Rust Engine (Optional / Contributors)

If developing or running the pure Rust nervous system (`core-native/`):

```bash
# Run all 105 native Rust unit and challenge tests
cargo test --manifest-path core-native/Cargo.toml

# Run the 5 native verification demo flows
cargo run --package kineti-cli -- test-all
```

---

## Repository Structure

```text
kineti/
├── bin/                 # TypeScript Governance CLI tools (single router: kineti.js)
│   ├── kineti.ts        # CLI router source (spend, saga, evidence, companion, mcp, ci)
│   ├── kineti-spend.ts  # Hardware spend circuit breaker ($50 ceiling, exit code 3)
│   ├── kineti-saga.ts   # LIFO undo stack & transactional rollback
│   ├── kineti-evidence.ts # Delimited SHA-256 test proofs bound to git tree hashes
│   ├── kineti-companion.ts # Apple HIG visual companion server (loopback only)
│   └── kineti-mcp.ts    # Model Context Protocol (MCP) server
├── core-native/         # Pure Rust Nervous System Workspace (8 crates, 0 external dependencies)
│   ├── Cargo.toml       # Workspace manifest
│   └── crates/
│       ├── kineti-core/ # Double-buffered snapshots, HLC, Kernel, Commit Gate
│       ├── kineti-memory/ # Epistemic Engine, Multi-scope context, Tombstones
│       ├── kineti-reflex/ # Fast sensory classification & emoji reflexes
│       ├── kineti-connectors/ # Protocolized connectors & permission gating
│       ├── kineti-actions/ # Action execution & confirmation gates
│       ├── kineti-gateway/ # Messaging webhooks & bridge
│       ├── kineti-harness/ # Outcome Verification Tickets (OVT) & Shadow workspaces
│       └── kineti-cli/  # Standalone native CLI binary
├── src/                 # Shared TypeScript libraries (governance, scheduler, security)
├── skills/              # 16 agent workflow skills (installed by setup.sh)
├── hosts/               # 10 editor configurations (Cursor, Claude, Antigravity, etc.)
├── hooks/               # 5 hook text blocks for host setup
├── public/              # Static documentation & legal assets
├── website/             # Kineti marketing & research portal (React + Vite)
├── docs/                # Architecture specifications & security reports
│   ├── PLAN.md          # Canonical production architecture spec
│   ├── AUDIT.md         # Audit summary & benchmarks
│   ├── SECURITY_REPORT.md # Security audit & origin gating analysis
│   └── TUTORIAL-first-run.md # Developer quickstart
├── tests/               # 168 TypeScript governance & causal test suites
├── .github/             # GitHub Actions CI, issue forms, PR template, Dependabot
├── AGENTS.md            # Universal rules for AI agents in this repository
├── SECURITY.md          # Vulnerability disclosure policy & SLAs
├── CONTRIBUTING.md      # Development setup, testing, and DCO sign-off
├── LICENSE              # MIT License
└── package.json         # kineti@0.3.4 npm package manifest
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
