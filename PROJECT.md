# Project: Kineti OS Architecture Audit & Empirical Benchmark Verification

## Architecture
- **Native Systems Engine (`core-native/`)**:
  - `kineti-core`: Lock-free atomic state snapshots with EBR (`ArcSwap`, dual-buffered RCU, atomic CAS), Universal 20-Entity Provenance Kernel, sub-50ms 3-way graph commit gate (topological rank acyclicity, HLC, Merkle DAG lineage), atomic fast-path spend circuit breaker ($50 ceiling, $47.50 trip).
  - `kineti-memory`: Dual-substrate context memory (vector embedding index + causal property graph traversal), runtime OTD engine with dynamic JMESPath bindings, tombstone masking for $O(1)$ SAGA LIFO rollback invalidation, calibrated temperature-scaled hybrid fusion scoring.
  - `kineti-reflex`: Sensory triage engine ($p99 < 1.0\,\text{ms}$), zero-token emoji reactions, dynamic style profiling (5-dim EWMA).
  - `kineti-connectors`: External integrations (Brave Search, FLUX image generation, Notion API).
  - `kineti-actions`: Action execution and 2-step confirmation gates (`ActionConfirmationGate`).
  - `kineti-gateway`: Omnichannel messaging gateways (Meta WhatsApp Cloud API webhooks, macOS AppleScript iMessage bridge).
  - `kineti-harness`: Developer safety harness, MCP proxy interceptor, git worktree shadow workspace, DNTI and Ed25519 dual-signed Outcome Verification Ticket (OVT) generation.
  - `kineti-cli`: Native binary CLI entrypoint and daemon runner (<8MB idle RSS).
- **TypeScript Governance Layer (`bin/`, `src/`, `tests/`)**:
  - `bin/`: CLI command routers, `kineti-state.ts` (13 stages), `kineti-spend.ts` ($50 global / $10 stage limit), `kineti-saga.ts` (LIFO undo stack), `kineti-evidence.ts` (cryptographic test proofs), `kineti-verify-gate.ts`, `kineti-companion.ts` (local dashboard, <12 KB gzipped payload, Apple HIG tokens).
  - `src/`: UI components, Apple design tokens, swarm coordinator (`coordinator.ts`) with Ed25519 OVT signing protocol.
  - `tests/`: 10 test suites with 82 automated governance tests (including 500-vector adversarial prompt injection suite).
