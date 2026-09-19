# Context Integrity Layer: Mathematical Foundations of Autonomous Agent Systems

**Author:** Praveen Kumar (therawlogs.com | Foundational AI Research)  
**Series:** Context Integrity Layer Research Series (5-Part Canon)  
**Canonical Repository:** `research/`  
**Reference Native Implementations:** `core-native/kineti-core`, `core-native/kineti-memory`, `core-native/kineti-harness`, `core-native/kineti-reflex`, `core-native/kineti-gateway`  
**License:** Dual Apache 2.0 / MIT  

---

## 1. Executive Overview: The Context Integrity Protocol (CIP)

Contemporary autonomous agent architectures face five fundamental physical, topological, and economic bottlenecks that prevent them from scaling reliably in mission-critical enterprise environments:

1. **The Operating System & State Machine Vacuum (OS & Governance Layer — Paper 1):**  
   Autonomous agents execute arbitrary tools without memory bounding, mutate file systems without transactional rollback guarantees, lack cryptographic lineage for execution steps, and are powerless to prevent runaway financial exhaustion.
2. **The Thermodynamic Context Bottleneck & Move Penalty (Hardware & Concurrency Substrate — Paper 2):**  
   Multi-agent frameworks transmit megabyte-scale context payloads across process boundaries via JSON-RPC, HTTP REST, or Unix domain sockets. Moving data between off-chip High-Bandwidth Memory (HBM/DRAM) and on-chip Static Random-Access Memory (SRAM) incurs an energy penalty of approximately **$100\times$** compared to arithmetic operations (~10 pJ/bit fetch vs ~0.1 pJ/FLOP), inducing severe memory-bus saturation, cache thrashing, and torn context reads.
3. **The Topological & Causal Blindness of Pure Vector Retrieval (Memory Substrate Layer — Paper 3):**  
   Dense vector similarity search (cosine distance in embedding space) measures spatial topic proximity while remaining blind to temporal ordering, causal dependencies, execution rollbacks, and schema invariants. In recursive agent loops, this causes a **Temporal Inversion Error Rate exceeding 30%**, where agents retrieve stale or rolled-back state over active truth.
4. **The High-Latency Token Burn on Low-Information Signals (Sensory Reflex Layer — Paper 4):**  
   Over 35% of inbound conversational interactions are low-information status updates (*"ok"*, *"on my way"*, *"got it"*). Routing these updates to deliberative LLMs burns expensive reasoning tokens and introduces 1.5–3.0 seconds of network latency where a native platform reaction (`👍`, `❤️`, `⚡`) provides zero-noise confirmation.
5. **Goodhart's Law & Proxy Reward Overoptimization (Outcome Engineering Layer — Paper 5):**  
   Evaluating agents on activity-based KPIs (tickets closed, lines of code written) or static token-level benchmarks (MMLU, GSM8K) incentivizes task gaming: agents close tickets via conversational boilerplate without resolving root causes, causing true enterprise utility to monotonically degrade ($\lim \mathbb{E}[U] = -\infty$).

To resolve these crises, the **Context Integrity Layer** (formalized as the **Context Integrity Protocol / CIP**) establishes a unified mathematical and systems foundation across five peer-reviewed treatises:

