# Kineti 1.0.0 — Offline AI-PR Gate Demo

A self-contained, zero-network demonstration of Kineti’s **AI-PR Verification & Anti-Tampering Gate** (Open-Core MIT). It initializes a disposable temporary Git repository (automatically deleted on exit), commits a protected base-branch `.kineti/gate.json` alongside Python, JS, and Rust test fixtures, and executes four live scenarios in under 15 seconds.

## Prerequisites

- `git` and `python3` on `PATH`
- A compiled native `kineti` `1.0.0` executable

Build from source if you have not already:

```bash
cargo build --release --locked --manifest-path core-native/Cargo.toml
```

## Run the Demo

```bash
./examples/pilot-demo/run-demo.sh
```

Or specify an explicit executable path:

```bash
KINETI_BIN=/absolute/path/to/kineti ./examples/pilot-demo/run-demo.sh
```

By default, `run-demo.sh` checks:

1. `$KINETI_BIN` (if set)
2. `core-native/target/release/kineti`
3. `core-native/target/debug/kineti`
4. `kineti` on `PATH`

No files inside this repository or your host `$HOME/.codex` directory are modified; all demo state runs inside a `mktemp -d` directory removed automatically via `trap`.

## The 4 Live Scenes

| Scene | Simulated AI PR Action | Command Executed | Expected Exit | What It Proves |
| --- | --- | --- | --- | --- |
| **1 — Blocked** | PR weakens `.kineti/gate.json` (`required_checks: []`, spoofed command) | `kineti pr-check --base-ref main --head-ref HEAD` | `1` | Policy is loaded strictly from `main`; unauthorized policy changes fail closed with `policy_tamper_detected`. |
| **2 — Blocked** | PR inserts `@pytest.mark.skip`, `it.skip`, `#[ignore]` and deletes a Python test | `kineti pr-check --base-ref main --head-ref HEAD` | `1` | Polyglot diff scanner flags added skip/ignore markers and net test deletions with exact file/line findings. |
| **3 — Blocked** | After a passing `--parser exit-code` run, 1 byte changes in `src/legacy/auth.py` (`3600` → `3601`) | `kineti test check --label unit` | `4` | Streaming `O_NOFOLLOW` freshness detects 1-byte content drift even inside `src/legacy/` (no silent directory blind spots). |
| **4 — Passed** | Clean fix PR tightens `src/legacy/auth.py` and adds a new test | `kineti test run` + `kineti pr-check --require-evidence unit --audit-out ... --summary-out ...` | `0` | Clean PR passes and emits an Ed25519-signed `kineti-pr-audit-v1` JSON bundle plus Markdown PR audit table. |

## Honest Scope & Limits

- **CI runner trust boundary:** This script runs locally to demonstrate CLI behavior offline. In production, real anti-tampering enforcement requires running `kineti pr-check` and `action.yml` on a protected Linux CI runner reading `.kineti/gate.json` from the protected base branch.
- **Parser transparency:** Scenes 3 and 4 run Python’s stdlib `unittest` fixture using `--parser exit-code`, which explicitly records `"count_observed": false` (distinct from built-in count-aware parsing for `cargo test` and multi-suite `bun test`).
- **Signatures vs. assertion quality:** Ed25519 signatures on `kineti-pr-audit-v1` establish record integrity relative to the signing key—not assertion quality (`"independent_assertion_quality_proven": false`).
- **No hidden hooks or sandboxing:** Kineti does not provide automatic command interception, protected writes, or kernel sandboxing. Cursor and Antigravity remain unverified hosts.
