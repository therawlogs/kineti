# Native Rust core internals

This page holds implementation names that used to sit in the README. The CLI a user can run is documented there: a hash-chained ledger, a commit check, and a spend breaker that exits with code 3 at 95% of recorded spend.

## `core-native/`

- **Double-Buffered Atomic Snapshots**: Two-slot `RwLock` snapshot design with thread-safe read paths and deferred epoch reclamation of retired instances. Latency figures are withheld until reproducible benchmark scripts land in the repo.
- **Universal 20-Entity Provenance Kernel**: Content-addressed RFC 8785 JSON canonicalization with BLAKE3 and SHA-256 digests (hand-rolled under a zero-external-dependency constraint; see `docs/PLAN.md` for rationale and test vectors).
- **Monotonic Hybrid Logical Clock (HLC)**: Physical and logical causality tracking under clock skew.
- **3-Way Graph Commit Gate**: Rejects causal inversions, topological DAG cycles, and single-byte state tampering.
- **Sensory Reflex Triage**: Fast sensory classification dispatching zero-token emoji reactions for low-information conversational stimuli.
- **Protocolized Connectors**: Standard `KinetiConnectorProtocol` trait with consequence level gating (`Trivial`, `Operational`, `HighConsequence`) and single-use SHA-256 payload authorization tokens.

The production architecture spec is [PLAN.md](./PLAN.md).
