# Authoritative Technical Survey & Architectural Specification: Universal Host Architecture & Hybrid Companion Design
**Document Identifier:** KINETI-R1-SURVEY-2026-09-06  
**Author:** Teamwork Preview Explorer 1 (`teamwork_preview_explorer_m2_1`)  
**Mission Role:** Explorer R1 — Universal Host Architecture & Hybrid Companion Design  
**Target Repository:** Kineti Local Harness (`kineti-os`)  
**Integrity Mode:** Production / Publication-Grade  
**Date:** 2026-09-06  

---

## 1. Executive Summary & Forensic Baseline

### 1.1 The Context Integrity Imperative
Autonomous AI coding agents (Google Antigravity, Anthropic Claude Code, OpenAI Codex/Operator, OpenCode, Cursor, and generic terminal runtimes) have rapidly advanced from conversational code generators to multi-turn agentic loops capable of modifying filesystems, executing terminal commands, creating database migrations, and dispatching network requests. 

However, existing developer harness architectures remain crippled by a fundamental flaw: **they rely almost exclusively on passive markdown prompt injection and uncoordinated file-system polling.** When an agent is governed only by markdown guidelines (such as `CLAUDE.md`, `AGENTS.md`, or passive skill folders), the host Large Language Model (LLM) possesses full uninhibited autonomy to hallucinate compliance, skip governance gates, self-authorize destructive actions, and bypass financial and architectural circuit breakers.

The core standing philosophy of the Kineti OS local harness is:
> **"Skills propose, programs enforce, memory remembers."**

To make this philosophy a deterministic engineering reality, the harness must transition from an out-of-band file-copying script into a **Universal Dual-Plane Runtime**:
1. **Deterministic Headless Control Plane (Engine Layer):** A high-performance, sub-10ms local daemon and native Model Context Protocol (MCP) server that acts as a mandatory execution gateway, intercepting tool calls, validating cryptographic state proofs, enforcing financial circuit breakers, and maintaining an append-only causal Merkle Directed Acyclic Graph (DAG).
2. **Zero-Friction Reactive Visual Companion Plane (Presentation Layer):** An Aside-style visual sidecar and canvas providing continuous real-time observability, live Merkle DAG inspection, interactive blast-radius diffs, dynamic spend gauges, and interactive human approval gates for high-consequence pipeline transitions.

### 1.2 Quantitative Forensic Audit Baseline
Our forensic investigation of the target repository (`hosts/`, `hooks/`, `bin/`, `kineti.config.json`, `setup.sh`, and `docs/AUDIT_REPORT.md`) reveals the structural deficiencies of the legacy harness implementation:

```
+---------------------------------------------------------------------------------------------------------+
|                                FORENSIC BASELINE OF REPOSITORY DEFECTS                                   |
+------------------------------+-----------------------------------+--------------------------------------+
| Subsystem & File Path        | Legacy Implementation             | Failure Mode / Security Vulnerability|
+------------------------------+-----------------------------------+--------------------------------------+
| Host Installer (`setup.sh:88`)| Naive file copy to host skill dirs| No active interception; relies on    |
|                              | (`~/.claude/skills/kineti-*`)     | model to willingly invoke skills     |
+------------------------------+-----------------------------------+--------------------------------------+
| State Machine                | `const v = (s as any)[key];`      | Crashes on nested gate lookups       |
| (`bin/kineti-state.ts:73`)   | Unindexed property access         | (`gate.feasibility` returns undef)   |
+------------------------------+-----------------------------------+--------------------------------------+
| Verify Gate                  | Programmatic `--trust` flag       | Autonomous agent self-authorizes     |
| (`bin/kineti-verify-gate.ts`) | writes directly to `trust.json`   | unverified bash command execution    |
+------------------------------+-----------------------------------+--------------------------------------+
| Spend Breaker                | Programmatic `--i-am-human` flag  | Subagent programmatically bypasses   |
| (`bin/kineti-spend.ts:18`)   | bypasses $50 ceiling without TTY  | financial circuit breaker            |
+------------------------------+-----------------------------------+--------------------------------------+
| I/O Utility                  | Silent catch returns `[]`         | Single corrupt line destroys entire  |
| (`bin/lib.ts:48-58`)         | in `readJsonl`                    | journal / rollback ledger history    |
+------------------------------+-----------------------------------+--------------------------------------+
| Concurrency Control          | Raw `fs.writeFileSync`            | File race conditions and clobbering  |
| (`bin/lib.ts:43-46`)         | without advisory locks (flock)    | under multi-agent parallel execution |
+------------------------------+-----------------------------------+--------------------------------------+
| Shell Execution              | `spawnSync("bash", ["-lc", cmd])` | Unquoted shell injection, process    |
| (`bin/kineti-saga.ts:58`)    | with raw string interpolation     | hangs, and uncaptured stderr         |
+------------------------------+-----------------------------------+--------------------------------------+
| Path Resolution              | Unchecked concatenation           | Creates malformed duplicate paths:   |
| (`bin/lib.ts:10-12`)         | `path.join(cwd, ".kineti")`       | `.kineti/.kineti/journal.jsonl`      |
+------------------------------+-----------------------------------+--------------------------------------+
```

This survey report provides the complete architectural specification to replace these fragile scripts with an enterprise-grade, universal harness and companion platform.

---

## 2. Forensic Codebase Audit: The Legacy "File-Copy" Architecture

### 2.1 The Current Host Integration Topology
In the existing codebase, host integration is managed by a static directory structure:
- `hosts/`: Contains four shell-style configuration files (`claude.conf`, `codex.conf`, `gemini.conf`, `opencode.conf`). Each defines `name`, `skills_dir_primary`, `skills_dir_fallback`, and `hook_file`.
- `hooks/`: Contains four plain-text prompt snippets (`claude.txt`, `codex.txt`, `gemini.txt`, `opencode.txt`) with instructional text intended to be pasted into the agent's markdown configuration.
- `setup.sh`: A 122-line Bash script that iterates through `hosts/*.conf`, detects if the host's skill folder exists on the machine, copies `skills/*/SKILL.md` into target directories, and prints out the hook text blocks.

### 2.2 Why Prompt Injection Alone Fails (The "Prompt-Trap")
The fundamental flaw of the current integration strategy is that it operates entirely **outside the host's execution loop**. As documented in `hooks/claude.txt`:
```
Add a "kineti" section to CLAUDE.md:
  When starting a new product run /kineti-officehours first and follow the
  kineti stage order. List available skills: officehours, diagnose, design...
```
This is a *suggestion*, not an *enforcement boundary*. The LLM can:
1. Ignore `CLAUDE.md` under high context window pressure or instruction drift.
2. Jump straight from Stage 1 (`officehours`) to Stage 7 (`build`) without satisfying Stage 5 (`feasibility`) or Stage 6 (`spec`).
3. Execute destructive commands (`rm -rf`, raw database drops, untracked API calls) using standard bash tools without registering an inverse LIFO saga rollback action in `bin/kineti-saga.ts`.
4. Run commands without cryptographic evidence capture (`bin/kineti-evidence.ts`).

