# Kineti Native Engine (`core-native/`)

Pure Rust workspace (8 crates) with zero external crate dependencies and `#![forbid(unsafe_code)]` in critical crates.

## Why hashing is hand-rolled

`kineti-core` implements BLAKE3 and SHA-256 in pure Rust instead of depending on audited crates. This is a deliberate supply-chain decision: the provenance kernel is the trust root for content addressing, so it must build with no third-party code. The tradeoff is correctness burden, carried by standard golden-vector tests in `crates/kineti-core/src/kernel.rs` (`test_sha256_golden_vector`, `test_blake3_golden_vector`).

## Performance figures

Do not quote latency or concurrency numbers for this engine unless they come from reproducible benchmark scripts in `benches/`. Module doc comments state only what the implementation does.
