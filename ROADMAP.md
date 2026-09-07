# ROADMAP.md — What Comes After v3.0.1

Everything below is deliberately not built yet. Each item names what
blocks it and the first concrete step when that blocker clears.

## v3.x — finishing touches

| Item | Blocked on | First step |
|---|---|---|
| Semantic memory search | An embedding API key (none on this machine). Default provider sunsets **2026-09-04**; after that date keyword recall still works, similarity search does not. | Pick a provider, then: `gbrain migrate embeddings --to voyage:voyage-4 --dim 1024 --dry-run` (preview cost) → drop `--dry-run`. Alternative keep-width option: `openai:text-embedding-3-small`. |
| Second-opinion in practice | A second vendor CLI installed and authenticated (`codex`, `gemini`, or `grok`). The skill exists and degrades cleanly today. | Install one CLI, then inside a review: `Load kineti-second-opinion` |
| Gemini MCP live test | Settings wired (`~/.gemini/settings.json` mcpServers → gbrain verbs). Needs one real gemini session to confirm tool discovery. | Open gemini CLI, ask it to `remember` a fact, then `gbrain search` it |

## v4 candidates

| Item | Why | First step |
|---|---|---|
| Eval gate (stage 10.5) | Ship currently proves tests pass; it cannot prove answers are good. Cohen's kappa ≥ 0.75 judge vs human-labeled golden set, blocking PRs like any other gate. | Collect 50 real input/output pairs from first v3 runs into `.kineti/golden.jsonl`; write `bin/kineti-eval.ts` |
| Native memory engine | Full Kineti causal fidelity: numeric evidence scoring on causal edges, loop checks at write time, SQL/PGQ store. gbrain + journal rules approximate this today by design. | Only if journal conventions feel tight: extend `kineti-memory-job` with edge scoring before considering a store swap |
| Team mode | Shared-brain scoping per login, required-vs-optional installs. Solo scope was a v3 decision. | gbrain already ships company-brain mode; start from its OAuth scoping tutorial |
| Token-exact context bill | Current audit counts words. Exact token accounting needs an off-machine count call (egress receipt first) or a local tokenizer. | Port gstack's context-bill approach onto skills/ tree |

## Bigger than Kineti

| Item | Note |
|---|---|
| Kineti Enterprise go-to-market | ICP, pricing, lighthouse customer plan; rewrite thesis-external around quantified buyer pain (friction-cost matrix as opening argument), per the sales manifesto. Kineti v3 is now the working proof artifact to point at. |
| Sales layer as pipeline artifacts | Mutual action plans and stakeholder maps already exist inside feasibility/dossiers; formalize them as named outputs a deal-flow skill maintains. |

## Done in v3 (for context)

Closed loop 13 stages · 3 gates · 7 harness programs · 17 skills ·
2 hosts wired · gbrain memory + TTL/cause-chain/hash-chain rules ·
weekly job scheduled · docs + audits · tag history in `git tag`.
