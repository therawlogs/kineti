# Kineti

**Agent = Model + Harness. Kineti is the harness.**

A single Rust binary (<3 MB) that turns any LLM into a governed agent:
context integrity enforced mechanically, every action chained cryptographically,
money capped by a real circuit breaker, and shipping refused unless proofs are fresh.

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
| No runaway spend | Circuit breaker halts mid-run; only a human flag file resumes | ASI-04 |
| No unapproved code | Stages 1–6 get read-only tools; SPEC gate requires literal human `y` | alignment-as-architecture |
| No silent history edits | Every record sha256-chained; one flipped byte fails verification | audit integrity |
| No injection execution | Tool output wrapped as DATA; instruction-shaped lines quarantined | ASI-01 |
| No unverified results | Schema/range validation between observation and reasoning | ASI-02 |
| No untraceable actions | Immutable root goal pinned into every envelope + record | goal drift |
| No stale ships | Test proofs bind to code fingerprints; STALE blocks ship | drift |
| No unlogged egress | Outbound sends recorded before they leave, keys redacted | ASI-05 |

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/iwpraveen/kineti/main/install.sh | sh
```

or from source:

```sh
cargo install --git https://github.com/iwpraveen/kineti
```

Requires an API key for one of the providers (`GEMINI_API_KEY` or
`XAI_API_KEY`, env vars only — never written to disk).

## Quickstart

```sh
kineti init                                   # scaffold .kineti/ + editable kineti.toml
kineti provider-test --provider gemini        # smoke-test the wire
kineti task --task "create hello.txt"         # one freeform governed task
kineti run --goal "build X"                   # full 13-stage pipeline with gates
kineti receipt                                # hash-chained run summary
```

## The pipeline

Thirteen stages, three hard gates. Feasibility failure loops back to
diagnosis automatically. The spec stage physically stops until a human types
`y`. Ship refuses without fresh proofs and a passed security checklist.

```
1 officehours ─▶ 2 diagnose ─▶ 3 design ─▶ 4 architecture ─▶ 5 FEASIBILITY GATE
     ▲                                                            │ fail → back to 2
     │                                                            ▼ pass
     └── 13 retro ◀─ 12 watch ◀─ 11 SHIP GATE ◀─ 10 security ◀─ 9 qa ◀─ 8 review ◀─ 7 build ─▶ 6 SPEC STOP (human)
```

## Memory that can't lie

`.kineti/journal.jsonl` records every action and observation as kernel-typed
records linked by causal edges (`caused · triggers · blocks · …`). Each edge
passes cycle detection and time-order validation before commit. Records age
active → warm → cold → archive; nothing is ever deleted.

## Configuration

Model names, prices, spend caps, context budget and verify commands live in
`kineti.toml` — data, never compiled in. Providers speak the OpenAI wire
format; Gemini and Grok work through one client today.

## Demo

<!-- demo gif placeholder -->

See [docs/DEMO.md](docs/DEMO.md) for the 90-second scripted walkthrough.

## Status

v0.1.0 — working core: ReAct loop, fenced tools, quarantine, journal chain,
DAG verifier, pre-context filter, validation layer, signal escalation,
authority tiers, saga undo, evidence proofs, egress firewall, spend breaker,
13-stage machine, receipts.

Roadmap v0.2: embedding-based semantic dedup, vector recall over the causal
graph, process reward models on step verification, true dual-LLM isolation,
speculative tool execution against shadow state.

## License

MIT — see [LICENSE](LICENSE).