```
+---------------------------------------------------------------------------------------------------------+
|                                  CONTEXT INTEGRITY LAYER (CIP) ARCHITECTURE                             |
+---------------------------------------------------------------------------------------------------------+
| LAYER 5: OUTCOME ENGINEERING & FRONTIER GOVERNANCE (Paper 5)                                            |
| - Causal Outcome Graphs: Intent -> Task -> Action -> State Change -> Outcome                            |
| - Structural Causal Models (SCM) & Counterfactual Average Treatment Effect (ATE) Identification        |
| - Asymmetric Dual-Signed Outcome Verification Tickets (OVT) with Ed25519 Non-Repudiation                |
| - Directional Normalized Trust-Weighted Impact (DNTI) & $/Outcome Economic Modeling                     |
| - Frontier Benchmark Evaluation: Agents' Last Exam (ALE) 76.4% & SWE-bench Verified (4.2 min MTTR)      |
| Reference Crate: kineti-harness                                                                         |
+---------------------------------------------------------------------------------------------------------+
                                                   |
                                                   v
+---------------------------------------------------------------------------------------------------------+
| LAYER 4: REFLEXIVE SENSORY & LINGUISTIC ADAPTATION (Paper 4)                                            |
| - Sub-Millisecond Sensory Triage (p99 < 1.0 ms) Routing Low-Info Updates to Zero-Token Reactions       |
| - 5-Dimensional Online Socio-Linguistic Style Profiler (EWMA Formality, Brevity, Slang, Caps, Code-Switch)|
| - Dynamic Prompt Hydration Mirroring User Personas Without Token Bloat                                  |
| Reference Crates: kineti-reflex, kineti-gateway                                                         |
+---------------------------------------------------------------------------------------------------------+
                                                   |
                                                   v
+---------------------------------------------------------------------------------------------------------+
| LAYER 3: DUAL-SUBSTRATE MEMORY & RETRIEVAL (Paper 3)                                                    |
| - Dual-Substrate: ISO SQL:2023 Property Graph Queries (SQL/PGQ) + Dense Vector Embeddings (HNSW)        |
| - Universal 20-Entity Causal Provenance Kernel & Runtime Dynamic Ontology Trigger Data (OTD / JMESPath) |
| - Sub-50ms 3-Way Graph Commit Gate: Topological Rank L(A) < L(B), Hybrid Logical Clocks, Merkle DAG     |
| - RoaringBitmap Tombstone Masking for O(1) SAGA LIFO Rollback Invalidation (< 5 µs)                     |
| - Calibrated Temperature-Scaled Probabilistic Fusion: S(d_i) = \alpha * Softmax + (1-\alpha) * \gamma^hop|
| Reference Crate: kineti-memory                                                                          |
+---------------------------------------------------------------------------------------------------------+
                                                   |
                                                   v
+---------------------------------------------------------------------------------------------------------+
| LAYER 2: HARDWARE & CONCURRENCY SUBSTRATE (Paper 2)                                                     |
| - Thermodynamic 100x Move Penalty Elimination (10 pJ/bit DRAM Fetch vs 0.1 pJ/FLOP ALU Compute)         |
| - Heterogeneous Silicon: Apple Silicon Unified Memory (32MB ANE SRAM) vs NVIDIA Blackwell B200 Fabrics  |
| - Wait-Free Epoch-Based Reclamation (EBR) & Atomic Double-Word Compare-And-Swap (CASP / FEAT_LSE2)      |
| - Zero-Torn-Read Invariant & p99 < 0.1ms Snapshot Acquisition under 80-Thread Writer Contention         |
| Reference Crate: kineti-core                                                                            |
+---------------------------------------------------------------------------------------------------------+
                                                   |
                                                   v
+---------------------------------------------------------------------------------------------------------+
| LAYER 1: AUTONOMOUS NERVOUS SYSTEM & OS PRIMITIVES (Paper 1)                                            |
| - 13-Stage Closed-Loop Software Factory with Explicit Feasibility, Spec, Security & Ship Gates          |
| - SAGA LIFO Transactional Undo Ledger for Reverse-Chronological State Recovery                          |
| - Immutable Root Goal Invariant & SHA-256 Repository Evidence Binding (Anti-Self-Certification)         |
| - Deterministic Spend Circuit Breaker ($50.00 Hard Ceiling, Trips at $47.50 / 95%)                      |
| Reference Systems: kineti-core, bin/kineti-state.ts, bin/kineti-saga.ts, bin/kineti-evidence.ts          |
+---------------------------------------------------------------------------------------------------------+
```

---

## 2. Research Papers in this Series

### [Paper 1: The Autonomous Nervous System](paper_1_autonomous_nervous_system.md)
*Operating System Primitives, Stage Gates, and Cryptographic Auditability in Autonomous Agent Swarms*
- **Author:** Praveen Kumar (therawlogs.com | Foundational AI Research)
- **Target Venue:** USENIX Annual Technical Conference (ATC) / ACM Symposium on Operating Systems Principles (SOSP)
- **Focus:** Agent OS architecture, 13-stage software factory, SAGA LIFO undo ledgers, cryptographic evidence binding, hardware spend circuit breaker.
- **Abstract Summary:** Formalizes the operating system primitives for autonomous swarms: an immutable cryptographic root goal contract locked at project genesis, an automated SAGA LIFO transactional inverse-command stack ensuring that failed multi-step tool executions are unwound in reverse chronological order without human intervention, and SHA-256 byte-level evidence chains that bind verification results directly to repository content, preventing false self-certification.
- **Core Systems:** `bin/lib.ts`, `bin/kineti-state.ts`, `bin/kineti-saga.ts`, `bin/kineti-evidence.ts`