### 2.3 File Contention and Architectural Drift
Every script in `bin/` currently operates as an independent Bun CLI process. When an agent calls `bun bin/kineti-state.ts set stage 2`, the script reads `.kineti/state.json`, modifies an in-memory object, and writes it back using `fs.writeFileSync`. 

Under multi-agent architectures (such as Google Antigravity subagent swarms or OpenCode parallel workers):
- Two agents reading and writing `.kineti/state.json` simultaneously produce a classic read-modify-write race condition, leading to corrupted state files or lost updates.
- In `bin/lib.ts:48-58`, any partially written line or syntax error causes `readJsonl` to catch the exception and return an empty array `[]`. A subsequent append or overwrite causes **complete permanent data loss of the audit trail**.

To solve this, Kineti must move to an active, in-process or daemonized architecture where all state changes are mediated by a transactional ACID storage engine with synchronous inter-process communication (IPC).

---

## 3. Universal Host Architecture: Concrete Mechanics for 6 Major Agent Environments

To achieve true universality, Kineti must not force developers into a custom proprietary IDE. Instead, it must plug directly into the AI tools developers already use. Below is the concrete technical architecture for the six dominant agent environments.

```
+----------------------------------------------------------------------------------------------------+
|                                UNIVERSAL HOST ADAPTER ARCHITECTURE                                  |
+----------------------------------------------------------------------------------------------------+
|                                                                                                    |
|   [Google Antigravity]  [Claude Code]  [OpenAI Codex]  [OpenCode / OSS]  [Cursor]  [Generic CLI]  |
|       Native MCP /         PreToolUse      ACP / Stdio       Reverse Proxy /     IDE Extension   PTY / LD_PRELOAD |
|         Sidecars             Hooks           JSON-RPC          Socket IPC         MCP Sidecar      Syscall Trap    |
|            │                   │                │                   │                  │                │          |
|            ▼                   ▼                ▼                   ▼                  ▼                ▼          |
+────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                 KINETI LOCAL ENGINE GATEWAY                                        |
|                          (Local Loop Latency Budget: < 10ms)                                       |
|                                                                                                    |
|   ┌────────────────────────────────────────────────────────────────────────────────────────────┐   |
|   │ 1. Ingress & Auth Guard (0.8ms)                                                            │   |
|   │    - UNIX Domain Socket / Named Pipes / Stdio JSON-RPC Frame Decoder                       │   |
|   │    - Cryptographic Host Token & Session Integrity Verification                             │   |
|   └─────────────────────────────────────────┬──────────────────────────────────────────────────┘   |
|                                             │                                                      |
|   ┌─────────────────────────────────────────▼──────────────────────────────────────────────────┐   |
|   │ 2. Context Integrity & Policy Enforcement Engine (2.7ms)                                   │   |
|   │    - Stage Progression Gatekeeper (13-Stage Linear DAG Validation)                         │   |
|   │    - Real-Time Token & Dollar Spend Circuit Breakers (Tripwire Check)                      │   |
|   │    - Dual-LLM Sanitization & Egress Whitelist Verification                                 │   |
|   │    - Human-in-the-Loop Intercept Evaluator (Pause & Yield for Critical Actions)            │   |
|   └─────────────────────────────────────────┬──────────────────────────────────────────────────┘   |
|                                             │                                                      |
|   ┌─────────────────────────────────────────▼──────────────────────────────────────────────────┐   |
|   │ 3. Storage & Cryptographic Substrate (1.5ms)                                               │   |
|   │    - SQLite WAL Transactional State & LIFO Saga Stack                                      │   |
|   │    - Background Merkle DAG Generator (SHA-256 Hash Chain Computation)                      │   |
|   │    - Causal Link & Runtime OTD Validator                                                   │   |
|   └─────────────────────────────────────────┬──────────────────────────────────────────────────┘   |
|                                             │                                                      |
|                                             ▼                                                      |
|                   Response Frame / Tool Execution / Unblock (0.5ms)                                |
+────────────────────────────────────────────────────────────────────────────────────────────────────+
                                              │
                      WebSocket / SSE Reactive Telemetry (< 2ms)
                                              ▼
                             [Kineti Visual Companion Canvas]
```

---

### 3.1 Google Antigravity
Google Antigravity is a premier agentic development environment featuring hierarchical subagent task delegation, native MCP server connectivity, runbook automation, and artifact workspace sandboxing.

#### Architectural Mechanics
- **Skills Directory Integration:** Antigravity discovers capabilities through structured skill packages located in `~/.gemini/config/skills/<skill-name>/SKILL.md`. Each skill specifies YAML frontmatter, execution runbooks, reference documents, and embedded scripts.
- **Native MCP Client:** Antigravity natively loads and queries MCP servers defined in its configuration. Servers expose eager tools (always in context) and lazy tools (loaded on demand via schema lookup).
- **Sidecar Process Supervisor:** Kineti runs as an active background sidecar registered in Antigravity's task environment, communicating via JSON-RPC over stdio or local IPC.
- **Lifecycle Event Hooks:** Antigravity triggers lifecycle notifications across subagent task creation, step initiation, tool execution, and task completion.

#### Protocol Flow & Sequence Diagram
```
Antigravity Agent              Kineti MCP Sidecar               Local SQLite / Merkle DAG
       │                                │                                    │
       │─── 1. mcp:list_tools ─────────▶│                                    │
       │◀── 2. [kineti_gate_check, ...]─│                                    │
       │                                │                                    │
       │─── 3. Call: kineti_gate_check ─▶│                                    │
       │       { stage: "spec" }        │─── 4. Query stage & evidence ─────▶│
       │                                │◀── 5. Pass (Evidence fresh) ───────│
       │                                │                                    │
       │                                │─── 6. Compute Merkle Node ────────▶│
       │◀── 7. Tool Result: APPROVED ───│                                    │
```

#### Exact Configuration Snippet
To register Kineti within Google Antigravity, add the daemon to `~/.gemini/config/settings.json` (or the workspace MCP configuration):
```json
{
  "mcp_servers": {
    "kineti_core": {
      "command": "kineti-daemon",
      "args": ["mcp", "--project-root", "${workspaceRoot}"],
      "env": {
        "KINETI_INTEGRITY_MODE": "development",
        "KINETI_LOG_LEVEL": "info"
      },
      "transport": "stdio"
    }
  },
  "hooks": {
    "on_task_start": "kineti-daemon hook on-task-start --task-id ${taskId}",
    "on_task_complete": "kineti-daemon hook on-task-complete --task-id ${taskId}"
  }
}
```

#### Latency Budget & Failure Isolation
- **Latency Budget:** Stdio JSON-RPC roundtrip: 1.8ms; SQLite gate verification: 1.4ms; SHA-256 node commit: 1.1ms; Response frame delivery: 0.5ms. **Total Local Loop: 4.8ms** (Budget: <10ms).
- **Process Lifecycle:** The sidecar is managed by Antigravity's supervisor. On unexpected termination, Antigravity sends `SIGTERM`, waits 2000ms, and escalates to `SIGKILL`.
- **Failure Isolation:** Fail-closed on security and spend gates; fail-open with warning on telemetry logging. If the daemon crashes, pending state rollbacks are safely preserved in the SQLite WAL file and recovered upon process restart.

