# Native Rust core internals

The native Rust crates provide local libraries and a separate command-line tool. This guide explains their parts and how they fit together. It does not describe a hosted service.

## `core-native/`

- **Double-Buffered Atomic Snapshots**: Two-slot `RwLock` snapshot design with thread-safe read paths and deferred epoch reclamation of retired instances. Latency figures are withheld until reproducible benchmark scripts land in the repo.
- **Universal 20-Entity Provenance Kernel**: Content-addressed RFC 8785 JSON canonicalization with BLAKE3 and SHA-256 digests (hand-rolled under a zero-external-dependency constraint; see `docs/PLAN.md` for rationale and test vectors).
- **Monotonic Hybrid Logical Clock (HLC)**: Physical and logical causality tracking under clock skew.
- **3-Way Graph Commit Gate**: Rejects causal inversions, topological DAG cycles, and single-byte state tampering.
- **Sensory Reflex Triage**: Fast sensory classification dispatching zero-token emoji reactions for low-information conversational stimuli.
- **Protocolized Connector Libraries**: Connector request builders and consequence checks are library code. This is not a hosted integration service; the local Companion does not connect service accounts or expose credential storage in v0.5.0.

The production architecture spec is [PLAN.md](./PLAN.md).