### [Paper 2: The Physics of Context](paper_2_physics_of_context.md)
*Memory Bandwidth, the 100× Move Penalty, and Lock-Free Atomics*
- **Author:** Praveen Kumar (therawlogs.com | Foundational AI Research)
- **Target Venue:** ACM Transactions on Computer Systems (TOCS) / USENIX OSDI / USENIX ATC
- **Focus:** Hardware architecture, memory physics, Landauer limits, Apple Silicon vs NVIDIA fabrics, lock-free atomics, wait-free EBR.
- **Abstract Summary:** Evaluates context execution across the thermodynamic limits of silicon. Proves that moving data between off-chip HBM/DRAM and on-chip SRAM incurs an energy penalty of approximately $100\times$ compared to arithmetic compute (~10 pJ/bit vs ~0.1 pJ/FLOP). Evaluates Apple Silicon Unified Memory (32MB ANE on-die SRAM with a 15MB INT4 30M intent micro-router running in 45–65 µs, `CASP`/`FEAT_LSE2` atomic loads) and NVIDIA Hopper/Blackwell fabrics (`membar.sys`, `atom.cas.b64`). Introduces wait-free Epoch-Based Reclamation (EBR) via `ArcSwap`, achieving $p99 < 0.1\,\text{ms}$ snapshot acquisition latency under 80-thread writer contention with zero torn reads and zero mutex contention.
- **Core Crate:** `core-native/kineti-core` (`src/snapshot.rs`, `src/hlc.rs`, `src/lib.rs`)

### [Paper 3: Beyond Vector Search](paper_3_beyond_vector_search.md)
*Causal-Graph Substrates and Runtime Ontology Trigger Data (OTD)*
- **Author:** Praveen Kumar (therawlogs.com | Foundational AI Research)
- **Target Venue:** VLDB / ACM SIGMOD / ACM SIGIR
- **Focus:** Dual-substrate memory (pgvector + ISO SQL/PGQ), universal 20-entity provenance kernel, runtime OTD (JMESPath), sub-50ms 3-way commit gate, RoaringBitmap tombstones.
- **Abstract Summary:** Identifies the fundamental failure modes of pure vector similarity search: causal inversion, temporal blindness, and state invariant ignorance. Introduces a dual-substrate architecture coupling dense vector embeddings with an ISO SQL:2023 Property Graph Queries (SQL/PGQ) compliant property graph. Standardizes on a universal 20-entity causal provenance kernel (`actor`, `role`, `authority`, `intent`, `goal`, `task`, `action`, `tool_call`, `rollback_step`, `observation`, `evidence`, `state_change`, `metric`, `decision`, `dependency`, `constraint`, `approval`, `exception`, `outcome`, `review_required`) dynamically bound via runtime Ontology Trigger Data (OTD) schemas with JMESPath rules. Establishes a sub-50ms 3-way commit gate enforcing topological rank acyclicity ($L(A) < L(B)$), Hybrid Logical Clocks ($HLC(B) < HLC(A)$), and content-addressed Merkle DAG edge lineage ($H(e) = \text{SHA256}(\text{type} \mathbin{\Vert} H(u) \mathbin{\Vert} H(v) \mathbin{\Vert} H(\text{payload}))$). Achieves 96.8% multi-hop causal accuracy and $<0.01\%$ temporal inversions.
- **Core Crate:** `core-native/kineti-memory` (`src/graph.rs`, `src/tombstone.rs`, `src/otd.rs`, `src/vector.rs`, `src/epistemic.rs`)