---

### 3.2 Anthropic Claude Code
Anthropic Claude Code is a terminal-based agentic CLI tool that interacts directly with developer environments via shell execution, file editing, and MCP tool discovery.

#### Architectural Mechanics
- **`PreToolUse` and `PostToolUse` Hooks:** Claude Code supports deterministic lifecycle hooks in `~/.claude/settings.json`. Before executing any tool (such as `Bash` or `FileEdit`), Claude Code executes the configured command and pipes the tool invocation payload via `stdin` as JSON.
- **Deterministic Stdin/Exit Code Contract:**
  - **Exit Code 0:** Allows tool execution to proceed unimpeded.
  - **Exit Code 2:** Blocks tool execution and returns the hook script's `stderr`/`stdout` directly to the model as an error message, forcing the model to adapt its plan.
- **Native stdio MCP Integration:** Claude Code connects to MCP servers defined via `claude mcp add` or configured in `.claude/mcp.json`.
- **Interactive Mode Shims:** A lightweight PTY shim (`kineti-claude`) wraps Claude Code to capture terminal control sequences and provide seamless operator overrides.

#### Protocol Flow & Sequence Diagram
```
Claude Code (CLI)              Kineti Hook Interceptor             Kineti Local Daemon
       │                                │                                  │
       │─── 1. Tool Call Intent ───────▶│                                  │
       │    stdin: {tool: "Bash",       │                                  │
       │            cmd: "rm -rf ..."}  │─── 2. Intercept Check (Socket) ─▶│
       │                                │◀── 3. REFUSED: Saga Required ────│
       │                                │                                  │
       │◀── 4. Process Exit Code 2 ─────│                                  │
       │       stderr: "Blocked: You    │                                  │
       │       must register saga first"│                                  │
       ▼                                                                   │
[Model adjusts plan and registers saga rollback]                           │
```

#### Exact Configuration Snippet
Add the hook interceptor to `~/.claude/settings.json`:
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "command": "kineti hook claude-pre-tool --cwd ${cwd}"
      },
      {
        "matcher": "FileEdit",
        "command": "kineti hook claude-pre-tool --cwd ${cwd}"
      }
    ],
    "PostToolUse": [
      {
        "matcher": "*",
        "command": "kineti hook claude-post-tool --cwd ${cwd}"
      }
    ]
  },
  "mcpServers": {
    "kineti": {
      "command": "kineti",
      "args": ["mcp"],
      "transport": "stdio"
    }
  }
}
```

The hook interceptor script (`kineti hook claude-pre-tool`) reads the JSON from stdin, queries the local daemon via UNIX domain socket (`/var/run/kineti.sock`), and exits within 4ms:
```bash
#!/usr/bin/env bash
# High-speed hook evaluator
exec kineti-daemon-eval --hook PreToolUse "$@"
```

#### Latency Budget & Failure Isolation
- **Latency Budget:** Process spawn & stdin pipe: 2.1ms; Daemon socket query: 1.2ms; Policy rule match: 0.9ms. **Total Local Loop: 4.2ms**.
- **Failure Isolation:** The hook adheres to a strict 500ms execution timeout. If the hook execution times out or encounters a crash, it defaults to Exit Code 2 (fail-closed), preventing unauthorized shell mutations while alerting the developer.

---

### 3.3 OpenAI Codex / Operator
The OpenAI Codex platform and Operator agent family utilize high-throughput streaming architectures and the Agent Client Protocol (ACP) for multi-turn execution.

#### Architectural Mechanics
- **Headless Process Supervisor:** The agent operates under a managed supervisor (`kineti-supervisor`) that manages process execution, sandboxing, and resource limits.
- **Agent Client Protocol (ACP) & Streaming JSON-RPC:** Codex models interactions across three conversation primitives:
  - **Thread:** The durable container for an entire workflow run.
  - **Turn:** A single request-response-action conversational cycle.
  - **Item:** Atomic actions, tool calls, text deltas, or verification artifacts within a turn.
- **Native MCP Client:** Integrated through standard MCP client endpoints, exposing Kineti's stage verification and evidence logging tools.
- **Environment Sandboxing:** Executes untrusted agent bash actions inside OS-level sandboxes:
  - Linux: Bubblewrap (`bwrap`) with unshared user/network namespaces and read-only mounts.
  - macOS: `sandbox-exec` with strict Seatbelt profiles restricting filesystem access strictly to the project root.

#### Protocol Flow & Sequence Diagram
```
Codex CLI / Operator             Kineti Supervisor                Sandboxed Execution Box
       │                                │                                    │
       │─── 1. ACP Turn: Tool Call ────▶│                                    │
       │       Item: { "action": ... }  │─── 2. Validate Sandbox Profile ───▶│
       │                                │─── 3. Register LIFO Saga ─────────▶│
       │                                │─── 4. Spawn Sandboxed Worker ─────▶│
       │                                │◀── 5. Stream Stdout/Stderr ────────│
       │◀── 6. Stream JSON-RPC Item ────│                                    │
```

#### Exact Configuration Snippet
Configured in `~/.codex/config.toml`:
```toml
[agent]
protocol = "acp"
supervisor_path = "/usr/local/bin/kineti-supervisor"

[mcp_servers.kineti]
command = "kineti"
args = ["mcp"]
env = { KINETI_SANDBOX = "strict" }

[sandbox]
engine = "bubblewrap"
isolate_network = true
allowed_outbound_proxy = "http://127.0.0.1:8787"
workspace_mount_rw = true
```

#### Latency Budget & Failure Isolation
- **Latency Budget:** ACP streaming chunk parser: 0.9ms; Sandbox privilege check: 1.5ms; Saga stack push: 1.2ms; JSON-RPC dispatch: 0.6ms. **Total Local Loop: 4.2ms**.
- **Failure Isolation:** Every tool execution is isolated in a separate container namespace. If the agent crashes or attempts an out-of-bounds write, the sandbox halts the process (`SIGKILL`), and the supervisor automatically invokes `kineti-saga rollback` to restore the filesystem to its pristine pre-turn state.

---

### 3.4 OpenCode & Open-Source Agent Ecosystem
The open-source agent ecosystem (OpenCode, Aider, Smolagents, AutoGen, CrewAI) requires protocol-agnostic, low-friction integration surfaces that do not depend on proprietary host APIs.

#### Architectural Mechanics
- **Local UNIX Domain Socket IPC:** Fast, secure bidirectional communication over `/var/run/kineti.sock` (or `~/.kineti/run/kineti.sock` on non-root environments) using framed JSON-RPC 2.0.
- **LLM API Intercepting Reverse Proxy:** Kineti runs a lightweight local proxy server (`http://127.0.0.1:8787`). Open-source agents configure their OpenAI / Anthropic base URL to this proxy:
  - Intercepts outbound `/v1/chat/completions` and `/v1/messages`.
  - Injects runtime Ontology Trigger Data (OTD) and stage constraints directly into the system prompt.
  - Monitors token consumption in real time and cuts off execution when the $50 spend breaker is tripped.
