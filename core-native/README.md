# Kineti Native Engine (`core-native/`)

Pure Rust workspace (8 crates). Critical crates forbid `unsafe`; `kineti-harness` uses the maintained `ed25519-dalek` library for OVT signatures. The Rust credential-vault module is an experimental legacy prototype and must not be used for real secrets.

## Why hashing is hand-rolled

`kineti-core` implements BLAKE3 and SHA-256 in pure Rust. The `kineti-harness` crate uses an established Ed25519 library for digital signatures rather than custom signature code.

## Performance figures

Do not quote latency or concurrency numbers for this engine unless they come from reproducible benchmark scripts in `crates/kineti-cli/benches/`. Module doc comments state only what the implementation does.
