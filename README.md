# Kineti

**Ship proof + spend fuse for any agent. The meter and the stamp.**

Claude, Cursor, Grok, and `fx` write the code. Kineti writes the receipt and blocks merge if tests do not match the files.

> **v0.2:** Kineti is a ship proof and spend fuse for any agent. The old 13-stage runner is no longer the default — it is frozen on tag `v0.1.0` (see `docs/v0.1.md`). `kineti run --goal` still exists as `kineti run --legacy` for that tag; the new default is three commands and a red X on merge.

```
Cursor / Claude / Grok / LangGraph
        │
        ▼
   your gateway  ← policy: cap, allow tools, require y
        │
        ├─► model API (OpenAI, xAI, Anthropic, …)
        │
        ▼
   receipt written  (who, model, $ spent, hashes, pass/fail)
        │
        ▼
   CI / merge  ← fail if receipt missing or stale
```

```sh
# 1. Any agent writes code (Cursor, Claude, Grok …) pointed at your URL:
#    base_url = https://gate.getkineti.com/v1   # or http://localhost:8787/v1 locally

# 2. Bind tests to the current files:
kineti evidence --cmd "cargo test"

# 3. Gate — CI runs this (see Action below):
kineti ship-check   # exit 1 = stale, 2 = missing

# 4. Inspect:
kineti verify && kineti receipt
```

Merge is blocked if `ship-check` fails. `verify` works offline from files you already have — no cloud required.

## Install

```sh
curl -fsSL https://getkineti.com/install.sh | sh
```

Pin a version:

```sh
curl -fsSL https://getkineti.com/install.sh | sh -s v0.2.0
# v0.1.0 13-stage runner stays available as tag v0.1.0
```

Supported: **macOS** (Apple silicon & Intel) and **Linux** x64/arm64 — Linux prefers musl-static builds.

### Uninstall

```sh
rm -f /usr/local/bin/kineti "$HOME/.local/bin/kineti" "$HOME/.cargo/bin/kineti"
rm -rf ~/.kineti/auth
rm -rf .kineti kineti.toml
```

Or from source: `cargo install --git https://github.com/therawlogs/kineti`

## What it keeps from v0.1

| Keep | Why it sells |
|---|---|
| Spend reserve / settle + hard stop (`src/ipc/pool.rs`) | Stops a bill. Finance understands it. |
| Ship proof bound to file hashes (`src/enforce/evidence.rs`) | Stops merge of untested agent edits. |
| Hash-chained journal + offline verify (`src/memory/journal.rs`) | Receipt someone else can check. |

Everything else is demo — stages 1–13 are now `docs/v0.1.md` / `examples/`.

## How it is distributed

Three install paths, all at once:

1. **Proxy URL** — `base_url = https://gate.getkineti.com/v1` — one line in their OpenAI-compatible SDK config. Gateway does `reserve` → forward → `settle`, writes receipt (hashes + metadata, never raw prompts). Hosted in `therawlogs/kineti-pro` (private); local demo via `kineti gateway --port 8787`.
2. **CI Action** — `kineti-receipt` required on the protected branch. Works even if they never change the agent. Weakest on spend, strongest on ship.
3. **Wrap command** — `kineti wrap -- claude -p "…"` or `kineti wrap -- fx ask "…"` — laptops/agencies. Does not catch Cursor Cloud unless cloud points at your URL.

Start with 1 + 2. Wrap is extra.

## GitHub Action (required check)

```yaml
# .github/workflows/kineti-receipt.yml
name: kineti-receipt
on: [pull_request]
jobs:
  receipt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: ./.github/actions/kineti-receipt
        with:
          verify-command: "cargo test --all"
```

Fail if receipt missing or fingerprint does not match current files. No server required for `verify`.

## Quickstart (local, no gateway)

```sh
kineti init                                   # .kineti/ + kineti.toml (cap, verify_command)
kineti evidence --cmd "cargo test"            # bind proof to fingerprint
kineti ship-check                             # gate
kineti receipt                                # spend + head + gates
kineti verify --all                           # full DAG check
kineti wrap -- cargo test                     # run anything under the cap
kineti gateway --port 8787 &                  # OpenAI proxy with reserve/settle (demo)
kineti serve &                                # local ledger daemon (optional)
```

Scale note: `verify` is offline. Gateway is stateless workers + one ledger per org; receipts are append-only to object storage. Never store raw prompts — hashes + counts only.

## Configuration

`kineti.toml` — model names, prices, caps, verify command. Data, never compiled in.

```toml
[limits]
global_usd = 50.0
per_stage_usd = 10.0        # 0 disables
max_worker_usd = 0
verify_command = "cargo test"

[execution]
mode = "single"
```

Agents that speak OpenAI wire format work through one client (`src/provider.rs`). Point them at the gateway via `[providers.*].base_url`.

## Receipt schema (v1)

`kineti receipt --json` (or `.kineti/receipt.jsonl` line) is versioned:

```json
{"v":"1","at":"2026-08-27T…Z","cmd":"cargo test","passed":true,"fingerprint":"abc…","chain_head":"def…","cost_usd":0.042}
```

`ship-check` exit codes: `0` fresh, `1` stale/failed, `2` missing.

## Command reference (default)

| Command | Purpose |
|---|---|
| `init` | scaffold `.kineti/` + `kineti.toml` |
| `evidence --cmd …` | bind proof to current fingerprint |
| `ship-check` | refuse if proof stale/missing (gate) |
| `verify [--all]` | offline chain check |
| `receipt` | spend + gates + DAG summary |
| `wrap -- <cmd>` | run any command under the cap |
| `gateway --port 8787` | OpenAI proxy with reserve/settle (demo; hosted in kineti-pro) |
| `clean-check` | scan for secrets / forbidden strings |
| `serve` | local ledger daemon on `.kineti/kineti.sock` |
| `provider-test` | smoke-test a provider |
| `login/logout/auth-status` | PKCE OAuth |
| `run --legacy` / `resume` / `undo` / `merge` | frozen 13-stage pipeline (v0.1.0, hidden) |

## Status

v0.2.0 — ship proof + spend fuse, 13-stage runner demoted. `verify` + `ship-check` + `receipt.v1` stable. Hosted gateway + dashboard live in `therawlogs/kineti-pro` (private). See `CHANGELOG.md`.

## License

Apache-2.0 — `v0.1.0` tag preserved, forkable.