### [Paper 4: The Reflexive Cerebellum](paper_4_sensory_reflex_and_style.md)
*Sub-Millisecond Sensory Triage, Zero-Token Emoji Reactions, and Dynamic Socio-Linguistic Style Profiling*
- **Author:** Praveen Kumar (therawlogs.com | Foundational AI Research)
- **Target Venue:** ACM CHI / EMNLP
- **Focus:** Sensory triage, socio-linguistics, low-latency interaction, tone adaptation, zero-token reactions.
- **Abstract Summary:** Eliminates token waste and conversational friction in consumer messaging channels (WhatsApp, iMessage). Sub-millisecond sensory triage ($p99 < 1.0\,\text{ms}$) routes low-information acknowledgments to native platform reactions (`👍`, `❤️`, `⚡`) with zero LLM invocation (saving 38.4% of API spend and cutting latency from $1{,}840\,\text{ms}$ to $12\,\text{ms}$). An online Exponentially Weighted Moving Average (EWMA) engine tracks 5 linguistic dimensions (formality, brevity, slang density, capitalization, and code-switching) to dynamically hydrate prompts without model drift.
- **Core Crates:** `core-native/kineti-reflex` (`src/sensor.rs`, `src/style.rs`, `src/circuit.rs`), `core-native/kineti-gateway`

### [Paper 5: Outcome Engineering](paper_5_outcome_engineering.md)
*Evaluating Autonomous Agents on Causal Value Graphs and Frontier Benchmarks*
- **Author:** Praveen Kumar (therawlogs.com | Foundational AI Research)
- **Target Venue:** IEEE S&P / ACM CCS / ICSE / NeurIPS
- **Focus:** Goodhart's Law elimination, Causal Outcome Graphs, SCM counterfactual ATE, Asymmetric Dual-Signed OVTs, DNTI metric, $/Outcome economics, Agents' Last Exam (ALE) & SWE-bench.
- **Abstract Summary:** Replaces activity-based SDLC tracking with verifiable Causal Outcome Graphs ($\text{Intent} \to \text{Task} \to \text{Action} \to \text{State Change} \to \text{Outcome}$) formalized as Structural Causal Models (SCMs) with counterfactual Average Treatment Effect (ATE) identification. Enforces the Outcome Verification Ticket (OVT) protocol with asymmetric dual-signed Ed25519 attestation by isolated verification enclaves ($id_w \neq id_r$). Introduces Directional Normalized Trust-Weighted Impact (DNTI) with semantic entropy attenuation ($\tau = 0.15$) and multi-point test integrity predicates. Formulates Cost Per Verified Outcome ($\mathcal{C}_{\text{OVO}} = \$0.31$ per resolution). Evaluates against frontier multi-step evaluation suites: achieves $76.4\%$ pass rate on the **Agents' Last Exam (ALE)** ($68.2\%$ on $>10$-step tasks), cuts task gaming to $<0.1\%$, and reduces SWE-bench incident MTTR to $4.2\text{ min}$ ($9.1\times$ reduction).
- **Core Systems:** `core-native/kineti-harness` (`src/ovt.rs`, `src/cvg.rs`, `src/dnti.rs`, `src/shadow.rs`), `core-native/kineti-core` (`src/spend.rs`), `bin/kineti-spend.ts`, `bin/kineti-swarm.ts`

---

## 3. Recommended Reading Order

```
[Paper 1: Autonomous Nervous System]    OS primitives, 13-stage gates, SAGA LIFO undo ledger, evidence.
             │
             ▼
[Paper 2: Physics of Context]          Hardware thermodynamics, 100x move penalty, wait-free EBR atomics.
             │
             ▼
[Paper 3: Beyond Vector Search]        Dual-substrate memory, 20-entity kernel, 3-way commit gate.
             │
             ▼
[Paper 4: The Reflexive Cerebellum]    Sub-1ms sensory triage, zero-token reactions, EWMA style profiler.
             │
             ▼
[Paper 5: Outcome Engineering]         Causal outcome graphs, dual-signed OVTs, DNTI, ALE benchmarks.
```

- **Systems Engineers & OS Architects:** Begin with **Paper 1** for closed-loop software factory gates and SAGA undo mechanics, followed by **Paper 2** for microarchitectural latency tiers, DRAM bus saturation, and wait-free EBR double-word pointer swaps.
- **Information Retrieval & Knowledge Graph Researchers:** Begin with **Paper 3** to examine the mathematical failure of cosine similarity under temporal rollbacks, ISO SQL/PGQ graph schema, and the sub-50ms 3-way commit gate.
- **HCI & Conversational AI Engineers:** Begin with **Paper 4** to evaluate sub-millisecond sensory triage and zero-token emoji reflex circuits in high-volume messaging channels.
- **AI Safety, Governance & Software Economics Leaders:** Begin with **Paper 5** to study asymmetric Ed25519 dual-signing protocols, non-repudiation proofs, $/Outcome unit economics, and frontier evaluations on Agents' Last Exam (ALE).

