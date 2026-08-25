# Kineti

**Agent = Model + Harness. Kineti is the harness.**

A single Rust binary (~3 MB) that turns any LLM into a governed agent — or a
governed swarm of them: context integrity enforced mechanically, every action
chained cryptographically, money capped by real circuit breakers, parallel
workers isolated in their own trees, and shipping refused unless proofs are
fresh.

Skills propose. Programs enforce. Memory remembers.

```text
$ kineti run --goal "Create greeting.sh that prints KINETI SHIPS"
╔══════════════════════════════════════╗
║ STAGE  6/13 — spec                   ║
╚══════════════════════════════════════╝
═══ SPEC HARD STOP ═══  … awaiting human approval [y/N]
── iteration 12 ── ⚙ bash → ./greeting.sh
✔ proofs FRESH (./greeting.sh)
🚢 SHIPPED.
```

## Why

Prompt-based guardrails fail in production because prompts don't survive
translation and have no enforcement mechanism. Kineti moves governance out of
the prompt and into the harness control flow:

| Guarantee | Mechanism | Threat model |
|---|---|---|
| No runaway spend | Reserve/settle ledger with per-stage + per-worker ceilings; only a human flag file resumes | ASI-04 |
| No unapproved code | Stages 1–6 get read-only tools; SPEC gate requires literal human `y` | alignment-as-architecture |
| No silent history edits | Every record sha256-chained into a branch-and-merge DAG; one flipped byte fails verification — per branch, by name | audit integrity |
| No injection execution | Tool output wrapped as DATA; instruction-shaped lines quarantined | ASI-01 |
| No unverified results | Schema/range validation between observation and reasoning | ASI-02 |
| No untraceable actions | Immutable root goal pinned into every envelope + record | goal drift |
| No stale ships | Test proofs bind to code fingerprints; STALE blocks ship | drift |
| No unlogged egress | Outbound sends recorded before they leave, keys redacted | ASI-05 |
| No partial history | Worker journals fold into the main chain via deterministic two-parent merge records; orphan branches block ship | audit integrity |
| No rogue parallel writes | Every swarm worker lives in its own git worktree (or scratchpad); the fence binds to it | race conditions |

## Install

```sh
curl -fsSL https://getkineti.com/install.sh | sh
```

The script verifies downloads against the release's `SHA256SUMS`, never asks
for sudo, and installs to `/usr/local/bin` when writable, else
`~/.local/bin` (override with `$KINETI_INSTALL_DIR`). Linux builds prefer the
musl-static variants.

or from source:

```sh
cargo install --git https://github.com/therawlogs/kineti
```

