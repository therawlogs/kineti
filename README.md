# Kineti OS `v0.3.7`

[![npm version](https://img.shields.io/npm/v/kineti.svg?color=teal)](https://www.npmjs.com/package/kineti)
[![crates.io](https://img.shields.io/crates/v/kineti-cli.svg?color=orange)](https://crates.io/crates/kineti-cli)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> **Spend cap, transactional undo, and proof receipts for AI coding agents.**  
> A lightweight open-source (MIT) safety harness that runs under Claude Code, Cursor, OpenCode, Codex, Antigravity, Cline, and fx.sh.

```text
$ kineti spend status
total $0 of $50; entries 0; tripped=false

$ kineti spend log --stage build --model sonnet --tokens-in 400000 --tokens-out 200000
logged $4.2 (stage build); run total $4.2

$ kineti spend log --stage build --model opus --tokens-in 100000 --tokens-out 60000
kineti: SPEND BREAKER TRIPPED: stage build total $10.2 reached ceiling $9.5 (exit code 3)

$ kineti spend check
kineti: TRIPPED: stage build total $10.2 reached ceiling $9.5 (exit code 3)

$ kineti spend reset
kineti: reset requires --i-am-human (breakers are human-only) (exit code 2)

$ kineti spend reset --i-am-human
breaker reset by human (exit code 0)
```

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
- **Spend Circuit Breaker**: Deterministic trip at 95% of recorded totals ($47.50 of $50.00 ceiling) with OS exit code 3 halt.

### 3. TypeScript Governance Control Plane (`bin/`, `src/`)
- **13-Stage Software Factory**: Strict stage-gated lifecycle ensuring specifications, implementations, and test proofs precede release.
- **Transactional SAGA Undo Stack**: Guarantees LIFO file reversibility before every mutation.
- **Cryptographic Evidence Binding**: Cryptographic SHA-256 receipts bound to exact workspace code fingerprints via `kineti-evidence.ts`.
- **Apple HIG Visual Companion**: Local web dashboard built with Apple Human Interface Guidelines (78 KB HTML, 15 KB gzipped, zero runtime JS frameworks).
- **Universal Model Context Protocol (MCP)**: 14 native governance tools exposed to Cursor, Claude Code, Antigravity, and Codex.

### 4. Verified Test Counts & Evaluation Roadmap
- **What is verified today**: 190 TypeScript governance tests and 251 native Rust tests (unit plus integration suites), 0 failures (441 total passed tests), bound to workspace code fingerprints through delimited SHA-256 evidence receipts (`bin/kineti-evidence.ts`).
- **Frontier figures in `src/harness/benchmark.ts` are design targets, not measured results**: the ALE 76.4% pass rate, SWE-bench 4.2 min MTTR, and $0.31 per-outcome numbers are goal constants for the evaluation program. They have not been produced by empirical runs.
- **Kineti Hostile 100 (in development)**: a public suite of 100 hostile tool calls against the gate with published method and published failures. This is the benchmark the project intends to be judged by.
- **Directional Normalized Trust-Weighted Impact (DNTI)**: three-factor loss-averse outcome verification ($\Phi \times \sigma_\tau(SE) \times \Psi(\mathcal{T})$) designed to resist Goodhart-style metric gaming.
- **Cost Per Verified Outcome ($/Outcome)**: design goal of pricing work in verified business outcomes instead of raw token consumption.

---

## Quickstart

### 1. Install via npm

> **Prerequisite**: The CLI is distributed on npm and runs on Node.js (>= 18), but requires Bun (>= 1.1) installed on the system for governance execution (`curl -fsSL https://bun.sh/install | bash`).

```bash
# Install globally (latest)
npm install -g kineti

# Or pin to the current stable release
npm install -g kineti@0.3.7

# Or run directly with npx
npx kineti --help
```

### 2. Connect to Your AI Editor or Agent Host

Run Kineti directly as an MCP governance server inside your AI editor to record spend against caps ($50 ceiling; the agent records each model call and the breaker trips on recorded totals), transactional SAGA undo, and cryptographic test verification.

#### Automatic Setup for All Hosts
Run Kineti's auto-configurator in your project directory:
```bash
kineti init
```
This detects your installed hosts (Claude, OpenCode, Codex, Cursor, Antigravity, Gemini, fx.sh), copies the 17 workflow skills into each one, writes the `~/.kineti/repo` pointer, and prints the safety-hook block to paste into your host settings. Use `kineti init --host <name>` to install into a single host, or `kineti init --install-root` to add the project rule files.

MCP server config is **not** written automatically. Add it with the manual step below for your editor.

#### Manual Configuration by Editor

**Claude Code (CLI)**:
```bash
claude mcp add kineti npx -y kineti mcp
```

**Cursor**:
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
Windsurf uses a different path: `~/.codeium/windsurf/mcp_config.json`, with the same `mcpServers` JSON.

**OpenCode**:
```bash
opencode mcp add kineti -- npx -y kineti mcp
```
Or add the block below to `~/.config/opencode/opencode.jsonc`:
```json
{
  "mcp": {
    "kineti": {
      "type": "local",
      "command": ["npx", "-y", "kineti", "mcp"]
    }
  }
}
```
Confirm with `opencode mcp list`.

**Codex (CLI)**:
Add to `${CODEX_HOME:-$HOME/.codex}/config.toml`:
```toml
[mcp_servers.kineti]
command = "npx"
args = ["-y", "kineti", "mcp"]
```
Confirm Codex sees Kineti:
```bash
codex mcp list
```
Kineti should appear in the list.

**Google Antigravity**:
Install the Kineti skills into Antigravity:
```bash
kineti init --host antigravity
```
Gemini uses the same installer:
```bash
kineti init --host gemini
```
Neither host ships an MCP config block here, so connect the MCP server through your editor's generic MCP settings with the fields in the next section.

**Fx.sh**:
Install skill rules directly to your fx environment:
```bash
kineti init --host fx
```

**Claude Desktop**:
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

**Cline / Roo Code**:
Add a stdio MCP server in settings (Cline: `.clinerules/mcp_settings.json`, Roo Code: `.roo/mcp_settings.json`):
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

# Run tests and save cryptographic proof receipt
kineti test -- echo hello

# Verify cryptographic test evidence freshness
kineti test check --label test

# Run stage-agnostic CI verification
kineti ci
```

### 4. Native Rust Engine & Crates.io

All 8 native crates are published on [crates.io](https://crates.io):

```bash
# Add native core nervous system to your Rust project
cargo add kineti-core

# Or install the native CLI binary directly
cargo install kineti-cli@0.3.7

# Run all 251 native Rust unit and integration tests locally
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
│   ├── kineti-spend.ts  # Self-reported spend circuit breaker ($50 ceiling, exit code 3, trips on recorded totals)
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
├── skills/              # 17 agent workflow skills (installed by setup.sh)
├── hosts/               # 7 editor configurations (Claude, Cursor, Codex, OpenCode, Gemini, Antigravity, fx.sh)
├── hooks/               # 6 hook text blocks for host setup
├── public/              # Static documentation & legal assets
├── website/             # Static landing page (index.html plus video poster assets)
├── docs/                # Architecture specifications & security reports
│   ├── README.md        # Docs index
│   ├── PLAN.md          # Canonical production architecture spec
│   ├── SECURITY_REPORT.md # Security audit & origin gating analysis
│   ├── TUTORIAL-first-run.md # Developer quickstart
│   ├── HOWTO-daily-loop.md # Daily operating loop
│   ├── SWARM_COORDINATION_AND_IDENTITY.md # Multi-agent identity & signing
│   └── MULTI_REPO_FLEET_AND_INTEGRATIONS.md # Fleet and integration design
├── tests/               # 190 TypeScript governance & causal test suites
├── .github/             # GitHub Actions CI, issue forms, PR template, Dependabot
├── AGENTS.md            # Universal rules for AI agents in this repository
├── SECURITY.md          # Vulnerability disclosure policy & SLAs
├── CONTRIBUTING.md      # Development setup, testing, and DCO sign-off
├── LICENSE              # MIT License
└── package.json         # kineti@0.3.7 npm package manifest
```

---

## Verification & Safety Properties

Today Kineti provides local ledger tooling, spend circuit breakers, and evidence verification that agents invoke during their lifecycle (conventions in `hooks/` plus self-reported proofs via `kineti-spend.ts` and `kineti-evidence.ts`). An in-line MCP proxy gate with deterministic allow/ask/deny interception is in active sprint.

1. **Safe Rust**: `#![forbid(unsafe_code)]` enforced across all critical crates.
2. **Spending Ceiling**: `$50.00` default cap per project, settable at mirror time ($1-$1000); tooling exits with code 3 at 95% of recorded totals.
3. **Cryptographic Proof Binding**: Test runs recorded as SHA-256 evidence receipts bound to the exact code fingerprint.
4. **Clean No over Dirty Yes**: Agents report genuine impossibilities clearly rather than silently violating budget or counterparty boundaries.
5. **Outcome Verification**: DNTI loss-averse scoring plus dollars-per-verified-outcome, targeting the Hostile 100 suite as the public bar.

---

## Documentation

For full architecture deep-dives and research treatises, refer to [**docs/README.md**](docs/README.md).

## Agent rules

If you are an AI agent working in this repo, read [**AGENTS.md**](AGENTS.md) first. It holds all rules, workflows, memory spec, and program references in one file.

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for more information.

## Contributing
 
Humans and agents follow the same steps in [**CONTRIBUTING.md**](CONTRIBUTING.md): setup, tests with proof, money rules, and the ship checklist.

## Contact

- **General & Community**: [hello@getkineti.com](mailto:hello@getkineti.com)
- **Security Disclosures**: [security@getkineti.com](mailto:security@getkineti.com) (disclosure policy and SLA in [SECURITY.md](SECURITY.md))
- **Website**: [getkineti.com](https://getkineti.com)
- **GitHub Issues**: [github.com/therawlogs/kineti/issues](https://github.com/therawlogs/kineti/issues)