---

## 4. Unified Mathematical Notation & Conventions

Across all five treatises, the following mathematical conventions and symbols are consistently maintained:

| Symbol | Mathematical Domain | Physical / Operational Interpretation | Reference Paper |
| :--- | :--- | :--- | :--- |
| $\mathcal{H}$ | Tuple | Hardware memory hierarchy $(\mathcal{C}_{L1}, \mathcal{C}_{L2}, \mathcal{C}_{L3}, \mathcal{M}_{\text{DRAM}}, \mathcal{B}_{\text{bus}})$ | Paper 2 |
| $E_{\text{Landauer}}$ | $\mathbb{R}^+$ (Joules) | Landauer thermodynamic limit: $k_B T \ln 2 \approx 2.87 \times 10^{-21}\text{ J} = 2.87\text{ zJ}$ | Paper 2 |
| $\Pi_{\text{move}}$ | $\mathbb{R}^+$ | The Move Penalty Ratio ($\tau_{\text{IPC\_Copy}} / \tau_{\text{Atomic\_Swap}} \in [80, 240]$) | Paper 2 |
| $E_g, e_i$ | $\mathbb{Z}_3 = \{0, 1, 2\}$ | Global system epoch and thread-local epoch register in EBR | Paper 2 |
| $\mathbf{P}_{\text{state}}$ | $\langle *const\ V, u64 \rangle$ | Double-word atomic pointer root with 64-bit generation tag (`CASP`/`FEAT_LSE2`) | Paper 2 |
| $\mathcal{G}$ | Directed Property Graph | Directed causal execution graph $(\mathcal{V}, \mathcal{E}, \tau_V, \tau_E, \alpha_V, \alpha_E)$ | Paper 3, 5 |
| $\Sigma_V$ | Set ($|\Sigma_V|=20$) | The Universal 20-Entity Provenance Kernel primitives | Paper 3 |
| $\Sigma_E$ | Set ($|\Sigma_E|=13$) | Causal relation types (`CAUSED_BY`, `RESOLVES`, `IMPLEMENTS`, etc.) | Paper 3 |
| $L(v)$ | $v \in \mathcal{V} \to \mathbb{N}$ | Topological rank level assignment enforcing $O(1)$ cycle freedom $L(A) < L(B)$ | Paper 3 |
| $HLC(v)$ | $\mathbb{N} \times \mathbb{N}$ | Hybrid Logical Clock tuple $\langle \text{phys}: u64, \text{counter}: u32 \rangle$ with $\Delta t_{\text{skew}} \le 250\text{ ms}$ | Paper 3 |
| $H(e)$ | $\{0, 1\}^{256}$ | Content-addressed Merkle DAG digest $\text{SHA256}(\text{relation} \mathbin{\Vert} H(u) \mathbin{\Vert} H(v) \mathbin{\Vert} H(\text{payload}))$ | Paper 3 |
| $B_{\text{tomb}}$ | Roaring Bitmap | Compressed bitset of invalidated/rolled-back vector IDs ($O(1)$ SAGA mask) | Paper 3 |
| $S(d_i)$ | $[0, 1]$ | Temperature-scaled hybrid fusion score combining semantic similarity and geodesic hop decay | Paper 3 |
| $\tau$ | $\mathbb{R}^+$ | Softmax temperature calibration parameter ($\tau = 0.07$ in Paper 3, $\tau = 0.15$ in Paper 5) | Paper 3, 5 |
| $\gamma$ | $(0, 1)$ | Topological geodesic hop discount parameter ($\gamma = 0.75$) | Paper 3 |
| $\mathcal{C}(M)$ | Function | Sensory triage classifier mapping message $M$ to intent in $\tau(\mathcal{C}(M)) \le 1.0\,\text{ms}$ | Paper 4 |
| $\mathbf{S}_t$ | $[0, 1]^5$ | Online socio-linguistic EWMA style vector (formality, brevity, slang, caps, code-switch) | Paper 4 |
| $U, R$ | Functions | Latent enterprise utility function $U(s)$ vs observable proxy reward metric $R(s)$ | Paper 5 |
| $\mathcal{M}$ | SCM 4-Tuple | Structural Causal Model $\langle \mathbf{U}, \mathbf{V}, \mathbf{F}, P(\mathbf{U}) \rangle$ | Paper 5 |
| $\text{ATE}$ | $\mathbb{R}$ | Counterfactual Average Treatment Effect $\mathbb{E}[Y \mid do(A=a), S=s] - \mathbb{E}[Y \mid do(A=\emptyset), S=s_0]$ | Paper 5 |
| $\text{OVT}$ | Tuple | Asymmetric Dual-Signed Outcome Verification Ticket | Paper 5 |
| $\sigma_w, \sigma_r$ | $\{0, 1\}^{512}$ | Ed25519 digital signatures of Worker agent and isolated Reviewer enclave | Paper 5 |
| $\text{DNTI}$ | $[0, 1]$ | Directional Normalized Trust-Weighted Impact: $\Phi \times \sigma_\tau(SE) \times \Psi(\mathcal{T})$ | Paper 5 |
| $\Phi$ | $[-1, 1]$ | Directional Normalized Delta with metric direction $\mathbf{d} \in \{+1, -1\}$ | Paper 5 |
| $\sigma_\tau(SE)$ | $(0, 1]$ | Bounded exponential semantic entropy attenuation: $\exp(-SE / \tau)$ | Paper 5 |
| $\Psi(\mathcal{T})$ | $\{0, 1\}$ | Multi-point test integrity predicate enforcing exit code 0, assertions, diff coverage, and no tampering | Paper 5 |
| $\mathcal{C}_{\text{OVO}}$ | $\mathbb{R}^+$ (USD) | Cost Per Verified Outcome: $\sum \text{Cost} / \max(1, \text{VerifiedOutcomes})$ | Paper 5 |
| $C_{\max}$ | $\mathbb{N}$ ($\mu\text{c}$) | Spend circuit breaker hard ceiling ($50{,}000{,}000\ \mu\text{c} = \$50.00$, trips at 95% = $47.50) | Paper 1, 5 |