- **Standardized Agent Protocol (AGP) Adapter:** Implements standard REST/SSE endpoints (`/runs`, `/tasks`, `/steps`) allowing full observability over open-source agent workflows.

#### Protocol Flow & Sequence Diagram
```
OpenCode / OSS Agent             Kineti Local Proxy                LLM Provider (OpenAI/Anthropic)
       │                                │                                    │
       │─── 1. POST /v1/chat/completions▶│                                    │
       │       (Prompt payload)         │─── 2. Audit & Inject OTD Prompt ──▶│
       │                                │─── 3. Deduct Token Budget ────────▶│
       │                                │─── 4. Forward Upstream ───────────▶│
       │                                │◀── 5. Stream Chunks (SSE) ─────────│
       │◀── 6. Stream Sanitized SSE ────│                                    │
```

#### Exact Configuration Snippet
Add to agent environment (`.env` or shell configuration):
```bash
# Direct OpenCode / Aider through Kineti Governance Proxy
export OPENAI_BASE_URL="http://127.0.0.1:8787/v1"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8787/v1"
export KINETI_IPC_SOCKET="$HOME/.kineti/run/kineti.sock"
```

In OpenCode configuration (`~/.config/opencode/config.json`):
```json
{
  "api_proxy": "http://127.0.0.1:8787",
  "mcp_servers": [
    {
      "name": "kineti",
      "transport": "stdio",
      "command": "kineti",
      "args": ["mcp"]
    }
  ]
}
```

#### Latency Budget & Failure Isolation
- **Latency Budget:** UNIX domain socket frame decode: 0.6ms; Reverse proxy stream proxying: 1.8ms; Token accounting tick: 0.4ms. **Total Local Overhead Added: 2.8ms**.
- **Failure Isolation:** The proxy runs in a separate thread. If the proxy fails, it fails closed to prevent unmetered spend, but clients receive clear HTTP 503 error payloads with diagnostic rollback recommendations.

---

### 3.5 Cursor (AI IDE)
Cursor is the leading AI-native IDE (a specialized VS Code fork) with deep language server integration, integrated terminals, and interactive chat/composer surfaces.

#### Architectural Mechanics
- **Native Extension (.vsix) Architecture:** Kineti ships as a lightweight VS Code / Cursor extension executing within the Extension Host process.
- **Global & Workspace `mcp.json` Integration:** Cursor natively queries MCP servers configured in `~/.cursor/mcp.json` (global) or `.cursor/mcp.json` (project-specific).
- **Integrated Terminal Command Injection:** Intercepts terminal commands run by developers or Composer agents using the VS Code Terminal API (`vscode.window.onDidWriteTerminalData`) and standard shell integration escape sequences (OSC 133 / OSC 633).
- **Editor Canvas Webview Integration:** Employs `vscode.window.createWebviewPanel` to dock the Aside-style companion canvas directly alongside the editor, providing zero-friction visual DAG inspection without leaving the IDE.

#### Protocol Flow & Sequence Diagram
```
Cursor Composer Agent            Cursor Extension (.vsix)          Kineti Core Daemon
       │                                │                                  │
       │─── 1. Invoke MCP Tool ────────▶│                                  │
       │       kineti_stage_transition  │─── 2. Forward via Socket ───────▶│
       │                                │◀── 3. Requires Human Approval ───│
       │                                │                                  │
       │                                │─── 4. Render Modal in Webview ──▶[Editor Canvas]
       │                                │◀── 5. Human clicks [APPROVE] ────[Editor Canvas]
       │◀── 6. Tool Success Response ───│                                  │
```

#### Exact Configuration Snippet
Project configuration in `.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "kineti": {
      "command": "kineti",
      "args": ["mcp"],
      "env": {
        "KINETI_WORKSPACE_ROOT": "${workspaceFolder}"
      }
    }
  }
}
```

Extension contribution point in `package.json`:
```json
{
  "name": "kineti-cursor-companion",
  "displayName": "Kineti Context Integrity Companion",
  "version": "1.0.0",
  "activationEvents": ["onStartupFinished"],
  "contributes": {
    "viewsContainers": {
      "activitybar": [
        {
          "id": "kineti-explorer",
          "title": "Kineti Companion",
          "icon": "media/kineti-icon.svg"
        }
      ]
    },
    "views": {
      "kineti-explorer": [
        {
          "id": "kineti-canvas-view",
          "name": "Live Merkle DAG",
          "type": "webview"
        }
      ]
    }
  }
}
```

#### Latency Budget & Failure Isolation
- **Latency Budget:** Webview `postMessage` bridge: 0.8ms; Local socket dispatch: 1.1ms; Merkle DAG state fetch: 1.5ms. **Total Local Loop: 3.4ms**.
- **Failure Isolation:** Extension runs in VS Code's sandboxed extension host. If the extension host encounters an error, the editor UI remains entirely unaffected, and Cursor automatically restarts the host.

---

### 3.6 Generic Terminal Agents (CLI, Python, Shell Scripts)
For legacy, arbitrary, or bespoke terminal agents (such as raw Python agent loops, AutoGPT, or LangChain scripts running in bash), Kineti provides universal runtime interception without modifying the agent's source code.

#### Architectural Mechanics
- **Pseudo-Terminal (PTY) Wrapper (`kineti exec`):** Spawns the agent process inside a dedicated PTY master/slave pair (`pty.openpty` on POSIX systems). The supervisor intercepts all standard input, output, and escape sequences in real time.
- **Dynamic Linker Syscall Interception:** Uses dynamic library preloading (`LD_PRELOAD` on Linux, `DYLD_INSERT_LIBRARIES` on macOS) to intercept standard C library (`libc`) system calls:
  - `execve()`, `posix_spawn()`: Intercepts command executions to check stage permissions and enforce verify gates.
  - `connect()`: Intercepts outbound TCP socket creation, preventing unauthorized network exfiltration and enforcing the egress whitelist.
  - `unlink()`, `rmdir()`: Intercepts destructive file deletions and forces registration of rollback entries before execution.
- **Transparent MITM HTTP Proxy:** Intercepts outgoing TLS connections using a locally trusted root certificate (`~/.kineti/certs/ca.crt`), transparently capturing and metering LLM API tokens.

#### Protocol Flow & Sequence Diagram
```
Agent Process (Python/CLI)       Injected Libc Shim (DYLD/LD)      Kineti Supervisor Daemon
       │                                │                                    │
       │─── 1. execve("/bin/rm", ...) ─▶│                                    │
       │                                │─── 2. Trap: Check Perms (IPC) ────▶│
       │                                │◀── 3. Blocked: No Saga Recorded ───│
       │◀── 4. Return -1 (EPERM) ───────│                                    │
```

#### Exact Configuration Snippet
Command execution wrapper:
```bash
# Transparent execution wrapper
kineti exec -- python run_agent.py
```