- **Foundational Research Series**: removed from the tree. Will be re-added when ready.

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| F1 | Paper 1 Canon & Invariants | 13-stage software factory, SAGA LIFO undo ledger, cryptographic evidence binding | M1 | Survey (Spec Miner) |
| F2 | Paper 2 Canon & Invariants | EBR atomics, zero-copy snapshots, 100x move penalty rule ($\Pi_{\text{move}} \in [80, 240]$) | M1 | Survey (Spec Miner) |
| F3 | Paper 3 Canon & Invariants | 20-entity active kernel, causal multi-tenant property graph, O(1) privacy tombstones | M1 | Survey (Spec Miner) |
| F4 | Paper 4 Canon & Invariants | Sub-millisecond sensory triage ($p99 < 1.0\,\text{ms}$), zero-token emoji reactions, dynamic style profiling | M1 | Survey (Spec Miner) |
| F5 | Paper 5 Canon & Invariants | Causal value graphs, Ed25519 dual-signed OVTs ($id_w \neq id_r$), $50 spend breaker | M1 | Survey (Spec Miner) |
| F6 | 80-Thread Concurrency Audit | Thread safety under 80-thread writer contention with zero torn reads and zero deadlocks | M2 | Survey (Explorer Rust) |
| F7 | Native Memory Safety Audit | Zero memory leaks, zero dangling pointers, zero UB across all crates (100% Safe Rust) | M2 | Survey (Explorer Rust) |
| F8 | Multi-Tenant Isolation Audit | Tenant isolation across graph and vector indices; gap analysis and fix specification | M2 | Survey (Explorer Rust) |
| F9 | Sensory Triage Benchmark | Empirical verification of sensory classification $p99 < 1.0\,\text{ms}$ before model invocation | M3 | Survey (Spec Miner / Rust) |
| F10 | Context Snapshot Benchmark | Empirical verification of context snapshot acquisition $p99 < 0.1\,\text{ms}$ under 80-writer contention | M3 | Survey (Explorer Rust) |
| F11 | Causal Graph Walk Benchmark | Empirical verification of 3-hop graph query traversal $p99 < 0.8\,\text{ms}$ across 100,000 nodes | M3 | Survey (Spec Miner / Rust) |
| F12 | Cryptographic Verification Latency | Empirical verification of Ed25519 signature validation $< 100\,\mu\text{s}$ | M3 | Survey (Spec Miner / Rust) |
| F13 | Memory Footprint Benchmark | Verification of daemon idle Resident Set Size (RSS) $< 25\,\text{MB}$ (actual $< 8\,\text{MB}$) | M3 | Survey (Explorer Rust) |
| F14 | Token Efficiency Benchmark | Prompt compression achieving $\ge 35\%$ token savings via 20-entity active kernel | M3 | Survey (Spec Miner) |
| F15 | Spend Circuit Breaker Audit | Deterministic trip at 95% ($47.50 of $50.00 ceiling) with OS exit code 3 halt | M4 | Survey (Explorer Governance) |
| F16 | 2-Step Financial Confirmation Gate | 100% of purchases require user confirmation; Ed25519 OVT binding verification | M4 | Survey (Explorer Governance) |
| F17 | SAGA Undo Stack Audit | 100% of file mutations preceded by inverse command; clean LIFO rollback under failure | M4 | Survey (Explorer Governance) |
| F18 | Waitlist Page Weight & HIG Audit | Static landing payload $< 35\,\text{KB}$ gzipped (actual 11.62 KB), 0 runtime JS frameworks, Apple HIG | M5 | Survey (Explorer Governance) |
| F19 | Omnichannel Gateway Resilience | WhatsApp Cloud API webhooks (challenge, inbound, outbound, emoji, photo) & iMessage AppleScript bridge | M5 | Survey (Explorer Governance) |
| F20 | Knowledge Architecture Audit | Notion sync integrity across all 7 workspace crates | M5 | Survey (Explorer Governance) |
| F21 | Adversarial Prompt Injection Audit | 0 prompt injection escapes across 500 adversarial benchmark test vectors | M5 | Survey (Explorer Governance) |
| F22 | Full Test Suite Verification | 83 native Rust tests passing + 82 TypeScript governance tests passing | M6 | Survey (All Explorers) |
| F23 | Cryptographic Evidence Freshness | Fresh cryptographic evidence receipt recorded in `.kineti/evidence.jsonl` | M6 | Survey (Explorer Governance) |
| F24 | Canonical Production Plan | Formulate and audit `implementation_plan.md` bridging research canon to production | M6 | Survey (Spec Miner) |
| F25 | Publication-Grade Architecture Audit Report | Exhaustive comparative benchmark matrices, gap remediation, certification signatures | M6 | Orchestration Synthesis |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Formal Research Canon & Mathematical Invariants | Audit Papers 1–5 theorems (2.1–2.3, 3.1–3.2, 5.1–5.3), 13-stage software factory, SAGA LIFO, 20-entity kernel, CVG, DNTI, spend breaker | none | COMPLETED |
| M2 | Native Rust Memory Safety & Concurrency Audit | Audit 80-thread writer contention, zero torn reads, 100% Safe Rust, Miri/ASan setup, multi-tenant isolation | M1 | COMPLETED |
| M3 | Empirical World-Class Micro-Benchmark Verification | Validate sensory triage p99 < 1.0ms, snapshot latency p99 < 0.1ms, graph walk p99 < 0.8ms, Ed25519 < 100µs, RSS < 25MB, token compression >= 35% | M2 | COMPLETED |
| M4 | Financial Safety & SAGA Reversibility Audit | Audit hardware spend ceiling deterministic trip at $47.50, 2-step financial confirmation, SAGA LIFO rollback | M1, M2 | COMPLETED |
| M5 | Production Readiness & Static Waitlist Audit | Audit waitlist payload < 35KB gzipped, Apple HIG, WhatsApp/iMessage gateway, Notion sync, 500 prompt injection vectors | M3, M4 | COMPLETED |
| M6 | Master Synthesis, Evidence Generation & Final Audit Report | Publish canonical architecture plan, fresh evidence in `.kineti/evidence.jsonl`, master audit report `docs/ARCHITECTURE_AUDIT_AND_BENCHMARK_REPORT.md` | M1, M2, M3, M4, M5 | COMPLETED |
| M7 | 360º Human Model & Epistemic Resilience | Verbatim root goal invariance, friction triage ladder, commitment verification, asymmetric gap-filling | M6 | COMPLETED |

