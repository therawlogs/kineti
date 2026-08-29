# CHANGELOG

## v0.2.2 — 2026-08-29 — docs polish + crates.io README sync

- `README.md` pin now `v0.2.2` + `cargo install kineti` note; `Status` now `v0.2.2 any-agent`.
- `docs/RELEASE.md` current `0.2.2`, added v0.2.1 go-live record; `docs/v0.1.md` points to `v0.2.1` as active.
- `SECURITY.md` scope now `v0.2.2` any-agent (`[artifacts]` + `swarm`); `CONTRIBUTING.md` product `v0.2.2` + any-cmd note.
- `.github/actions/kineti-receipt` now `verify-command` + `proof-command` alias, description any-agent (`pytest`/`npm test`/`./verify.sh`).
- No code change beyond docs; fingerprint & wire format still `v1`.

## v0.2.1 — 2026-08-29 — any-agent artifacts + single-command swarm

> Ship proof and spend cap for **any** agent — code, docs, data, configs. Same meter and stamp, now with generic artifact selection and `kineti swarm` in one command.

### Added
- `[artifacts]` in `kineti.toml` — `include`/`exclude` globs + `max_file_bytes` + `follow_symlinks`. Fingerprint now honors it via `src/enforce/evidence.rs` + `src/tools.rs:walk_all` (defaults skip `.git/.kineti/target/node_modules/dist/build/.next/coverage/tmp/.cache/legacy`). Works for any work product, not just code.
- `[proof].command` — default `evidence`/`ship-check` verification (any cmd: `cargo test` | `pytest` | `npm test` | `make check` | `./verify.sh`). Legacy `limits.verify_command` still honored as alias. `kineti evidence` now accepts optional `--cmd` and falls back to `[proof].command`.
- `kineti swarm` — one command fan-out: `kineti swarm --tasks tasks.jsonl --cap 10 [--provider grok] [--max-parallel 4] [--dry-run]`. Tasks file supports JSON array or JSONL with `{id, prompt}` (LLM agent) or `{id, command}` (shell). Parallel waves, isolated journal branches, worktree isolation per `execution.worker_isolation`, auto git merge + `kineti merge` for journals, then auto `evidence → receipt → ship-check`.

### Changed
- Docs: `README.md`, `docs/DEMO.md`, `kineti.toml` defaults now show any-agent examples (`pytest`, `npm test`, swarm `tasks.jsonl`). Fingerprint docs say *artifacts* not *code*; `src/enforce/evidence.rs` errors now say `artifacts changed` / `no proof evidence`.
- Site: `kineti-web/src/pages/index.astro` hero + table now say *any agent / artifacts*; `docs/getting-started.astro` documents `[artifacts]` + `swarm`; `docs/commands.astro` lists `evidence [--cmd]` + `swarm`.
- `src/enforce/evidence.rs:20` `fingerprint(root)` now uses `Config::load_from(root).artifacts` (was `cwd`); `src/tools.rs:384` `walk_all` now uses `load_from(root)`.
- Spend cap docs: `[limits].per_stage_usd` now described as per-scope cap for any agent (gateway/wrap/swarm), not legacy stage-only.

### Compatibility
- Proof `v` stays `"1"`; wire formats unchanged. New fingerprint excludes (`dist/build/.next/coverage/tmp/.cache/legacy`) + `max_file_bytes` may make prior proofs stale if those dirs or >4 MiB files exist — re-run `kineti evidence --cmd "..."`.
- `0.2.0` tag preserved; `0.2.1` is drop-in — no `install.sh` change, so `https://getkineti.com/install.sh` stays byte-identical (no sync needed); `kineti-website` auto-deploy on push will carry new docs.

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