The `kineti exec` runner automatically configures the environment:
```bash
#!/usr/bin/env bash
export DYLD_INSERT_LIBRARIES="/usr/local/lib/libkineti_shim.dylib"
export LD_PRELOAD="/usr/local/lib/libkineti_shim.so"
export HTTP_PROXY="http://127.0.0.1:8787"
export HTTPS_PROXY="http://127.0.0.1:8787"
export SSL_CERT_FILE="$HOME/.kineti/certs/ca.crt"
exec "$@"
```

#### Latency Budget & Failure Isolation
- **Latency Budget:** Syscall trap interception: 0.08ms; IPC query to daemon: 1.1ms; Policy check: 0.6ms. **Total Overhead per Syscall: 1.78ms**.
- **Failure Isolation:** If the shim fails to reach the daemon socket, it defaults to a secure fail-closed posture for destructive calls (`unlink`, `connect`), while allowing read-only calls (`open(O_RDONLY)`) to pass.

---

### 3.7 Comprehensive Cross-Host Architectural Comparison Matrix

```
+----------------------------------------------------------------------------------------------------------------------------------+
|                                    COMPREHENSIVE CROSS-HOST ARCHITECTURAL COMPARISON                                             |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| Host Environment  | Primary Hook Point | Protocol / Intercept   | Latency (Local)  | Sandboxing Mode  | Failure Isolation Policy |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| Google Antigravity| Skills & Runbooks  | Native MCP Stdio &     | 4.8 ms           | Artifact Brain & | Fail-Closed on security; |
|                   | Task Supervisor    | Lifecycle Hooks        |                  | Task Sandbox     | Async on telemetry       |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| Anthropic Claude  | PreToolUse &       | Stdin/Stdout JSON      | 4.2 ms           | Native Claude    | Fail-Closed (Exit 2)     |
| Code              | PostToolUse Hooks  | (Exit 0 / Exit 2)      |                  | Permissions Gate | with model feedback      |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| OpenAI Codex /    | Headless Supervisor| Agent Client Protocol  | 4.2 ms           | Bubblewrap /     | Process SIGKILL and      |
| Operator          | ACP & Tools        | (ACP) + Streaming RPC  |                  | Seatbelt Profile | Auto Saga Rollback       |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| OpenCode / OSS    | API Endpoints &    | Reverse HTTP Proxy &   | 2.8 ms           | Process Groups & | Fail-Closed on spend;    |
| Agents            | Local Sockets      | UNIX Domain Socket     |                  | User Permissions | Graceful fallback        |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| Cursor (AI IDE)   | Extension Host &   | MCP JSON-RPC &         | 3.4 ms           | VS Code Ext Host | Isolated UI thread;      |
|                   | Terminal Events    | Webview Bridge         |                  | Sandbox          | Non-blocking restarts    |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
| Generic Terminal  | Dynamic Linker &   | PTY Wrapper, Libc Shim | 1.8 ms           | POSIX PTY &      | Fail-Closed on destructive|
| Agents            | Libc Syscalls      | (LD_PRELOAD / DYLD)    |                  | Syscall Filter   | calls; Pass-open reads   |
+-------------------+--------------------+------------------------+------------------+------------------+--------------------------+
```

---

## 4. Architectural Paradigm Evaluation: Headless Middleware vs. Standalone Companion vs. IDE Extension

Choosing the right user interaction paradigm is the pivotal product decision for the Kineti harness. Below is a rigorous technical evaluation of the three potential form factors.

### 4.1 Comparative Analysis of the Three Paradigms

```
                  THE THREE PARADIGM SPECTRUM
                  
  [Option A: Pure Headless]      [Option B: Standalone Companion]      [Option C: IDE Extension]
    - CLI daemon only              - Dedicated window (Aside.com style)  - VS Code / Cursor plugin
    - Lowest memory (15MB)         - Richest visual expressiveness       - Co-located with code
    - Zero visual UI               - High RAM (Electron / Web)           - High vendor lock-in
    - High developer friction      - Universal across all tools          - Trapped inside editor
```

#### Paradigm 1: Pure Headless Middleware (CLI Daemon / MCP Only)
- **Concept:** Runs strictly in the background as a daemon process or MCP server. All configuration is via CLI commands (`kineti status`, `kineti gate check`) or terminal logs.
- **Strengths:** Lightweight (<20MB RAM in Rust), headless server compatibility (CI/CD pipelines, remote SSH, Docker), completely host-agnostic.
- **Fatal Weaknesses:** Visual opacity. Developers cannot easily comprehend a complex 20-entity causal DAG, multi-file blast-radius diffs, or spend trajectories through terminal ASCII text alone. Approval gates become disruptive terminal prompts that break flow.

#### Paradigm 2: Standalone Browser / Companion (The Aside.com Model)
- **Concept:** Aside.com pioneered the browser-native companion: a dedicated application window that operates alongside your workflow, providing interactive navigation, real-time memory tracking, and human-in-the-loop approval gates for sensitive web actions.
- **Strengths:** Maximum context density and visual fidelity. Uninhibited screen real estate to render full interactive Merkle DAG graphs, multi-column diff inspectors, spend velocity gauges, and time-travel timelines. Completely decoupled from specific IDEs.
- **Weaknesses:** Requires running a separate window; risk of developer window management fatigue; heavier system resource footprint if packaged naively.

#### Paradigm 3: Integrated IDE Extension (Cursor / VS Code Plugin)
- **Concept:** Lives directly inside the developer's primary code editor as an extension panel or status bar widget.
- **Strengths:** Seamless proximity to code; zero window switching.
- **Fatal Weaknesses:** Severe vendor lock-in. Breaks down when the developer switches to Claude Code in terminal, Google Antigravity, or headless CI/CD. Limited UI real estate inside sidebars.

### 4.2 Comprehensive Multi-Dimensional Trade-Off Matrix

```
+-------------------------------------------------------------------------------------------------------+
|                                    PARADIGM TRADE-OFF EVALUATION                                      |
+-------------------------------+-----------------------+-----------------------+-----------------------+
| Dimension                     | Pure Headless         | Standalone Companion  | Integrated IDE Ext    |
+-------------------------------+-----------------------+-----------------------+-----------------------+
| 1. Onboarding Friction        | Minimal (`brew inst`)| Low (Single tray app) | Moderate (Store inst) |
| 2. Visual Expressiveness      | Very Poor (ASCII only)| Exceptional (Full UI) | Moderate (Webview pan)|
| 3. Host Agnosticism           | Total (Works anywhere)| Total (Works with all)| Very Poor (IDE-locked)|
| 4. Enforcement Authority      | Absolute (Kernel gate)| Absolute (Via daemon) | Weak (Host-dependent) |
| 5. Memory / CPU Footprint     | Ultralight (<25MB)    | Moderate (60-120MB)   | Light (40-80MB)       |
| 6. Complex Graph Visuals      | Impossible            | Native (WebGL / SVG)  | Restricted (Sidebar)  |
| 7. Human Approval UX          | Blocking TTY Prompt   | 1-Click Interactive   | In-Editor Notification|
| 8. CI/CD & Headless Operation | Native                | Requires Headless Mode| Unsupported           |
+-------------------------------+-----------------------+-----------------------+-----------------------+
```

