# Kineti Native Engine (`core-native/`)

The native Rust workspace contains eight crates and the kineti-cli command. It provides local libraries and demos; the npm package supplies the separate Bun-based governance commands.

## Why hashing is hand-rolled

`kineti-core` implements BLAKE3 and SHA-256 in pure Rust. The `kineti-harness` crate uses an established Ed25519 library for digital signatures rather than custom signature code.

## Performance figures

Do not quote latency or concurrency numbers for this engine unless they come from reproducible benchmark scripts in `crates/kineti-cli/benches/`. Module doc comments state only what the implementation does.