Credentials: an API key per provider (`GEMINI_API_KEY`, `XAI_API_KEY`) **or**
an OAuth token via `kineti login` (see [Auth](#auth)). Keys live in env vars;
tokens in `~/.kineti/auth/` with mode 0600. Nothing secret is ever logged.

## Quickstart

```sh
kineti init                                   # scaffold .kineti/ + editable kineti.toml
kineti provider-test --provider gemini        # smoke-test the wire
kineti task --task "create hello.txt"         # one freeform governed task
kineti run --goal "build X"                   # full 13-stage pipeline with gates
kineti receipt                                # hash-chained run summary
kineti verify --all                           # full DAG history check
```

## The pipeline

Thirteen stages, three hard gates. Feasibility failure loops back to
diagnosis automatically. The spec stage physically stops until a human types
`y`. Ship refuses without fresh proofs, a passed security checklist, clean
journal history, and a clean-files scan.

```
1 officehours ─▶ 2 diagnose ─▶ 3 design ─▶ 4 architecture ─▶ 5 FEASIBILITY GATE
     ▲                                                            │ fail → back to 2
     │                                                            ▼ pass
     └── 13 retro ◀─ 12 watch ◀─ 11 SHIP GATE ◀─ 10 security ◀─ 9 qa ◀─ 8 review ◀─ 7 build ─▶ 6 SPEC STOP (human)
```

## Single vs Swarm

One flag switches topology:

```sh
# linear pipeline (default):
kineti run --goal "Refactor database queries" --mode single

# parallel feature workers after ONE spec approval:
kineti run --goal "Build auth, billing, and an admin dashboard" \
    --mode swarm --cap 100
```

In swarm mode the coordinator runs stages 1–6 and its architecture document
must declare a mechanical **Task Partition** — one line per worker with
scope globs and dependencies:

```markdown
## Task Partition
- T1: database layer | scope: src/db/** ; migrations/** | deps: -
- T2: api routes     | scope: src/api/**             | deps: T1
```

Overlapping scopes or cyclic dependencies bounce back to stage 4 before any
code is written. The single `y` at the spec stop approves the whole split.

Workers then execute build → review → qa inside isolated worktrees, in
dependency waves up to `[execution].max_parallel_workers`. Integration is a
strict ladder (§R2): sequential merges → if conflicts, **exactly one**
arbitrator attempt → otherwise halt and hand the diff to a human. The merged
tree needs fresh proofs before stages 10–13 run.

## Daemon

`kinetid` (`kineti serve`) owns the authoritative spend pool and cached
journal heads on `.kineti/kineti.sock` (mode 0600): appends become O(1),
concurrent CLI processes share one governed ledger, and warm round-trips are
tens of microseconds. Every command silently falls back to direct execution
when no daemon answers — same code behind both transports, proven identical
by CI. Force behavior with `KINETI_FORCE_DIRECT=1` / `KINETI_NO_DAEMON=1`.
Full chain verification stays out of the hot path and remains mandatory at
the ship gate and `kineti verify`.

## Memory that can't lie

`.kineti/journal.jsonl` records every action and observation as kernel-typed
records linked by causal edges (`caused · triggers · blocks · …`). Each edge
passes cycle detection and time-order validation before commit. Records age
active → warm → cold → archive; nothing is ever deleted.

Swarm workers append to their own chains (`journal.w-<task>.jsonl`); after a
verified integration those branches are folded into the main chain through
deterministic two-parent merge records. `kineti verify --all` checks every
branch, flags tamper by name, and refuses to ship with orphaned or extended-
after-the-fact history.

## Money

Spending is a reservation protocol against an atomic micro-dollar pool:
every model call reserves an estimate first and settles actuals after, so
parallel workers can never collectively overshoot. Ceilings: global
(`limits.global_usd`), per-stage (`limits.per_stage_usd`, enforced — set 0 to
disable), per-worker (`limits.max_worker_usd`). Crossing any settled ceiling
halts everything immediately; only `.kineti/spend.reset`, created by a human,
resumes spending. Standalone concurrent runs serialize on an advisory ledger
lock and fail closed rather than race it — run the daemon when you need true
concurrency.

## Auth

Providers accept a stored OAuth token or the classic env key; a valid token
wins, expired tokens attempt one refresh, then fall back to the env key.

```sh
kineti login --provider grok     # PKCE flow, opens browser, waits on loopback
kineti auth-status               # expiry state of stored tokens
kineti logout --provider grok
```

Enable per provider in `kineti.toml`:

```toml
[providers.grok.auth]
client_id     = "kineti-cli"
authorize_url = "https://idp.example.com/authorize"
token_url     = "https://idp.example.com/token"
scopes        = "openid profile"
```

Tokens persist at `~/.kineti/auth/<provider>.json` (mode 0600). RFC 7636
S256 end-to-end, state validated against CSRF.

## Configuration

Model names, prices, spend caps, execution topology, auth endpoints and
verify commands live in `kineti.toml` — data, never compiled in. Providers
speak the OpenAI wire format; Gemini and Grok work through one client today.

```toml
[execution]
mode = "single"              # "single" | "swarm"
max_parallel_workers = 4
worker_isolation = "auto"    # auto | git | scratchpad

[limits]
global_usd = 50.0
per_stage_usd = 10.0         # enforced; 0 disables
max_worker_usd = 0           # swarm-only ceiling; 0 disables
context_char_budget = 24000
verify_command = ""          # proof the ship gate binds to
```

## C API

`libkineti` (built alongside every release) exposes three calls for
embedding in C, Python (ctypes/cffi), Rust-external hosts:

```c
#include "kineti.h"   /* grab kineti.h from the Release assets */

KinetiResult r = kineti_run("{\"goal\":\"create hello.txt\","
                             "\"provider\":\"gemini\"}");
/* r.ok ? JSON payload : error text — free with kineti_free_string(&r.payload) */
KinetiResult v = kineti_verify("{\"all\":true}");
KinetiResult q = kineti_receipt(NULL);
kineti_free_string(r.payload); kineti_free_string(v.payload);
kineti_free_string(q.payload);
```

Panics never cross the boundary; failures arrive as `ok=false`. The calling
process must have the project directory as its working directory.

## Demo

<!-- demo gif placeholder -->

See [docs/DEMO.md](docs/DEMO.md) for the 90-second scripted walkthrough and
[docs/RELEASE.md](docs/RELEASE.md) for the release checklist.

## Command reference

| Command | Purpose |
|---|---|
| `init` | scaffold `.kineti/` + editable `kineti.toml` |
| `run --goal … [--mode …] [--cap …]` | governed 13-stage pipeline (single or swarm) |
| `resume` | continue from the last saved stage |
| `task --task …` | one freeform governed task |
| `undo` | saga rollback, newest-first |
| `evidence --cmd …` | bind a proof to the current code fingerprint |
| `ship-check` | run the ship proof gate alone |
| `receipt` | unified summary: spend across workers, gates timeline, DAG, egress, clean-files |
| `verify [--all]` | chain check / full DAG check incl. orphan closure |
| `merge --branch <id>` | fold a worker journal into the main chain |
| `clean-check` | scan project files for names/home-paths/secrets (§8.2) |
| `login/logout/auth-status` | PKCE OAuth lifecycle |
| `serve` | run the governance daemon |
| `provider-test` | smoke-test a provider's wire |

## Status

v0.1 — shipped and test-pinned (109 tests): ReAct loop, fenced tools,
quarantine, pre-context filter, validation layer, signal escalation,
authority tiers, saga undo, evidence fingerprints, egress firewall,
13-stage machine with all gates, UDS daemon with shared reserve/settle
ledger and per-stage ceilings, branch-and-merge DAG journals, worktree
isolation, swarm orchestrator with arbitrator ladder, unified receipt,
PKCE OAuth, and a C-ABI embedding surface.

Deferred deliberately (tracked for later): syscall sandboxing for spawned
processes, WASI compilation, semantic/vector recall over the causal graph.

## License

Apache-2.0 — see [LICENSE](LICENSE).
