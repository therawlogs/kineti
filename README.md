# Kineti OS `v0.5.0`

[![npm version](https://img.shields.io/npm/v/kineti.svg?color=teal)](https://www.npmjs.com/package/kineti)
[![crates.io](https://img.shields.io/crates/v/kineti-cli.svg?color=orange)](https://crates.io/crates/kineti-cli)
[![license: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Kineti helps AI coding agents run tests, save a record of the files tested, keep undo commands, and track reported spending. It shows when later edits make a test record stale and can help CI stop a release. It does not control every agent action or verify provider bills. Protected writes are experimental and do not provide a secure sandbox.

From a source checkout, run the repository's Bun CLI with an explicit label and required test inventory:

Prerequisites: Bun is installed and project dependencies are available (`bun install`).

```bash
bun bin/kineti-evidence.ts run --label release-check --required-test tests -- bun test tests/
bun bin/kineti-evidence.ts check --label release-check
```

After a source edit, the check reports a stale receipt. Rerun the same `run` command, then check again. With an installed package, use `kineti test run --label release-check --required-test tests -- bun test tests/` and `kineti test check --label release-check` from the project directory.

Example blocked bad release: a release script checks the receipt, receives a non-zero `STALE` result after a source edit, and stops before packaging. This is a check your script must run; Kineti does not stop unrelated release commands or intercept every editor action.

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

### 1. Rust core (`core-native/`)
Rust core with a hash-chained ledger and a commit check. Spend breaker exits with code 3 at 95% of recorded spend.

### 2. TypeScript Governance Control Plane (`bin/`, `src/`)
- **13-Stage Project State**: A local workflow record for a goal, stage, and named gates.
- **Transactional SAGA Undo Stack**: Newest-first rollback of inverses the agent registered. A failed inverse prints `rollback incomplete`, exits 1, and that step stays pending.
- **Versioned Test Receipts**: Kineti records the test command, parsed test summary, required-test inventory, workspace fingerprint, and a local Ed25519 signature. The signature identifies the local issuer key; it does not establish a trusted organization or prove tests were adequate. The separate verifier can check the declared evidence when given the trusted public key.
- **Visual Companion**: A local dashboard for project state, activity, team details, and local budgets.
- **Model Context Protocol (MCP)**: 15 tools: 14 governance tools plus the plain-talk helper. The opt-in protected-write launcher targets Cursor; real host acceptance remains unverified.

### 3. Test Inventory & Evaluation Roadmap
- **Test inventory**: 270 TypeScript governance test cases and 250 native Rust test cases (520 total test cases); passing tests verify tested paths, not every external integration or agent action.
- **Frontier figures in `src/harness/benchmark.ts` are design targets, not measured results**: the ALE 76.4% pass rate, SWE-bench 4.2 min MTTR, and $0.31 per-outcome numbers are goal constants for the evaluation program. They have not been produced by empirical runs.
- **Not provided**: a complete agent sandbox or cloud connector service.

---

## Quickstart

### 1. Install via npm

> **Prerequisite**: The CLI is distributed on npm and runs on Node.js (>= 18), but requires Bun (>= 1.1) installed on the system for governance execution (`curl -fsSL https://bun.sh/install | bash`).

```bash
# Install globally (latest)
npm install -g kineti

# After v0.5.0 is published to npm, pin to that version
npm install -g kineti@0.5.0

# Or run directly with npx
npx kineti --help
```

### 2. Connect to Your AI Editor or Agent Host

Run Kineti as an MCP server in your editor for local test receipts, registered rollback, and a spend log. Spend totals are reported by the agent; Kineti does not independently measure provider billing or stop the agent from making calls outside Kineti.

#### Automatic Setup for All Hosts
Run Kineti's auto-configurator in your project directory:
```bash
kineti init
```
This detects your installed hosts (Claude, OpenCode, Codex, Cursor, Antigravity, Gemini, fx.sh), copies the 17 workflow skills into each one, writes the `~/.kineti/repo` pointer, and prints the safety-hook block to paste into your host settings. Use `kineti init --host <name>` to install into a single host, or `kineti init --install-root` to add the project rule files.

MCP server config is **not** written automatically. Add it with the manual step below for your editor.

The opt-in protected file-write launcher targets Cursor in this release; real Cursor host acceptance remains unverified. `kineti mcp init` safely merges the Kineti launcher into `.cursor/mcp.json`; writes are gated only when the agent uses Kineti's `kineti_protected_write` MCP tool. Cursor-native file tools bypass this gate. Other hosts' gate adapters are unsupported.

Initialize local project state separately from editor setup:

```bash
kineti state init --project my-project --goal "Describe the outcome for this run"
```

`kineti init` installs agent instructions and skills. `kineti state init` creates local task state. In CI, use `kineti seed` to create only missing state and spend files from `kineti.config.json`; it keeps existing locked state unchanged.

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
To merge the protected launcher into the existing Cursor MCP configuration, run `kineti mcp init` from the project directory. It keeps other servers and saves a backup.
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
Neither host has a host-specific MCP command here. To connect the server, use the host's MCP settings to add a local stdio server with command `npx` and arguments `-y kineti mcp`. The exact settings format depends on the host.

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
# Start the visual companion dashboard
kineti companion
# Open http://127.0.0.1:8788

# Start the MCP governance server directly
kineti mcp

# Check spend circuit breaker status ($50 default ceiling)
kineti spend check

# Run tests, declare the required inventory, and save a versioned receipt
bun bin/kineti-evidence.ts run --label unit --required-test tests/lib.test.ts -- bun test tests/lib.test.ts

# Check local test receipt freshness
kineti test check --label test

# Check a receipt's declared evidence elsewhere using a trusted public key
kineti evidence-verify --receipt receipt.json --trusted-key .kineti/evidence-issuer.pem --workspace .

# Prepare runtime state on a fresh CI checkout (run before tests and `kineti ci`)
kineti seed

# Run CI checks with required evidence labels
kineti ci --require-evidence typecheck --require-evidence unit-tests
```

CI workflows should pass every required test label to the gate. For example:

```bash
kineti ci --require-evidence typecheck --require-evidence unit-tests
```

The CI report checks the required receipts against the current workspace fingerprint and reports **checks passed** or **blocked**. It reports declared receipt checks only.

### 4. Native Rust Engine & Crates.io (`kineti-cli`)

The Rust workspace contains eight separately versioned crates on [crates.io](https://crates.io). The v0.5.0 commands below are available after that release finishes publishing. Note that the native binary `kineti-cli` provides native Rust demo and epistemic commands, and does not replace the Node/Bun npm `kineti` governance router (`kineti mcp`, `kineti spend`, `kineti test`, `kineti ci`).

```bash
# Add the native core to your Rust project after v0.5.0 is published
cargo add kineti-core

# Or install the native CLI binary (kineti-cli)
cargo install kineti-cli@0.5.0

# Run the native Rust test suite locally
cargo test --manifest-path core-native/Cargo.toml

# Run the 5 native verification demo flows
cargo run --package kineti-cli -- test-all
```

---

## Repository Structure

```text
kineti/
├── bin/                 # TypeScript governance tools (single router: kineti.js)
│   ├── kineti.ts        # CLI router source
│   ├── kineti-seed.ts   # Creates missing CI state and spend files from config
│   ├── kineti-spend.ts  # Self-reported spend circuit breaker ($50 ceiling, exit code 3, trips on recorded totals)
│   ├── kineti-saga.ts   # LIFO undo stack & transactional rollback
│   ├── kineti-evidence.ts # Test exit codes with workspace fingerprints
│   ├── kineti-ci.ts    # Required evidence and local state report
│   ├── kineti-companion.ts # Local dashboard server (loopback only)
│   └── kineti-mcp.ts    # Model Context Protocol (MCP) server
├── core-native/         # Native Rust workspace (8 crates; harness uses Ed25519 library)
│   ├── Cargo.toml       # Workspace manifest
│   └── crates/
│       ├── kineti-core/ # Double-buffered snapshots, HLC, Kernel, Commit Gate
│       ├── kineti-memory/ # Epistemic Engine, Multi-scope context, Tombstones
│       ├── kineti-reflex/ # Fast sensory classification & emoji reflexes
│       ├── kineti-connectors/ # Protocolized connectors & permission gating
│       ├── kineti-actions/ # Action execution & confirmation gates
│       ├── kineti-gateway/ # Messaging webhooks & bridge
│       ├── kineti-harness/ # OVT signature demo with caller-supplied key trust; worktrees are not sandboxes
│       └── kineti-cli/  # Standalone native CLI binary
├── src/                 # Shared TypeScript libraries (governance, scheduler, security)
├── skills/              # 17 agent workflow skills (installed by setup.sh)
├── hosts/               # 7 editor configurations (Claude, Cursor, Codex, OpenCode, Gemini, Antigravity, fx.sh)
├── hooks/               # 6 hook text blocks for host setup
├── public/              # Static legal pages
├── website/             # Static landing page (index.html plus video poster assets)
├── docs/                # Architecture specifications & security reports
│   ├── README.md        # Docs index
│   ├── PLAN.md          # Historical architecture proposal
│   ├── ARCHITECTURE.md  # Rust core internals (not the CLI pitch)
│   ├── research/        # Research notes, not part of the CLI
│   ├── SECURITY_REPORT.md # Security audit & origin gating analysis
│   ├── TUTORIAL-first-run.md # Developer quickstart
│   ├── HOWTO-daily-loop.md # Daily operating loop
│   ├── SWARM_COORDINATION_AND_IDENTITY.md # Multi-agent identity & signing
│   └── MULTI_REPO_FLEET_AND_INTEGRATIONS.md # Fleet and integration design
├── tests/               # TypeScript governance and utility tests
├── .github/             # GitHub Actions CI, issue forms, PR template, Dependabot
├── AGENTS.md            # Universal rules for AI agents in this repository
├── SECURITY.md          # Vulnerability disclosure policy & SLAs
├── CONTRIBUTING.md      # Development setup, testing, and DCO sign-off
├── LICENSE              # MIT License
└── package.json         # Kineti npm package manifest
```

---

## Verification & Safety Properties

Today Kineti provides local ledger tooling, registered rollback, spend totals based on agent reports, and test receipts bound to a workspace fingerprint. These tools do not mediate every agent action.

1. **Safe Rust**: `#![forbid(unsafe_code)]` is enforced in the listed Rust crates.
2. **Recorded-spend limit**: `$50.00` default local ceiling, configurable in project settings; spend is agent-reported, and the logging command exits with code 3 near the recorded limit.
3. **Test receipts**: New receipts are signed by a project-local Kineti key and bind a command, parsed test summary, required test files, dependency lockfiles, tool versions, and workspace fingerprint. Old receipts remain readable with limited results and cannot authorize the Cursor protected-write gate. A copied receipt can be checked elsewhere only against its declared evidence and a separately trusted issuer public key; the verifier does not run tests on that other machine. Text files normalize CRLF to LF; binary files are hashed as raw bytes. Paths are workspace-relative with `/` separators. Files over 50 MiB retain the existing size-and-mtime limitation.
4. **Experimental protected write**: Writes routed through `kineti_protected_write` are authorized against a separately provisioned local issuer key, a receipt no older than 240 minutes, current declared inputs, and the exact destination, arguments, and bytes. Set `.kineti/protected-write-policy.json` to JSON with `version: 1`, `trusted_issuer_key` copied from `.kineti/evidence-issuer.pem`, and empty `revoked_run_ids` and `revoked_issuer_keys` arrays; set its file mode to `0600`. The policy and signing key are controlled by the same local account; this does not protect against an adversarial builder or account owner. Add run IDs or issuer keys to the corresponding revocation list to block them. Missing or malformed policy blocks writes. Captured summaries and printed file names are untrusted test observation. A same-command regression reproduces a forged-success receipt being accepted by the writer. Independent evaluator results and a trusted required-case inventory remain unresolved; stronger gate acceptance is blocked. A run is marked used before the final rename; a failure after that point requires a new receipt. The check, filesystem rechecks, temporary write, and rename are separate operations under a lock used only by cooperating Kineti writers, so the authorization-to-effect gap remains unresolved. Cursor’s direct filesystem tools and other out-of-band writers bypass this gate.
5. **Scriptable results**: Receipt checks use `0` for fresh, `4` for stale, `5` for missing, `6` for limited legacy receipts, and `7` for invalid or unsupported evidence. Protected writes use `0` for allowed, `1` for blocked, `2` for invalid input, and `3` for a write failure.
6. **Agent guidance**: Installed workflow instructions ask agents to report genuine impossibilities rather than silently exceed stated limits. Kineti does not enforce this across every tool or action.
7. **Prompt-injection tests**: The repository includes 550 test inputs for the local prompt filter. Passing that suite is not a general security guarantee. A public 100 hostile-tool-call benchmark is not included.

---

## Documentation

For protected-write trust setup and guarantees, see [Protected write policy and limits](docs/PROTECTED-WRITE.md).

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