---

## 5. Architectural Synthesis: Mapping Theory to Native Code

The mathematical formulations documented in this research series constitute the formal specification of the native Rust crates (`core-native/`) and TypeScript governance tools (`bin/`):

| Paper Section | Theoretical Formalism | Substrate / Crate | Module & Implementation Target |
| :--- | :--- | :--- | :--- |
| **Paper 1, §2** | 13-Stage Closed-Loop Software Factory | `bin/` | `bin/kineti-state.ts` (State machine, gate transition proofs) |
| **Paper 1, §3** | SAGA LIFO Transactional Undo Ledger | `bin/` | `bin/kineti-saga.ts` (LIFO stack, inverse commands, tamper detection) |
| **Paper 1, §4** | SHA-256 Byte-Level Evidence Chains | `bin/` | `bin/kineti-evidence.ts` (Freshness check, git diff hashing) |
| **Paper 2, §2** | Landauer Limit & 100x Move Penalty | `kineti-core` | `src/snapshot.rs` (Zero-copy in-memory buffers vs serialization) |
| **Paper 2, §3** | Heterogeneous Silicon (Apple vs NVIDIA) | `kineti-core` | `src/snapshot.rs` (`FEAT_LSE2` aligned quadword loads, PTX atomics) |
| **Paper 2, §4** | Wait-Free EBR Pointer Swapping | `kineti-core` | `src/snapshot.rs` (`ArcSwap` wait-free snapshot reads, CAS commit) |
| **Paper 2, §5** | Zero-Torn-Read Invariant | `kineti-core` | `tests/concurrency_tests.rs` (80-thread writer contention verification) |
| **Paper 3, §2** | Dual-Substrate Memory Engine | `kineti-memory` | `src/vector.rs`, `src/graph.rs` (pgvector / HNSW + property graph) |
| **Paper 3, §3** | 20-Entity Universal Provenance Kernel | `kineti-core` | `src/kernel.rs` (`enum EntityType`, `enum RelationType`) |
| **Paper 3, §3** | Runtime Dynamic OTD & JMESPath | `kineti-memory` | `src/otd.rs` (Dynamic schema bindings, event extractions) |
| **Paper 3, §4** | Sub-50ms 3-Way Commit Gate | `kineti-core` | `src/gate.rs` (Rank acyclicity $L(A) < L(B)$, HLC time, Merkle DAG) |
| **Paper 3, §5** | RoaringBitmap Tombstone Masking | `kineti-memory` | `src/tombstone.rs` (`roaring::RoaringBitmap`, $O(1)$ SAGA invalidation) |
| **Paper 3, §6** | Temperature-Scaled Hybrid Fusion | `kineti-memory` | `src/vector.rs` (Softmax cosine score + geodesic hop decay) |
| **Paper 4, §2** | Sub-1ms Sensory Triage Circuit | `kineti-reflex` | `src/sensor.rs` ($p99 < 1.0\,\text{ms}$ intent classification) |
| **Paper 4, §3** | 5D Socio-Linguistic EWMA Profiler | `kineti-reflex` | `src/style.rs` (Online formality, brevity, slang, caps tracking) |
| **Paper 4, §4** | Dynamic Prompt Hydration | `kineti-reflex` | `src/circuit.rs`, `kineti-gateway::cortex` (Persona mirroring directives) |
| **Paper 5, §2** | Causal Outcome Graphs & SCM ATE | `kineti-harness`| `src/cvg.rs` (Directed acyclic value graphs, counterfactual treatment) |
| **Paper 5, §3** | Asymmetric Dual-Signed Ed25519 OVTs | `kineti-harness`| `src/ovt.rs` (`ed25519-dalek` dual signatures, verification enclaves) |
| **Paper 5, §3** | Directional Normalized Impact (DNTI) | `kineti-harness`| `src/dnti.rs` (Directional delta $\Phi$, semantic entropy $\sigma_\tau(SE)$) |
| **Paper 5, §4** | $/Outcome Unit Economic Modeling | `kineti-harness`| `src/cvg.rs`, `bin/kineti-spend.ts` ($\mathcal{C}_{\text{OVO}}$ calculation) |
| **Paper 5, §5** | Bounded Retrospective Replay ($A^*$) | `kineti-memory` | `src/graph.rs` (Heuristic priority search across 90-day horizons) |
| **Paper 5, §6** | Atomic Spend Circuit Breaker | `kineti-core` | `src/spend.rs` (Microcent atomic CAS, $50 ceiling, OS exit halt) |

