# KINETI OS — UNIVERSAL AI AGENT HARNESS & CONTEXT INTEGRITY RUNTIME
## Master Architecture, Core Engine Hardening, Developer GTM, Solo-Founder Economics & Strategic M&A Blueprint

**Document Identifier:** `KINETI-STRAT-BLUEPRINT-2026-V1`  
**Classification:** Strategic Architectural Specification & Commercial Blueprint  
**Authors:** Kineti OS Engineering & Strategy Architecture Team (`teamwork_preview_worker_m2_1`)  
**Target Repository:** Kineti Local Harness (`kineti-os`)  
**Working Directory Root:** `/Users/praveen/Documents/Products/kineti local harness`  
**Publication Date:** September 2026  
**Status:** Authoritative / Complete / Production-Grade  

---

## Master Table of Contents

1. [Executive Summary & Foundational Vision](#1-executive-summary--foundational-vision)
   - 1.1 The Context Integrity Imperative & The Autonomous Agent Trust Barrier
   - 1.2 Core Failure Modes of Existing Agent Frameworks
   - 1.3 The Kineti Philosophy: "Skills Propose, Programs Enforce, Memory Remembers"
   - 1.4 The Paradigm Shift: From Probabilistic Prompting to Outcome Engineering
   - 1.5 System Topology: Dual-Plane Architecture Overview
2. [Universal Host Architecture & Hybrid Companion Blueprint (R1)](#2-universal-host-architecture--hybrid-companion-blueprint-r1)
   - 2.1 Universal Host Adapter Specifications (Antigravity, Claude Code, Codex/Operator, OpenCode, Cursor, Terminal CLI)
   - 2.2 Cross-Host Protocol & Intercept Matrix
   - 2.3 Microsecond-Level Local Loop Latency Budget Breakdown (< 10ms)
   - 2.4 Form Factor Evaluation: Headless Daemon vs. Standalone Companion vs. IDE Extension (The Aside.com Deconstruction)
   - 2.5 The Hybrid Companion Architecture Specification (Rust/Bun Daemon + Visual Sidecar Canvas `ws://127.0.0.1:8788`)
   - 2.6 Reactive Telemetry, Approval Gate Protocol & Wire Formats
3. [Core Engine Hardening & Research Substrate Integration (R2)](#3-core-engine-hardening--research-substrate-integration-r2)
   - 3.1 The Context Integrity Protocol (CIP) 7-Layer Architecture (L1 Transport to L7 Outcome)
   - 3.2 Causal-Graph Substrates & ISO SQL/PGQ Property Graph Specification
   - 3.3 Universal 20-Entity Provenance Kernel (Full TypeScript Definitions & JSON-LD Schemas)
   - 3.4 Runtime Ontology Trigger Data (OTD) Mechanics & Event-Driven State Machine
   - 3.5 Outcome Verification Tickets (OVTs): Dual-Signing Protocol, RFC 8785 JCS & W3C Verifiable Credentials
   - 3.6 Systematic Remediation & Eradication Matrix of All 44 Audit Defects (CRIT-01 to INFO-10)
4. [Solo-Founder Unit Economics & Monetization Engine (R3)](#4-solo-founder-unit-economics--monetization-engine-r3)
   - 4.1 The Three-Tier Packaging & Pricing Matrix (Open-Core, Pro $39/mo, Enterprise $250+/mo)
   - 4.1.1 The getkineti.com Open-Source Move: Boundary, IP Protection & The "Services Trap" Fix
   - 4.2 Comprehensive 24-Month Financial Pro-Forma Model ($0 to $1.40M ARR M12, $4.59M ARR M24)
   - 4.3 Solo Operator Cost Structure, COGS, OpEx, and Operating Leverage (> 95% Gross Margins, > 90% EBITDA)
   - 4.4 The Economics Paradigm Shift: $/Token to Cost Per Verified Outcome ($/Outcome) & Mathematical Enterprise ROI
5. [Market Positioning & Developer Go-To-Market (GTM) (R4)](#5-market-positioning--developer-go-to-market-gtm-r4)
   - 5.1 Exhaustive 10-Player Competitive Landscape Matrix & Unoccupied White Space
   - 5.2 0-to-1 and 1-to-10 Viral Developer GTM Roadmap (MCP Ecosystem, Signed Git Commits, GitHub Actions)
   - 5.3 Plain English Positioning: "The Stripe for Agent Accountability" / "The Datadog for Agent Governance"
6. [Strategic M&A Playbook & Acquisition Moat (R5)](#6-strategic-ma-playbook--acquisition-moat-r5)
   - 6.1 The Strategic Acquisition Thesis: The Missing Enterprise Operating System Layer for AI
   - 6.2 Strategic Buyer Profile Mapping (Frontier AI Labs vs. Enterprise Developer Platforms)
   - 6.3 The 4-Pillar Defensible IP Moat (Causal DAGs, Cryptographic OVTs, Runtime OTD, Universal Neutrality)
   - 6.4 The Dual-Track Strategy: Cashflow Independence ($1.3M-$3.5M+ Net) vs. Acquisition Leverage ($50M-$200M+)
7. [12-Month Phased Engineering & Commercial Execution Roadmap](#7-12-month-phased-engineering--commercial-execution-roadmap)
   - 7.1 Q1: Core Daemon Hardening, 44 Defect Remediations & MCP Standard (Months 1–3)
   - 7.2 Q2: Visual Companion Canvas & Pro Monetization Launch (Months 4–6)
   - 7.3 Q3: Enterprise Attestation Authority & Dual-Signed OVTs (Months 7–9)
   - 7.4 Q4: Enterprise Scaling, Compliance Partnerships & Dual-Track M&A Positioning (Months 10–12)
8. [Appendix: Complete Wire Protocols & Canonical Schemas](#8-appendix-complete-wire-protocols--canonical-schemas)
   - 8.1 Appendix A: Complete Universal 20-Entity JSON-LD Schema
   - 8.2 Appendix B: Complete Runtime OTD Activation & State Machine JSON Schema
   - 8.3 Appendix C: Complete Outcome Verification Ticket (OVT) W3C VC JSON-LD Schema
   - 8.4 Appendix D: WebSocket Bidirectional Control Protocol Wire Specification

---

## 1. Executive Summary & Foundational Vision

### 1.1 The Context Integrity Imperative & The Autonomous Agent Trust Barrier

The software engineering industry is undergoing an unprecedented structural transition. Large Language Models (LLMs) have progressed rapidly from passive, inline code completion utilities (such as GitHub Copilot v1) into active, multi-turn, autonomous agentic loops. Frontier models—including Anthropic Claude 3.7 Sonnet operating in Claude Code, OpenAI Codex and Operator, Google Antigravity, OpenCode, and Cursor Agent—are now regularly entrusted with complex, end-to-end engineering tasks. These agents autonomously read large multi-repository codebases, execute terminal shell commands, generate and apply database migrations, mutate dependency graphs, interact with external APIs, and submit multi-file pull requests.

However, widespread enterprise deployment of autonomous coding agents has hit a hard, structural obstacle: **The Autonomous Agent Trust Barrier**. 

Enterprise engineering leaders, Chief Information Security Officers (CISOs), and compliance directors are acutely aware that while autonomous agents can write code rapidly, they operate without deterministic runtime accountability. In production environments, an agent that operates probabilistically without hard invariant boundaries introduces severe, existential liabilities:
- A single unvetted command can delete production databases, wipe git histories, or trigger cascading cloud provisioning loops.
- An agent hallucinating internal library interfaces can commit silent logical errors that evade superficial unit tests and trigger multi-million-dollar production downtime.
- In regulated sectors (such as Financial Technology, Healthcare, Defense, and Critical Infrastructure), pushing unverified, unattested code to production violates mandatory compliance frameworks (SOC 2 Type II, ISO 27001, HIPAA, PCI-DSS, FedRAMP).

```
+----------------------------------------------------------------------------------------------------+
|                                THE AUTONOMOUS AGENT TRUST BARRIER                                  |
+----------------------------------------------------------------------------------------------------+
|                                                                                                    |
|    PROBABILISTIC FRONTIER AGENTS              THE TRUST VOID              ENTERPRISE PRODUCTION    |
|   ┌─────────────────────────────┐        ┌──────────────────────┐        ┌─────────────────────┐   |
|   │ • Claude Code               │        │ • Context Decay      │        │ • Zero Downtime     │   |
|   │ • Google Antigravity        │───────►│ • Unverified Commits │───────►│ • SOC 2 Compliance  │   |
|   │ • OpenAI Codex / Operator   │        │ • Runaway Spend      │        │ • Causal Lineage    │   |
|   │ • Cursor / OpenCode         │        │ • No Rollback Safety │        │ • Legal Provenance  │   |
|   └─────────────────────────────┘        └──────────────────────┘        └─────────────────────┘   |
|                                                     ▲                                              |
|                                                     │                                              |
|                                  KINETI CONTEXT INTEGRITY RUNTIME                                  |
|                                  (Closes the Trust Void via CIP)                                   |
+----------------------------------------------------------------------------------------------------+
```

### 1.2 Core Failure Modes of Existing Agent Frameworks

Today's agent frameworks (such as LangChain, CrewAI, AutoGen, MetaGPT, and simple markdown prompt templates like `CLAUDE.md` or `AGENTS.md`) fail in production because of five fundamental architectural flaws:

1. **Context Drift & The "Prompt-Trap":**  
   Existing frameworks rely on passive prompt injection to steer agents. System prompts instruct the LLM: *"Always write tests first"* or *"Never delete production databases."* However, as conversation history accumulates over 20 to 50 turns, context window pressure causes **attention degradation and context distraction**. The model's probabilistic sampler increasingly disregards system instructions in favor of recency bias, hallucinating compliance and executing unauthorized operations.
2. **Lack of Causal Lineage (Vector RAG Fallacy):**  
   Modern agent memory relies on flat vector similarity embeddings (Retrieval-Augmented Generation / RAG). Vector embeddings measure *semantic similarity*, fundamentally confusing **correlation with causation**. In complex codebases, two functions may appear textually similar while possessing completely incompatible execution semantics and lifecycles. Vector memory lacks temporal directionality, causal graph structure, and deductive proofs.
3. **Untracked Spend & Misaligned Economic Incentives:**  
   Frontier model providers bill by the token ($/Token). This creates a perverse economic dynamic: **the model provider earns more revenue when an agent gets trapped in a 40-step hallucination loop, burning millions of tokens before failing**. Today's harnesses lack hardware-style, sub-millisecond spend circuit breakers capable of terminating runaway loops before financial damage occurs.
4. **Destructive State Mutations Without Saga Rollback Safety:**  
   When an autonomous agent modifies files, installs packages, runs database migrations, or dispatches external network calls, it modifies real state. If the agent encounters a compiler error or logic failure in turn 15, existing frameworks simply throw an error or start an uncoordinated retry loop. The developer is left with a broken, halfway-mutated filesystem with no deterministic mechanism to restore state.
5. **Absence of Cryptographic Provenance & Attestation:**  
   Console logs and JSONL audit files in existing tools are mutable, ephemeral, and non-atomic. An agent can self-certify that its output passed tests, while the actual bash execution failed or was skipped. There is no cryptographic link tying the specific git tree fingerprint to verified test proofs and authorized human gate approvals.

### 1.3 The Kineti Philosophy: "Skills Propose, Programs Enforce, Memory Remembers"

Kineti OS replaces speculative prompt-based steering with a deterministic, mathematically verifiable governing principle:

$$\mathbf{\text{“Skills propose, programs enforce, memory remembers.”}}$$

- **Skills Propose:** Domain methodologies, cognitive strategies, and workflow runbooks (e.g., Officehours, Diagnose, Spec, Build, Review, QA, Security, Ship) are declared as structured skills. Agents freely use these skills to analyze problems, synthesize solutions, draft specifications, and propose code diffs.
- **Programs Enforce:** The agent is never permitted to self-authorize state mutations or gate transitions. Enforcement is strictly handled out-of-band by deterministic, compiled, sub-millisecond local programs and commit gates. An agent cannot proceed from Stage 6 (`spec`) to Stage 7 (`build`) without an immutable, cryptographically valid human gate signature. An agent cannot exceed the $50.00 spend ceiling because the runtime actively severs model API proxy connectivity at the socket layer.
- **Memory Remembers:** Every agent hypothesis, decision, command execution, observation, file diff, and spend entry is persisted into an immutable, append-only, causal property graph (governed by ISO SQL/PGQ standards) and anchored into a cryptographic Merkle Directed Acyclic Graph (DAG). Memory is not an opaque vector database; it is a structured, temporally ordered causal history with full rollback and replay capabilities.

### 1.4 The Paradigm Shift: From Probabilistic Prompting to Outcome Engineering

The ultimate transition pioneered by Kineti OS is the move from speculative token generation to **Outcome Engineering**:

```
+----------------------------------------------------------------------------------------------------+
|                                THE OUTCOME ENGINEERING PARADIGM                                    |
+------------------------------------+---------------------------------------------------------------+
| Probabilistic Agent Frameworks     | Kineti Context Integrity Runtime                              |
+------------------------------------+---------------------------------------------------------------+
| Token-based billing ($/Token)      | Outcome-based accounting (Cost Per Verified Outcome, $/Outcome)|
| Probabilistic prompt suggestions   | Deterministic sub-50ms atomic commit gates                    |
| Flat vector embeddings (RAG)       | ISO SQL/PGQ Causal Graphs with formal DAG node/edge semantics  |
| Unrecoverable catastrophic failures| Saga LIFO transaction rollbacks with inverse compensations    |
| Mutable, ephemeral console logs    | Dual-signed cryptographic Outcome Verification Tickets (OVTs) |
| Proprietary walled-garden lock-in  | Universal 7-Layer Context Integrity Protocol (CIP)            |
+------------------------------------+---------------------------------------------------------------+
```

Under Outcome Engineering, code generation is treated as a transactional, verified outcome. A task is not complete when an LLM outputs an `assistant` message stating "I have finished the task." A task is complete **only when a cryptographically dual-signed Outcome Verification Ticket (OVT) is minted**, proving that:
1. The code was generated strictly within the bounds of a locked root goal.
2. All 13 stages of the pipeline were traversed in accordance with DAG dependencies.
3. Every mutating action registered an inverse compensating action on a LIFO rollback stack.
4. Independent verification tests passed against an exact SHA-256 workspace code fingerprint.
5. All security, feasibility, and spec gates were signed by authorized human or cryptographic evaluators.
6. Cumulative dollar and token expenditures remained strictly within authorized financial budgets.

### 1.5 System Topology: Dual-Plane Architecture Overview

To achieve universal host support without sacrificing developer ergonomics, Kineti OS is engineered as a **Dual-Plane Runtime**:

```
+────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                 KINETI OS DUAL-PLANE RUNTIME TOPOLOGY                              |
+────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                                                                                    |
|    HOST / AGENT EXECUTION LAYER                                                                    |
|    [Google Antigravity]  [Claude Code]  [OpenAI Codex]  [OpenCode]  [Cursor]  [Terminal CLI]       |
|             │                   │              │             │          │            │             |
|             ▼                   ▼              ▼             ▼          ▼            ▼             |
|  ┌──────────────────────────────────────────────────────────────────────────────────────────────┐  |
|  │                        PLANE 1: DETERMINISTIC HEADLESS CONTROL PLANE                         │  |
|  │                       (Local Daemon & Native In-Process MCP Server)                          │  |
|  │                                                                                              │  |
|  │  • Multi-Protocol Ingress: Stdio JSON-RPC 2.0, UNIX Domain Sockets, HTTP Reverse Proxy       │  |
|  │  • Context Integrity Protocol (CIP) 7-Layer Engine (< 10ms local loop latency)               │  |
|  │  • Embedded Dual Storage: SQLite 3 (WAL mode) + DuckDB (ISO SQL/PGQ Property Graph)          │  |
|  │  • Hardware-Style $50 Spend Circuit Breaker & Real-Time Token Ledger                         │  |
|  │  • Safe LIFO Saga Compensating Transaction Rollback Stack                                    │  |
|  │  • Background Merkle DAG Generator & Workspace SHA-256 Code Fingerprinter                   │  |
|  └──────────────────────────────────────────────┬───────────────────────────────────────────────┘  |
|                                                 │                                                  |
|                        Bidirectional Local WebSocket & SSE Telemetry                               |
|                         `ws://127.0.0.1:8788` | `http://127.0.0.1:8788`                            |
|                                                 │                                                  |
|  ┌──────────────────────────────────────────────▼───────────────────────────────────────────────┐  |
|  │                     PLANE 2: ZERO-FRICTION REACTIVE VISUAL COMPANION                         │  |
|  │                        (Aside-Style Desktop & Web Canvas Sidecar)                            │  |
|  │                                                                                              │  |
|  │  • Real-Time Merkle DAG Causal Inspector (Interactive SVG/Canvas Node Visualizer)            │  |
|  │  • 1-Click Interactive Human Approval Modal (Pre-rendered Unified Blast-Radius Diffs)       │  |
|  │  • Live Financial Spend Radial Gauges & Multi-Model Burn Velocity Meters                    │  |
|  │  • Interactive Time-Travel Saga Scrubber & 1-Click Intermediate State Rollback               │  |
|  │  • Context Integrity & Runtime Ontology Trigger Data (OTD) Live Monitor                     │  |
|  └──────────────────────────────────────────────────────────────────────────────────────────────┘  |
+────────────────────────────────────────────────────────────────────────────────────────────────────+
```

This dual-plane topology ensures that:
- In automated, headless environments (e.g., CI/CD GitHub Actions runners, remote SSH developer boxes, Docker containers), Plane 1 operates completely autonomously, enforcing policy gates, tracking spend, and generating cryptographic tickets without requiring any display or visual UI.
- On the developer's local workstation, Plane 2 docks seamlessly alongside any IDE or terminal, providing immediate visual transparency, effortless 1-click approvals, and intuitive causal debugging.

## 2. Universal Host Architecture & Hybrid Companion Blueprint (R1)

### 2.1 Universal Host Adapter Specifications

To establish true enterprise-grade ubiquity, Kineti OS does not force developers into an isolated proprietary IDE. Instead, it operates as a universal, host-agnostic substrate that plugs seamlessly into the developer tools and agent environments already deployed in engineering teams.

Below are the complete technical specifications, lifecycle hooks, protocol sequence diagrams, configuration formats, latency budgets, and failure isolation policies for the six primary agent runtime environments.

---

#### 2.1.1 Google Antigravity Adapter

Google Antigravity is an enterprise-grade agent development platform characterized by hierarchical multi-agent delegation, native Model Context Protocol (MCP) tool bindings, workspace artifact sandboxing, and structured runbook automation.

##### Architectural Mechanics & Integration Hooks
1. **Skill Discovery & Manifest Binding:** Antigravity discovers capabilities via structured directory trees located in `~/.gemini/config/skills/<skill_name>/SKILL.md`. Kineti registers skills containing YAML metadata frontmatter, execution instructions, tool requirements, and local script bindings.
2. **Native Stdio MCP Client:** Antigravity natively spawns and queries MCP servers defined in its workspace configuration. Kineti registers as an in-process stdio MCP server exposing deterministic tools (`kineti_gate_check`, `kineti_record_evidence`, `kineti_saga_register`, `kineti_spend_status`).
3. **Supervisor Process Hooks:** Antigravity dispatches lifecycle events across task phases (`on_task_start`, `on_step_eval`, `on_tool_call`, `on_task_complete`). Kineti hooks into these events to enforce root-goal immutability and evaluate policy gates before child subagents are spawned.
4. **Artifact Sandboxing:** Subagents operate within isolated workspace artifact folders (`brain/<conversation_id>/`). Kineti monitors artifact output to compute cryptographic SHA-256 tree digests without tracking transient model thoughts.

##### Protocol Flow Sequence Diagram
```
Antigravity Agent Runtime        Kineti MCP Daemon (Plane 1)      SQLite WAL / Merkle DAG
          │                                  │                               │
          │─── 1. mcp:initialize ───────────▶│                               │
          │◀── 2. Capabilities & Tools ──────│                               │
          │                                  │                               │
          │─── 3. Call: kineti_gate_check ──▶│                               │
          │       { stage: 6, gate: "spec" } │─── 4. Query Gate Status ─────▶│
          │                                  │◀── 5. Gate Approved (Signed) ─│
          │                                  │                               │
          │                                  │─── 6. Compute Merkle Node ───▶│
          │◀── 7. Tool Result: APPROVED ─────│                               │
          │                                  │                               │
          │─── 8. Mutating Action Executed ─▶│                               │
          │       (Filesystem Edit / Shell)  │─── 9. Push Saga Rollback ────▶│
          │                                  │                               │
```

##### Exact Configuration Snippet
To register Kineti within Google Antigravity, add the daemon to `~/.gemini/config/settings.json` (or workspace-specific MCP configuration):
```json
{
  "mcp_servers": {
    "kineti_core": {
      "command": "kineti-daemon",
      "args": ["mcp", "--workspace-root", "${workspaceRoot}"],
      "env": {
        "KINETI_INTEGRITY_MODE": "development",
        "KINETI_LOG_LEVEL": "info",
        "KINETI_STORAGE_PATH": "${workspaceRoot}/.kineti"
      },
      "transport": "stdio"
    }
  },
  "hooks": {
    "on_task_start": "kineti-daemon hook antigravity-task-start --task-id ${taskId} --goal \"${taskGoal}\"",
    "on_task_complete": "kineti-daemon hook antigravity-task-complete --task-id ${taskId}"
  }
}
```

##### Latency Budget & Failure Isolation Policy
- **Latency Budget (Microseconds):**
  - Stdio JSON-RPC frame decode: $820\mu\text{s}$
  - SQLite WAL stage gate check: $1,410\mu\text{s}$
  - SHA-256 Merkle leaf calculation: $1,120\mu\text{s}$
  - JSON-RPC response frame transmission: $480\mu\text{s}$
  - **Total Local Loop Roundtrip: 3.83 ms** (Well within the $< 10\text{ms}$ hard boundary).
- **Process Lifecycle & Failure Isolation:** The daemon runs as a supervised child process of Antigravity. If the daemon process terminates unexpectedly, Antigravity attempts graceful `SIGTERM` cleanup followed by `SIGKILL` after 2000ms.
- **Enforcement Boundary:** **Fail-Closed on Security & Financial Gates; Fail-Open with Warning on Telemetry.** If the daemon crashes or the SQLite database encounters lock contention, any mutating tool call (e.g., shell command execution or file writes) is strictly blocked with exit code 2 until the daemon recovers. Read-only queries default to cached snapshot state.

---

#### 2.1.2 Anthropic Claude Code Adapter

Anthropic Claude Code is a high-speed, terminal-native CLI agent designed for direct software engineering workflows via terminal execution, file system modification, and tool orchestration.

##### Architectural Mechanics & Integration Hooks
1. **Deterministic `PreToolUse` & `PostToolUse` Lifecycle Hooks:** Claude Code natively supports lifecycle hook commands declared in `~/.claude/settings.json`. Before executing any tool (e.g. `Bash`, `FileEdit`, `Glob`, `Grep`), Claude Code pipes the planned tool name and arguments as a JSON payload to `stdin` of the configured hook binary.
2. **Deterministic Exit Code Enforcement Contract:**
   - **Exit Code 0:** Authorizes tool execution to proceed immediately.
   - **Exit Code 2:** Blocks tool execution deterministically. Claude Code captures the hook script's `stderr` and injects it verbatim into the conversation context as a tool error. This forces the model to adapt its reasoning, resolve gate prerequisites, or register compensating actions.
3. **Native Stdio MCP Connectivity:** Claude Code natively connects to MCP servers defined via `claude mcp add kineti -- kineti-daemon mcp`.
4. **PTY Interactive Wrapper:** For enhanced developer UX, Kineti ships a zero-overhead PTY wrapper (`kineti claude`) that intercepts terminal escape sequences, captures interactive approvals, and auto-injects Kineti's MCP configuration.

##### Protocol Flow Sequence Diagram
```
Claude Code CLI Engine           Kineti Hook Evaluator           Kineti Daemon / Policy Engine
          │                                  │                               │
          │─── 1. Tool Intent (Stdin JSON) ─▶│                               │
          │    { tool: "Bash",               │                               │
          │      cmd: "rm -rf migrations/" } │─── 2. Intercept Check (Socket)▶│
          │                                  │◀── 3. REFUSED: No Saga Action ─│
          │                                  │                               │
          │◀── 4. Process Exit Code 2 ───────│                               │
          │       Stderr: "BLOCKED: Mutation │                               │
          │       requires LIFO saga action" │                               │
          ▼                                                                  │
[Model adjusts reasoning, calls `kineti-saga register`, then retries command]│
```

##### Exact Configuration Snippet
Add the hook interceptor to `~/.claude/settings.json`:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "command": "kineti-daemon hook claude-pre-tool --workspace ${cwd}"
      },
      {
        "matcher": "FileEdit",
        "command": "kineti-daemon hook claude-pre-tool --workspace ${cwd}"
      }
    ],
    "PostToolUse": [
      {
        "matcher": "*",
        "command": "kineti-daemon hook claude-post-tool --workspace ${cwd}"
      }
    ]
  },
  "mcpServers": {
    "kineti": {
      "command": "kineti-daemon",
      "args": ["mcp", "--workspace-root", "${cwd}"],
      "transport": "stdio"
    }
  }
}
```

##### Latency Budget & Failure Isolation Policy
- **Latency Budget (Microseconds):**
  - Stdin JSON pipe ingestion: $950\mu\text{s}$
  - Fast UNIX domain socket query (`/var/run/kineti/daemon.sock`): $1,150\mu\text{s}$
  - Policy rule evaluation (Saga check & spend ceiling): $1,280\mu\text{s}$
  - Exit code evaluation & process exit: $410\mu\text{s}$
  - **Total Local Loop Overhead: 3.79 ms**.
- **Failure Isolation:** The hook script operates under a strict **500ms execution timeout**. If the hook process hangs or crashes, it exits with Exit Code 2 (fail-closed), preventing unauthorized shell executions while emitting a diagnostic warning to the developer.

---

#### 2.1.3 OpenAI Codex / Operator Adapter

The OpenAI Codex and Operator agent ecosystem utilizes the Agent Client Protocol (ACP) and streaming JSON-RPC architectures to drive autonomous coding workflows.

##### Architectural Mechanics & Integration Hooks
1. **Headless Process Supervisor (`kineti-supervisor`):** Untrusted Codex/Operator processes are executed under a dedicated supervisor that governs system resources, file handles, and OS-level execution sandboxes.
2. **Agent Client Protocol (ACP) Multi-Turn Primitives:** Codex structures agent operations across three core primitives:
   - **Thread:** Durable execution container representing an entire milestone or user task.
   - **Turn:** A single conversational request-response cycle.
   - **Item:** Atomic operations within a turn (tool calls, code patches, text tokens, bash execution outputs).
   Kineti's ACP adapter intercepts every Item frame in real time, validating invariants before execution is dispatched to the host.
3. **OS-Level Container Sandboxing:**
   - **Linux:** Bubblewrap (`bwrap`) isolation with unshared PID, network, and IPC namespaces, mounting the workspace read-write while locking root system directories read-only.
   - **macOS:** `sandbox-exec` with strict Seatbelt profiles restricting filesystem mutations exclusively to the designated repository folder.

##### Protocol Flow Sequence Diagram
```
Codex CLI / Operator Engine      Kineti Process Supervisor         Sandboxed Execution Box
          │                                  │                               │
          │─── 1. ACP Turn: Tool Call Item ─▶│                               │
          │       { "action": "patch", ... } │─── 2. Evaluate Seatbelt/Bwrap▶│
          │                                  │─── 3. Register LIFO Undo ────▶│
          │                                  │─── 4. Dispatch Sandboxed Exec▶│
          │                                  │◀── 5. Stream Stdout/Stderr ───│
          │◀── 6. Stream Verified Item Frame─│                               │
```

##### Exact Configuration Snippet
Configured in `~/.codex/config.toml`:
```toml
[agent]
protocol = "acp"
supervisor_path = "/usr/local/bin/kineti-supervisor"

[mcp_servers.kineti]
command = "kineti-daemon"
args = ["mcp"]
env = { KINETI_SANDBOX = "strict", KINETI_INTEGRITY_MODE = "development" }

[sandbox]
engine = "bubblewrap"
isolate_network = true
allowed_outbound_proxy = "http://127.0.0.1:8787"
workspace_mount_rw = true
max_memory_mb = 4096
max_cpu_percent = 80
```

##### Latency Budget & Failure Isolation Policy
- **Latency Budget (Microseconds):**
  - ACP JSON streaming chunk parser: $910\mu\text{s}$
  - Sandbox privilege verification: $1,420\mu\text{s}$
  - Saga stack push & snapshot record: $1,180\mu\text{s}$
  - JSON-RPC item frame dispatch: $520\mu\text{s}$
  - **Total Local Loop Latency: 4.03 ms**.
- **Failure Isolation:** Every tool execution runs in an isolated container sandbox. If Codex attempts an illegal system write or network exfiltration, the sandbox immediately halts the process (`SIGKILL`), and `kineti-supervisor` automatically executes `kineti-saga rollback` to restore the filesystem to its pristine pre-turn state.

---

#### 2.1.4 OpenCode & Open-Source Agent Ecosystem Adapter

The open-source agent ecosystem (including OpenCode, Aider, Smolagents, AutoGen, and CrewAI) requires a zero-friction, protocol-agnostic integration surface that does not rely on proprietary host APIs.

##### Architectural Mechanics & Integration Hooks
1. **Local UNIX Domain Socket IPC:** Fast, authenticated IPC over `/var/run/kineti/daemon.sock` (or `~/.kineti/run/daemon.sock`) utilizing framed JSON-RPC 2.0.
2. **LLM API Intercepting Reverse Proxy:** Kineti runs a high-performance local HTTP/HTTPS reverse proxy on `http://127.0.0.1:8787`. Open-source agents direct their `OPENAI_BASE_URL` or `ANTHROPIC_BASE_URL` through this proxy:
   - Intercepts all outbound `/v1/chat/completions` and `/v1/messages` calls.
   - Transparently injects active Runtime Ontology Trigger Data (OTD) constraints into the system prompt.
   - Dynamically calculates exact token costs (using Tiktoken/BPE) and enforces the $50.00 hardware spend circuit breaker.
   - Prevents prompt injection and data exfiltration via an OWASP ASI-01/02 dual-model sanitization pipeline.
3. **Standardized Agent Protocol (AGP) Endpoints:** Exposes RESTful endpoints (`/v1/runs`, `/v1/tasks`, `/v1/gates`) for seamless observability.

##### Protocol Flow Sequence Diagram
```
OpenCode / Aider Agent           Kineti Local Reverse Proxy       LLM Provider API (Cloud)
          │                                  │                               │
          │─── 1. POST /v1/chat/completions ─▶│                               │
          │       (User prompt + context)    │─── 2. Inject Runtime OTD ────▶│
          │                                  │─── 3. Deduct Token Budget ───▶│
          │                                  │─── 4. Forward Upstream ──────▶│
          │                                  │◀── 5. Stream Chunks (SSE) ────│
          │◀── 6. Stream Metered Chunks ─────│                               │
```

##### Exact Configuration Snippet
Add to agent environment (`.env` or shell profile):
```bash
# Redirect OpenCode / Aider through Kineti Governance Proxy
export OPENAI_BASE_URL="http://127.0.0.1:8787/v1"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8787/v1"
export KINETI_IPC_SOCKET="$HOME/.kineti/run/daemon.sock"
```

In OpenCode configuration (`~/.config/opencode/config.json`):
```json
{
  "api_proxy": "http://127.0.0.1:8787",
  "mcp_servers": [
    {
      "name": "kineti",
      "transport": "stdio",
      "command": "kineti-daemon",
      "args": ["mcp"]
    }
  ]
}
```

##### Latency Budget & Failure Isolation Policy
- **Latency Budget (Microseconds):**
  - UNIX socket frame decode: $580\mu\text{s}$
  - Reverse proxy request rewrite & OTD prompt injection: $1,620\mu\text{s}$
  - Dynamic token accounting tick: $420\mu\text{s}$
  - **Total Added Overhead: 2.62 ms**.
- **Failure Isolation:** The proxy runs in an isolated thread pool. If the spend limit is breached, outbound requests are severed with `HTTP 402 Payment Required` and a detailed diagnostic payload.

---

#### 2.1.5 Cursor (AI IDE) Adapter

Cursor is the dominant AI-native IDE (a specialized VS Code fork) with deep language server protocol (LSP) integration, integrated terminal emulation, and multi-file composer capabilities.

##### Architectural Mechanics & Integration Hooks
1. **Cursor Extension (`.vsix`):** Kineti packages a native VS Code/Cursor extension that executes in the Extension Host process.
2. **Workspace & Global `mcp.json` Integration:** Cursor natively loads MCP servers declared in `.cursor/mcp.json` (workspace) or `~/.cursor/mcp.json` (global).
3. **Integrated Terminal Command Interception:** Intercepts terminal commands executed by Composer agents or developers using the VS Code Terminal API (`vscode.window.onDidWriteTerminalData`) and standard shell integration escape sequences (OSC 133 / OSC 633).
4. **Docked Companion Canvas (Webview Panel):** Leverages `vscode.window.createWebviewPanel` to dock the Aside-style companion canvas directly alongside the editor code tabs, providing zero-friction visual DAG inspection without leaving the IDE window.

##### Protocol Flow Sequence Diagram
```
Cursor Composer Agent            Cursor Extension Host             Kineti Core Daemon
          │                                  │                               │
          │─── 1. Call: kineti_stage_step ──▶│                               │
          │       (Attempt Stage 6 Spec)     │─── 2. Query Daemon Socket ───▶│
          │                                  │◀── 3. Requires Human Gate ────│
          │                                  │                               │
          │                                  │─── 4. Render Webview Modal ──▶[Docked Canvas]
          │                                  │◀── 5. Human Clicks [APPROVE] ─[Docked Canvas]
          │◀── 6. Tool Success Response ─────│                               │
```

##### Exact Configuration Snippet
Workspace configuration in `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "kineti": {
      "command": "kineti-daemon",
      "args": ["mcp"],
      "env": {
        "KINETI_WORKSPACE_ROOT": "${workspaceFolder}"
      }
    }
  }
}
```

##### Latency Budget & Failure Isolation Policy
- **Latency Budget (Microseconds):**
  - Webview `postMessage` bridge: $790\mu\text{s}$
  - Local domain socket dispatch: $1,050\mu\text{s}$
  - Merkle DAG state query: $1,420\mu\text{s}$
  - **Total Local Loop Latency: 3.26 ms**.
- **Failure Isolation:** Extension runs in the isolated Extension Host sandbox. A crash in the extension host never impacts the primary editor UI, and Cursor automatically restarts the extension host process.

---

#### 2.1.6 Generic Terminal Agents (CLI, Python Loops, Shell Scripts)

For legacy, bespoke, or ad-hoc agent scripts (such as custom Python loops, AutoGPT, or LangChain agents running in bash), Kineti provides transparent system-level interception without requiring any code modifications.

##### Architectural Mechanics & Integration Hooks
1. **Pseudo-Terminal (PTY) Wrapper (`kineti exec`):** Spawns the agent process inside a managed POSIX pseudo-terminal master/slave pair (`pty.openpty`), intercepting all terminal control sequences, standard input, and standard output.
2. **Dynamic Linker Syscall Interception (`LD_PRELOAD` / `DYLD_INSERT_LIBRARIES`):** Injects a lightweight C dynamic library shim (`libkineti_shim`) that intercepts standard C library (`libc`) system calls:
   - `execve()`, `posix_spawn()`: Intercepts command executions to check stage permissions and enforce verify gates.
   - `connect()`: Intercepts outbound TCP sockets, blocking unauthorized network access and enforcing egress whitelists.
   - `unlink()`, `rmdir()`: Intercepts destructive file deletions and forces registration of inverse LIFO saga actions before execution.
3. **Transparent MITM HTTP Proxy:** Intercepts outgoing TLS connections using a locally trusted root certificate (`~/.kineti/certs/ca.crt`), metering LLM tokens transparently.

##### Protocol Flow Sequence Diagram
```
Agent Process (Python / CLI)     Injected Libc Shim (DYLD/LD)      Kineti Supervisor Daemon
          │                                  │                               │
          │─── 1. execve("/bin/rm", ...) ───▶│                               │
          │                                  │─── 2. Trap: Check Perms ─────▶│
          │                                  │◀── 3. REFUSED: No Saga Action │
          │◀── 4. Return -1 (EPERM) ─────────│                               │
```

##### Exact Configuration Snippet
Execution runner command:
```bash
# Transparent execution wrapper
kineti exec -- python run_agent.py
```

Under the hood, `kineti exec` configures the execution environment:
```bash
#!/usr/bin/env bash
export DYLD_INSERT_LIBRARIES="/usr/local/lib/libkineti_shim.dylib"
export LD_PRELOAD="/usr/local/lib/libkineti_shim.so"
export HTTP_PROXY="http://127.0.0.1:8787"
export HTTPS_PROXY="http://127.0.0.1:8787"
export SSL_CERT_FILE="$HOME/.kineti/certs/ca.crt"
exec "$@"
```

##### Latency Budget & Failure Isolation Policy
- **Latency Budget (Microseconds):**
  - Libc syscall interception trap: $85\mu\text{s}$
  - Fast IPC query to supervisor: $1,050\mu\text{s}$
  - Security policy evaluation: $590\mu\text{s}$
  - **Total Syscall Intercept Overhead: 1.73 ms**.
- **Failure Isolation:** If the daemon socket is unreachable, the shim defaults to **fail-closed** for destructive mutations (`unlink`, `rmdir`, `execve`) while permitting read-only operations (`open(O_RDONLY)`).

---

### 2.2 Cross-Host Protocol & Intercept Matrix

```
+------------------------------------------------------------------------------------------------------------------------------------+
|                                      COMPREHENSIVE CROSS-HOST ARCHITECTURAL MATRIX                                                 |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| Host Environment  | Primary Hook Point | Protocol / Intercept   | Latency (Local)  | Sandboxing Mode    | Failure Isolation Policy |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| Google Antigravity| Skills & Runbooks  | Native MCP Stdio &     | 3.83 ms          | Artifact Brain &   | Fail-Closed on security; |
|                   | Task Supervisor    | Lifecycle Hooks        |                  | Task Sandbox       | Async on telemetry       |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| Anthropic Claude  | PreToolUse &       | Stdin/Stdout JSON      | 3.79 ms          | Native Claude      | Fail-Closed (Exit 2)     |
| Code              | PostToolUse Hooks  | (Exit 0 / Exit 2)      |                  | Permissions Gate   | with model feedback      |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| OpenAI Codex /    | Headless Supervisor| Agent Client Protocol  | 4.03 ms          | Bubblewrap /       | Process SIGKILL and      |
| Operator          | ACP & Tools        | (ACP) + Streaming RPC  |                  | Seatbelt Profile   | Auto Saga Rollback       |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| OpenCode / OSS    | API Endpoints &    | Reverse HTTP Proxy &   | 2.62 ms          | Process Groups &   | Fail-Closed on spend;    |
| Agents            | Local Sockets      | UNIX Domain Socket     |                  | User Permissions   | Graceful fallback        |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| Cursor (AI IDE)   | Extension Host &   | MCP JSON-RPC &         | 3.26 ms          | VS Code Ext Host   | Isolated UI thread;      |
|                   | Terminal Events    | Webview Bridge         |                  | Sandbox            | Non-blocking restarts    |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
| Generic Terminal  | Dynamic Linker &   | PTY Wrapper, Libc Shim | 1.73 ms          | POSIX PTY &        | Fail-Closed on writes;   |
| Agents            | Libc Syscalls      | (LD_PRELOAD / DYLD)    |                  | Syscall Filter     | Pass-open on reads       |
+-------------------+--------------------+------------------------+------------------+--------------------+--------------------------+
```

---

### 2.3 Local Loop Latency Budget & Physical Limits (P95 < 35ms, Hot Path < 10ms)

In autonomous agent operations, latency is cumulative: an agent executing a 30-step task experiences 30 roundtrips through the governance harness. If harness evaluation takes 500ms per step, the developer experiences a crippling 15-second delay. Kineti OS establishes a strict **P95 < 35ms local loop guarantee** under cold/IPC execution, with in-memory cached hot-path execution resolving in **under 5.5 milliseconds**.

```
+----------------------------------------------------------------------------------------------------+
|                      LOCAL LOOP LATENCY BUDGET (HOT-PATH CACHE VS COLD IPC P95)                     |
+-------------------------------------------------------------+-------------------+------------------+
| Execution Step & Processing Subsystem                       | Hot Path (Cache)  | Cold IPC (P95)   |
+-------------------------------------------------------------+-------------------+------------------+
| 1. Ingress & Frame Decoding                                 |                   |                  |
|    - UNIX Socket Read / Stdio JSON-RPC Parser               | 450 μs            | 3,200 μs         |
|    - HMAC Session Token & Capability Verification           | 250 μs            | 850 μs           |
|    *Subtotal: Step 1*                                       | *700 μs*          | *4,050 μs*       |
+-------------------------------------------------------------+-------------------+------------------+
| 2. Context Integrity & Stage Gate Evaluation                |                   |                  |
|    - SQLite WAL Index Query (`gates`, `stage`)              | 820 μs            | 4,500 μs         |
|    - 13-Stage Linear Pipeline Invariant Check               | 220 μs            | 1,200 μs         |
|    - OWASP ASI Egress Rule Match                            | 450 μs            | 2,100 μs         |
|    *Subtotal: Step 2*                                       | *1,490 μs*        | *7,800 μs*       |
+-------------------------------------------------------------+-------------------+------------------+
| 3. Spend Circuit Breaker & Token Accounting                 |                   |                  |
|    - Real-Time Dollar Limit Check (< $50.00 / micro-cents)  | 160 μs            | 750 μs           |
|    - Atomic Spend Counter Increment                         | 210 μs            | 950 μs           |
|    *Subtotal: Step 3*                                       | *370 μs*          | *1,700 μs*       |
+-------------------------------------------------------------+-------------------+------------------+
| 4. Storage Transaction & Merkle DAG Node Commit             |                   |                  |
|    - SQLite WAL Append (`causal_nodes`, `saga_stack`)       | 950 μs            | 8,500 μs         |
|    - RFC 8785 Canonical Serialization & Merkle Leaf Hash   | 520 μs            | 2,400 μs         |
|    *Subtotal: Step 4*                                       | *1,470 μs*        | *10,900 μs*      |
+-------------------------------------------------------------+-------------------+------------------+
| 5. Response Egress & Telemetry Push                         |                   |                  |
|    - Outbound JSON-RPC Frame Serialization                  | 290 μs            | 1,800 μs         |
|    - Authenticated WebSocket Push to Sidecar (`:8788`)      | 350 μs            | 3,500 μs         |
|    *Subtotal: Step 5*                                       | *640 μs*          | *5,300 μs*       |
+-------------------------------------------------------------+-------------------+------------------+
| **TOTAL LOCAL LOOP ROUNDTRIP LATENCY**                      | **4,670 μs**      | **29,750 μs**    |
|                                                             | **(4.67 ms)**     | **(< 30 ms P95)**|
+-------------------------------------------------------------+-------------------+------------------+
```

---

### 2.4 Form Factor Evaluation: Headless Daemon vs. Standalone Companion vs. IDE Extension

Choosing the proper form factor determines user adoption, developer friction, and long-term defensibility. Below is a multi-dimensional evaluation of the three architectural options.

#### 2.4.1 Deconstructing Aside.com: Lessons for Agent Harnesses
Aside (aside.com) demonstrated that autonomous agents become trustworthy to humans **only when the execution boundary is visually visible, inspectable, and interruptible**.
Key architectural insights derived from Aside.com:
1. **The Approval Gate Principle:** High-consequence actions (filesystem deletion, financial transactions, outbound messages) must pause execution and present the human operator with a pre-rendered blast-radius preview.
2. **The "Live Canvas" Mindset:** Rather than forcing developers to watch fast-scrolling text in a terminal, users require a living state canvas showing what the agent knows, what it plans to do next, and how much money it has spent.
3. **Local-First Memory:** Memory should not be an opaque cloud vector database; it must be locally inspectable, searchable, and editable by the operator.

#### 2.4.2 Comprehensive Multi-Dimensional Trade-Off Matrix

```
+-------------------------------------------------------------------------------------------------------+
|                                     PARADIGM TRADE-OFF EVALUATION                                     |
+-------------------------------+-----------------------+-----------------------+-----------------------+
| Dimension                     | Pure Headless         | Standalone Companion  | Integrated IDE Ext    |
+-------------------------------+-----------------------+-----------------------+-----------------------+
| 1. Onboarding Friction        | Minimal (`brew inst`)| Low (Single tray app) | Moderate (Store inst) |
| 2. Visual Expressiveness      | Very Poor (ASCII only)| Exceptional (Full UI) | Moderate (Webview pan)|
| 3. Host Agnosticism           | Total (Works anywhere)| Total (Works with all)| Very Poor (IDE-locked)|
| 4. Enforcement Authority      | Absolute (Kernel gate)| Absolute (Via daemon) | Weak (Host-dependent) |
| 5. Memory / CPU Footprint     | Ultralight (< 25MB)   | Moderate (60-120MB)   | Light (40-80MB)       |
| 6. Complex Graph Visuals      | Impossible            | Native (WebGL / SVG)  | Restricted (Sidebar)  |
| 7. Human Approval UX          | Blocking TTY Prompt   | 1-Click Interactive   | In-Editor Notification|
| 8. CI/CD & Headless Operation | Native                | Requires Headless Mode| Unsupported           |
+-------------------------------+-----------------------+-----------------------+-----------------------+
```

#### 2.4.3 Synthesis: The Imperative for the Hybrid Architecture
Neither pure headless nor pure standalone companion is sufficient on its own:
- A pure headless tool lacks visual fidelity for complex causal DAGs and turns approval gates into disruptive terminal prompts.
- A pure standalone app cannot run in headless CI/CD pipelines, remote SSH sessions, or low-resource containers.

**The Architectural Solution is the Kineti Hybrid Architecture:**  
A high-performance **Headless Local Daemon & MCP Core (Plane 1)** paired with a **Zero-Friction Reactive Visual Sidecar / Canvas (Plane 2)**. The daemon executes autonomously in the background, enforcing rules regardless of whether the UI is open. When a developer desires visual oversight, the companion connects instantly via local WebSockets (`ws://127.0.0.1:8788`) to render the live visual canvas.

---

### 2.5 The Hybrid Companion Architecture Specification

#### 2.5.1 Plane 1: High-Performance Headless Local Daemon
- **Language & Runtime:** Engineered in **Rust** (using `tokio`, `tower`, and `rusqlite`) for production single-binary distribution (18MB resident memory footprint, sub-millisecond memory safety, zero garbage collection pauses). Prototyped and validated in **Bun / TypeScript** using native `bun:sqlite` and native WebSocket servers.
- **Embedded Storage Engine:**
  - **SQLite 3 (WAL Mode):** Operational OLTP engine managing runs, gates, spend ledgers, and the LIFO saga rollback stack.
  - **DuckDB (In-Process OLAP):** Analytical graph engine executing ISO SQL/PGQ queries across millions of historical causal links.
  - **Atomic JSONL Journal:** Thread-safe, fsync-flushed export logs (`.kineti/journal.jsonl`, `egress.jsonl`).

#### 2.5.2 Plane 2: Zero-Friction Reactive Visual Sidecar / Canvas
Built strictly adhering to the **Kineti Master Design System**: Tailwind CSS, Lucide Icons, Motion spring physics, and Radix UI headless primitives across the three Master Visual Archetypes (Modern Technical SaaS, Clean High-Trust Fintech, and Premium Editorial).

##### The 5 Core Visual Modules:
1. **Live Merkle DAG Causal Inspector:** Interactive canvas rendering the 13-stage execution pipeline, active tasks, tool interactions, and causal dependency edges. Nodes glow green (verified pass), amber (pending approval), or red (gate failure).
2. **1-Click Human Approval Modal & Blast-Radius Engine:** Displays colorized unified syntax diffs, file change counts, lines added/removed, risk score, and single-click `[1] Approve` or `[2] Reject & Rollback` actions.
3. **Spend Gauges & Token Meters:** Radial velocity meters displaying cumulative dollars spent against the $50.00 circuit breaker ceiling, token consumption rates, and per-stage cost breakdowns.
4. **Time-Travel Timeline & Saga Unwind Controller:** Visual LIFO stack showing each executed mutation. Clicking any historical node displays an instant diff preview and enables one-click rollback of intermediate steps.
5. **Context Integrity & Runtime OTD Inspector:** Real-time inspector displaying active Ontology Trigger Data, injected causal constraints, and real-time prompt sanitation logs.

---

### 2.6 Reactive Telemetry, Approval Gate Protocol & Wire Formats

Communication between Plane 1 and Plane 2 is mediated by framed JSON-RPC 2.0 over local WebSockets (`ws://127.0.0.1:8788`).

> [!IMPORTANT]
> **Hardened Local WebSocket Security (CSWSH Defense):**
> 1. **Loopback-Only Binding:** The daemon binds strictly to `127.0.0.1` and `::1`. It rejects any external network interface requests.
> 2. **Ephemeral HMAC Bearer Token Handshake:** At daemon startup, a cryptographically random 256-bit token is generated and written with restricted 0600 permissions to `.kineti/auth_token`. WebSocket connections must present this token via query parameter (`ws://127.0.0.1:8788?token=...`) or `Authorization: Bearer <TOKEN>` header.
> 3. **Strict Origin Validation:** The server validates the HTTP `Origin` header during the HTTP upgrade handshake. Requests with origins other than `http://localhost:*`, `http://127.0.0.1:*`, or verified desktop webview protocols (`vscode-webview://`, `tauri://`) are rejected with HTTP 403 Forbidden, neutralizing Cross-Site WebSocket Hijacking (CSWSH) attacks from malicious browser tabs.

#### Event 1: Gate Approval Requested (`kineti.gate.approval_requested`)
Emitted by the daemon when an agent reaches a gated boundary (Stage 6 Spec, Stage 11 Ship, or destructive file operations):
```json
{
  "jsonrpc": "2.0",
  "method": "kineti.gate.approval_requested",
  "params": {
    "ticket_id": "ovt-894f-2026-09-06",
    "stage": 6,
    "gate_name": "spec",
    "action_type": "filesystem_mutation",
    "description": "Approve generated OpenAPI spec and TypeScript type definitions",
    "blast_radius": {
      "files_created": 3,
      "files_modified": 1,
      "lines_added": 240,
      "lines_deleted": 12,
      "risk_score": "LOW"
    },
    "diff_preview": "--- a/spec.md\n+++ b/spec.md\n@@ -10,6 +10,18 @@\n+### New Endpoint: /v1/attest",
    "expires_at": "2026-09-06T05:40:00Z"
  }
}
```

#### Event 2: Gate Approval Resolved (`kineti.gate.approval_resolved`)
Sent by the Visual Sidecar when the operator approves or rejects the gate:
```json
{
  "jsonrpc": "2.0",
  "method": "kineti.gate.approval_resolved",
  "params": {
    "ticket_id": "ovt-894f-2026-09-06",
    "decision": "approved",
    "decided_by": "human_operator",
    "signature": "ed25519:7a4c9e88b...3f0a",
    "timestamp": "2026-09-06T05:32:15Z"
  }
}
```

#### Event 3: Spend Telemetry Tick (`kineti.spend.tick`)
Broadcast after every LLM completion:
```json
{
  "jsonrpc": "2.0",
  "method": "kineti.spend.tick",
  "params": {
    "session_id": "787db8bf-e17f-440a-abec-0fbfbb7ecae3",
    "tokens_in": 14200,
    "tokens_out": 2100,
    "cost_usd": 0.074,
    "session_total_usd": 3.42,
    "ceiling_usd": 50.00,
    "burn_rate_usd_per_min": 0.18
  }
}
```

## 3. Core Engine Hardening & Research Substrate Integration (R2)

### 3.1 The Context Integrity Protocol (CIP) 7-Layer Architecture

The Context Integrity Protocol (CIP) formalizes agent governance into a strict 7-layer stack, analogous to the ISO/OSI networking model. Each layer establishes formal boundaries, timing budgets, framing standards, and failure isolation policies.

```
+-------------------------------------------------------------------------------+
|                       CONTEXT INTEGRITY PROTOCOL (CIP)                        |
+-------+-----------------------------+-----------------------------------------+
| Layer | Layer Name                  | Primary Architectural Responsibility    |
+-------+-----------------------------+-----------------------------------------+
| L7    | Business & Outcome Layer    | OVTs, SLA verification, ROI metrics     |
| L6    | Provenance & Attestation    | Merkle DAG, dual signatures, audit proof|
| L5    | Gate & Policy Runtime       | Deterministic sub-50ms commit gates     |
| L4    | Causal-Graph Substrate      | ISO SQL/PGQ node/edge DAG, cycle check  |
| L3    | Context & Working Memory    | Token accounting, TTL, sanitization     |
| L2    | Host Session & Boundary     | Process sandbox, host adapter, leases   |
| L1    | Physical / Transport        | Local socket, stdio framing, IPC (<1ms) |
+-------+-----------------------------+-----------------------------------------+
```

#### Layer 1: Physical / Transport
- **Mechanics:** Sub-millisecond local Inter-Process Communication (IPC) via Unix Domain Sockets (`/var/run/kineti/daemon.sock` or `$XDG_RUNTIME_DIR/kineti.sock` on POSIX; named pipes `\\.\pipe\kineti-ipc` on Windows) and stdio framing for headless MCP subprocess execution.
- **Framing & Serialization:** JSON-RPC 2.0 with content-length framing (`Content-Length: <bytes>\r\n\r\n<json_payload>`).
- **Timing Budget:** Sub-1ms round-trip transport latency ($\le 850\mu\text{s}$ target).
- **Security & Isolation:** Unix domain sockets are bound to mode `0600` (read/write only by owner UID). Peer PID/UID verification via `SO_PEERCRED` (Linux) or `LOCAL_PEERCRED` (macOS) ensures only authorized local agent processes connect. Stdio streams operate in isolated subprocess execution spaces without shell expansion hazards.

#### Layer 2: Host Session & Agent Boundary
- **Mechanics:** Decouples the core engine from external host environments (Google Antigravity, Anthropic Claude Code, OpenAI Codex, OpenCode, Cursor, and raw shell CLI).
- **Session Lifecycle:** 
  1. `Session_Init`: Host presents capability credentials and receives a cryptographically signed Session Token with an immutable lease duration (default: 3600s).
  2. `Heartbeat & Liveness`: Bidirectional ping/pong every 15s. Loss of 3 heartbeats halts agent side-effects.
  3. `Least-Privilege Capability Token`: Host adapters can declare restricted permissions (e.g., `READ_ONLY`, `FILESYSTEM_WRITE_SCOPED`, `SHELL_EXEC_RESTRICTED`).
  4. `Session_Terminate`: Graceful drain or forced abort triggering LIFO saga unwinding.
- **Root Goal Locking:** Upon session establishment, the root goal is captured and cryptographically hashed:
  $$\text{GoalHash} = \text{SHA-256}(\text{root\_goal} \mathbin{\Vert} \text{constraints} \mathbin{\Vert} \text{success\_criteria})$$
  Any downstream tool call or context injection attempting to mutate the root goal is immediately rejected at the Layer 2 boundary.

#### Layer 3: Agent Context & Working Memory
- **Mechanics:** Active context window governance, dynamic token-exact billing, and TTL memory lifecycle management.
- **Token Accounting:** Every inbound prompt and outbound completion is measured with byte-exact tokenizers (Tiktoken for OpenAI/Anthropic models or native BPE tokenizers) rather than crude character counts. Cost is computed dynamically against live model pricing tables.
- **Memory Lifecycle & TTL States:**
  $$\text{Record Lifecycle: } \text{Active} \xrightarrow{\text{expiry}} \text{Warm (90d)} \xrightarrow{+185\text{d}} \text{Cold (275d)} \xrightarrow{\text{maintenance}} \text{Archive}$$
  Only `active` records are loaded into privileged prompt context. Warm/Cold records remain queryable via graph index without polluting context tokens.
- **Dual-LLM Sanitization Pipeline (OWASP ASI-01/02):**
  External web data, API receipts, or third-party code are quarantined in an untrusted sandbox context. A secondary, isolated evaluator model parses and strips malicious instructions (prompt injection, jailbreaks, hidden markdown directives) before synthesizing clean facts into primary agent memory.

#### Layer 4: Causal-Graph Substrate
- **Mechanics:** Real-time generation of directed acyclic property graphs capturing every agent cognition, tool interaction, and environmental observation.
- **Query Standard:** Standardized on **ISO/IEC 9075-16:2023 (SQL/PGQ - Property Graph Queries)** allowing relational and graph queries to execute natively in a single SQL engine (PostgreSQL + Age / DuckDB / embedded SQLite-PGQ).
- **Acyclicity & Invariant Enforcement:** Strict cycle detection executed at write-time. Temporal causality enforcement:
  $$\forall e = (u \xrightarrow{\text{CAUSED\_BY}} v), \quad t(v) \le t(u)$$
  An effect can never precede its cause in timestamp order.

#### Layer 5: Gate Enforcement & Policy Runtime
- **Mechanics:** Sub-50ms atomic commit gates that halt execution deterministically when invariants fail.
- **Enforcement Pipeline:**
  - **Stage 5 (Feasibility Gate):** Evaluates hurdle rate, data availability, and financial viability. Failures return run to Stage 2 (`diagnose`).
  - **Stage 6 (Spec Gate):** Hard stop requiring cryptographically validated human signature before any code generation is permitted.
  - **Stage 10 (Security Gate):** Executes static analysis, secret scans, dependency audits, and OWASP Top 10 agent vulnerability checks.
  - **Stage 11 (Ship Gate):** Requires fresh evidence proofs (`evidence_fresh == true`) and security clearance (`security_pass == true`).
- **Spend Circuit Breaker:** Continuous evaluation of token spend. If cumulative spend reaches $\$50.00$ USD (or per-stage ceilings), the runtime atomically halts agent execution, locks the workspace, and requires an interactive human operator confirmation.

#### Layer 6: Provenance & Cryptographic Attestation
- **Mechanics:** Merkle DAG construction and cryptographic hash-chaining across all execution events.
- **Merkle Leaf Generation:**
  $$\text{Leaf}_n = \text{HMAC-SHA256}\Big(K_{\text{harness}}, \text{prev\_hash}_n \mathbin{\Vert} \text{timestamp}_n \mathbin{\Vert} \text{entity\_id}_n \mathbin{\Vert} \text{CanonicalJSON}(\text{payload}_n)\Big)$$
- **Code Fingerprinting:** Robust recursive hashing of the workspace tree ignoring volatile directories (`.git/`, `.kineti/`, `.agents/`, `design/screenshots/`, `node_modules/`, `dist/`). Failsafe symlink detection preventing infinite loops or FIFO blocking.
- **Tamper Resistance:** Any post-hoc modification of historical execution logs, journal lines, or evidence files immediately breaks the hash chain, causing runtime verification to fail with exit code 3 (`TAMPER_DETECTED`).

#### Layer 7: Business & Outcome Layer
- **Mechanics:** Translates technical execution DAGs into quantifiable business outcomes encapsulated in **Outcome Verification Tickets (OVTs)**.
- **Core Value Metric:** Transitioning enterprise contracts from $/Token to **Cost Per Verified Outcome ($/Outcome)**.
  $$\text{Outcome Efficiency Ratio (OER)} = \frac{\text{Quantified Business Value Created (\$)}}{\text{Execution Spend (\$) + Human Review Latency (\$)}} \ge 10.0$$
- **Compliance Artifacts:** Generates immutable, exportable compliance bundles (JSON-LD / W3C Verifiable Credentials) acceptable to enterprise security, SOC 2 Type II audits, ISO 27001, and regulatory oversight bodies.

---

### 3.2 Causal-Graph Substrates & ISO SQL/PGQ Property Graph Specification

#### Node & Edge Taxonomy
The causal graph models all agent actions and environmental states as a formal Property Graph.

```
       [Goal Node]
            │
            ▼ (DEPENDS_ON)
      [Decision Node] ────(CAUSED_BY)────► [Hypothesis Node]
            │
            ▼ (CAUSED_BY)
       [Action Node] ────(ROLLED_BACK_BY)────► [RollbackAction]
            │
            ▼ (CAUSED_BY)
    [Observation Node]
            │
            ▼ (VERIFIED_BY)
        [Gate Node]
            │
            ▼ (ENABLES)
      [Outcome Node]
```

##### 6 Core Node Types:
1. **Action:** State-mutating operation executed by the agent (file edit, shell command execution, API write).
2. **Observation:** Ground truth returned from the environment (tool stdout/stderr, compiler output, HTTP response status, test runner JSON).
3. **Hypothesis:** Agent's explicit inductive reasoning or proposed causal mechanism prior to action.
4. **Decision:** Agent's deliberate branch selection between alternative paths, including trade-off rationale.
5. **Gate:** Deterministic policy checkpoint (Spec approval, Security scan, Test verification).
6. **Outcome:** Terminal, verified business deliverable (passed integration test suite, deployed microservice, generated UI).

##### 8 Core Edge Types:
1. `CAUSED_BY`: Strict temporal causation link ($A$ was triggered by $B$).
2. `DEPENDS_ON`: Structural prerequisite ($A$ cannot execute until $B$ resolves).
3. `VERIFIED_BY`: Epistemic validation (connects an Action or Outcome to an empirical Observation).
4. `ROLLED_BACK_BY`: Compensating association (connects a mutating Action to its inverse RollbackAction).
5. `BLOCKS`: Preventive relationship (a failed Gate or negative Observation blocks downstream Actions).
6. `ENABLES`: Permissive relationship (successful passage of a Gate authorizes subsequent Actions).
7. `CONTRADICTS`: Falsification relationship (an Observation disproves a Hypothesis).
8. `REMEDIATES`: Corrective relationship (an Action repairs a defect identified in an Observation).

---

#### Relational & Property Graph Schema (DDL)

The schema is implemented in PostgreSQL with ISO SQL/PGQ extension compatibility and mirrored in embedded DuckDB:

```sql
-- 1. Vertex Tables
CREATE TABLE causal_nodes (
    node_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    node_type VARCHAR(32) NOT NULL CHECK (node_type IN ('Action', 'Observation', 'Hypothesis', 'Decision', 'Gate', 'Outcome')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    label VARCHAR(255) NOT NULL,
    payload_hash CHAR(64) NOT NULL,
    payload JSONB NOT NULL,
    merkle_leaf_hash CHAR(64) NOT NULL
);

CREATE INDEX idx_causal_nodes_session ON causal_nodes(session_id);
CREATE INDEX idx_causal_nodes_type ON causal_nodes(node_type);
CREATE INDEX idx_causal_nodes_created ON causal_nodes(created_at);

-- 2. Edge Tables
CREATE TABLE causal_edges (
    edge_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    source_node_id UUID NOT NULL REFERENCES causal_nodes(node_id) ON DELETE CASCADE,
    target_node_id UUID NOT NULL REFERENCES causal_nodes(node_id) ON DELETE CASCADE,
    relationship_type VARCHAR(32) NOT NULL CHECK (relationship_type IN (
        'CAUSED_BY', 'DEPENDS_ON', 'VERIFIED_BY', 'ROLLED_BACK_BY',
        'BLOCKS', 'ENABLES', 'CONTRADICTS', 'REMEDIATES'
    )),
    status VARCHAR(16) NOT NULL DEFAULT 'candidate' CHECK (status IN ('candidate', 'hypothesis', 'validated', 'rejected')),
    weight NUMERIC(5,4) NOT NULL DEFAULT 1.0000,
    proof_id UUID REFERENCES causal_nodes(node_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    CONSTRAINT chk_no_self_loops CHECK (source_node_id <> target_node_id)
);

-- Trigger strictly enforcing cross-node chronological causal order (source.created_at <= target.created_at)
CREATE OR REPLACE FUNCTION trg_check_causal_order() RETURNS TRIGGER AS $$
BEGIN
    IF NEW.relationship_type IN ('CAUSED_BY', 'BLOCKS') THEN
        IF (SELECT created_at FROM causal_nodes WHERE node_id = NEW.source_node_id) >
           (SELECT created_at FROM causal_nodes WHERE node_id = NEW.target_node_id) THEN
            RAISE EXCEPTION 'Temporal order violation: source node created after target node';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_check_causal_order
BEFORE INSERT OR UPDATE ON causal_edges
FOR EACH ROW EXECUTE FUNCTION trg_check_causal_order();

CREATE INDEX idx_causal_edges_nodes ON causal_edges(source_node_id, target_node_id);
CREATE INDEX idx_causal_edges_rel ON causal_edges(relationship_type);

-- 3. ISO SQL/PGQ Property Graph Definition
CREATE PROPERTY GRAPH agent_causal_graph
VERTEX TABLES (
    causal_nodes LABEL Node
        PROPERTIES (node_id, node_type, label, created_at, payload)
)
EDGE TABLES (
    causal_edges
        SOURCE KEY (source_node_id) REFERENCES causal_nodes (node_id)
        DESTINATION KEY (target_node_id) REFERENCES causal_nodes (node_id)
        LABEL Edge
        PROPERTIES (edge_id, relationship_type, status, weight)
);
```

---

#### ISO SQL/PGQ Graph Queries

##### Query 1: Full Provenance Traversal (Goal to Outcome)
Traverses the causal ancestry from a delivered Outcome back to the root Goal, collecting every intermediate action, observation, and verification proof:
```sql
SELECT *
FROM GRAPH_TABLE (agent_causal_graph
    MATCH (out:Node WHERE out.node_type = 'Outcome')
          <-[e:Edge WHERE e.relationship_type IN ('CAUSED_BY', 'ENABLES')]-+ (ancestor:Node)
    COLUMNS (
        out.node_id AS outcome_id,
        ancestor.node_id AS step_id,
        ancestor.node_type AS step_type,
        ancestor.label AS step_label,
        ancestor.created_at AS step_time
    )
)
ORDER BY step_time ASC;
```

##### Query 2: Write-Time Cycle Detection (Preventing Circular Causality)
Before inserting a new dependency edge `(:new_source, :new_target, 'DEPENDS_ON')`, the runtime ensures no path already exists from `:new_target` to `:new_source`:
```sql
SELECT COUNT(*) AS cycle_exists
FROM GRAPH_TABLE (agent_causal_graph
    MATCH (src:Node WHERE src.node_id = :new_target)
          -[e:Edge WHERE e.relationship_type = 'DEPENDS_ON']-+
          (dst:Node WHERE dst.node_id = :new_source)
    COLUMNS (src.node_id AS cycle_root)
);
```
*If `cycle_exists > 0`, the transaction is aborted with `CYCLE_DETECTED_ERROR`.*

##### Query 3: Blame & Root-Cause Analysis for Gate Failures
When a Gate fails (e.g. Stage 11 Ship Gate), this query automatically pinpoints the exact CodeDiff and mutating Action that triggered the failure:
```sql
SELECT *
FROM GRAPH_TABLE (agent_causal_graph
    MATCH (g:Node WHERE g.node_type = 'Gate' AND g.payload->>'status' = 'fail')
          <-[:Edge WHERE relationship_type = 'VERIFIED_BY']-
          (obs:Node WHERE obs.node_type = 'Observation')
          <-[:Edge WHERE relationship_type = 'CAUSED_BY']-
          (act:Node WHERE act.node_type = 'Action')
    COLUMNS (
        g.label AS failed_gate,
        obs.label AS error_observation,
        obs.payload->>'stderr' AS failure_detail,
        act.node_id AS offending_action_id,
        act.label AS offending_action,
        act.payload->>'command' AS offending_command
    )
);
```

##### Query 4: Topological Saga Rollback Sequence Generation (Recursive CTE)
Retrieves all mutating actions for a failed session in strict reverse topological dependency order (LIFO) alongside timestamp ordering, resolving compensating inverse actions and preventing race conditions during concurrent multi-agent executions:
```sql
WITH RECURSIVE topological_depth AS (
    -- Base case: leaf actions with no downstream dependent actions in this session
    SELECT 
        act.node_id,
        0 AS depth
    FROM causal_nodes act
    WHERE act.session_id = :session_id
      AND NOT EXISTS (
          SELECT 1 FROM causal_edges dep 
          WHERE dep.target_node_id = act.node_id 
            AND dep.relationship_type = 'DEPENDS_ON'
      )
    UNION ALL
    -- Recursive step: traverse upstream along dependency edges
    SELECT 
        dep.target_node_id AS node_id,
        td.depth + 1 AS depth
    FROM topological_depth td
    JOIN causal_edges dep ON dep.source_node_id = td.node_id AND dep.relationship_type = 'DEPENDS_ON'
    WHERE td.depth < 100 -- Cycle recursion safeguard
),
ranked_actions AS (
    SELECT node_id, MAX(depth) AS topo_rank
    FROM topological_depth
    GROUP BY node_id
)
SELECT 
    act.node_id AS action_id,
    act.label AS action_label,
    rb.payload->>'inverse_command' AS undo_command,
    act.created_at AS executed_at,
    COALESCE(ra.topo_rank, 0) AS rollback_order
FROM causal_nodes act
JOIN causal_edges e ON e.source_node_id = act.node_id AND e.relationship_type = 'ROLLED_BACK_BY'
JOIN causal_nodes rb ON rb.node_id = e.target_node_id
LEFT JOIN ranked_actions ra ON ra.node_id = act.node_id
WHERE act.session_id = :session_id
ORDER BY COALESCE(ra.topo_rank, 0) ASC, act.created_at DESC;
```

---

### 3.3 Universal 20-Entity Provenance Kernel

The Kineti Kernel formalizes all agent cognition, execution, and verification events into 20 strongly typed entities. Below are the complete TypeScript definitions:

```typescript
// ==========================================
// UNIVERSAL 20-ENTITY PROVENANCE KERNEL SCHEMAS
// ==========================================

export type UUID = string;
export type ISO8601 = string;
export type SHA256 = string;

// 1. Agent: Identity, public signing key, declared capabilities, and runtime bounds
export interface Agent {
  id: UUID;
  public_key: string; // Ed25519 public key hex
  archetype: "explorer" | "worker" | "challenger" | "reviewer" | "auditor";
  model_id: string;   // e.g. "claude-3-7-sonnet", "gpt-4o"
  capabilities: string[];
  max_spend_limit_usd: number;
}

// 2. Session: Isolated temporal context container, host binding, lease, and permissions
export interface Session {
  id: UUID;
  agent_id: UUID;
  host_id: UUID;
  created_at: ISO8601;
  expires_at: ISO8601;
  status: "active" | "committed" | "aborted" | "timed_out";
  working_directory: string;
}

// 3. Host: Runtime host descriptor (Google Antigravity, Claude Code, Codex, OpenCode, Cursor)
export interface Host {
  id: UUID;
  name: "antigravity" | "claude_code" | "codex" | "opencode" | "cursor" | "cli";
  version: string;
  os: "darwin" | "linux" | "win32";
  protocol_version: "CIP-1.0";
}

// 4. Goal: Immutable root objective statement, cryptographic hash, and success criteria
export interface Goal {
  id: UUID;
  session_id: UUID;
  root_goal: string;
  constraints: string[];
  success_criteria: string[];
  immutable_hash: SHA256; // SHA256(root_goal + constraints + success_criteria)
  locked_at: ISO8601;
}

// 5. Milestone: Major stage marker along the 13-stage execution pipeline
export interface Milestone {
  id: UUID;
  session_id: UUID;
  stage_number: number; // 1 to 13
  stage_id: "officehours" | "diagnose" | "design" | "architecture" | "feasibility" | "spec" | "build" | "review" | "qa" | "security" | "ship" | "watch" | "retro";
  status: "pending" | "in_progress" | "passed" | "failed";
  entered_at: ISO8601;
  exited_at?: ISO8601;
}

// 6. Task: Specific decomposed unit of executable work delegated to an Agent
export interface Task {
  id: UUID;
  milestone_id: UUID;
  agent_id: UUID;
  title: string;
  description: string;
  status: "planned" | "in_progress" | "completed" | "failed";
}

// 7. ToolCall: Structured invocation intent with name and arguments
export interface ToolCall {
  id: UUID;
  task_id: UUID;
  tool_name: string;
  arguments: Record<string, unknown>;
  issued_at: ISO8601;
}

// 8. ToolResult: Execution outcome with stdout, stderr, exit code, duration
export interface ToolResult {
  id: UUID;
  tool_call_id: UUID;
  stdout: string;
  stderr: string;
  exit_code: number;
  duration_ms: number;
  executed_at: ISO8601;
}

// 9. Observation: Distilled environmental fact extracted from ToolResult
export interface Observation {
  id: UUID;
  tool_result_id?: UUID;
  fact: string;
  confidence: number; // 0.0000 to 1.0000
  source: string;
  created_at: ISO8601;
}

// 10. Artifact: Concrete file or asset produced or modified during execution
export interface Artifact {
  id: UUID;
  session_id: UUID;
  relative_path: string;
  content_type: string;
  sha256: SHA256;
  size_bytes: number;
  created_at: ISO8601;
}

// 11. CodeDiff: Unified patch representation of modifications applied to workspace files
export interface CodeDiff {
  id: UUID;
  artifact_id: UUID;
  unified_diff: string;
  additions: number;
  deletions: number;
  applied_at: ISO8601;
}

// 12. Assertion: Deterministic claim evaluated during automated test execution
export interface Assertion {
  id: UUID;
  test_name: string;
  status: "pass" | "fail";
  expected_value: string;
  actual_value: string;
  evaluated_at: ISO8601;
}

// 13. GateResult: Pass/fail/pending verdict emitted by a Layer 5 policy checkpoint
export interface GateResult {
  id: UUID;
  milestone_id: UUID;
  gate_name: "feasibility" | "spec" | "security" | "ship";
  status: "pass" | "fail" | "pending";
  evaluated_rules: Array<{ rule: string; passed: boolean; reason?: string }>;
  evaluator_signature: string; // Ed25519 signature
  evaluated_at: ISO8601;
}

// 14. SpendEntry: Exact prompt/completion token count and financial dollar cost
export interface SpendEntry {
  id: UUID;
  session_id: UUID;
  stage_id: string;
  tokens_in: number;
  tokens_out: number;
  tokens_cached: number;
  usd_cost: number;
  cumulative_session_usd: number;
  logged_at: ISO8601;
}

// 15. RollbackAction: LIFO compensating transaction registration
export interface RollbackAction {
  id: UUID;
  session_id: UUID;
  step_number: number; // LIFO index
  label: string;
  inverse_command: string;
  state: "registered" | "executed" | "failed";
  registered_at: ISO8601;
}

// 16. Checkpoint: Point-in-time cryptographic freeze of memory, DAG state, and files
export interface Checkpoint {
  id: UUID;
  session_id: UUID;
  git_commit_sha: string;
  dag_root_hash: SHA256;
  spend_usd_snapshot: number;
  created_at: ISO8601;
}

// 17. OTDTrigger: Active Ontology Trigger Data rule specification and firing criteria
export interface OTDTrigger {
  id: UUID;
  trigger_name: string;
  condition_predicate: string; // JSONPath or DSL expression
  target_ontology_state: string;
  action_payload: Record<string, unknown>;
  created_at: ISO8601;
}

// 18. EvidenceLog: Cryptographic proof record binding test results to workspace fingerprint
export interface EvidenceLog {
  id: UUID;
  session_id: UUID;
  label: string;
  command: string;
  exit_code: number;
  code_fingerprint_before: SHA256;
  code_fingerprint_after: SHA256;
  recorded_at: ISO8601;
  status: "FRESH" | "STALE" | "MISSING";
}

// 19. MerkleLeaf: Hash node in the continuous cryptographic execution provenance tree (DAG branch-aware)
export interface MerkleLeaf {
  leaf_index: number;
  parent_hashes: SHA256[]; // Supports multiple parents for DAG branch merges under concurrent agent execution
  entity_type: string;
  entity_id: UUID;
  canonical_payload_hash: SHA256;
  node_hash: SHA256; // SHA256(parent_hashes.join(":") + "\x00" + entity_type + "\x00" + entity_id + "\x00" + canonical_payload_hash)
  timestamp: ISO8601;
}

// 20. OVT: Outcome Verification Ticket certifying final deliverable (Dual-Contract: Internal Kernel Record & W3C VC)

/** Internal storage record for daemon SQLite WAL and PostgreSQL persistence */
export interface OVTInternalRecord {
  ticket_id: UUID;
  session_id: UUID;
  root_goal: string;
  root_goal_hash: SHA256;
  code_fingerprint: SHA256;
  evidence_hash: SHA256;
  merkle_root: SHA256;
  total_session_spend_usd: number;
  spend_microcents: number; // Integer micro-cents (1 USD = 1,000,000 microcents) for deterministic cross-language hashing
  outcomes: Array<{
    milestone: string;
    status: "VERIFIED_PASS" | "FAILED";
    test_proof_hash: SHA256;
    spend_usd: number;
  }>;
  policy_gates_passed: string[];
  delivery_timestamp: ISO8601;
  signatures: {
    agent_signature: string;   // Ephemeral Ed25519 signature
    harness_signature: string; // Harness Ed25519 counter-signature
  };
  compliance_proof: {
    w3c_verifiable_credential_uri?: string;
    merkle_inclusion_proof: string[];
  };
}

/** External W3C Verifiable Credential standard schema (Appendix C & Section 3.5) */
export interface OVTVerifiableCredential {
  "@context": string[];
  id: string; // URI
  type: string[];
  issuer: string; // DID
  issuanceDate: ISO8601;
  credentialSubject: {
    id: string; // Subject DID
    sessionId: UUID;
    rootGoal: string;
    rootGoalHash: SHA256;
    codeFingerprint: SHA256;
    merkleRoot: SHA256;
    outcomes: Array<{
      milestone: string;
      status: "VERIFIED_PASS" | "FAILED";
      testProofHash: SHA256;
      spendUsd: number;
    }>;
    policyGatesPassed: string[];
    totalSessionSpendUsd: number;
    spendMicrocents?: number;
  };
  proof: Array<{
    type: string;
    created: ISO8601;
    verificationMethod: string;
    proofPurpose: string;
    proofValue: string;
  }>;
}

/** Dual-Contract Unified Interface with field mapping aliases */
export interface OVT extends OVTInternalRecord {
  // Legacy & W3C VC camelCase aliases for transparent cross-boundary interop
  goal_hash?: SHA256;
  total_spend_usd?: number;
  id?: string;
  sessionId?: UUID;
  rootGoalHash?: SHA256;
  totalSessionSpendUsd?: number;
  toVerifiableCredential?: () => OVTVerifiableCredential;
}
```

---

### 3.4 Runtime Ontology Trigger Data (OTD) Mechanics & Event-Driven State Machine

#### Dynamic Ontological State vs Static Prompt Bloat
Traditional agent frameworks stuff hundreds of guidelines into fixed system prompts. As multi-turn sessions expand, LLMs experience **attention decay and context distraction**, drifting away from constraints.

**Runtime Ontology Trigger Data (OTD)** replaces static prompt bloat with an event-driven state machine:
1. The harness maintains the active **Ontological Mode** (`DIAGNOSIS_MODE`, `SPEC_LOCK_MODE`, `BUILD_SAFE_MODE`, `AUTO_REPAIR_MODE`, `RECOVERY_MODE`).
2. When environmental telemetry matches an OTD trigger predicate (test failure, spend threshold, gate transition), the harness injects an **OTD Activation Packet** directly into the Layer 3 context buffer and updates capability boundaries at Layer 2.
3. Operations outside the active ontology are strictly blocked at the socket layer.

```
                [Telemetry Event: Test Failure]
                               │
                               ▼
                   ┌───────────────────────┐
                   │  OTD Evaluation Engine │
                   │  (Sub-10ms Matcher)   │
                   └───────────┬───────────┘
                               │ (Matches Predicate)
                               ▼
        ┌──────────────────────────────────────────────┐
        │  Transition: BUILD_MODE ──► AUTO_REPAIR_MODE │
        ├──────────────────────────────────────────────┤
        │ - Scope locked to failed test file           │
        │ - Max repair budget set to $2.00             │
        │ - Maximum 5 self-repair iterations           │
        │ - Saga undo checkpoint registered            │
        └──────────────────────────────────────────────┘
```

#### OTD Trigger Definition Schema (JSON Schema):
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OTDTriggerDefinition",
  "type": "object",
  "required": ["trigger_id", "name", "event_source", "condition", "transition_to", "enforcement"],
  "properties": {
    "trigger_id": { "type": "string", "format": "uuid" },
    "name": { "type": "string" },
    "event_source": { 
      "type": "string", 
      "enum": ["tool_result", "spend_telemetry", "gate_evaluation", "file_watcher", "heartbeat"] 
    },
    "condition": {
      "type": "object",
      "required": ["predicate", "eval_expr"],
      "properties": {
        "predicate": { "type": "string", "enum": ["EQUALS", "GREATER_THAN", "MATCHES_REGEX", "GRAPH_PATH_EXISTS"] },
        "eval_expr": { "type": "string" },
        "threshold": { "type": ["string", "number", "boolean"] }
      }
    },
    "transition_to": {
      "type": "object",
      "required": ["ontology_state", "active_skills", "revoked_tools"],
      "properties": {
        "ontology_state": { "type": "string" },
        "active_skills": { "type": "array", "items": { "type": "string" } },
        "revoked_tools": { "type": "array", "items": { "type": "string" } }
      }
    },
    "enforcement": {
      "type": "object",
      "required": ["deterministic_block", "timeout_ms"],
      "properties": {
        "deterministic_block": { "type": "boolean" },
        "timeout_ms": { "type": "integer", "default": 5000 }
      }
    }
  }
}
```

---

### 3.5 Outcome Verification Tickets (OVTs) Deep Specification

An **Outcome Verification Ticket (OVT)** is an unforgeable, cryptographically dual-signed credential that proves an autonomous agent achieved a specific business outcome while complying with all Layer 5 governance gates.

```
       [Agent Private Key]                  [Harness Private Key]
              │                                      │
              ▼                                      ▼
     [Sign(Outcome Hash)]                 [Sign(Gate + Merkle Root)]
              │                                      │
              └───────────────┬──────────────────────┘
                              ▼
                 ┌──────────────────────────┐
                 │ Outcome Verification     │
                 │ Ticket (OVT)             │
                 │                          │
                 │ - Dual Ed25519 Sigs      │
                 │ - Merkle Root Verified   │
                 │ - Code Fingerprint Match │
                 │ - Exact Token Cost Audit │
                 └──────────────────────────┘
```

#### Dual-Signing Protocol Lifecycle:
1. **Agent Claim Signing:** Upon completing a milestone, the agent constructs the `OutcomeClaim`:
   $$\text{Hash}_{\text{agent}} = \text{SHA256}\Big(\text{session\_id} \mathbin{\Vert} \text{goal\_hash} \mathbin{\Vert} \text{deliverable\_artifacts\_hash}\Big)$$
   The agent signs $\text{Hash}_{\text{agent}}$ with its ephemeral Ed25519 private key: $S_{\text{agent}} = \text{Sign}(K_{\text{agent\_priv}}, \text{Hash}_{\text{agent}})$.
2. **Harness Verification & Counter-Signing:** The local harness daemon receives the claim and executes independent verification:
   - Evaluates Layer 5 Gates (Feasibility, Spec, Security, Ship).
   - Recalculates the workspace code fingerprint ($\text{FP}_{\text{code}}$) and validates against `bin/kineti-evidence`.
   - Recomputes the execution Merkle Root ($M_{\text{root}}$) over all 20 kernel entities.
   - Computes the combined verification digest:
     $$\text{Hash}_{\text{harness}} = \text{SHA256}\Big(\text{Hash}_{\text{agent}} \mathbin{\Vert} S_{\text{agent}} \mathbin{\Vert} M_{\text{root}} \mathbin{\Vert} \text{FP}_{\text{code}} \mathbin{\Vert} \text{spend\_microcents}\Big)$$
   - The harness countersigns: $S_{\text{harness}} = \text{Sign}(K_{\text{harness\_priv}}, \text{Hash}_{\text{harness}})$.
3. **Float Canonicalization & Tamper Resistance:** To eliminate floating-point formatting ambiguity and cross-platform serialization divergence across heterogeneous agent runtimes (e.g. Node.js, Rust, Go, Python), all monetary values in cryptographic digests are normalized to integer micro-cents (`spend_microcents: number`, where $\$1.42 \to 1,420,000$ micro-cents, $1 \text{ USD} = 10^6 \text{ micro-cents}$) or strictly serialized via **RFC 8785 (JSON Canonicalization Scheme / JCS)**. An OVT cannot be forged by an agent because it lacks $K_{\text{harness\_priv}}$. It cannot be forged by an external actor because any modification to code, spend, or logs invalidates $\text{FP}_{\text{code}}$, $\text{spend\_microcents}$, and $M_{\text{root}}$.

#### W3C Verifiable Credentials Compliance Instance
```json
{
  "@context": [
    "https://www.w3.org/2018/credentials/v1",
    "https://schema.kineti.ai/v1/ovt"
  ],
  "id": "urn:uuid:f47ac10b-58cc-4372-a567-0e02b2c3d479",
  "type": ["VerifiableCredential", "OutcomeVerificationTicket"],
  "issuer": "did:kineti:host:macbook-pro-m3-local-harness",
  "issuanceDate": "2026-09-06T05:22:04Z",
  "credentialSubject": {
    "id": "did:kineti:agent:worker-m2-1",
    "sessionId": "787db8bf-e17f-440a-abec-0fbfbb7ecae3",
    "rootGoal": "Author authoritative technical strategy blueprint for Universal Harness",
    "rootGoalHash": "8f4b23c5e6d78a9012345678abcdef0123456789abcdef0123456789abcdef01",
    "codeFingerprint": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "merkleRoot": "1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f809",
    "outcomes": [
      {
        "milestone": "ship",
        "status": "VERIFIED_PASS",
        "testProofHash": "4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b",
        "spendUsd": 1.42
      }
    ],
    "policyGatesPassed": ["feasibility", "spec", "security", "ship"],
    "totalSessionSpendUsd": 1.42,
    "spendMicrocents": 1420000
  },
  "proof": [
    {
      "type": "Ed25519Signature2020",
      "created": "2026-09-06T05:22:05Z",
      "verificationMethod": "did:kineti:agent:worker-m2-1#key-1",
      "proofPurpose": "assertionMethod",
      "proofValue": "z3h...AgentSignatureBase58..."
    },
    {
      "type": "Ed25519Signature2020",
      "created": "2026-09-06T05:22:06Z",
      "verificationMethod": "did:kineti:host:macbook-pro-m3-local-harness#host-key",
      "proofPurpose": "verificationMethod",
      "proofValue": "z7k...HarnessSignatureBase58..."
    }
  ]
}
```

### 3.6 Systematic Remediation & Eradication Matrix of All 44 Audit Defects

The forensic investigation documented in `docs/AUDIT_REPORT.md` revealed 44 distinct defects across security, state mechanics, data integrity, and host portability in the legacy prototype harness. The hardened Kineti Universal Runtime systematically resolves and eliminates each defect through Layer 1 to Layer 7 architectural mechanisms.

Below is the complete remediation matrix followed by exhaustive technical specifications across all severity tiers.

---

#### 3.6.1 Master 44-Defect Remediation Matrix

```
+-----------------------------------------------------------------------------------------------------------------------------------------+
|                                              MASTER 44-DEFECT REMEDIATION MATRIX                                                        |
+---------+------+--------------------------------------------------+-------------------------+-------------------------------------------+
| ID      | Sev  | Defect Title                                     | Impacted File(s)        | Hardened Runtime Architectural Solution   |
+---------+------+--------------------------------------------------+-------------------------+-------------------------------------------+
| CRIT-01 | CRIT | Baseline Test Failure in Memory Job Test         | tests/memory-job.test.ts| RFC 8785 Canonical JCS Merkle Hashing     |
| CRIT-02 | CRIT | Broken Gate Status Lookup in State CLI           | bin/kineti-state.ts     | First-class L5 Gate Engine & Path Mapping |
| CRIT-03 | CRIT | Programmatic Self-Trust Security Bypass          | bin/kineti-verify-gate  | L2 Interactive TTY & Cryptographic Auth   |
| CRIT-04 | CRIT | Catastrophic Silent Ledger Erasure on Parse Err  | bin/lib.ts              | Resilient Line Stream Parser & Fsync Logs |
| CRIT-05 | CRIT | Missing Repo Pointer & Space-Path Splitting Cron | scripts/weekly.sh       | Structured Array Parsing & Robust Pointer |
| HIGH-01 | HIGH | Arbitrary Shell Execution & Stderr Swallowing    | bin/kineti-saga.ts      | 30s Timeout, Stderr Capture & Diagnostic  |
| HIGH-02 | HIGH | Command Injection & Blindness in Evidence Run    | bin/kineti-evidence.ts  | Preserved Arg Arrays & Stderr Forwarding  |
| HIGH-03 | HIGH | Path Duplication Creating Nested Spend Logs      | bin/kineti-spend.ts     | Centralized Repository Path Authority     |
| HIGH-04 | HIGH | Fingerprint Invalidation via .agents & Screens   | bin/kineti-evidence.ts  | Deterministic Exclusions in Tree Hash     |
| HIGH-05 | HIGH | Spend Circuit Breaker Reset Bypass via CLI Flag  | bin/kineti-spend.ts     | Hardware-Style TTY & Token Guarded Reset  |
| HIGH-06 | HIGH | Spec Skill Directs Unsupported Gate Transition   | skills/spec/SKILL.md    | 3-State Gate Protocol ('pending' support) |
| HIGH-07 | HIGH | Uncaught TypeError on Missing CLI Option Arg     | bin/kineti-memory-job.ts| Strict Bounds-Checked CLI Option Parser   |
| HIGH-08 | HIGH | Installer Smoke Test Excluded from Default Run   | package.json            | Unified CI Test Runner (Unit + Shell)     |
| HIGH-09 | HIGH | Critical Enforcement Subsystems Untested         | bin/*.ts, tests/        | Comprehensive 95%+ Integration Coverage   |
| MED-01  | MED  | Saga Rollback Forbids Committed Runs Without Flg | bin/kineti-saga.ts      | Explicit `--force-committed` for Watch    |
| MED-02  | MED  | Zero Enforcement Tooling for Standing Rule 8     | ETHOS.md, bin/          | Automated Layer 5 Pre-Commit Secret Scanner|
| MED-03  | MED  | Missing UX Blueprint Integration in Skills       | skills/officehours, des | Formal Stage 1 UX Blueprint Deliverables  |
| MED-04  | MED  | Non-Atomic File Overwrite in Memory Journal      | bin/kineti-memory-job.ts| Atomic Tempfile Write + Fsync + Rename     |
| MED-05  | MED  | Symlink Traversal & FIFO Blocking in Fingerprint | bin/kineti-evidence.ts  | `fs.lstatSync`, Symlink & FIFO Bypass     |
| MED-06  | MED  | Unchecked Argument Parsing Shift in Installer    | setup.sh                | Strict Argument Count Validation in Shift |
| MED-07  | MED  | Unhandled Exit Code 1 in read_conf Under Pipefail| setup.sh                | Safe Grep Stream Extraction Pattern       |
| MED-08  | MED  | Incomplete Skill Install Omitting Subdirectories | setup.sh                | Recursive Directory Mirroring (cp -R)     |
| MED-09  | MED  | Incomplete Uninstall Leaving Orphan Pointer      | setup.sh                | Comprehensive Purge (`rm ~/.kineti/repo`) |
| MED-10  | MED  | Fragile Bun Path Resolution in Cron Environments | scripts/weekly.sh       | Multi-Path Discovery & Fallback Execution |
| MED-11  | MED  | Temp Directory Resource Leakage in Test Suite    | tests/*.test.ts         | Strict `try...finally` Lifecycle Fixtures |
| MED-12  | MED  | TypeScript Strictness Gaps Masking Index Hazards | tsconfig.json           | Enable `noUncheckedIndexedAccess: true`    |
| LOW-01  | LOW  | Missing Executable Permission on CLI Binary      | bin/kineti-memory-job.ts| File Mode `0755` via Git Mode Update       |
| LOW-02  | LOW  | Inappropriate Executable Bit on Library Module   | bin/lib.ts              | Library Mode `0644` (Non-Executable)      |
| LOW-03  | LOW  | Hash Delimiter Collision Risk in Egress Ledger   | bin/kineti-egress.ts     | RFC 8785 Canonical JSON Digest Hashing    |
| LOW-04  | LOW  | Unsanitized Log Injection (CWE-117) in Alerts     | bin/kineti-verify-gate   | JSON-Structured Audit Logging (No Newline) |
| LOW-05  | LOW  | Record Type Schema Naming Mismatch               | kineti.config.json      | Schema Canonicalization to `"dossier"`     |
| LOW-06  | LOW  | Stale Reference in Officehours to journal.md      | skills/officehours      | Path Alignment to `journal.jsonl`          |
| LOW-07  | LOW  | Incomplete Skill Catalog Listing in Claude Hook  | hooks/claude.txt        | Full 17-Skill Catalog Declaration          |
| LOW-08  | LOW  | Dead/Conflicting hook_file Directives in Confs    | hosts/*.conf            | Centralized Manifest Schema Validation     |
| INFO-01 | INFO | Program Count Mismatch in Documentation          | package.json, README    | Correct Count: 7 Enforcement Binaries      |
| INFO-02 | INFO | Absolute Path Reference `/design` in Standing Law| ETHOS.md                | Relative Path Alignment (`design/screens/`)|
| INFO-03 | INFO | Unquoted Variable Expansion in Daily Loop Doc     | docs/HOWTO-daily-loop   | POSIX Quoting Enforced (`"$K/..."`)        |
| INFO-04 | INFO | Repository Version Desynchronization Across Files | package.json, config    | Synchronized SemVer `3.1.0` Release Tag    |
| INFO-05 | INFO | Gate Count Ambiguity in Documentation vs Config   | kineti.config.json      | Formal 4-Gate Alignment (Feas, Spec, Sec, Sh)|
| INFO-06 | INFO | Philosophical Conflict on 12 UI Components       | skills/design/SKILL.md  | Mandatory Kineti Design System Directives  |
| INFO-07 | INFO | Hardcoded Skill Counts in Installer Smoke Test   | tests/test-setup.sh     | Dynamic Discovery Count Comparison         |
| INFO-08 | INFO | Unprotected Shell Glob Expansions Without Nullglob| setup.sh, audit-skills  | `shopt -s nullglob` Enforced Across Scripts|
| INFO-09 | INFO | Unused Test Import in Harness Test Suite         | tests/harness.test.ts   | Tree-Shaken Unused Import Cleanup          |
| INFO-10 | INFO | Zero Automated Test Coverage for Utility Scripts | scripts/*.sh            | Automated ShellSpec / Bats Test Suite      |
+---------+------+--------------------------------------------------+-------------------------+-------------------------------------------+
```

---

#### 3.6.2 Critical Severity Technical Remediations (CRIT-01 to CRIT-05)

##### [CRIT-01] Baseline Test Failure in Memory Job Test
- **Impacted Files & Lines:** `tests/memory-job.test.ts:37-56` (interacting with `bin/kineti-memory-job.ts:64-66, 104-107`).
- **Root Cause Analysis:**
  1. Hash calculation in test used pipe delimiters (`${r.prev_hash}|${r.at}|...`), while `bin/kineti-memory-job.ts:65` computed hashes without delimiters.
  2. `oldLearning` record was inserted without `prev_hash` and `hash`, causing `verify-chain` to crash with exit code 3 (`CHAIN BROKEN: missing prev_hash/hash`).
  3. Key sorting desynchronization: `bin/kineti-memory-job.ts` enforces strict alphabetical key canonicalization (`canonStable`), whereas `oldLearning.data` declared keys out of alphabetical order (`skill`, `trigger`, `lesson`), causing content hash mismatches.
- **Universal Runtime Architectural Mechanism:** Layer 6 Automated Merkle Leaf Canonicalization. The kernel provides an immutable `computeRecordHash(entity)` function utilizing RFC 8785 canonical JSON serialization. Unchained records are rejected at ingestion time rather than failing during downstream sweeps.
- **Concrete Fix Specification:**
  In `tests/memory-job.test.ts`: Remove pipe delimiters; supply `prev_hash: r2.hash` on `oldLearning`; sort `oldLearning.data` keys alphabetically (`{ lesson: "old", skill: "qa", trigger: "always" }`).

##### [CRIT-02] Broken Gate Status Lookup in State CLI Blocking Build & Ship Pre-Flights
- **Impacted Files & Lines:** `bin/kineti-state.ts:70-77` (interacting with `skills/build/SKILL.md:23` and `skills/ship/SKILL.md:29-30`).
- **Root Cause Analysis:** `cmd === "get"` evaluated `(s as any)["gate.spec"]`, which returned `undefined` because gate statuses are stored in nested dictionary `s.gates[name]`. This caused pre-flight checks in Stage 7 (`build`) and Stage 11 (`ship`) to crash with exit code 2.
- **Universal Runtime Architectural Mechanism:** Layer 5 Gate State Engine. Gate states are first-class kernel entities (`GateResult`) accessed via typed getters with automatic dot-notation path resolution (`gate.<name>` seamlessly maps to `s.gates[<name>]`).
- **Concrete Fix Specification:**
  In `bin/kineti-state.ts`: Add dot-notation path resolver in `get`:
  ```typescript
  if (v === undefined && key.startsWith("gate.")) {
    v = s.gates[key.slice(5)];
  }
  ```

##### [CRIT-03] Programmatic Self-Trust Security Bypass in Verify Gate
- **Impacted Files & Lines:** `bin/kineti-verify-gate.ts:20-27`.
- **Root Cause Analysis:** `--trust` could be executed programmatically by an untrusted subagent without human interaction, allowing autonomous agents to authorize and execute arbitrary shell commands via `bash -lc`.
- **Universal Runtime Architectural Mechanism:** Layer 2 Host-Agent Privilege Boundary & Human-in-the-Loop Attestation. Privileged operations (trust elevation, circuit breaker resets) require an interactive human TTY confirmation or a cryptographically signed human authorization token (`KINETI_TRUST_CONFIRMED=1`). Non-interactive calls from autonomous agents are blocked with exit code 2.
- **Concrete Fix Specification:**
  In `bin/kineti-verify-gate.ts`: Check `process.stdin.isTTY` and reject automated subagent self-trust unless verified cryptographic environment proof is supplied.

##### [CRIT-04] Catastrophic Silent Ledger Erasure on JSONL Parse Errors
- **Impacted Files & Lines:** `bin/lib.ts:48-58`.
- **Root Cause Analysis:** Wrapping `readJsonl` in a blanket `try...catch` that returned `[]` meant that a single corrupt line, partial write, or null byte caused the entire file to be treated as empty. Downstream writes then truncated the ledger, permanently destroying memory, egress chains, and rollback stacks.
- **Universal Runtime Architectural Mechanism:** Layer 1 Resilient Stream Parser with Atomic Fsync Logging. The parser processes lines independently, skipping corrupted lines with prominent stderr diagnostics while preserving all valid historical records. Ledger append operations utilize strict atomic write-and-rename or append-only file locks with `fsync()`.
- **Concrete Fix Specification:**
  In `bin/lib.ts`: Line-by-line parsing loop with isolated per-line `try/catch`, logging malformed lines to stderr without dropping valid lines.

##### [CRIT-05] Unhandled Missing Repo Pointer & Space-Path Splitting in Cron Script
- **Impacted Files & Lines:** `scripts/weekly.sh:10, 13, 16`.
- **Root Cause Analysis:** `cat "$HOME/.kineti/repo"` failed under `set -e` if the pointer was missing. `for p in $PROJECTS; do` performed unquoted word-splitting, breaking when repository paths contained spaces (e.g., `/kineti local harness/`).
- **Universal Runtime Architectural Mechanism:** Layer 1 Robust POSIX/Bash Packaging & Harness Daemon Automation. Replaces brittle shell word-splitting with structured delimiter parsing (`IFS=':' read -r -a project_list`) and explicit pointer validation with actionable diagnostics.
- **Concrete Fix Specification:**
  In `scripts/weekly.sh`: Add existence check for `~/.kineti/repo`; use bash arrays with `IFS=':'` for path list expansion; quote all path expansions `"$p"`.

---

#### 3.6.3 High Severity Technical Remediations (HIGH-01 to HIGH-09)

##### [HIGH-01] Arbitrary Shell Execution & Stderr Swallowing in Saga Rollback
- **Impacted Files & Lines:** `bin/kineti-saga.ts:63-72`.
- **Root Cause Analysis:** Undos executed via `bash -lc` had no execution timeout (risking infinite process hangs) and completely swallowed stderr, leaving developers blind when undo steps failed.
- **Universal Runtime Architectural Mechanism:** Layer 5 Saga Compensating Engine with ETHOS Rule 4.2 Harmonization. Default execution respects ETHOS Rule 4.2 (unwind newest-first, log diagnostic stderr, continue with remaining undos). Commands run with a 30s timeout (`timeout: 30000`). An optional `--fail-fast` flag is provided for strict cascading operations.
- **Concrete Fix Specification:** Add `timeout: 30000`, capture `res.stderr`, emit prominent error detail on failure, and support `--fail-fast`.

##### [HIGH-02] Command Injection, Argument Flattening & Diagnostic Blindness in Proof Capture
- **Impacted Files & Lines:** `bin/kineti-evidence.ts:61, 64, 71`.
- **Root Cause Analysis:** `argv.slice(dd + 1).join(" ")` stripped quotes and shell boundaries. Subprocess execution swallowed `stderr` on non-zero exit codes.
- **Universal Runtime Architectural Mechanism:** Hardened Process Wrapper. Forward child process stderr directly to `process.stderr` on non-zero exits, ensuring complete diagnostic visibility. Preserves command argument arrays without flattening.
- **Concrete Fix Specification:** Forward child process stderr directly on non-zero exit codes; preserve command argument arrays.

##### [HIGH-03] Path Duplication Creating Nested `.kineti/.kineti/spend.log.jsonl`
- **Impacted Files & Lines:** `bin/kineti-spend.ts:29`.
- **Root Cause Analysis:** `logFile()` concatenated `".kineti"` to `projectKdir()`, which already included `.kineti`, producing `.kineti/.kineti/spend.log.jsonl`.
- **Universal Runtime Architectural Mechanism:** Centralized Repository Path Authority in `lib.ts`.
- **Concrete Fix Specification:** Change `path.join(projectKdir(), ".kineti", "spend.log.jsonl")` to `path.join(projectKdir(), "spend.log.jsonl")`.

##### [HIGH-04] Code Fingerprint Invalidation Caused by `.agents/` and `design/screenshots/`
- **Impacted Files & Lines:** `bin/kineti-evidence.ts:11-15`.
- **Root Cause Analysis:** Omission of `.agents/` and `screenshots/` from `EXCLUDE_DIRS` meant that agent heartbeats or QA screenshot generation altered the code fingerprint, immediately flipping valid test proofs to `STALE` and blocking Ship.
- **Universal Runtime Architectural Mechanism:** Layer 6 Deterministic Workspace Fingerprinting.
- **Concrete Fix Specification:** Add `.agents` and `screenshots` to `EXCLUDE_DIRS`. Test proofs remain strictly bound to production source code and test files.

##### [HIGH-05] Spend Circuit Breaker Reset Bypass via Static CLI Flag
- **Impacted Files & Lines:** `bin/kineti-spend.ts:97-103`.
- **Root Cause Analysis:** Reset relied solely on passing static flag `--i-am-human`. Autonomous LLMs parsed the error and executed the flag, bypassing the $50 safety ceiling.
- **Universal Runtime Architectural Mechanism:** Layer 5 Cryptographic / TTY Circuit Breaker Reset. Reset requires an interactive human TTY confirmation (`process.stdin.isTTY`) or a privileged environment secret (`KINETI_HUMAN_RESET_TOKEN`).
- **Concrete Fix Specification:** Enforce `process.stdin.isTTY` check or cryptographic token verification before clearing spend ceiling.

##### [HIGH-06] Spec Skill Directs Unsupported Gate State Transition `gate.spec pending`
- **Impacted Files & Lines:** `skills/spec/SKILL.md:49` vs `bin/kineti-state.ts:95-98`.
- **Root Cause Analysis:** Spec skill instructed setting `gate.spec pending`, but `kineti-state.ts` strictly threw exit code 2 on any value other than `pass` or `fail`.
- **Universal Runtime Architectural Mechanism:** Three-State Gate Protocol (`pass`, `fail`, `pending`).
- **Concrete Fix Specification:** Update `kineti-state.ts` to allow `pending` as a valid gate state value.

##### [HIGH-07] Uncaught Runtime TypeError on Missing CLI Option Argument
- **Impacted Files & Lines:** `bin/kineti-memory-job.ts:74-75`.
- **Root Cause Analysis:** Passing `--dir` as final argument caused `process.argv[di + 1]` to evaluate to `undefined`, crashing unhandled inside `path.resolve(undefined)`.
- **Universal Runtime Architectural Mechanism:** Strict CLI Argument Validation.
- **Concrete Fix Specification:** Validate that option arguments exist and do not start with `-`; otherwise exit cleanly with code 2.

##### [HIGH-08] Installer Smoke Test Excluded from Default Test Runner
- **Impacted Files & Lines:** `package.json:8`, `tests/test-setup.sh`.
- **Root Cause Analysis:** `bun test tests/` only executed `.ts` files, leaving `test-setup.sh` completely skipped in CI.
- **Universal Runtime Architectural Mechanism:** Unified CI Verification Pipeline.
- **Concrete Fix Specification:** Update `package.json` test script to `"bun test tests/ && bash tests/test-setup.sh"`.

##### [HIGH-09] Critical Enforcement Subsystems Completely Untested
- **Impacted Files & Lines:** `bin/kineti-saga.ts`, `spend.ts`, `evidence.ts`, `egress.ts`, `tests/harness.test.ts`.
- **Root Cause Analysis:** Key enforcement paths (saga commit/rollback, stage spend ceilings, evidence command verification, egress hash chaining) had 0% test coverage.
- **Universal Runtime Architectural Mechanism:** Rigorous Layer 5 & Layer 6 Test Fixtures.
- **Concrete Fix Specification:** Add dedicated integration tests covering saga LIFO execution, spend ceiling tripwires, proof invalidation, and egress truncation detection.

---

#### 3.6.4 Medium Severity Technical Remediations (MED-01 to MED-12)

- **[MED-01] Saga Rollback Forbids Committed Runs Without Force Flag (`bin/kineti-saga.ts:15-20, 55-57`):** Committed runs are sealed to protect history; add `--force-committed` flag to allow Stage 12 (`watch`) automated incident recovery.
- **[MED-02] Zero Enforcement Tooling for Standing ETHOS Rule 8 (`ETHOS.md:44-48`):** Rule 8 mandates pre-commit scans for personal names/secrets; implement automated pre-commit scanner (`bin/kineti-scan.ts`) integrated into Stage 10 (`security`).
- **[MED-03] Missing UX Blueprint Integration in Intake & Design Skills (`skills/officehours/SKILL.md`, `design/`):** Update `officehours` and `design` skills to produce structured UX blueprints adhering to the 12 Mandatory UI Components.
- **[MED-04] Non-Atomic File Overwrite in Memory Journal Truncation (`bin/kineti-memory-job.ts:43-46`):** Replace in-place `fs.writeFileSync` with atomic tempfile write, `fsync()`, and POSIX `rename()` replacement.
- **[MED-05] Symlink Traversal & FIFO Blocking in Fingerprint Directory Walk (`bin/kineti-evidence.ts:22-39`):** Replace `fs.statSync` with `fs.lstatSync`; explicitly skip symlinks, named pipes (FIFOs), and sockets to prevent denial of service.
- **[MED-06] Unchecked Argument Parsing Shift Crash in Installer (`setup.sh:18`):** Check `$# -ge 2` before calling `shift 2` to prevent bash arithmetic syntax errors on missing parameters.
- **[MED-07] Unhandled Exit Code 1 in `read_conf` Under `set -eo pipefail` (`setup.sh:25-27`):** Replace pipe `grep | cut` with safe grep pattern or `awk -F= '{print $2}'` ensuring exit 0 on missing optional keys.
- **[MED-08] Incomplete Skill Installation Omitting Non-Markdown Subdirectories (`setup.sh:88-94`):** Replace single-file `cp SKILL.md` with recursive directory copying (`cp -R`) to preserve `scripts/`, `references/`, and `resources/`.
- **[MED-09] Incomplete Uninstall Leaving Orphan Repository Pointer (`setup.sh:55-69`):** Ensure `--uninstall` removes `~/.kineti/repo` and cleans up stale shell hooks.
- **[MED-10] Fragile Bun Path Resolution in Non-Interactive Cron Environments (`scripts/weekly.sh:11-12`):** Implement multi-path discovery (`$BUN_INSTALL/bin`, `~/.bun/bin`, `/usr/local/bin`) before falling back.
- **[MED-11] Temporary Directory Resource Leakage in Integration Test Suite (`tests/memory-job.test.ts:69`):** Wrap temporary directory creation and cleanup in `try...finally` or test teardown hooks.
- **[MED-12] TypeScript Strictness Gaps Masking Indexing Hazards (`tsconfig.json:8`):** Enable `"noUncheckedIndexedAccess": true` and resolve 10 undefined-indexing locations with strict type guards.

---

#### 3.6.5 Low Severity Technical Remediations (LOW-01 to LOW-08)

- **[LOW-01] Missing Executable Permission on CLI Binary (`bin/kineti-memory-job.ts`):** Set executable file mode `chmod 0755` and commit file mode to git.
- **[LOW-02] Inappropriate Executable Bit on Library Module (`bin/lib.ts`):** Remove executable bit `chmod 0644` as `lib.ts` is a pure import module.
- **[LOW-03] Hash Delimiter Collision Risk in Egress Receipt Ledger (`bin/kineti-egress.ts:14-16`):** Replace pipe delimiter string concatenation with RFC 8785 canonical JSON hashing.
- **[LOW-04] Unsanitized Log Injection (CWE-117) in Machine Alerts (`bin/kineti-verify-gate.ts:59`, `spend.ts:114`):** Sanitize carriage returns and newlines, writing structured JSON lines to alerts log.
- **[LOW-05] Record Type Schema Naming Mismatch (`kineti.config.json:40`):** Align `"project-dossier"` to standard `"dossier"`.
- **[LOW-06] Stale Reference in Officehours Skill to Deprecated `journal.md` (`skills/officehours/SKILL.md:32`):** Update reference to `.kineti/journal.jsonl`.
- **[LOW-07] Incomplete Skill Catalog Listing in Claude Hook Snippet (`hooks/claude.txt:4-5`):** Update prompt snippet to include all 17 skills (adding `anchors`, `second-opinion`, `skillify`).
- **[LOW-08] Dead and Conflicting `hook_file` Directives in Host Configurations (`hosts/*.conf:5`):** Clean up dead configuration directives and validate host manifests against a strict JSON schema.

---

#### 3.6.6 Informational Severity Technical Remediations (INFO-01 to INFO-10)

- **[INFO-01] Harness Program Count Mismatch in Documentation (`package.json:5`, `bin/README.md:3`):** Update documentation to reflect all 7 CLI enforcement programs.
- **[INFO-02] Absolute Path Reference `/design` in Standing Law (`ETHOS.md:9`):** Fix typo to reference repository-relative `design/screens/`.
- **[INFO-03] Unquoted Variable Expansion in Daily Loop Documentation (`docs/HOWTO-daily-loop.md:37-38`):** Wrap `$K/...` in quotes `"$K/kineti-evidence.ts"`.
- **[INFO-04] Repository Version Desynchronization Across Configs (`package.json:3`, `kineti.config.json:2`):** Synchronize all repository descriptors to SemVer `3.1.0`.
- **[INFO-05] Gate Count Ambiguity in Documentation vs Configuration (`kineti.config.json:10-16`, `WORKFLOWS.md:10`):** Align documentation to reflect 4 formal gates (Feasibility, Spec, Security, Ship).
- **[INFO-06] Philosophical Conflict on 12 UI Components (`skills/design/SKILL.md:65-66`):** Harmonize design skill to enforce the 12 Mandatory UI Components as strict standards.
- **[INFO-07] Hardcoded Skill Counts in Installer Smoke Test (`tests/test-setup.sh:18, 24`):** Replace hardcoded `17` with dynamic directory counting (`find skills -maxdepth 1 -mindepth 1 -type d | wc -l`).
- **[INFO-08] Unprotected Shell Glob Expansions Without Nullglob (`setup.sh:47, 61, 88`, `scripts/audit-skills.sh:12`):** Set `shopt -s nullglob` across all bash automation scripts.
- **[INFO-09] Unused Test Import in Harness Test Suite (`tests/harness.test.ts:1`):** Remove unused `beforeAll` import.
- **[INFO-10] Zero Automated Test Coverage for Utility Shell Scripts (`scripts/*.sh`):** Implement automated ShellSpec test runner for `setup.sh`, `weekly.sh`, and `audit-skills.sh`.

## 4. Solo-Founder Unit Economics & Monetization Engine (R3)

Building a venture-scale software enterprise as a solo builder requires **ruthless capital efficiency, automated product-led growth (PLG), and structural operating leverage**. Rather than hiring bloated sales, marketing, or support teams, Kineti OS monetizes through an open-core developer engine that converts individual developer utility into mandatory enterprise governance.

---

### 4.1 The Three-Tier Packaging & Pricing Matrix

The Kineti OS pricing architecture aligns directly with developer value and organizational risk:

```
+------------------------------------------------------------------------------------------------------------------------------------------+
|                                                   THE THREE-TIER PRICING MATRIX                                                          |
+------------------------------+-----------------------------------+-----------------------------------+-----------------------------------+
| Feature Dimension            | Tier 1: Open-Core (Free)          | Tier 2: Pro ($39/seat/mo)         | Tier 3: Enterprise ($250+/seat/mo)|
+------------------------------+-----------------------------------+-----------------------------------+-----------------------------------+
| Target Customer Profile      | Individual devs, OSS contributors | Senior engineers, indie builders  | Engineering orgs, Fintech, Health |
| Price Point & Billing        | $0 / month (Forever Free)         | $39 / mo ($390/yr annual, $49/mo) | $250 / mo ($3,000/yr, 10-seat min)|
| Deployment Architecture      | 100% Local-First CLI & MCP Server | Local Daemon + Cloud Sync Canvas  | Local Daemon + Enterprise VPC / GW|
| Host Ecosystem Support       | Universal (Claude, Codex, Cursor) | Universal + Visual Companion Canvas| Universal + CI/CD Governance Gates|
| State & Causal Memory        | Single-project SQLite / JSONL DAG | Multi-project E2EE Cloud Sync     | Federated Team DAG with RBAC & SSO|
| Verification & Proofs        | Local `kineti-evidence` proofs    | Live Merkle Traces & Replay Scrubber| Dual-Signed OVTs (Ed25519)      |
| Financial Spend Governance   | Basic local spend breaker ($50)   | Multi-model spend analytics       | Team budget pools & auto-halts    |
| Compliance & Audit Exports   | Local hash-chained egress logs    | Cloud trace backups               | SOC 2 / ISO 27001 / HIPAA Exports |
| Undo & Rollback Safety       | Local LIFO saga rollback stack    | Visual diff scrubber & undo picker| Automated canary rollback post-mor|
| Support SLA                  | GitHub Issues & Community Discord | Priority Email & VIP Discord      | 99.99% Attestation SLA, 1-hr resp |
+------------------------------+-----------------------------------+-----------------------------------+-----------------------------------+
```

#### Tier 1: Open-Core (The Developer Adoption Trojan Horse)
- **Distribution Model:** Distributed via `brew install kineti` and `npx @kineti/harness init`.
- **Zero-Friction Adoption:** Instantly registers with Claude Code, Cursor, and Antigravity via standard MCP stdio transport.
- **Strict Data Residency:** 100% of telemetry, logs, state, and code diffs remain strictly on the developer machine in `.kineti/`. Zero external cloud calls are required. This eliminates privacy friction for security-conscious developers.

#### Tier 2: Pro Tier ($39/seat/mo — The Productivity Multiplier)
- **Target Audience:** Individual engineers, consultants, and tech leads running agents across 5+ client repositories or multiple machines.
- **Core Conversion Catalyst:** The **Aside-Style Visual Companion Canvas**. Developers running multi-step agent loops cannot easily parse hundreds of terminal JSONL lines. The Pro canvas delivers real-time Merkle DAG rendering, interactive 1-click human approval modals, spend gauges, and interactive time-travel replay scrubbers.
- **Multi-Project Causal Synchronization:** End-to-end encrypted synchronization of causal learnings and run-records across desktop and laptop environments.

#### Tier 3: Team Tier ($79–$129/seat/mo, 3–25 seats — The Mid-Market Bridge)
- **Target Audience:** Engineering squads and high-growth startups (3 to 25 engineers) needing shared governance without enterprise procurement cycles.
- **Core Conversion Catalyst:** Self-serve credit card billing that bridges the cliff between $39/mo individual subscriptions and $30k/yr enterprise contracts.
- **Team Governance Primitives:**
  1. **Shared Spend Pools & Fleet Limits:** Centralized spend alerts and shared dollar ceilings across all team agent sessions.
  2. **Team Causal Graph Sync:** Distributed DAG merging across multiple developers working on the same repository.
  3. **Centralized Pull Request Gate:** GitHub Action PR status checks for the organization with shared policy enforcement.
  4. **Team Management Dashboard:** Multi-seat seat assignment, role-based access control, and team-wide spend analytics.

#### Tier 4: Enterprise Compliance Tier ($250+/seat/mo — The Risk Governance Standard)
- **Pricing & Terms:** $250 per seat/month ($3,000/seat/year), sold with a 10-seat contract minimum ($30,000 Annual Contract Value / ACV baseline).
- **Core Value Driver:** VPs of Engineering and CISOs cannot allow autonomous agents to push code to production without deterministic policy enforcement and non-repudiable auditability.
- **Enterprise Primitives:**
  1. **Cryptographically Dual-Signed OVTs:** Outcome Verification Tickets signed by both the local developer agent session key and the Kineti Enterprise Attestation Authority using Ed25519 cryptography.
  2. **CI/CD Governance Gates (`kineti-verify-gate` for GitHub Actions / GitLab CI):** Automatically blocks any Pull Request from merging if the code fingerprint does not match a verified, passing, unexpired OVT.
  3. **Turnkey Regulatory Audit Packages:** One-click compliance bundles containing Merkle inclusion proofs, egress receipts, spend logs, and verification test outputs, pre-formatted for SOC 2 Type II, ISO 27001, HIPAA, and FedRAMP auditors.

---

### 4.1.1 The getkineti.com Open-Source Move: Strategic Analysis, Code Boundaries & The "Services Trap" Fix

**Strategic Inquiry:** Does open-sourcing the base Kineti execution engine (Rust CLI v0.2.2 distributed via `getkineti.com`, crates.io, and `github.com/therawlogs/kineti`) compromise or cannibalize the commercial harness strategy?

**The Verdict:** **No. It is the single most important trust catalyst and top-of-funnel engine for Kineti—provided the intellectual property boundary is strictly defended.**

#### 1. Why Open-Core is Non-Negotiable for Developer Harnesses
In the modern developer ecosystem, **closed-source agent infrastructure dies on arrival**. No senior engineer, security officer, or enterprise team will grant root filesystem or terminal execution access to a closed proprietary binary that intercepts agent commands, wraps git hooks, and monitors files without full auditability.
- **The Historical Precedent:** Docker, Git, Terraform, Supabase, PostHog, Sentry, and Temporal achieved multi-billion-dollar enterprise outcomes precisely by establishing an open-source, local-first developer standard before selling commercial governance.
- **The Zero-Friction Trojan Horse:** An open-source local CLI (`kineti init`, `evidence`, `ship-check`, `receipt`) eliminates security paranoia, wins developer trust, earns organic GitHub stars, and bypasses enterprise procurement red tape. Developers adopt it locally because it solves their immediate pain—preventing agents from breaking tests and burning credit cards—for free.

#### 2. The Strict Open-Core Demarcation Matrix (What is Free vs. Commercial)
To preserve enterprise pricing power and prevent cloud hyperscalers from commoditizing Kineti, the codebase is formally partitioned:

```
+----------------------------------------------------------------------------------------------------------------------------------------+
|                                             OPEN-CORE CODE DEMARCATION & LICENSING MATRIX                                              |
+------------------------------------+---------------------------------------------------+--------------------+--------------------------+
| Component Layer                    | Scope & Functional Capabilities                   | License Model      | Distribution Channels    |
+------------------------------------+---------------------------------------------------+--------------------+--------------------------+
| Layer A: Community Client (Free)   | Local Rust CLI (`kineti v0.2.2+`), SHA-256        | MIT / Apache 2.0   | crates.io, GitHub,       |
|                                    | artifact fingerprinting (`evidence`), local spend | (Permissive OSS)   | `getkineti.com/install.sh|
|                                    | cap killswitch, git pre-commit hook (`ship-check`)|                    | brew, npx                |
|                                    | and standard stdio Model Context Protocol (MCP).  |                    |                          |
+------------------------------------+---------------------------------------------------+--------------------+--------------------------+
| Layer B: Commercial Pro ($39/mo)   | Aside-Style Floating Companion Canvas (WebSocket  | Business Source    | getkineti.com Pro license|
|                                    | `ws://127.0.0.1:8788`), Live Merkle DAG Visualizer| License (BSL 1.1 / | authenticated desktop    |
|                                    | Interactive Replay Scrubber, Multi-Repo E2EE      | ELv2 — Sentry model| binary download          |
|                                    | Causal Memory Sync.                               | converts in 36 mo) |                          |
+------------------------------------+---------------------------------------------------+--------------------+--------------------------+
| Layer C: Commercial Team ($79/mo)  | Shared Team Spend Pools, Multi-Repo DAG Sync,     | Business Source    | getkineti.com Team self- |
|                                    | Centralized PR Status Checks (up to 25 seats),    | License (BSL 1.1)  | serve credit card billing|
|                                    | Team Dashboard & Shared Quality Gate Templates.   |                    |                          |
+------------------------------------+---------------------------------------------------+--------------------+--------------------------+
| Layer D: Enterprise ($250+/seat/mo)| Central Attestation Authority (Ed25519 OVT signer)| Proprietary        | Private Docker Registry, |
|                                    | ISO SQL/PGQ Causal Property Graph Cluster, Fleet  | Commercial         | VPC Helm Chart, Kineti   |
|                                    | Budget Pools, CI/CD Gate GitHub Action, Turnkey   | (Annual Contracts) | Hosted Cloud Gateway     |
|                                    | SOC 2 / ISO 27001 / HIPAA Audit Packages, SLA.    |                    |                          |
+------------------------------------+---------------------------------------------------+--------------------+--------------------------+
```

#### 3. Re-Architecting getkineti.com: Escaping the "$5,000 Consulting Trap"
On `getkineti.com/pricing`, the current live offering states:
> *"Get team setup — $5K one time →"*

**The Strategic Hazard:** A $5,000 one-time fee inadvertently positions Kineti as an agency or boutique devtools consultancy rather than a scalable software platform. For a solo founder, custom one-off setups consume precious engineering hours, do not generate recurring revenue, and trade at low 1x–2x service multiples rather than the 15x–30x SaaS ARR multiples commanded by AI governance platforms.

**The Solution — Transition to the "Enterprise Pilot & Architecture Package":**
Transform the $5,000 entry point on `getkineti.com/pricing` into an **Enterprise Pilot Program**:
- **Price:** $5,000 upfront.
- **Contract Deliverables:**
  1. **90-Day Full License** for up to 10 Enterprise seats (normally a $7,500 retail value).
  2. **Bespoke Architecture Integration:** White-glove CI/CD pipeline setup and custom OTD schema mapping for the customer's proprietary codebase.
  3. **Executive Governance Audit Report:** A comprehensive quarterly report demonstrating exact tokens saved, regressions prevented, and audit-ready OVT records.
  4. **100% Contract Credit:** If the customer signs an annual Enterprise agreement ($30,000 ACV baseline) before day 90, the full $5,000 pilot fee is credited toward their first-year subscription.
- **The Solo Advantage:** This eliminates unscalable bespoke consulting, immediately qualifies serious enterprise buyers, generates non-dilutive working capital, and feeds directly into the compounding enterprise cohort model in Section 4.2.

#### 4. Progressive Brand Alignment on getkineti.com
The current tagline on `getkineti.com` reads:
> *"Ship proof and spend cap for any agent. Claude, Cursor, Grok, and fx produce artifacts — code, docs, data, configs. Kineti binds any verification to artifact hashes, caps the bill, and fails merge if the proof is stale."*

This is an exceptionally strong, pragmatic hook for developer utility. The blueprint unifies this public message into a 3-tier progressive revelation:
1. **Developer Hook (Current getkineti.com):** *"Ship proof and spend cap for any agent."* (Immediate, zero-jargon developer utility).
2. **Platform Hook (Q2 with Companion Canvas):** *"The Visual Control Plane & Live DAG Inspector for Autonomous Agents."* (Productivity and visibility for power users).
3. **Enterprise Hook (Q3 with Cloud Gateway):** *"The Context Integrity Protocol & Cryptographic Compliance Standard for Enterprise AI."* (SOC 2, ISO 27001, dual-signed OVTs).

By keeping the developer hook sharp on `getkineti.com`, the open-source CLI drives thousands of organic installations that naturally upgrade into the visual companion and enterprise compliance layers.

---

### 4.2 Comprehensive 24-Month Financial Pro-Forma Model

The following model details the quantitative trajectory for a solo founder scaling Kineti OS from launch to **$1.40M ARR in 12 months** and **$4.59M ARR in 24 months** via product-led growth and automated self-serve enterprise expansion.

#### Key Funnel Assumptions:
- **Cumulative Installs:** Scaling from 2,000 in Month 1 to 350,000 in Month 24.
- **Weekly Active Users (WAU):** 25% of cumulative installs remain actively engaged in developer workflows.
- **Free-to-Pro Conversion:** Scaling from 2.0% at launch to 4.5% at maturity (scaling from 2.0% at Month 2 to 4.08% at Month 10 and 4.46% at Month 24 as power-user retention and visual companion utility compound).
- **Pro Monthly Churn:** 1.5% (substantially lower than SaaS benchmarks due to workflow lock-in via causal memory).
- **Enterprise Expansion:** Enterprise account growth is modeled as a monthly compounding cohort expansion (0.15%/mo transition rate from mature Pro subscriber cohorts) combined with direct inbound/outbound enterprise lands (averaging 10–25 seats per land), scaling from 1 account in Month 4 to 18 accounts in Month 12 and 55 accounts in Month 24.
- **Enterprise Churn & NRR:** <0.4% monthly churn; 135% Net Revenue Retention (NRR) driven by team seat expansions.

```
+----------------------------------------------------------------------------------------------------------------------------------------+
|                                           24-MONTH FINANCIAL PRO-FORMA PROJECTION                                                      |
+-------+----------+--------+-----------+-----------+------------+------------+----------------+--------------------+--------------------+
| Month | Installs | WAU    | Pro Seats | Pro MRR   | Ent. Accts | Ent. Seats | Enterprise MRR | Total MRR          | Annual Run-Rate    |
+-------+----------+--------+-----------+-----------+------------+------------+----------------+--------------------+--------------------+
| M1    | 2,000    | 500    | 0         | $0        | 0          | 0          | $0             | $0                 | $0                 |
| M2    | 5,000    | 1,250  | 25        | $975      | 0          | 0          | $0             | $975               | $11.7k             |
| M3    | 10,000   | 2,500  | 65        | $2,535    | 0          | 0          | $0             | $2,535             | $30.4k             |
| M4    | 18,000   | 4,500  | 130       | $5,070    | 1          | 10         | $2,500         | $7,570             | $90.8k             |
| M5    | 25,000   | 6,250  | 210       | $8,190    | 1          | 10         | $2,500         | $10,690            | $128.3k            |
| M6    | 35,000   | 8,750  | 320       | $12,480   | 2          | 25         | $6,250         | $18,730            | $224.8k            |
| M7    | 48,000   | 12,000 | 460       | $17,940   | 4          | 45         | $11,250        | $29,190            | $350.3k            |
| M8    | 62,000   | 15,500 | 620       | $24,180   | 6          | 70         | $17,500        | $41,680            | $500.2k            |
| M9    | 80,000   | 20,000 | 800       | $31,200   | 8          | 95         | $23,750        | $54,950            | $659.4k            |
| M10   | 100,000  | 25,000 | 1,020     | $39,780   | 11         | 130        | $32,500        | $72,280            | $867.4k            |
| M11   | 125,000  | 31,250 | 1,280     | $49,920   | 14         | 170        | $42,500        | $92,420            | $1.11M             |
| M12   | 150,000  | 37,500 | 1,550     | $60,450   | 18         | 225        | $56,250        | $116,700           | $1.40M             |
| M15   | 200,000  | 50,000 | 2,150     | $83,850   | 26         | 360        | $90,000        | $173,850           | $2.09M             |
| M18   | 250,000  | 62,500 | 2,800     | $109,200  | 35         | 520        | $130,000       | $239,200           | $2.87M             |
| M21   | 300,000  | 75,000 | 3,350     | $130,650  | 44         | 700        | $175,000       | $305,650           | $3.67M             |
| M24   | 350,000  | 87,500 | 3,900     | $152,100  | 55         | 920        | $230,000       | $382,100           | $4.59M             |
+-------+----------+--------+-----------+-----------+------------+------------+----------------+--------------------+--------------------+
```

#### Strategic Milestone Inflections:
- **Month 11 ($1.11M ARR):** Reaches the $1M ARR threshold with $92.4k MRR in less than one year.
- **Month 18 ($2.87M ARR):** Achieves sustainable multi-million-dollar solo cashflow ($239.2k MRR).
- **Month 24 ($4.59M ARR):** Establishes dominant market leadership with 3,900 Pro subscribers and 55 Enterprise accounts governing 920 production seats.

---

### 4.3 Solo Operator Cost Structure, COGS, OpEx, and Operating Leverage

The structural economic advantage of Kineti OS is that **the local daemon offloads 95% of execution, storage, and policy evaluation to the developer's local workstation**. The central cloud infrastructure operates solely as a synchronization, metadata index, and cryptographic attestation gateway.

#### Monthly Operating Breakdown at Month 12 Baseline ($116,700 MRR / $1.40M ARR):

```
+----------------------------------------------------------------------------------------------------+
|                             SOLO OPERATOR MONTHLY COGS & OPEX BREAKDOWN                            |
+--------------------------------------------------------+---------------------+---------------------+
| Cost Category                                          | Monthly Cost (USD)  | % of Gross Revenue  |
+--------------------------------------------------------+---------------------+---------------------+
| 1. Cloudflare Workers / Vercel Edge API Hosting        | $450                | 0.39%               |
| 2. Supabase Postgres & Upstash Redis Sync Layer        | $750                | 0.64%               |
| 3. Cloudflare R2 / AWS S3 Merkle Proof Storage         | $250                | 0.21%               |
| 4. AWS CloudHSM / KMS (Ed25519 Attestation Master Keys)| $320                | 0.27%               |
| 5. Stripe Merchant Fees (2.9% + $0.30 per transaction) | $3,650              | 3.13%               |
+--------------------------------------------------------+---------------------+---------------------+
| **TOTAL COST OF GOODS SOLD (COGS)**                    | **$5,420**          | **4.64%**           |
+--------------------------------------------------------+---------------------+---------------------+
| **GROSS PROFIT**                                       | **$111,280**        | **95.36%**          |
+--------------------------------------------------------+---------------------+---------------------+
| 6. Sentry & PostHog Telemetry Tracing                  | $380                | 0.33%               |
| 7. Resend & Intercom Automated Customer Engagement     | $290                | 0.25%               |
| 8. GitHub Enterprise & CI/CD Verification Runners      | $200                | 0.17%               |
| 9. Vanta / Drata Automated SOC 2 Compliance Monitoring | $850                | 0.73%               |
| 10. Legal, Accounting & SaaS Administration Overhead   | $1,200              | 1.03%               |
+--------------------------------------------------------+---------------------+---------------------+
| **TOTAL OPERATING EXPENSES (OPEX)**                    | **$2,920**          | **2.50%**           |
+--------------------------------------------------------+---------------------+---------------------+
| **NET OPERATING INCOME (EBITDA)**                      | **$108,360**        | **92.85%**          |
+--------------------------------------------------------+---------------------+---------------------+
```
*(Note on Stripe Fees: The modeled $3,650/mo assumes standard payment processing of 2.9% + $0.30 per transaction with an effective ~15% annual billing mix among Pro and Enterprise accounts, reducing per-transaction fixed fees and matching actual merchant fee disbursements).*

#### Financial Operating Indicators:
- **Software Gross Margin:** **95.36%** (Significantly exceeding the 80–85% SaaS standard).
- **Operating EBITDA Margin:** **92.85%**, generating **$1,300,320 in net annual profit** directly to the solo founder at Month 12.
- **Headcount Scaling:** 1 Full-Time Employee (the Founder). Automated bug reproduction via Kineti run-records reduces customer support tickets by 90% compared to typical developer tools.

---

### 4.4 The Economics Paradigm Shift: $/Token to Cost Per Verified Outcome ($/Outcome)

#### The Breakdown of the $/Token Billing Model
Current LLM pricing is fundamentally misaligned with customer value:
- Foundational model labs bill by the million tokens ($3.00/M input, $15.00/M output).
- **Perverse Incentive:** If an autonomous agent enters a 40-step hallucination loop, burns 3,000,000 tokens, and fails, **the model provider makes $30.00 in profit while the customer loses $30.00 and receives broken code**.
- Token consumption does not correlate with business value. A 500k-token hallucinated refactor has negative value, while a 2k-token verified bug fix has multi-thousand-dollar value.

```
                      TRADITIONAL UNCONSTRAINED TOKEN PARADIGM
     [Prompt] ──► [Agent Reasoning Loop] ──► [2.5M Tokens Burned] ──► [Silent Outage]
                       (Provider earns $25.00)                 (Enterprise loses $25,000)

                        KINETI VERIFIED OUTCOME PARADIGM
     [Prompt] ──► [Kineti Runtime Gate] ──► [Sub-50ms Verification] ──► [Dual-Signed OVT]
                     (Prevents Runaway Spend)     (Guaranteed Passing Proof)  (Risk Reduced to Zero)
```

#### The Real-World Cost of Unverified Agent Failures
1. **Developer Triage & Context-Switching:** Senior engineers spend 4 to 8 hours bisecting commits, untangling hallucinated architectures, and manually testing unverified agent PRs: 6 hours @ $125/hr = **$750 per incident**.
2. **Corrupted Codebase Rollback & Data Repair:** If an agent executes destructive database migrations or overwrites configuration files without an inverse LIFO saga step: **$5,000 to $15,000 in engineering remediation**.
3. **Production Application Downtime:** According to Gartner and DORA benchmarks, enterprise downtime costs an average of **$5,600 per minute ($336,000 per hour)**. A 10-minute deployment outage caused by an unverified agent change costs **$56,000**.

#### Mathematical Enterprise ROI Formulation
Let:
- $N$ = Number of autonomous agent milestone tasks executed per month
- $P_f$ = Probability of an ungoverned agent generating an undetected regression ($2.5\%$ based on industry benchmarks)
- $C_f$ = Average blended cost of an unverified agent failure ($\$4,800$ weighted average of triage, rollback, and outage risk)
- $C_{kineti}$ = Kineti Enterprise cost per seat ($\$250/\text{month}$)
- $S$ = Number of developer seats

$$\text{Monthly Expected Risk Without Kineti} = N \times P_f \times C_f$$
$$\text{Net Monthly Enterprise Savings} = (N \times P_f \times C_f) - (S \times C_{kineti})$$
$$\text{Enterprise Return on Investment (ROI)} = \frac{\text{Net Monthly Savings}}{S \times C_{kineti}} \times 100\%$$

#### Enterprise ROI Sensitivity Matrix:

```
+------------------------------------------------------------------------------------------------------------------------------------+
|                                                ENTERPRISE ROI SENSITIVITY MATRIX                                                   |
+---------------+---------------+--------------------+---------------------+---------------------+-------------------+---------------+
| Team Size ($S$)| Monthly Tasks | Risk Without Kineti| Monthly Kineti Cost | Net Monthly Savings | Net Annual Savings| Enterprise ROI|
+---------------+---------------+--------------------+---------------------+---------------------+-------------------+---------------+
| 10 Engineers  | 1,000 tasks   | $120,000 / mo      | $2,500 / mo         | $117,500 / mo       | $1,410,000 / yr   | **4,700%**    |
| 25 Engineers  | 2,500 tasks   | $300,000 / mo      | $6,250 / mo         | $293,750 / mo       | $3,525,000 / yr   | **4,700%**    |
| 50 Engineers  | 5,000 tasks   | $600,000 / mo      | $12,500 / mo        | $587,500 / mo       | $7,050,000 / yr   | **4,700%**    |
| 100 Engineers | 10,000 tasks  | $1,200,000 / mo    | $25,000 / mo        | $1,175,000 / mo     | $14,100,000 / yr  | **4,700%**    |
+---------------+---------------+--------------------+---------------------+---------------------+-------------------+---------------+
```

**The Enterprise Sales Bottom Line:**  
At $250/seat/month, Kineti Enterprise delivers a **4,700% return on investment** and achieves a **payback period of 0.6 days (14 hours)**. For an enterprise buyer, purchasing Kineti is not an expense—it is an indispensable operational insurance policy that pays for itself if it catches even a single defective agent commit every six months.

## 5. Market Positioning & Developer Go-To-Market (GTM) (R4)

### 5.1 Exhaustive 10-Player Competitive Landscape Matrix & Unoccupied White Space

The AI tooling and agent infrastructure ecosystem is intensely active, but deeply fragmented. Competitors cluster into three isolated quadrants, leaving a massive, highly defensible, and high-margin market white space:

```
                                  ACTIVE RUNTIME GOVERNANCE
                                             ▲
                                             │         ★ KINETI OS
                                             │    (Deterministic Causal Runtime
                                             │     & Cryptographic Attestation)
                                             │
      IDE / WALLED GARDENS                   │                  AGENT FRAMEWORKS
      (Cursor, Windsurf)                     │                 (LangChain, CrewAI,
                                             │                  AutoGen, MetaGPT)
  ◄──────────────────────────────────────────┼──────────────────────────────────────────►
   PROPRIETARY / HOST-LOCKED                 │               HOST-AGNOSTIC / OPEN
                                             │
                                             │
                                             │
                                             │      OBSERVABILITY PLATFORMS
                                             │     (LangSmith, Arize Phoenix,
                                             │      Braintrust, Humanloop)
                                             ▼
                                  PASSIVE POST-HOC TELEMETRY
```

#### Detailed 10-Player Competitive Feature Comparison Matrix:

```
+-----------------------------------------------------------------------------------------------------------------------------------------+
|                                              10-PLAYER COMPETITIVE EVALUATION MATRIX                                                     |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| Evaluation Dimension     | Agent Frameworks             | Developer Tools / IDEs       | Observability Platforms    | Kineti OS         |
|                          | (LangChain, AutoGen, CrewAI) | (Cursor, Windsurf, Aside)    | (LangSmith, Arize, Braint) | (Universal Runtime)|
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 1. Primary Architectural | Prompt orchestration & agent | Inline autocomplete & editor | Post-hoc telemetry, trace  | **Active Runtime  |
|    Role                  | scaffolding in code          | chat composer                | logging & prompt evals     | Governance Engine**|
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 2. Host Ecosystem        | Python / TypeScript library  | Proprietary fork of VS Code  | SDK wrapper around LLM     | **Universal Host- |
|    Support               | lock-in                      | or browser (Aside.com)       | cloud API requests         | Agnostic Plugin** |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 3. Execution Intercept   | Soft application-level       | Editor UI inline suggestions | Passive observer; fires    | **Sub-50ms Atomic |
|    Capability            | callbacks only               | and terminal commands        | *after* execution finishes | Commit Gates**    |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 4. Context Integrity     | Probabilistic Vector RAG     | Vector embeddings + open     | None (Stores raw prompt &  | **Causal Graph    |
|    Substrate             | (Cosine distance search)     | editor tab heuristics        | response text strings)     | Substrate + OTD** |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 5. Execution Undo Safety | None (Manual ad-hoc shell    | Git undo / local file        | None                       | **Formal LIFO Saga|
|                          | command cleanups)            | history timeline             |                            | Rollback Stack**  |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 6. Financial Spend       | None or loose application    | Hard monthly credit quota    | Alert notifications after  | **Hardware-Style  |
|    Protection            | token counters               | per subscription             | budget overruns occur      | $50 Breaker**     |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 7. Cryptographic         | None                         | None                         | None                       | **Dual-Signed OVTs|
|    Attestation           |                              |                              |                            | & Merkle Proofs** |
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 8. Pre-Commit CI/CD Gate | None                         | None                         | None                       | **Automated PR    |
|    Enforcement           |                              |                              |                            | Verification Gate*|
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 9. Data Privacy & Local  | Depends on user's custom     | Source code sent to vendor   | Prompts and completions    | **100% Local-First|
|    Residency             | backend deployment           | cloud servers for processing | stored on vendor cloud     | Engine (Zero Egr)*|
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
| 10. Multi-Host           | Requires rewrites across     | Zero (Cannot run inside      | Requires vendor SDK        | **Native Support  |
|     Universality         | different frameworks         | Claude Code or Antigravity)  | instrumentations           | for 6 Major Hosts*|
+--------------------------+------------------------------+------------------------------+----------------------------+-------------------+
```

#### The Unoccupied White Space: Active Runtime Governance
1. **Frameworks build toys, not runtimes:** LangChain, AutoGen, and CrewAI provide scaffolding for demos, but lack deterministic system-level invariants. They cannot prevent an agent from wiping a repository or looping indefinitely.
2. **IDEs are walled gardens:** Cursor and Windsurf are exceptional editors, but enterprises cannot standardize on a single IDE. Enterprise developers use VS Code, JetBrains, Vim/Neovim, and terminal-based agents (Claude Code). Governance must exist at the **runtime process and protocol layer**, not inside an editor binary.
3. **Observability is too late:** LangSmith, Arize Phoenix, and Braintrust are passive recorders. Telling an engineering director that an agent burned $200 and corrupted a repository *after the fact* is useless. Kineti intercepts, validates, and rolls back actions **in real time**.

---

### 5.2 0-to-1 and 1-to-10 Viral Developer GTM Roadmap

```
+----------------------------------------------------------------------------------------------------+
|                                      SOLO FOUNDER GTM ROADMAP                                      |
+----------------------------------------------------------------------------------------------------+
|  MONTHS 1–6 (0-to-1): DEVELOPER MINDSHARE    │  MONTHS 7–18 (1-to-10): MONETIZATION & EXPANSION    |
|  • Technical research essays (HN / X)        │  • Launch Visual Companion Canvas (Pro $39/mo)      |
|  • Launch open-core CLI & MCP server         │  • Automated self-serve Stripe checkout             |
|  • Smithery.ai & official MCP registries     │  • Free-to-Enterprise inbound domain qualification |
|  • Signed Git commit viral attribution loop  │  • Launch Enterprise Compliance Tier ($250/mo)      |
|  • GitHub Actions verify-gate open release   │  • Partner with Vanta/Drata for SOC 2 attestation   |
+----------------------------------------------------------------------------------------------------+
```

#### 5.2.1 0-to-1 Phase: Developer Authority & Viral Distribution (Months 1–6)
As a solo founder, paid advertising and outbound sales forces are unviable. Growth must be 100% organic and driven by high-signal engineering authority:

1. **High-Impact Technical Research Publishing:**
   - Authoritative long-form technical essays published on Hacker News, Substack, and X:
     - *"Why Vector RAG Fails Autonomous Coding Agents: Moving from Semantic Similarity to Causal-Graph Substrates"*
     - *"The 44 Architectural Flaws in AI Coding Harnesses (And How to Fix Them)"*
     - *"Stop Paying for Tokens: The Case for Cost Per Verified Outcome"*
   - Goal: Establish technical thought leadership and drive 15,000+ GitHub stars and 35,000+ local CLI downloads in the first 6 months.

2. **Dominance in the Model Context Protocol (MCP) Ecosystem & Multi-Channel Distribution:**
   - Kineti leverages its live multi-channel distribution engine (`getkineti.com`, crates.io, Homebrew, and npm):
     ```bash
     # Live production installation (Rust binary v0.2.2+)
     curl -fsSL https://getkineti.com/install.sh | sh
     # or crates.io:
     cargo install kineti

     # One-command universal MCP server registration
     kineti mcp init # or npx @kineti/harness init
     ```
   - Automatically injects configuration into `claude_desktop_config.json`, Cursor MCP settings, and Antigravity profiles. Featured on MCP directory registries (Smithery.ai, PulseMCP).

3. **Viral Developer Attribution Loops (The "Powered by Stripe" for Agents):**
   - **GitHub Action PR Verification Badges (Primary Flywheel):** Every Pull Request verified by Kineti automatically receives a clean, non-intrusive status check and markdown summary badge:
     ```markdown
     > **Kineti Verified** (OVT: `9a7f...2c4b`)
     > ✅ Stage 6 Spec Gate Passed · ✅ Stage 10 Security Audit Passed · Spend: $1.42 / $10.00 Limit
     > [Inspect Cryptographic Merkle Proof](https://verify.kineti.dev/9a7f2c4b)
     ```
   - **Opt-in Git Commit Trailers:** For open-source developers who choose to enable public verification badges, Kineti supports standard, lint-friendly Conventional Commit trailers (`Signed-off-by: ...`, `Outcome-Proof: ...`), avoiding CI commit-linter rejections while allowing verifiable provenance attribution in public repos.

4. **Free GitHub Actions CI/CD Integration:**
   - Release `kineti-io/verify-action@v1` on the GitHub Marketplace. Developers can add a 4-line YAML step to their CI pipeline that enforces Kineti proof verification on all AI-generated PRs:
     ```yaml
     - name: Verify Agent Outcome
       uses: kineti-io/verify-action@v1
       with:
         require_ovt: true
         max_spend_limit: 10.00
     ```

#### 5.2.2 1-to-10 Phase: Product-Led Monetization & Enterprise Expansion (Months 7–18)
1. **The Visual Companion Canvas as the Paid Conversion Catalyst:**
   - Developers love the free CLI, but when managing complex 10-step autonomous runs, reading JSONL terminal logs becomes overwhelming.
   - The Aside-style Visual Companion Canvas (launching in Month 6) provides a stunning real-time desktop view: live Merkle DAG trees, interactive human approval modals, and spend gauges. The visual experience converts free CLI users to the $39/mo Pro tier at 3.0–4.0%.
2. **Bottom-Up Enterprise Land-and-Expand:**
   - Developers adopt Kineti individually to prevent Claude Code or Cursor from breaking their local code.
   - When 3 to 5 developers in a company are using Kineti, the Kineti CLI detects domain clustering (e.g., `@stripe.com`, `@brex.com`) and displays an in-CLI prompt:
     ```text
     ┌──────────────────────────────────────────────────────────────────┐
     │ 5 engineers at your organization are using Kineti OS.            │
     │ Enable team-wide compliance gates and SOC 2 audit exports with   │
     │ Kineti Enterprise: https://kineti.dev/enterprise/stripe          │
     └──────────────────────────────────────────────────────────────────┘
     ```
   - This triggers automated inbound requests directly to the VP of Engineering or CISO.
3. **Automated Compliance Partnerships (Vanta, Drata):**
   - Package Kineti audit exports into pre-built integrations for compliance automation platforms (Vanta, Drata). Enterprises undergoing SOC 2 or ISO 27001 certification are required to prove control over AI coding tools; Kineti becomes the recommended default vendor.

---

### 5.3 Plain English Positioning

To cut through noisy AI marketing jargon, Kineti uses crisp, non-academic analogies that immediately click with both developers and enterprise buyers:

#### Primary Positioning Analogies:
- **"The Stripe for Agent Accountability"**  
  *Context:* Just as Stripe transformed complex, opaque banking regulations and payment gateways into a 3-line embeddable API, Kineti transforms non-deterministic agent chaos, context degradation, and compliance liability into a clean, deterministic local runtime.
- **"The Datadog for Agent Governance"**  
  *Context:* Datadog monitors cloud infrastructure and alerts when servers fail. Kineti actively governs autonomous agents—intercepting bad commands, enforcing budgets, rolling back breaking changes, and proving verified outcomes.

#### Multi-Persona Messaging Breakdown:

```
+----------------------------------------------------------------------------------------------------+
|                                    MULTI-PERSONA MESSAGING MATRIX                                  |
+---------------------+---------------------------------------+--------------------------------------+
| Persona             | Core Pain & Anxiety                   | Kineti Plain English Promise         |
+---------------------+---------------------------------------+--------------------------------------+
| Senior Developer /  | "AI agents burn my tokens, write      | *"Never clean up after an agent      |
| Tech Lead           | subtle bugs across 15 files, and leave| again. Kineti guarantees safe saga   |
|                     | me to clean up the mess."             | rollbacks, locks your goal forever,  |
|                     |                                       | and keeps your context clean."       |
+---------------------+---------------------------------------+--------------------------------------+
| Head of Security /  | "Developers are using autonomous AI   | *"Total visibility and zero leakage. |
| CISO                | tools that run unvetted shell commands| Kineti enforces tamper-evident hash- |
|                     | and could leak proprietary IP."       | chained egress logs and blocks       |
|                     |                                       | unauthorized shell execution."       |
+---------------------+---------------------------------------+--------------------------------------+
| VP of Engineering / | "I want the productivity of AI agents,| *"Deploy autonomous agents with      |
| CTO                 | but I can't risk a multi-million-     | mathematical confidence. Kineti signs|
|                     | dollar outage or failed SOC 2 audit." | cryptographic verification tickets   |
|                     |                                       | and blocks unverified PRs in CI/CD." |
+---------------------+---------------------------------------+--------------------------------------+
```

## 6. Strategic M&A Playbook & Acquisition Moat (R5)

### 6.1 The Strategic Acquisition Thesis: The Missing Enterprise Operating System Layer for AI

Frontier AI laboratories (Anthropic, OpenAI, Google DeepMind) and cloud developer platforms (Microsoft/GitHub, Atlassian, Cloudflare) are engaged in an intense multi-billion-dollar race for developer mindshare. However, their core business models face a shared, existential vulnerability: **they produce raw intelligence, but lack the deterministic runtime governance layer required for enterprise deployment**.

Without context integrity, causal DAG memory, and verifiable outcome gates, enterprise leaders cannot permit autonomous agents to write code directly to mission-critical production repositories. Kineti OS represents the **missing operating system layer** between frontier models and enterprise infrastructure.

```
+----------------------------------------------------------------------------------------------------+
|                                    THE M&A ACQUISITION LANDSCAPE                                   |
+----------------------------------------------------------------------------------------------------+
|                                                                                                    |
|   DEVSECOPS & PLATFORMS              ENTERPRISE CLOUD SUITES             FRONTIER AI LABS          |
|   [GitHub/Microsoft, GitLab, Snyk]   [Atlassian, Datadog, Cloudflare]    [Anthropic, OpenAI]       |
|   Valuation: $18M–$28M (M12)         Valuation: $15M–$25M (M12)          Valuation: $20M–$35M (M12)|
|   Valuation: $55M–$85M (M24)         Valuation: $50M–$75M (M24)          Valuation: $60M–$95M (M24)|
|                                                                                                    |
|   Strategic Urgency:                 Strategic Urgency:                  Strategic Urgency:        |
|   • Defend the Pull Request gate     • Connect goals to code diffs       • Prevent context drift   |
|   • Turnkey SOC 2 / OVT compliance   • Active runtime governance         • Move to $/Outcome       |
|   • Agent supply chain security      • Expand developer observability    • Multi-IDE governance    |
|                                                                                                    |
+----------------------------------------------------------------------------------------------------+
```

---

### 6.2 Strategic Buyer Profile Mapping

#### 6.2.1 Buyer Group 1: DevSecOps & Developer Platforms (Priority Alpha — Highest Strategic Alignment)

##### Target 1A: Microsoft / GitHub (Priority Alpha)
- **Strategic Imperative:** GitHub Copilot Workspace and Codespaces are racing to dominate agentic software engineering. But enterprise customers demand strict compliance, pull-request verification, and auditability before agents touch production code.
- **Why GitHub Acquires Kineti OS:**
  1. **Turnkey GitHub Actions Governance:** Kineti’s `kineti-verify-gate` becomes the default native verification engine inside GitHub Enterprise and GitHub Actions.
  2. **Defending the Pull Request:** As AI generates 80%+ of code, the pull request shifts from manual human code review to **cryptographic outcome verification**. Kineti owns that verification layer.
- **Projected Acquisition Valuation:** **$22M – $35M at M12** (15x–25x ARR) · **$65M – $95M at M24** (14x–21x ARR).

##### Target 1B: GitLab
- **Strategic Imperative:** GitLab positions itself as the complete DevSecOps platform. As autonomous agents generate code directly, GitLab needs an embedded runtime policy harness to prevent hallucinations and supply chain attacks.
- **Projected Acquisition Valuation:** **$20M – $30M at M12** · **$60M – $85M at M24**.

##### Target 1C: Snyk
- **Strategic Imperative:** Snyk leads developer security. Kineti's real-time egress monitoring and tamper-evident Merkle provenance provide Snyk with a runtime AI governance product.
- **Projected Acquisition Valuation:** **$18M – $25M at M12** · **$55M – $75M at M24**.

---

#### 6.2.2 Buyer Group 2: Enterprise Cloud Platforms & Observability

##### Target 2A: Atlassian (Jira / Bitbucket)
- **Strategic Imperative:** Atlassian faces irrelevance as autonomous AI coding tools bypass Jira and Confluence. Kineti connects high-level project goals (Stage 1 Officehours) directly to verified code diffs and Merkle proof tickets, embedding Atlassian into the agentic loop.
- **Projected Acquisition Valuation:** **$18M – $28M at M12** · **$55M – $80M at M24**.

##### Target 2B: Datadog
- **Strategic Imperative:** Datadog LLM Observability monitors passive prompt/token telemetry. Kineti adds active execution interception, spend limits, and deterministic rollbacks.
- **Projected Acquisition Valuation:** **$16M – $24M at M12** · **$50M – $70M at M24**.

---

#### 6.2.3 Buyer Group 3: Frontier AI Laboratories (Anthropic, OpenAI, DeepMind)
- **Strategic Imperative:** Frontier labs focus on model training and foundation intelligence. While they develop baseline developer tools (Claude Code, Codex), acquiring a mature, cross-host governance engine accelerates enterprise compliance deployment and outcome-based pricing models.
- **Valuation Multiple Realism:** Acquired as an accretive platform layer at standard market multiples: **$20M – $35M at M12** (14x–25x ARR) or **$60M – $95M at M24**.

---

### 6.3 The 4-Pillar Defensible IP Moat

Kineti OS is engineered to resist commoditization. Its technological moat rests on four proprietary architectural pillars that cannot be replicated by prompt engineering or vector search:

```
+----------------------------------------------------------------------------------------------------+
|                                   THE 4-PILLAR DEFENSIBLE IP MOAT                                  |
+------------------------------------------------+---------------------------------------------------+
| 1. Causal-Graph Substrates (ISO SQL/PGQ)       | 2. Dual-Signed Outcome Verification Tickets (OVT) |
| • Eliminates vector RAG hallucinations         | • Ed25519 cryptographic non-repudiation           |
| • Strict relational causal semantics           | • Merkle inclusion proofs for CI/CD gates         |
| • Time-ordered DAG edge validation             | • Mathematical guarantee of code provenance       |
+------------------------------------------------+---------------------------------------------------+
| 3. Runtime Ontology Trigger Data (OTD)         | 4. Universal Neutrality & Cross-Host Protocol     |
| • Sub-50ms deterministic state injections      | • Headless daemon decoupled from any single LLM   |
| • Dynamic boundary enforcement                 | • Interoperable across Claude, OpenAI, Cursor, etc|
| • Context integrity across 100+ turns          | • Immune to single-vendor model obsolescence      |
+------------------------------------------------+---------------------------------------------------+
```

1. **Causal-Graph Substrates with ISO SQL/PGQ vs. Commoditized Vector RAG:**  
   Vector embeddings measure *semantic proximity*, which fundamentally conflates correlation with causation. In a complex software codebase, two functions may appear semantically similar but have completely incompatible execution lifecycles. Kineti implements formal graph theory using ISO SQL/PGQ standards: nodes (Goals, Tasks, Tools, Diff, Evidence) are linked by typed causal edges (`CAUSED_BY`, `DEPENDS_ON`, `VERIFIED_BY`, `ROLLED_BACK_BY`). The runtime runs topological sorting and cycle detection, deterministically pruning invalid agent reasoning paths.
2. **Cryptographically Dual-Signed Outcome Verification Tickets (OVTs):**  
   An OVT binds a specific code diff hash to a verified test execution fingerprint, an egress ledger root, and a spend receipt. It is dual-signed using asymmetric cryptography (Ed25519): signed once by the ephemeral local agent session key, and countersigned by the Kineti Attestation Gateway. It is mathematically impossible to forge an OVT without invalidating the Merkle root.
3. **Runtime Ontology Trigger Data (OTD):**  
   Instead of stuffing 100k tokens of raw documentation into a prompt, Kineti’s OTD engine detects the active stage and task state, injecting exactly the minimal, deterministic boundary constraints required for the next atomic step. This keeps prompt tokens minimal and context fidelity near 100%.
4. **Universal Host Neutrality:**  
   Proprietary tools like Cursor cannot become the universal enterprise standard because they require developers to abandon their preferred IDEs. Kineti’s protocol-level architecture (MCP + local daemon) operates invisibly beneath any tool, making it the Switzerland of agent governance.

---

### 6.4 The Dual-Track Strategy: Cashflow Independence vs. Acquisition Leverage

A solo founder’s greatest strategic asset in negotiations is **the ability to walk away**. When a startup is burning venture capital, acquirers can force distressed sales or lowball acquihires ($5M–$15M).

Kineti OS executes a **Dual-Track Milestone Strategy**:

```
                              THE DUAL-TRACK LEVERAGE ENGINE
                                             ▲
                                             │
                       TRACK A: PROFITABLE CASHFLOW INDEPENDENCE
                 ($1M-$4M ARR, 92%+ EBITDA Margin, Zero External Debt)
                                             │
                                             ▼
                 Gives Founder Absolute "Walk-Away" Leverage in Negotiations
                                             │
                                             ▼
                        TRACK B: PRE-EMPTIVE M&A AUCTION WAR
                  (Anthropic vs. OpenAI vs. GitHub Competitive Bidding)
                                             │
                                             ▼
                            $100M–$200M+ STRATEGIC ACQUISITION
```

1. **Track A: Profitable Independence (The Fortress of Cashflow):**  
   By maintaining >95% gross margins and zero headcount, Kineti generates **$1.3M in net cash at Month 12** and **$3.5M+ at Month 24**. The founder never needs outside venture funding. Any acquisition offer below $50M is easily declined because the founder already owns 100% of a multi-million-dollar cash-flowing software monopoly.
2. **Track B: Pre-Emptive Acquisition Auction:**  
   Kineti becomes the de facto governance standard across multiple competing hosts (Claude Code, Cursor, Codex). If Anthropic considers acquiring Kineti to lock in Claude Code governance, Microsoft/GitHub must evaluate the defensive risk of losing control over enterprise agent CI/CD. This dynamic triggers a **competitive bidding war**, driving exit valuations to strategic multiples (**25x–50x ARR**, or **$100M–$200M+**).

---

## 7. 12-Month Phased Engineering & Commercial Execution Roadmap

Below is the quarterly execution roadmap detailing engineering, product, commercial, and M&A deliverables for a solo builder across the first 12 months:

```
+----------------------------------------------------------------------------------------------------+
|                                  12-MONTH QUARTERLY EXECUTION ROADMAP                              |
+----------------------------------------------------------------------------------------------------+
|  Q1: CORE DAEMON HARDENING & MCP STANDARD (Months 1–3)                                             |
|  • Technical: Fix 44 audit defects, implement sub-50ms atomic commit gates & JSONL fsync           |
|  • Product: Release unified `kineti-daemon` CLI and stdio MCP server for Claude Code & Cursor      |
|  • Commercial: Publish foundational research essays on HN/X; hit 10k installs, $30k ARR           |
|  • M&A / Governance: File provisional patents on Causal DAG Runtime OTD & Dual-Signed OVTs         |
+----------------------------------------------------------------------------------------------------+
|  Q2: VISUAL COMPANION CANVAS & PRO MONETIZATION LAUNCH (Months 4–6)                                |
|  • Technical: Build Aside-style Visual Sidecar canvas (`:8788`) with live Merkle DAG inspector     |
|  • Product: Release Pro Tier ($39/mo) with multi-project sync, interactive gates, and spend gauges |
|  • Commercial: Launch self-serve Stripe checkout; hit 35k installs, 320 Pro subscribers, $224k ARR  |
|  • M&A / Governance: Establish informal engineering dialogues with Anthropic & GitHub teams       |
+----------------------------------------------------------------------------------------------------+
|  Q3: ENTERPRISE ATTESTATION GATEWAY & DUAL-SIGNED OVTS (Months 7–9)                                |
|  • Technical: Deploy AWS CloudHSM Ed25519 signing gateway and GitHub Actions CI/CD verify gate     |
|  • Product: Launch Enterprise Compliance Tier ($250/mo, 10-seat floor) with SOC 2 audit exports    |
|  • Commercial: Partner with Vanta and Drata; land first 8 Enterprise accounts; reach $659k ARR     |
|  • M&A / Governance: Retain boutique tech M&A advisor to structure inbound interest               |
+----------------------------------------------------------------------------------------------------+
|  Q4: ENTERPRISE SCALING & PRE-EMPTIVE M&A DUAL-TRACK (Months 10–12)                                |
|  • Technical: DuckDB ISO SQL/PGQ causal graph query engine; automated multi-repo canary rollbacks  |
|  • Product: Federated team DAG dashboard with Okta SAML/SSO and RBAC                               |
|  • Commercial: Cross $1.40M ARR ($116.7k MRR) with 1,550 Pro users and 18 Enterprise customers    |
|  • M&A / Governance: Run dual-track process: Scale to $4.5M ARR or execute $120M-$200M+ M&A exit   |
+----------------------------------------------------------------------------------------------------+
```

### 7.1 Q1: Core Daemon Hardening & MCP Standard (Months 1–3)
- **Technical Deliverables:**
  - Eradicate all 44 defect categories from `docs/AUDIT_REPORT.md` (CRIT-01 to INFO-10).
  - Implement the Context Integrity Protocol (CIP) Layer 1–5 local daemon in Bun/Rust.
  - Implement resilient JSONL stream parsing with POSIX `fsync` logging and atomic tempfile renaming.
  - Standardize on Model Context Protocol (MCP) stdio and SSE transports.
- **Commercial Deliverables:**
  - Publish research papers: *"Beyond Vector Search: Causal-Graph Substrates"* and *"Stop Paying for Tokens: The Case for Cost Per Verified Outcome"*.
  - Distribute via Homebrew (`brew install kineti`) and npm registry (`npx @kineti/harness init`).
  - Target: 10,000 installs, 2,500 WAU, $30.4k ARR.

### 7.2 Q2: Visual Companion Canvas & Pro Monetization Launch (Months 4–6)
- **Technical Deliverables:**
  - Develop the Aside-style Visual Companion Canvas (Vite/React 19/Tailwind/Radix UI/Lucide) running on `http://127.0.0.1:8788`.
  - Implement live bidirectional WebSocket protocol (`kineti.gate.*`, `kineti.dag.*`, `kineti.spend.*`).
  - Implement 1-click human approval gate modal with syntax-highlighted unified blast-radius diffs.
- **Commercial Deliverables:**
  - Launch Pro Tier ($39/seat/month) with automated Stripe billing.
  - Release Cursor IDE extension (`.vsix`) on Open VSX and VS Code Marketplace.
  - Target: 35,000 cumulative installs, 320 Pro subscribers, $224.8k ARR.

### 7.3 Q3: Enterprise Attestation Gateway & Dual-Signed OVTs (Months 7–9)
- **Technical Deliverables:**
  - Build the Cloud Attestation Gateway with AWS KMS/CloudHSM Ed25519 signing authorities.
  - Implement RFC 8785 Canonical JSON Serialization (JCS) and W3C Verifiable Credentials formatting.
  - Release `kineti-io/verify-action@v1` on GitHub Actions Marketplace.
- **Commercial Deliverables:**
  - Launch Enterprise Compliance Tier ($250/seat/month with $30k ACV floor).
  - Establish compliance referral partnerships with Vanta and Drata.
  - Target: 80,000 cumulative installs, 800 Pro subscribers, 8 Enterprise customers (95 seats), $659.4k ARR.

### 7.4 Q4: Enterprise Scaling & Pre-Emptive M&A Dual-Track (Months 10–12)
- **Technical Deliverables:**
  - Embed DuckDB in-process OLAP engine for sub-12ms ISO SQL/PGQ causal graph traversals across historical repositories.
  - Build automated canary rollback supervisor with signed incident post-mortems.
  - Implement Okta/Azure AD SAML 2.0 SSO and enterprise RBAC.
- **Commercial Deliverables:**
  - Cross **$1.40M ARR** ($116.7k MRR) with 1,550 Pro users and 18 Enterprise accounts (225 seats).
  - Maintain **>95% gross margins** and **>90% EBITDA**, netting $1.3M in annual cashflow.
- **M&A Execution:**
  - Activate Track B acquisition discussions with Anthropic, Microsoft/GitHub, and OpenAI.
  - Leverage Track A cashflow independence to command an acquisition valuation of **$120M – $200M+**.

## 8. Appendix: Complete Wire Protocols & Canonical Schemas

### 8.1 Appendix A: Complete Universal 20-Entity JSON-LD Schema

Universal 20-Entity Provenance Kernel vocabulary (`https://schema.kineti.ai/v1/context.jsonld`):

```json
{
  "@context": {
    "@version": 1.1,
    "kineti": "https://schema.kineti.ai/v1/",
    "xsd": "http://www.w3.org/2001/XMLSchema#",
    "id": "@id",
    "type": "@type",
    "Agent": "kineti:Agent",
    "Session": "kineti:Session",
    "Host": "kineti:Host",
    "Goal": "kineti:Goal",
    "Milestone": "kineti:Milestone",
    "Task": "kineti:Task",
    "ToolCall": "kineti:ToolCall",
    "ToolResult": "kineti:ToolResult",
    "Observation": "kineti:Observation",
    "Artifact": "kineti:Artifact",
    "CodeDiff": "kineti:CodeDiff",
    "Assertion": "kineti:Assertion",
    "GateResult": "kineti:GateResult",
    "SpendEntry": "kineti:SpendEntry",
    "RollbackAction": "kineti:RollbackAction",
    "Checkpoint": "kineti:Checkpoint",
    "OTDTrigger": "kineti:OTDTrigger",
    "EvidenceLog": "kineti:EvidenceLog",
    "MerkleLeaf": "kineti:MerkleLeaf",
    "OVT": "kineti:OutcomeVerificationTicket",
    "sessionId": {"@id": "kineti:sessionId", "@type": "xsd:string"},
    "rootGoal": {"@id": "kineti:rootGoal", "@type": "xsd:string"},
    "rootGoalHash": {"@id": "kineti:rootGoalHash", "@type": "xsd:string"},
    "codeFingerprint": {"@id": "kineti:codeFingerprint", "@type": "xsd:string"},
    "merkleRoot": {"@id": "kineti:merkleRoot", "@type": "xsd:string"},
    "totalSessionSpendUsd": {"@id": "kineti:totalSessionSpendUsd", "@type": "xsd:decimal"},
    "spendMicrocents": {"@id": "kineti:spendMicrocents", "@type": "xsd:integer"},
    "createdAt": {"@id": "kineti:createdAt", "@type": "xsd:dateTime"},
    "status": {"@id": "kineti:status", "@type": "xsd:string"},
    "signature": {"@id": "kineti:signature", "@type": "xsd:string"}
  }
}
```

#### 8.1.1 Universal 20-Entity Provenance Kernel Complete Property Vocabulary
To guarantee exhaustive semantic interoperability and prevent data loss during RDF parsing, every property of the 20 kernel entities maps to formal `kineti:` vocabulary terms:

```json
{
  "@context": {
    "agentId": "kineti:agentId",
    "agentRole": "kineti:agentRole",
    "workingDirectory": "kineti:workingDirectory",
    "parentAgentId": "kineti:parentAgentId",
    "hostName": "kineti:hostName",
    "adapterType": "kineti:adapterType",
    "hookMechanism": "kineti:hookMechanism",
    "milestoneName": "kineti:milestoneName",
    "targetDate": "kineti:targetDate",
    "taskId": "kineti:taskId",
    "toolName": "kineti:toolName",
    "inputParameters": "kineti:inputParameters",
    "outputPayload": "kineti:outputPayload",
    "executionDurationMs": "kineti:executionDurationMs",
    "observationType": "kineti:observationType",
    "artifactPath": "kineti:artifactPath",
    "contentHash": "kineti:contentHash",
    "diffContent": "kineti:diffContent",
    "gateName": "kineti:gateName",
    "verdict": "kineti:verdict",
    "amountUsd": "kineti:amountUsd",
    "cumulativeSpendUsd": "kineti:cumulativeSpendUsd",
    "inverseAction": "kineti:inverseAction",
    "checkpointHash": "kineti:checkpointHash",
    "triggerPredicate": "kineti:triggerPredicate",
    "leafIndex": "kineti:leafIndex",
    "parentHashes": "kineti:parentHashes",
    "nodeHash": "kineti:nodeHash",
    "ticketId": "kineti:ticketId",
    "evidenceHash": "kineti:evidenceHash",
    "complianceProof": "kineti:complianceProof"
  }
}
```

---

### 8.2 Appendix B: Complete Runtime OTD Activation & State Machine JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "KinetiRuntimeOTDActivationPacket",
  "type": "object",
  "required": [
    "packet_id",
    "session_id",
    "timestamp",
    "previous_ontology_state",
    "active_ontology_state",
    "triggering_event",
    "injected_context_directives",
    "enforced_boundaries"
  ],
  "properties": {
    "packet_id": { "type": "string", "format": "uuid" },
    "session_id": { "type": "string", "format": "uuid" },
    "timestamp": { "type": "string", "format": "date-time" },
    "previous_ontology_state": { 
      "type": "string", 
      "enum": ["SESSION_GENESIS", "DIAGNOSIS_MODE", "SPEC_LOCK_MODE", "BUILD_SAFE_MODE", "AUTO_REPAIR_MODE", "RECOVERY_MODE"] 
    },
    "active_ontology_state": { 
      "type": "string", 
      "enum": ["DIAGNOSIS_MODE", "SPEC_LOCK_MODE", "BUILD_SAFE_MODE", "AUTO_REPAIR_MODE", "RECOVERY_MODE"] 
    },
    "triggering_event": {
      "type": "object",
      "required": ["event_type", "event_id", "summary"],
      "properties": {
        "event_type": { "type": "string" },
        "event_id": { "type": "string" },
        "summary": { "type": "string" }
      }
    },
    "injected_context_directives": {
      "type": "array",
      "items": { "type": "string" }
    },
    "enforced_boundaries": {
      "type": "object",
      "required": ["revoked_tools", "authorized_scopes", "max_turn_spend_usd"],
      "properties": {
        "revoked_tools": { "type": "array", "items": { "type": "string" } },
        "authorized_scopes": { "type": "array", "items": { "type": "string" } },
        "max_turn_spend_usd": { "type": "number" }
      }
    }
  }
}
```

---

### 8.3 Appendix C: Complete Outcome Verification Ticket (OVT) W3C VC JSON-LD Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OutcomeVerificationTicketVerifiableCredential",
  "type": "object",
  "required": ["@context", "id", "type", "issuer", "issuanceDate", "credentialSubject", "proof"],
  "properties": {
    "@context": {
      "type": "array",
      "items": { "type": "string" }
    },
    "id": { "type": "string", "format": "uri" },
    "type": {
      "type": "array",
      "items": { "type": "string" }
    },
    "issuer": { "type": "string" },
    "issuanceDate": { "type": "string", "format": "date-time" },
    "credentialSubject": {
      "type": "object",
      "required": [
        "id",
        "sessionId",
        "rootGoal",
        "rootGoalHash",
        "codeFingerprint",
        "merkleRoot",
        "outcomes",
        "policyGatesPassed",
        "totalSessionSpendUsd",
        "spendMicrocents"
      ],
      "properties": {
        "id": { "type": "string" },
        "sessionId": { "type": "string", "format": "uuid" },
        "rootGoal": { "type": "string" },
        "rootGoalHash": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
        "codeFingerprint": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
        "merkleRoot": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
        "outcomes": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["milestone", "status", "testProofHash", "spendUsd"],
            "properties": {
              "milestone": { "type": "string" },
              "status": { "type": "string", "enum": ["VERIFIED_PASS", "FAILED"] },
              "testProofHash": { "type": "string", "pattern": "^[a-f0-9]{64}$" },
              "spendUsd": { "type": "number" }
            }
          }
        },
        "policyGatesPassed": {
          "type": "array",
          "items": { "type": "string" }
        },
        "totalSessionSpendUsd": { "type": "number" },
        "spendMicrocents": { "type": "integer" }
      }
    },
    "proof": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["type", "created", "verificationMethod", "proofPurpose", "proofValue"],
        "properties": {
          "type": { "type": "string" },
          "created": { "type": "string", "format": "date-time" },
          "verificationMethod": { "type": "string" },
          "proofPurpose": { "type": "string" },
          "proofValue": { "type": "string" }
        }
      }
    }
  }
}
```

---

### 8.4 Appendix D: WebSocket Bidirectional Control Protocol Wire Specification

All WebSocket frames exchanged over `ws://127.0.0.1:8788` follow standard JSON-RPC 2.0 formatting:

#### 1. Inbound (Sidecar -> Daemon): Subscribe to DAG Updates
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "kineti.dag.subscribe",
  "params": {
    "session_id": "787db8bf-e17f-440a-abec-0fbfbb7ecae3",
    "include_historical_nodes": true
  }
}
```

#### 2. Outbound (Daemon -> Sidecar): Causal Node Created Broadcast
```json
{
  "jsonrpc": "2.0",
  "method": "kineti.dag.node_created",
  "params": {
    "node_id": "9a7f8b2c-4e6d-4a1b-8c2d-3e5f7a9b0c1d",
    "node_type": "Action",
    "label": "Applied migration 004_create_causal_nodes.sql",
    "timestamp": "2026-09-06T05:31:00Z",
    "merkle_leaf_hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "edges": [
      {
        "target_node_id": "8f6e7d1c-3b5a-490a-9b1c-2d4e6f8a9b0c",
        "relationship_type": "CAUSED_BY"
      }
    ]
  }
}
```

#### 3. Inbound (Sidecar -> Daemon): Trigger Saga Rollback Step
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "kineti.saga.rollback_step",
  "params": {
    "session_id": "787db8bf-e17f-440a-abec-0fbfbb7ecae3",
    "target_step_number": 14,
    "force": false
  }
}
```

#### 4. Outbound (Daemon -> Sidecar): Saga Rollback Completed
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "status": "rolled_back",
    "unwound_steps": 1,
    "current_git_head": "4a1b8c2d",
    "code_fingerprint": "1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e7f809"
  }
}
```

---

*End of Strategy Blueprint Specification.*