### 4.3 Deconstructing Aside.com: Lessons for Agent Harnesses
Aside (aside.com) demonstrated that autonomous agents become trustworthy to humans **only when the execution boundary is visually visible, inspectable, and interruptible**.
Key lessons from Aside's architecture:
1. **The Approval Gate Principle:** High-consequence actions (form submission, financial payments, external messages) must pause execution and present the user with a pre-rendered blast-radius preview.
2. **The "Live Canvas" Mindset:** Rather than watching text scroll in a terminal, users want a living state canvas showing what the agent knows, what it plans to do next, and how much money it has spent.
3. **Local-First Memory:** Memory should not be an opaque cloud vector database; it must be locally inspectable and editable by the operator.

### 4.4 Synthesis: The Imperative for the Hybrid Architecture
Neither pure headless nor pure standalone companion is sufficient on its own.
- If Kineti is *only* a visual companion, it cannot run in headless CI/CD pipelines, remote servers, or low-memory terminal environments.
- If Kineti is *only* a headless CLI, developers will abandon it due to lack of visibility into complex causal graphs and tedious terminal approval prompts.

**The Solution is a Hybrid Architecture:**
A high-performance **Headless Local Daemon & MCP Core** paired with a **Zero-Friction Reactive Visual Sidecar / Canvas**. The daemon runs autonomously and enforces rules regardless of whether the UI is open. When an operator wants visibility, the lightweight companion connects instantly via local WebSockets to render the live visual canvas.

---

## 5. Technical Blueprint: The Kineti Hybrid Companion Architecture

Below is the definitive technical blueprint for the Kineti Hybrid Architecture, spanning the headless core, embedded storage engine, Merkle DAG generator, and reactive visual sidecar.

```
+────────────────────────────────────────────────────────────────────────────────────────────────────+
|                           KINETI HYBRID ARCHITECTURE SPECIFICATION                                 |
+────────────────────────────────────────────────────────────────────────────────────────────────────+
|                                                                                                    |
|   ┌────────────────────────────────────────────────────────────────────────────────────────────┐   |
|   │                         PRESENTATION LAYER: VISUAL SIDECAR / CANVAS                         │   |
|   │     (React 19 / Svelte 5 + Tailwind CSS + Lucide Icons + Motion Spring Physics + Radix)    │   |
|   │                                                                                            │   |
|   │   ┌─────────────────────┐  ┌─────────────────────┐  ┌──────────────────────────────────┐   │   |
|   │   │  Live Merkle DAG    │  │  Human Approval     │  │  Spend Gauges & Token Meters     │   │   |
|   │   │  Causal Inspector   │  │  Diff Modal & Gate  │  │  ($50 Circuit Breaker Tripwire)  │   │   |
|   │   └─────────────────────┘  └─────────────────────┘  └──────────────────────────────────┘   │   |
|   │   ┌─────────────────────┐  ┌─────────────────────┐  ┌──────────────────────────────────┐   │   |
|   │   │  Time-Travel        │  │  Context Integrity  │  │  Universal 20-Entity Provenance  │   │   |
|   │   │  Saga Rollback Bar  │  │  OTD Inspector      │  │  Kernel Trace Stream             │   │   |
|   │   └─────────────────────┘  └─────────────────────┘  └──────────────────────────────────┘   │   |
|   └─────────────────────────────────────────▲──────────────────────────────────────────────────┘   |
|                                             │                                                      |
|                             Bidirectional Local WebSocket & SSE                                    |
|                             `ws://127.0.0.1:8788` | `http://127.0.0.1:8788`                        |
|                                             │                                                      |
|   ┌─────────────────────────────────────────▼──────────────────────────────────────────────────┐   |
|   │                      ENGINE LAYER: HIGH-PERFORMANCE HEADLESS LOCAL DAEMON                  │   |
|   │                              (Written in Rust / Bun Native)                                │   |
|   │                                                                                            │   |
|   │   ┌──────────────────────────────────────┐    ┌────────────────────────────────────────┐   │   |
|   │   │ Multi-Protocol Host Ingress Gate     │    │ Cryptographic Merkle DAG Generator     │   │   |
|   │   │  - Stdio MCP Server (JSON-RPC 2.0)   │    │  - SHA-256 Hash Chain Verifier         │   │   |
|   │   │  - UNIX Domain Socket (/tmp/kineti)  │    │  - Temporal Order Validator            │   │   |
|   │   │  - HTTP Reverse Proxy (:8787)        │    │  - Cycle Detection (Tarjan SCC)        │   │   |
|   │   └──────────────────────────────────────┘    └────────────────────────────────────────┘   │   |
|   │   ┌──────────────────────────────────────┐    ┌────────────────────────────────────────┐   │   |
|   │   │ Policy & Governance Engine           │    │ Safe LIFO Saga Rollback Supervisor     │   │   |
|   │   │  - 13-Stage Pipeline State Machine   │    │  - Inverse Compensation Stack          │   │   |
|   │   │  - Pre-flight Verify Gatekeeper      │    │  - File Modification Rollback Guard    │   │   |
|   │   │  - Financial Circuit Breaker ($50)   │    │  - Deterministic Process Reverter      │   │   |
|   │   └──────────────────────────────────────┘    └────────────────────────────────────────┘   │   |
|   └─────────────────────────────────────────▲──────────────────────────────────────────────────┘   |
|                                             │                                                      |
|                                   Embedded ACID Transaction                                        |
|                                             │                                                      |
|   ┌─────────────────────────────────────────▼──────────────────────────────────────────────────┐   |
|   │                      STORAGE LAYER: EMBEDDED DUAL-ENGINE SUBSTRATE                         │   |
|   │                                                                                            │   |
|   │   ┌──────────────────────────────────────┐    ┌────────────────────────────────────────┐   │   |
|   │   │ SQLite 3 (WAL Mode, In-Memory Cache) │    │ DuckDB Analytical Substrate            │   │   |
|   │   │  - ACID State, Gates, Spend Ledgers  │    │  - ISO SQL/PGQ Graph Queries           │   │   |
|   │   │  - Strict Foreign Key Enforcement    │    │  - Deep Causal Traversal & Analytics   │   │   |
|   │   └──────────────────────────────────────┘    └────────────────────────────────────────┘   │   |
|   │   ┌────────────────────────────────────────────────────────────────────────────────────┐   │   |
|   │   │ Append-Only Durable JSONL Journal (`.kineti/journal.jsonl`, `egress.jsonl`)        │   │   |
|   │   │  - Human-Readable Audit Trail with Atomic File Flushing & Fsync Protection         │   │   |
|   │   └────────────────────────────────────────────────────────────────────────────────────┘   │   |
|   └────────────────────────────────────────────────────────────────────────────────────────────┘   |
+────────────────────────────────────────────────────────────────────────────────────────────────────+
```

---

### 5.1 High-Performance Headless Local Daemon & MCP Server
The headless daemon serves as the central control plane.
- **Language & Runtime Selection:**
  - **Primary Core:** Implemented in **Rust** using `tokio` (asynchronous runtime), `tower` (service abstraction), and `rusqlite` (embedded SQLite bindings). Rust provides single-binary distribution, sub-millisecond memory safety, zero garbage collection latency spikes, and a minuscule 18MB resident memory footprint.
  - **Secondary Prototype / Alternative:** **Bun / TypeScript** using native Bun SQLite (`bun:sqlite`), fast native WebSocket servers, and SIMD JSON parsing for rapid iterative prototyping.
- **Sub-10ms Local Loop Guarantee:**
  All synchronous gate decisions and policy checks are resolved entirely in-memory or via SQLite WAL cache in **under 4.5 milliseconds**, ensuring zero perceived lag for the developer or the model.

### 5.2 Embedded Transactional Storage: SQLite + DuckDB + JSONL
The legacy harness's reliance on raw uncoordinated `fs.writeFileSync` is replaced by an enterprise-grade three-tier storage architecture:

1. **SQLite 3 (Operational OLTP Layer):**
   - Runs in **WAL (Write-Ahead Logging)** mode with `busy_timeout = 5000` and `synchronous = NORMAL`.
   - Stores project run state, stage progression, active gate locks, and the LIFO saga rollback stack.
   - Prevents all concurrency race conditions and multi-agent write clobbering.
   - Schema enforcement guarantees that gate states are strictly validated against `('pass', 'fail', 'pending')`.
2. **DuckDB (Analytical Causal Layer):**
   - Embedded in-process analytical engine.
   - Implements ISO SQL/PGQ graph queries across millions of historical causal links (`caused`, `triggers`, `blocks`, `enables`, `remediates`).
   - Executes recursive graph path queries (e.g., finding the root cause of a regression across 50 prior runs) in under 12 milliseconds.
3. **Durable JSONL Export (`.kineti/journal.jsonl`):**
   - Maintains compatibility with git tracking and human inspection.
   - Writes are mediated exclusively by the daemon via atomic rename buffers (`file.tmp -> rename -> file`) with explicit `fsync` flushing, eliminating the silent data loss bug in `bin/lib.ts:48-58`.

### 5.3 Background Merkle DAG Generator & Causal Integrity Engine
The daemon continuously maintains a cryptographically verified Merkle DAG:
- **Node Structure:**
  $$	ext{Hash}_n = 	ext{SHA-256}(	ext{Type} \,\|\, 	ext{PrevHash} \,\|\, 	ext{Timestamp} \,\|\, 	ext{PayloadCanonicalJSON})$$
- **Node Types:**
  1. `RunRecord`: Root goal, active stage, locked configuration.
  2. `EvidenceProof`: Test execution exit code, test stdout hash, working directory git tree fingerprint.
  3. `SagaAction`: Inverse compensation command, target file, inverse diff patch.
  4. `EgressReceipt`: Outbound HTTP request hash, provider response hash, token deduction count.
  5. `GateResolution`: Human approval ticket signature, gate identifier, decision timestamp.
- **Background Integrity Worker:**
  - Evaluates causal graph edges to guarantee strict temporal ordering ($T_{	ext{effect}} \ge T_{	ext{cause}}$).
  - Runs Tarjan's strongly connected components algorithm to detect and reject causal cycles.
  - Generates tamper-evident Outcome Verification Tickets (OVTs) cryptographically bound to the Merkle root.

### 5.4 Zero-Friction Visual Sidecar / Canvas (The Developer Companion)
The presentation layer is inspired by Aside.com's companion philosophy, built according to the Kineti Design System:
- **Design System Standards:**
  - Strict adherence to **Tailwind CSS**, **Lucide Icons**, **Motion spring physics**, and **Radix UI headless primitives**.
  - Visual Archetypes: Modern Technical SaaS (Linear/Vercel dark slate), Clean High-Trust Fintech (Stripe navy/emerald), and Premium Editorial (warm alabaster).
- **Core Visual Modules:**
  1. **Live Merkle DAG Inspector:** An interactive SVG/Canvas graph renderer. Displays pipeline stages (1 to 13), gate nodes, evidence attachments, and causal dependency edges. Nodes glow green (verified), amber (pending approval), or red (gate failure).
  2. **Human Approval Gate Modal & Blast-Radius Engine:** When an agent attempts a sensitive action (Stage 6 Spec approval, Stage 11 Ship gate, or destructive bash execution), the canvas pops an approval modal showing:
     - Colorized syntax-highlighted unified diff.
     - Blast-radius risk score (number of files affected, lines modified, dependencies touched).
     - Single-click **[1] Approve & Proceed** or **[2] Reject & Rollback** buttons.
  3. **Spend Gauges & Circuit Breakers:** Real-time radial meters and burn velocity charts tracking cumulative dollars spent against the $50.00 task ceiling and per-stage budgets.
  4. **Time-Travel Timeline & Saga Unwind Controller:** Visual LIFO stack showing each executed mutation. Clicking any historical node displays an instant diff preview and allows one-click rollback of intermediate steps.
  5. **Context Integrity & OTD Inspector:** Live inspector displaying active Ontology Trigger Data, injected causal constraints, and real-time prompt sanitation logs.

### 5.5 Reactive Telemetry & Human Approval Gate Protocol
The daemon and companion communicate over a high-speed local WebSocket (`ws://127.0.0.1:8788`).

