# Kineti 1.0.1 — AI-PR Verification & Anti-Tampering Gate

**Keep AI pull requests honest before they merge.**

Kineti is an open-core (MIT) offline Rust CLI and Linux GitHub Actions merge gate for engineering teams shipping code with AI coding agents (Claude Code, Codex CLI, Cursor). It locks test rules to your protected base branch, catches skipped or deleted tests and workflow edits in PR diffs, proves tests ran on the exact final code, and outputs tamper-evident signed audit bundles.

> **Honest scope:** Kineti checks participating workflows, detects changed inputs and PR-diff tampering, and blocks participating CI commands when evidence or reported spending fails policy. It does **not** provide automatic local interception, kernel sandboxing, protected writes, or provider-bill verification.

---

## Why Teams Use Kineti

When an AI coding agent hits a stubborn test failure on a pull request, standard CI has a blind spot—it executes whatever tests and workflow scripts sit on the PR branch. Standard CI happily turns green if an agent:

1. **Weakens policy or workflows** — edits `.kineti/gate.json`, `.github/workflows/**`, `pytest.ini`, or `jest.config.*`.
2. **Skips or deletes tests** — slips in `@pytest.mark.skip`, `@pytest.mark.xfail`, `it.skip`, `test.only`, `#[ignore]`, `t.Skip`, deletes a test function, or appends `--passWithNoTests`.
3. **Claims tests passed locally before a final edit** — even a 1-byte code edit after tests ran invalidates prior receipts (exit `4`).

Kineti closes that gap on your own Linux CI runner (`permissions: contents: read`) with zero data leaving your repository:

- **Base-branch policy lock:** Reads `.kineti/gate.json` strictly from the protected base commit (`git show <base>:.kineti/gate.json`) and blocks unauthorized edits to gate rules or CI workflows (`policy_tamper_detected`, exit `1`).
- **Polyglot AI test-tampering scanner:** Deterministically scans PR diffs across **Rust, Python, JS/TS, and Go** for deleted required test files, net test deletions, skip/ignore/only/xfail directives, and weakened test-runner flags—returning exact file and line numbers.
- **Freshness & supervised execution:** Runs exact allowlisted commands under a POSIX process-group deadline with streaming `O_NOFOLLOW` workspace hashing (two-tier: Git index + nested `.gitignore` hashing, or pure-Rust bounded fallback when `git` is absent).
- **Signed audit packs:** Emits canonical Ed25519-signed `kineti-pr-audit-v1` JSON bundles and formatted GitHub Step Summary tables for engineering and SOC 2 change-control reviews.

---

## Open-Core (MIT) & Founding Team Pilot

| Tier | License / Price | What You Get |
| --- | --- | --- |
| **Open-Core CLI & Action** | **Free · MIT License** | Full native Rust `kineti` binary, `kineti pr-check`, `action.yml`, polyglot diff scanner, supervised test receipts, `kineti-pr-audit-v1` signing, local MCP server, reported-spend checks, and offline demo. |
| **Founding Team Setup** | **$1,500 one-time** | Hands-on Week 1 rollout across unlimited repositories in your org, base-branch `.kineti/gate.json` + CODEOWNERS policy design, GitHub Actions & audit-pack wiring, and 30 days of direct founder tuning. |
| **Design-Partner Pilot** | **$299 / month per org** | Ongoing org support, rule/parser tuning across unlimited repos during pilot, priority CI hotfixes, and direct roadmap input. Cancel anytime with 30 days' notice. |

See [docs/PILOT-OFFER-AND-OUTREACH.md](docs/PILOT-OFFER-AND-OUTREACH.md) or email [hello@getkineti.com](mailto:hello@getkineti.com?subject=Kineti%20Founding%20Team%20Pilot) for Founding Team Pilot onboarding.

---

## Install & Try It in 15 Seconds

### Option 1 — Build from Source (Works Today)

Requires Rust/Cargo on Linux or macOS (`x64` or `arm64`). Runtime needs neither Node.js nor Bun:

```sh
cargo build --release --locked --manifest-path core-native/Cargo.toml
./core-native/target/release/kineti --version
```

Run the self-contained 4-scene offline demo (zero network access, cleans up automatically):

```sh
./examples/pilot-demo/run-demo.sh
```

### Option 2 — GitHub Actions PR Gate (`action.yml`)

Commit `.kineti/gate.json` on your protected `main` branch, then add Kineti to `.github/workflows/kineti-pr-gate.yml`:

```yaml
name: Kineti PR Gate
on:
  pull_request:
permissions:
  contents: read
jobs:
  kineti:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: therawlogs/kineti@<pinned-commit-sha>
        with:
          base-ref: ${{ github.event.pull_request.base.sha }}
          head-ref: ${{ github.event.pull_request.head.sha }}
          audit-out: .kineti/pr-audit.json
          summary-out: .kineti/pr-summary.md
          write-step-summary: "true"
```

### Option 3 — Cargo / npm (`1.0.1` Pending Registry Publication)

> **Publication status:** `1.0.1` is currently an open-core release candidate. Registry publication to `crates.io` and `npm` requires final multi-target archive acceptance and separate owner approval. Historical `0.6.1` packages are retired (deprecated on npm, yanked on crates.io).

After approved `1.0.1` registry publication:

```sh
# Cargo (requires Rust)
cargo install kineti-cli --version '=1.0.1' --locked

# npm (requires Node.js with install scripts enabled; runtime needs neither Node nor Bun)
npm install -g kineti@1.0.1
```

---

## Quickstart

### 1. Initialize a repository

Preview initialization first (Kineti requires an explicit goal and never overwrites an existing locked goal):

```sh
kineti init --goal "Ship verified pull requests"
kineti init --goal "Ship verified pull requests" --apply
```

