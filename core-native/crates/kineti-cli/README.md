# `kineti-cli` 1.0.1 — AI-PR Verification & Anti-Tampering Gate

**Keep AI pull requests honest before they merge.**

`kineti-cli` ships the standalone, offline, MIT-licensed `kineti` executable for Linux and macOS (`x64` and `arm64`). It locks test policy to your protected base branch, scans PR diffs across Rust, Python, JS/TS, and Go for skipped or deleted tests and workflow edits, supervises exact allowlisted test runs with streaming `O_NOFOLLOW` workspace freshness hashing, and emits Ed25519-signed `kineti-pr-audit-v1` bundles.

> **Published open-core release (`1.0.1`):** Published on `crates.io`, `npm`, and GitHub Release `v1.0.1`, with post-publication exact-archive acceptance verified across `linux-x64`, `linux-arm64`, `darwin-x64`, and `darwin-arm64`. Historical `0.6.1` packages remain retired (yanked on crates.io, deprecated on npm). Older pre-0.6.0 assistant releases remain in Git history; harness execution, model routing, delegation, swarm, dashboard, memory, evaluator services, and protected writes are removed from the active product.

## Install & Run

Install from `crates.io` (requires Rust/Cargo; runtime needs neither Node.js nor Bun):

```sh
cargo install kineti-cli --version '=1.0.1' --locked
```

Or build directly from source:

```sh
cargo build --release --locked --manifest-path core-native/Cargo.toml
./core-native/target/release/kineti --version
```

## Core Workflow

```sh
# 1. Preview and apply repo initialization (keep .kineti/ runtime files out of Git)
kineti init --goal "Ship verified pull requests"
kineti init --goal "Ship verified pull requests" --apply

# 2. Run supervised tests (count-aware for cargo/bun, exit-code for polyglot runners)
kineti test run --label rust-unit --required-test tests/ --parser auto -- cargo test
kineti test run --label py-unit --required-test tests/test_auth.py --parser exit-code -- pytest -q tests/

# 3. Verify freshness and enforce base-branch policy on pull requests
kineti test check --label rust-unit
kineti ci --require-evidence rust-unit
kineti pr-check --base-ref origin/main --head-ref HEAD --require-ci --require-evidence rust-unit \
  --audit-out .kineti/pr-audit.json --summary-out .kineti/pr-summary.md

# 4. Optional local utilities
kineti status [--project-only]
kineti spend check
kineti undo push --label cleanup -- rm generated.tmp
kineti undo rollback
kineti mcp --root .
kineti self-test
```

## Honest Limits

- Ed25519 signatures prove record integrity relative to the signing key—not assertion quality (`"independent_assertion_quality_proven": false`).
- Count-aware parsing (`--parser auto` or explicit `--parser junit`) covers `cargo test`, multi-suite `bun test`, and JUnit XML; `--parser exit-code` runners explicitly record `"count_observed": false`.
- Real anti-tampering enforcement requires running `kineti pr-check` and `action.yml` on a protected Linux CI runner reading `.kineti/gate.json` from the protected base branch.
- `kineti mcp` exposes 6 tools (undo/saga tools are CLI-only) and strictly enforces `.kineti/gate.json` command allowlists on `kineti_evidence_record`. Cursor and Antigravity remain unverified.
- Receipts persist exact command `argv` without automatic secret redaction; pass credentials via environment/CI secrets only.
- Reported spending records operator-entered amounts, not provider bills.

## Links

- Documentation & Open-Core Pilot: [https://getkineti.com](https://getkineti.com)
- Repository: [https://github.com/therawlogs/kineti](https://github.com/therawlogs/kineti)
- License: MIT