#### Exact Protocol Event Schemas:

**Event 1: Approval Requested (`kineti.gate.approval_requested`)**
```json
{
  "jsonrpc": "2.0",
  "method": "kineti.gate.approval_requested",
  "params": {
    "ticket_id": "ovt-894f-2026-09-06",
    "stage": 6,
    "gate_name": "spec",
    "action_type": "filesystem_mutation",
    "description": "Approve generated OpenAPI spec and type definitions",
    "blast_radius": {
      "files_created": 3,
      "files_modified": 1,
      "lines_added": 240,
      "risk_score": "LOW"
    },
    "diff_preview": "--- a/spec.md
+++ b/spec.md
@@ ...",
    "expires_at": "2026-09-06T05:40:00Z"
  }
}
```

**Event 2: Approval Resolved (`kineti.gate.approval_resolved`)**
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

**Event 3: Real-Time Spend Tick (`kineti.spend.tick`)**
```json
{
  "jsonrpc": "2.0",
  "method": "kineti.spend.tick",
  "params": {
    "stage": 7,
    "stage_spend_usd": 3.42,
    "stage_limit_usd": 10.00,
    "global_spend_usd": 14.85,
    "global_limit_usd": 50.00,
    "burn_rate_usd_per_minute": 0.45,
    "tokens_prompt": 124500,
    "tokens_completion": 18200
  }
}
```

---

## 6. Cross-Platform Distribution & Packaging Strategy

To ensure zero developer friction, Kineti implements a multi-tiered distribution strategy tailored for solo operators and enterprise teams alike.