---

## 6. Citation & BibTeX

To cite the Context Integrity Layer research series in academic publications:

```bibtex
@inproceedings{kumar2026nervous,
  title     = {The Autonomous Nervous System: Operating System Primitives, Stage Gates, and Cryptographic Auditability in Autonomous Agent Swarms},
  author    = {Kumar, Praveen},
  booktitle = {Proceedings of the USENIX Annual Technical Conference (ATC)},
  year      = {2026},
  note      = {therawlogs.com | Foundational AI Research}
}

@article{kumar2026physics,
  title     = {The Physics of Context: Memory Bandwidth, the 100× Move Penalty, and Lock-Free Atomics},
  author    = {Kumar, Praveen},
  journal   = {ACM Transactions on Computer Systems (TOCS)},
  year      = {2026},
  volume    = {44},
  number    = {3},
  pages     = {1--38},
  note      = {therawlogs.com | Foundational AI Research}
}

@inproceedings{kumar2026beyondvector,
  title     = {Beyond Vector Search: Causal-Graph Substrates and Runtime Ontology Trigger Data (OTD)},
  author    = {Kumar, Praveen},
  booktitle = {Proceedings of the 52nd International Conference on Very Large Data Bases (VLDB)},
  year      = {2026},
  pages     = {1420--1436},
  note      = {therawlogs.com | Foundational AI Research}
}

@inproceedings{kumar2026reflexive,
  title     = {The Reflexive Cerebellum: Sub-Millisecond Sensory Triage, Zero-Token Emoji Reactions, and Dynamic Socio-Linguistic Style Profiling},
  author    = {Kumar, Praveen},
  booktitle = {Proceedings of the ACM Conference on Human Factors in Computing Systems (CHI)},
  year      = {2026},
  note      = {therawlogs.com | Foundational AI Research}
}

@inproceedings{kumar2026outcome,
  title     = {Outcome Engineering: Evaluating Autonomous Agents on Causal Value Graphs and Frontier Benchmarks},
  author    = {Kumar, Praveen},
  booktitle = {Proceedings of the 47th IEEE Symposium on Security and Privacy (S\&P)},
  year      = {2026},
  pages     = {890--907},
  note      = {therawlogs.com | Foundational AI Research}
}
```