Keep `.kineti/` runtime state (`state.json`, `evidence.jsonl`, keys, spend logs) out of Git commits. Commit **only** `.kineti/gate.json` on your protected base branch:

```json
{
  "version": 1,
  "required_checks": ["unit"],
  "commands": {
    "unit": ["pytest", "-q", "tests/"]
  },
  "parsers": {
    "unit": "exit-code"
  },
  "required_tests": ["tests/test_auth.py"],
  "protected_paths": [".kineti/gate.json", ".github/workflows/", "pytest.ini"],
  "max_deleted_tests": 0,
  "max_age_minutes": 240
}
```

### 2. Run supervised tests & check PR tampering

```sh
# Built-in count-aware verification (cargo test and multi-suite bun test)
kineti test run --label rust-unit --required-test tests/ --parser auto -- cargo test

# Allowlisted polyglot verification (pytest, jest, vitest, go test, node --test, make test)
kineti test run --label unit --required-test tests/test_auth.py --parser exit-code -- pytest -q tests/

# Verify freshness, exact argv, required test presence, and Ed25519 signature
kineti test check --label unit

# Enforce all base-branch checks
kineti ci --require-evidence unit

# Scan PR diff against base branch and emit signed kineti-pr-audit-v1 + Markdown summary
kineti pr-check --base-ref origin/main --head-ref HEAD \
  --require-ci --require-evidence unit \
  --audit-out .kineti/pr-audit.json \
  --summary-out .kineti/pr-summary.md
```

### 3. Additional CLI Commands

```sh
kineti status                      # Inspect project goal, evidence, and reported spend
kineti status --project-only       # Inspect project state even if spend records need recovery
kineti spend check                 # Check operator-reported spending against total/stage limits & 95% stop
kineti undo push --label cleanup -- rm generated.tmp
kineti undo rollback               # Preview newest-first undo plan (CLI-only; removed from MCP)
kineti mcp --root .                # Stdio JSON-RPC MCP server (strictly allowlisted commands only)
kineti self-test                   # Disposable self-check covering staleness, signatures & fabricated output
```

---

## Exit Codes

| Exit | Context | Meaning |
| --- | --- | --- |
| `0` | All commands | Check or operation succeeded (fresh evidence, clean PR diff, policy satisfied) |
| `1` | `pr-check`, `ci`, `verify`, `test run`, `undo` | Policy/tampering/check failure (`policy_tamper_detected`, skip/deletion finding, failing test, or missing command mapping) |
| `2` | All commands | Invalid arguments, malformed options, disallowed path, or missing explicit confirmation |
| `3` | `spend`, `undo`, `store`, `receipt` | Spending stop, symlink (`O_NOFOLLOW`) refusal, or damaged/uncertain state or cleanup |
| `4` | `test check` | Stale, expired, or changed workspace inputs (including 1-byte drift after test run) |
| `5` | `test check` | Missing required evidence for the requested label |
| `6` | `test check` | Legacy or unsupported receipt cannot authorize exact-input CI gate |
| `7` | `test check` | Invalid, altered, or untrusted signature or mismatched command `argv` |
| `9` | `verify` | Missing `verify` declaration or absent/stale command trust |

---

## Honest Limits

- **Signatures vs. test quality:** Ed25519 signatures on receipts and `kineti-pr-audit-v1` bundles prove record integrity relative to the signing key—not that assertions are strong (`"independent_assertion_quality_proven": false`).
- **Two-tier test parsing:** `--parser auto` verifies observed pass/fail/skip counts for `cargo test` and multi-suite `bun test` (with optional `min_passed` floors). Allowlisted polyglot runners via `--parser exit-code` verify exit `0`, exact `argv`, required test presence, untruncated capture, and workspace freshness while explicitly recording `"count_observed": false`.
- **Trust boundary:** Real anti-tampering enforcement happens on your protected Linux CI runner reading `.kineti/gate.json` from the protected base branch. Local laptop runs cannot stop a same-account agent with write access from bypassing Kineti.
- **Host integrations & MCP:** `kineti mcp` exposes 6 tools (undo/saga tools are intentionally CLI-only) and strictly enforces `.kineti/gate.json` `commands` allowlists on `kineti_evidence_record`. Historical Codex integration checks covered macOS arm64 Codex `0.163.0-alpha.11.2` and `0.164.0`; **Cursor and Antigravity remain unverified**.
- **No automatic secret redaction in `argv`:** Receipts persist exact command `argv`; pass secrets via CI secrets/environment variables, never on command lines.
- **Reported spending:** `kineti spend` tracks operator-reported integer amounts, not provider billing invoices.

---

## Documentation & History

- Website & full reference: [website/index.html](website/index.html) · [website/docs.html](website/docs.html)
- Offline 4-scene demo: [examples/pilot-demo/README.md](examples/pilot-demo/README.md)
- Founding Team Pilot playbook: [docs/PILOT-OFFER-AND-OUTREACH.md](docs/PILOT-OFFER-AND-OUTREACH.md)
- Static hosting migration (`$0` Cloudflare / GitHub Pages): [docs/HOSTING-MIGRATION-CHECKLIST.md](docs/HOSTING-MIGRATION-CHECKLIST.md)
- Security policy & solo-maintainer response process: [SECURITY.md](SECURITY.md)
- Contributing & DCO guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Release history: [CHANGELOG.md](CHANGELOG.md) · [0.6.0 contract](docs/GATE-0.6.0-CONTRACT.md) · [0.6.0 closeout](docs/GATE-0.6.0-CLOSEOUT.md) · [historical pre-0.6.0 docs](docs/history/pre-0.6.0/README.md)

Licensed under the [MIT License](LICENSE).