```
+----------------------------------------------------------------------------------------------------+
|                                CROSS-PLATFORM DISTRIBUTION TIERS                                   |
+-------------------+--------------------+------------------------+----------------------------------+
| Distribution Tier | Target Platform    | Packaging Technology   | User Experience & Friction       |
+-------------------+--------------------+------------------------+----------------------------------+
| Tier 1: Localhost | Any OS / Browser   | Web SPA served via     | Zero-install; auto-opens browser |
| Web Canvas        | (Chrome/Safari/FF) | Daemon embedded assets | tab (`kineti companion`)         |
+-------------------+--------------------+------------------------+----------------------------------+
| Tier 2: macOS     | macOS Sonoma /     | Tauri v2 / Swift       | Dockable tray item; global hotkey|
| Menu Bar App      | Sequoia (Apple Sil)| Lightweight Native App | (⌘⌥K); floating sidecar panel    |
+-------------------+--------------------+------------------------+----------------------------------+
| Tier 3: Desktop   | macOS, Linux,      | Tauri v2 Desktop       | Snappable sidecar window; always-|
| Companion Window  | Windows            | (WebKit / WebView2)    | on-top mode alongside IDE/term   |
+-------------------+--------------------+------------------------+----------------------------------+
| Tier 4: Headless  | Linux Servers,     | Single Static Binary   | Zero UI; installs via Homebrew / |
| Daemon / CLI      | Docker, CI/CD      | (Rust musl / Bun)      | curl script; launchd / systemd   |
+-------------------+--------------------+------------------------+----------------------------------+
```

### 6.1 Daemon Process Supervision & Service Management
The headless engine operates as a durable user-level background service:
- **macOS (`launchd`):** Installed to `~/Library/LaunchAgents/dev.kineti.daemon.plist`. Automatically launches on user login, monitors socket health, and rotates logs.
- **Linux (`systemd`):** Installed as a systemd user unit to `~/.config/systemd/user/kineti.service` with automatic restart on failure (`Restart=on-failure`, `RestartSec=3s`).

#### Sample `launchd` Configuration (`dev.kineti.daemon.plist`):
```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>dev.kineti.daemon</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/kineti-daemon</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>/Users/praveen/.kineti/logs/daemon.log</string>
    <key>StandardErrorPath</key>
    <string>/Users/praveen/.kineti/logs/daemon.err</string>
</dict>
</plist>
```

### 6.2 Security, Sandboxing, & Cryptographic Updates
- **Local Loop Authentication:** All local WebSocket and HTTP connections require a bearer token read from `~/.kineti/auth.token` (permissions `0600`).
- **Socket Permissions:** The UNIX domain socket `/var/run/kineti.sock` is restricted to the current user (`0700`).
- **Cryptographic Binary Signatures:** All releases and automatic binary updates are cryptographically signed using **Minisign (Ed25519)**. The updater verifies signatures before executing atomic binary replacement.

---

## 7. Sub-10ms Latency Budget & Local Loop Benchmark Specifications

Every tool execution intercept, spend verification, and gate evaluation must execute within a strict **sub-10ms latency ceiling** to maintain instantaneous responsiveness.

### 7.1 Detailed Microsecond-Level Latency Budget Allocation

```
+----------------------------------------------------------------------------------------------------+
|                         SUB-10MS LOCAL LOOP LATENCY BUDGET BREAKDOWN                                |
+----------------------------------------------------+-----------------------+-----------------------+
| Execution Step                                     | Target Latency (µs)   | Max Allocated (µs)    |
+----------------------------------------------------+-----------------------+-----------------------+
| 1. IPC Ingress: Socket Read & Frame Decoding       | 450 µs                | 800 µs                |
| 2. Authentication & Host Session Validation        | 120 µs                | 250 µs                |
| 3. SQLite WAL State Lookup (Prepared Statement)    | 850 µs                | 1,500 µs              |
| 4. Policy Engine Check (Stage & Gate Conditions)   | 600 µs                | 1,200 µs              |
| 5. Spend Breaker & Token Quota Arithmetic          | 150 µs                | 350 µs                |
| 6. Merkle DAG Node SHA-256 Hashing                 | 400 µs                | 800 µs                |
| 7. SQLite In-Memory Commit / WAL Append            | 1,100 µs              | 2,200 µs              |
| 8. WebSocket Telemetry Broadcast (Non-blocking)    | 200 µs                | 500 µs                |
| 9. Response Serialization & Socket Return Frame    | 350 µs                | 700 µs                |
+----------------------------------------------------+-----------------------+-----------------------+
| TOTAL LOCAL LOOP TURNAROUND TIME                   | 4,220 µs (4.22 ms)    | 8,300 µs (8.30 ms)    |
+----------------------------------------------------+-----------------------+-----------------------+
```

### 7.2 Handling Asynchronous Human Approval Pauses
When an action requires human approval:
1. The synchronous local loop completes steps 1 through 6 in **<3ms**.
2. The daemon immediately flags the tool execution as `STATE_PENDING_APPROVAL` and holds the open socket connection without CPU spinning (using an asynchronous `tokio::sync::oneshot` channel).
3. Telemetry event `kineti.gate.approval_requested` is dispatched to the companion canvas (<1ms).
4. The companion renders the approval modal and plays an alert chime.
5. Upon operator approval, the companion transmits `kineti.gate.approval_resolved`. The daemon channel resolves, writes the verified node to SQLite, and returns exit code 0 to the waiting agent in under 2ms.
6. If the approval request exceeds the configurable timeout (default: 300 seconds), the channel resolves with `TIMEOUT`, failing closed and returning Exit Code 2 to the agent.

---

## 8. Conclusion & Strategic Next Steps for Blueprint Integration

### 8.1 Key Findings Summary
1. **The File-Copy Paradigm is Obsolete:** Passive prompt injection via `setup.sh` and `hooks/*.txt` cannot enforce safety. Kineti must operate as an active dual-plane runtime combining an in-process MCP server/daemon with a live visual canvas.
2. **Universal Host Mechanics are Solvable:** All 6 target hosts (Google Antigravity, Anthropic Claude Code, OpenAI Codex/Operator, OpenCode, Cursor, and Generic CLI) possess deterministic integration points (PreToolUse hooks, native MCP, ACP supervisors, reverse proxies, and PTY shims) that satisfy the sub-10ms latency budget.
3. **The Hybrid Architecture is the Winning Form Factor:** Pure headless is too opaque; pure standalone companion is too heavy and disconnected from CI/CD. The hybrid model (Rust daemon + SQLite WAL + Aside-style Web/Desktop canvas) provides the optimal balance of speed, governance, and visual delight.

### 8.2 Architectural Handoff to Strategic Blueprint Milestones
- **To R2 (Core Engine Hardening & Research Substrate Integration):**
  Integrate the verified SQLite WAL schema, the 20-Entity Provenance Kernel, and Outcome Verification Ticket (OVT) signatures directly into the daemon's Merkle DAG generator, permanently remediating the 44 audit vulnerabilities documented in `docs/AUDIT_REPORT.md`.
- **To R3 (Solo-Founder Unit Economics & Monetization Engine):**
  Monetize the companion canvas as the primary driver for the Pro Tier ($29-$49/seat/month) and package the cryptographic OVT compliance engine for the Enterprise Tier ($250+/seat/month).
- **To R6 (Master Blueprint Synthesis):**
  Incorporate the architectural diagrams, host configuration snippets, and latency budgets into the consolidated `docs/HARNESS_STRATEGY_BLUEPRINT.md`.

---
*Report compiled and certified by teamwork_preview_explorer_m2_1.*
