# Kineti

**Ship proof + spend fuse for any agent. The meter and the stamp.**

Claude, Cursor, Grok, and `fx` write code. Kineti writes the receipt and blocks merge if tests don't match files.

> **v0.2:** Old 13-stage runner is frozen on tag `v0.1.0` (`docs/v0.1.md`). Use `kineti run --legacy --goal "..."` for the old pipeline — it is hidden and prints a legacy warning (`kineti run --help` shows `--legacy`). The product is three commands and a red X on merge.

```
agent → gateway (cap, policy) → model API
              ↓
       receipt (hashes, $, pass/fail)
              ↓
        CI / merge ← fail if stale or missing
```

```sh
kineti evidence --cmd "cargo test"   # bind tests to current files
kineti ship-check                    # gate: 0 fresh, 1 stale, 2 missing
kineti verify && kineti receipt      # offline check + summary
```

`verify` is offline — no server required.

## Install

```sh
curl -fsSL https://getkineti.com/install.sh | sh
curl -fsSL https://getkineti.com/install.sh | sh -s v0.2.0  # pin — GitHub release body holds CHANGELOG v0.2.0 notes
```

macOS (arm64/x64) and Linux (x64/arm64, musl-static preferred). From source: `cargo install --git https://github.com/therawlogs/kineti`

Uninstall: `rm -f /usr/local/bin/kineti ~/.local/bin/kineti ~/.cargo/bin/kineti && rm -rf ~/.kineti/auth .kineti kineti.toml`

## GitHub Action (required check)

Uses the composite at `.github/actions/kineti-receipt` (exists on `main`). No server.

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

- With `verify-command` set, the action runs `kineti evidence --cmd "<verify-command>"` then `ship-check` + `verify --all` — no prior `.kineti/` needed.
- Without it, run `kineti evidence --cmd "..."` in a prior step first, then the action runs `ship-check`/`verify`.

Fails if receipt missing, stale, or fingerprint doesn't match current files.

## Quickstart (local, no gateway)

```sh
kineti init                                   # .kineti/ + kineti.toml
kineti evidence --cmd "cargo test"
kineti ship-check
kineti receipt
kineti verify --all
kineti wrap -- cargo test                     # run any command under the cap
kineti gateway --port 8787 &                  # OpenAI proxy demo (reserve/settle)
```

## Configuration

`kineti.toml` — caps, model prices, verify command. Never compiled in.

```toml
[limits]
global_usd = 50.0
per_stage_usd = 10.0        # 0 disables
verify_command = "cargo test"
[execution]
mode = "single"
```

Agents using OpenAI wire format point at the gateway via `[providers.*].base_url` (`src/provider.rs`). Gateway is stateless workers + one ledger per org; receipts are append-only, hashes + counts only — never raw prompts.

## Receipt (v1)

`kineti receipt --json` line:

```json
{"v":"1","at":"2026-08-27T…Z","cmd":"cargo test","passed":true,"fingerprint":"abc…","chain_head":"def…","cost_usd":0.042}
```

`ship-check` codes: `0` fresh, `1` stale/failed, `2` missing, `3` chain broken.

## Commands

| Command | Purpose |
|---|---|
| `init` | scaffold `.kineti/` + `kineti.toml` |
| `evidence --cmd …` | bind proof to fingerprint |
| `ship-check` | gate — refuse if stale/missing |
| `verify [--all]` | offline hash-chain check |
| `receipt` | spend + gates + DAG |
| `wrap -- <cmd>` | run any command under the cap |
| `gateway --port 8787` | OpenAI proxy demo (hosted in `kineti-pro`) |
| `clean-check` | scan for secrets / forbidden strings |
| `serve` | local ledger daemon |
| `provider-test` / `login` / `auth-status` | provider smoke-test + OAuth |
| `run --legacy --goal …` / `resume` / `undo` / `merge` | frozen 13-stage pipeline (v0.1.0, hidden, see `kineti run --help`) |

## Status

v0.2.0 — `verify` + `ship-check` + `receipt.v1` stable. Hosted gateway + dashboard in `therawlogs/kineti-pro` (private). See `CHANGELOG.md`.

Legacy 13-stage code remains in `src/` (`stages.rs`, `swarm.rs`, etc.) behind hidden `kineti run --legacy`; no `examples/` directory — `docs/v0.1.md` is the frozen reference.

Docs: [`docs/DEMO.md`](docs/DEMO.md) (30s receipt demo + legacy 90s) · [`docs/RELEASE.md`](docs/RELEASE.md) (cut + release-notes) · [`docs/v0.1.md`](docs/v0.1.md) (frozen runner) · [`CONTRIBUTING.md`](CONTRIBUTING.md) · [`SECURITY.md`](SECURITY.md)

Apache-2.0 — `v0.1.0` tag preserved, forkable.
