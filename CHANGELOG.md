# CHANGELOG

## v0.2.0 — 2026-08-27 — cut to gateway + receipt

> Kineti is a ship proof and spend fuse for any agent. The old 13-stage runner is no longer the default. Claude, Cursor, Grok, and fx write the code. Kineti writes the receipt and blocks merge if tests do not match the files.

### Kept (this is 0.2 / future 1.0)

- `evidence --cmd` bound to fingerprint (`src/enforce/evidence.rs`)
- `ship-check` — exits 1 stale/failed, 2 missing
- `verify` / `verify --all` — offline hash-chain check (`src/memory/journal.rs`)
- `receipt` (v1 JSON) + `clean-check` + `serve` local ledger (`src/daemon.rs`)
- Spend reserve/settle + `.kineti/spend.reset` human gate (`src/ipc/pool.rs`)
- `wrap -- <cmd>` — run any command under the cap
- `provider-test`, `login`/`logout`/`auth-status`, `task` (thin)

### Demoted (not default, code kept but hidden)

- `kineti run --legacy --goal` 13-stage pipeline → `kineti run --legacy` hidden, prints legacy warning, see `docs/v0.1.md` / tag `v0.1.0`
- `resume`, `undo`, `merge`, swarm/worktrees, `officehours → retro` stages 1–13 as product
- `review` / `qa-as-stage` / `security` as pipeline stages → replaced by caller's `evidence --cmd`
- `watch` / `retro` removed from default docs

### Removed from homepage

- "Only mathematically verifiable harness" claim; greeting.sh 13-stage demo
- Default `run` in README quickstart; swarm pitch as default

### Added

- `wrap` command, receipt `v` field, documented `ship-check` exit codes
- Reusable GitHub Action at `.github/actions/kineti-receipt/` (offline, no server)
- `docs/v0.1.md` — frozen runner reference

### Compatibility

- Apache-2.0 stays. `v0.1.0` tag preserved on GitHub. Fork the old pipeline if you want it.
- `v0.2.0` bumps `Cargo.toml` version; binaries still `kineti` (no rename).
- `kineti verify` and `kineti receipt` wire formats unchanged except added `v` in JSON.

## v0.1.0 — 2026-08-25

Initial harness: ReAct loop, 13-stage machine, swarm, DAC, evidence fingerprints, hash-chained DAG journals, UDS daemon with shared ledger, branch-and-merge, PKCE OAuth, C-ABI. Tagged and published with 8 assets + SHA256SUMS.
