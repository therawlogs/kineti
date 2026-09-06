# Project: Universal AI Agent Harness and Context Integrity Runtime Blueprint

## Architecture
- **Layered Protocol Architecture**: Context Integrity Protocol (CIP) 7-Layer Stack spanning Physical Transport (L1) to Business/Outcome Attestation (L7).
- **Hybrid System Topology**:
  - **Local Runtime Core**: Sub-10ms Headless Daemon / In-Process MCP Server in Rust / Bun with embedded SQLite (WAL mode) and DuckDB (ISO SQL/PGQ causal graph traversal).
  - **Visual Sidecar / Companion**: Aside-style lightweight canvas (`ws://127.0.0.1:8788`) delivering real-time Merkle DAG inspection, 1-click human gate approval, spend/token gauges, and saga time-travel rollback.
  - **Universal Host Adapters**: Google Antigravity (skills/sidecar), Anthropic Claude Code (bash/pre-tool hooks), OpenAI Codex/Operator (JSON-RPC supervisor), OpenCode (stdio/proxy), Cursor (extension/LSP), and Generic Terminal Agents (pty/proxy).
- **Core State & Graph Substrate**: ISO SQL/PGQ Causal DAG with 6 vertex types, 8 edge types, Universal 20-Entity Provenance Kernel, and Runtime OTD state machine.
- **Verification Engine**: Ed25519 dual-signed Outcome Verification Tickets (OVTs) anchored in cryptographic Merkle trees.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Universal Host Adapter Mechanics | Protocol flows, configs, and lifecycle hooks for Antigravity, Claude Code, Codex, OpenCode, Cursor, and Terminal Agents | M1 | Survey R1 |
| 2 | Sub-10ms Local Loop Latency Engine | Microsecond-budgeted local transport, SQLite WAL query, and SHA-256 DAG commit | M1 | Survey R1 |
| 3 | Aside-Style Hybrid Visual Sidecar | Real-time WebSocket/SSE companion canvas with Merkle DAG inspector, approval gates, and spend gauges | M1 | Survey R1 |
| 4 | CIP 7-Layer Protocol Specification | Full formal stack from L1 Physical Transport to L7 Outcome Attestation | M1 | Survey R2 |
| 5 | Causal-Graph Substrate with ISO SQL/PGQ | Formal vertex/edge DDL, 6 node types, 8 edge types, and SQL/PGQ query patterns | M1 | Survey R2 |
| 6 | Universal 20-Entity Provenance Kernel | Definitive JSON/JSON-LD schemas and TypeScript definitions for all 20 kernel entities | M1 | Survey R2 |
| 7 | Runtime OTD State Machine | Ontology Trigger Data schemas, sub-10ms condition matchers, and transition rules | M1 | Survey R2 |
| 8 | Outcome Verification Tickets (OVTs) | Ed25519 dual-signing protocol, RFC 8785 canonical serialization, and W3C VC schema | M1 | Survey R2 |
| 9 | Systematic Resolution of 44 Audit Defects | Concrete architectural remediations for all 44 defect categories from `docs/AUDIT_REPORT.md` (CRIT-01 to INFO-10) | M1 | Survey R2 |
| 10 | Solo-Founder Unit Economics & Pricing | Three-tier model (Open-Core, Pro $39/mo, Enterprise $250+/mo) with 95%+ gross margins | M1 | Survey R3 |
| 11 | 24-Month Pro-Forma Financial Model | Quantitative trajectory to $1.4M ARR (M12) and $4.59M ARR (M24) for a solo builder | M1 | Survey R3 |
| 12 | $/Token to $/Outcome Paradigm Shift | Enterprise ROI proof showing 4,700% return by preventing production outages | M1 | Survey R3 |
| 13 | Developer GTM & Competitive Landscape | 10-player matrix, white space capture, viral MCP/GitHub Actions distribution loops | M1 | Survey R4 |
| 14 | Strategic M&A Thesis ($50M-$200M+) | Acquisition target profiles (AI labs & dev platforms), valuation multiples, and walk-away cashflow leverage | M1 | Survey R5 |
| 15 | 12-Month Phased Engineering Roadmap | Four quarterly milestones (Q1-Q4) detailing solo-founder engineering deliverables | M1 | Survey R5 |
| 16 | Deliverable Compilation (`docs/HARNESS_STRATEGY_BLUEPRINT.md`) | Exhaustive, publication-grade blueprint compiling all sections with full schemas and diagrams | M1 | Survey R6 |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Master Strategy Blueprint Compilation | Compile exhaustive, publication-grade strategy blueprint at `docs/HARNESS_STRATEGY_BLUEPRINT.md` resolving all 44 defects and detailing R1-R6 | Survey (Phase 0) | DONE (Pass: Unanimous Approval & Clean Audit) |
| M2 | Multi-Agent Review & Empirical Challenge | Independent reviews (2x Reviewers) and empirical verification (2x Challengers) | M1 | DONE |
| M3 | Forensic Integrity Audit & Sentinel Handoff | Forensic audit (teamwork_preview_auditor) and completion reporting to Sentinel | M2 | DONE |

## Interface Contracts
### Host ↔ Daemon Interface (L1/L2)
- **Protocol**: JSON-RPC 2.0 over stdio or Unix Domain Socket (`/var/run/kineti/kineti.sock` or `~/.kineti/kineti.sock`).
- **MCP Endpoints**: `tools/call`, `resources/read`, `prompts/get`, `notifications/initialized`.
- **Latency Budget**: Total handshake < 5ms; local policy verification < 5ms.

### Daemon ↔ Companion Sidecar (L2/L3)
- **Protocol**: WebSocket (`ws://127.0.0.1:8788/events`) and HTTP REST (`http://127.0.0.1:8788/api/v1`).
- **Data Shape**: Streaming JSON graph updates (DAG nodes/edges), spend ticks, human gate requests with interactive accept/reject payloads.

### Causal Graph Substrate ↔ Engine (L4/L5/L6)
- **Engine**: SQLite 3.45+ (WAL mode, foreign keys enabled) + DuckDB (ISO SQL/PGQ extensions).
- **Transactions**: Sub-50ms atomic commit gates; LIFO saga rollback logs with SHA-256 state hashing.

## Code Layout & Deliverable Location
- **Deliverable Document**: `/Users/praveen/Documents/Products/kineti local harness/docs/HARNESS_STRATEGY_BLUEPRINT.md`
- **Audit Reference Document**: `/Users/praveen/Documents/Products/kineti local harness/docs/AUDIT_REPORT.md`
- **Agent Metadata & Artifacts**: `/Users/praveen/Documents/Products/kineti local harness/.agents/`