## Interface Contracts
- **Research Invariants to Codebase**:
  - Theorem 2.1 (Zero Torn Reads): Reader acquires active pointer clone without writer locks; validated via `test_80_writer_zero_torn_reads_stress`.
  - Theorem 2.2 (Bounded Latency): Read snapshot latency $p99 < 0.1\,\text{ms}$; validated via `test_80_writer_p99_snapshot_latency`.
  - Theorem 5.2 (Spend Halt Invariant): Total microcents $\ge 47{,}500{,}000 \implies$ process exits with code 3.
  - Theorem 5.1 (OVT Separation of Authority): $id_w \neq id_r \land pk_w \neq pk_r$; validated via `coordinator.ts`.
- **Cross-Layer Evidence Binding**:
  - `bun bin/kineti-evidence.ts run --label <label> -- <command>` creates SHA-256 fingerprint bound to git tree hash.
  - `bun bin/kineti-evidence.ts check --label <label>` verifies state is `FRESH`.

## Code Layout
```

core-native/
├── Cargo.toml                               (Workspace manifest: 7 active member crates)
└── crates/
    ├── kineti-core/                         (Snapshot, HLC, Provenance Kernel, Commit Gate, Spend Breaker)
    ├── kineti-memory/                       (Property Graph, Vector Index, OTD Engine, Tombstone Mask)
    ├── kineti-reflex/                       (Sensory Triage Sensor, Emoji Reactions, Style Profiler)
    ├── kineti-connectors/                   (Brave, FLUX, Notion integrations)
    ├── kineti-actions/                      (Action execution, 2-step Confirmation Gate)
    ├── kineti-gateway/                      (WhatsApp Cloud API webhook, AppleScript iMessage)
    ├── kineti-cli/                          (Command router, daemon runner, test-all)
    └── kineti-harness/                      (Stub crate for M4 MCP interceptor / shadow workspace)

bin/
├── kineti.ts                                (Main CLI router)
├── kineti-spend.ts                          ($50 spend circuit breaker)
├── kineti-saga.ts                           (LIFO undo stack)
├── kineti-evidence.ts                       (Cryptographic test proofs)
├── kineti-verify-gate.ts                    (Pre-flight gate verification)
└── kineti-companion.ts                      (Local dashboard, 11.62 KB gzipped, Apple HIG)

src/
├── design/apple-design-tokens.ts            (SF Pro typography, materials, squircle radii)
└── swarm/coordinator.ts                     (Swarm coordinator, Ed25519 dual-signed OVT)

tests/                                       (82 TypeScript tests across 10 suites)
docs/                                        (Architecture, audit reports, design guides)
.kineti/                                     (state.json, evidence.jsonl, spend.json, saga.jsonl)
```
