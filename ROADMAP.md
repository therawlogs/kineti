# Kineti Roadmap - Kineti only, user first

Goal: user talks normal. Kineti does safety, spend, undo, and proof in the background.
User only sees result and advice. One switch for on and off. No skill names. No special commands.

This replaces the old roadmap. No Jev. Kineti native only.

---

## What stays - adds value today

1. `core-native/` 8 crates: core, memory, reflex, connectors, actions, gateway, harness, cli. All keep. Link `kineti-harness` into workspace.
2. `bin/` core 7: state, spend, saga, evidence, verify-gate, companion, mcp. Keep.
3. `src/swarm`, `src/security`, `src/privacy`, `src/design`, `src/components`. Keep.
4. `tests/` core 10: harness, exec-safe, mcp, ci, companion, swarm, ui, apple_design, memory-job, adversarial_prompt_injection. Keep.
5. `docs/` keep 5: README, TUTORIAL-first-run, HOWTO-daily-loop, SECURITY_REPORT, SWARM_COORDINATION_AND_IDENTITY. Keep.
6. `public/waitlist.html` one landing page, the audited artifact. Keep.
7. Completed in v4.0: 360 human model with scope isolation and certainty tiers, Safe Rust core, snapshots under 0.1 ms with 80 writers, 20-entity graph, spend stop at 50.00 with trip at 47.50, reflex triage under 1.0 ms, dual-signed tickets, WhatsApp and iMessage gateway, 500 attack vectors with 0 escapes, 83 Rust tests plus 82 TypeScript tests.

## What to clear up - low value or duplicate

1. `docs/archive/` 2 old files - keep 1 zip only, out of main docs.
2. `docs/AUDIT_REPORT.md` plus `ARCHITECTURE_AUDIT_AND_BENCHMARK_REPORT.md` plus `CANONICAL_ARCHITECTURE_PLAN.md` - merge to 1 `AUDIT.md` plus 1 `PLAN.md`.
3. `public/waitlist.html` is the audited artifact referenced by audit evidence and the payload gate. `web/index.html` was a byte-identical duplicate, removed. `web/privacy.html` and `web/terms.html` stay as legal pages.
4. `CLAUDE.md`, `CODEX.md`, `ETHOS.md`, `MEMORY.md`, `MIGRATION.md`, `WORKFLOWS.md`, `PROJECT.md` - merge to 1 `AGENTS.md` plus `README.md`.
5. `bin/` extra 8: stripe, invite, schedule, epistemic, privacy, egress, swarm, memory-job - keep code, backend only. Remove from user docs.
6. `skills/` 17 folders - backend only. No trigger words for user.
7. `src/browser`, `src/growth`, `src/telephony`, `src/multimodal`, `src/documents`, `src/scheduler` - freeze unless used. Remove from default build if unused.
8. `tests/` extra 8: autonomous_capabilities, real_capabilities, autonomous_vault, saas_connectors, stripe_autonomous_payments, omnichannel_ingestion, frontier_benchmarks_and_metrics, plus setup overlap - merge into core 10.
9. Rename all `kineti-local-harness` to `kineti`. Full name `Kineti OS` only in docs title.

---

## Phase 1 - Trust base - v4.0.1

1. Rename to `kineti` in `.kineti/state.json`, `docs/MULTI_REPO_FLEET_AND_INTEGRATIONS.md`, `bin/kineti-companion.ts`.
2. Add write-only audit log at `~/.kineti/audit.log.jsonl` with hash chain. Log price change with who, when, old value, new value, reason. Log tamper tries. Read-only for user, view-only in dashboard.
3. Fix leak: add `user_id` filter to `VectorIndex::search` in `core-native/crates/kineti-memory/src/vector.rs`.
4. Lock dashboard to 127.0.0.1 only, token on writes, lock settings keys, add install checksum.
5. Done when: name is clean, audit log shows test entry, leak test passes, security check passes.

## Phase 2 - Auto experience - v4.1

1. Hide skills. Add intent router, Kineti native. User talks normal like Muse or Instinct app. System picks step.
2. Natural yes: yes, ok, go ahead, do it, looks good counts as yes. Change X counts as fix request.
3. Natural ops: how much spent, undo that, did tests pass, Kineti on, Kineti off. No commands to learn.
4. All checks default on: spend, tests, safety, undo save. User sees only outcome plus 1 to 3 numbered choices.
5. Menu bar helper: spend, goal, undo button, on and off switch.
6. Rewrite `docs/TUTORIAL-first-run.md` plus `docs/HOWTO-daily-loop.md` to plain talk only.
7. Done when: user can finish a task without typing any skill or command name.

## Phase 3 - Memory plus safety fix - v4.2

1. Build missing `src/otd.rs` runtime OTD engine with JMESPath bindings from Paper 3.
2. RoaringBitmap tombstones in `tombstone.rs`, HNSW index in `vector.rs`, fusion score mixing meaning plus graph distance.
3. Link `kineti-harness` in `core-native/Cargo.toml`, add native Ed25519 tickets, native cause graph plus trust score, git worktree shadow folder.
4. Keep ask before send, pay, delete. 10 minute expiry. Low confidence or high risk always asks human.
5. Privacy: forget X deletes everywhere plus proof line, expiry times, purge log, opt-out of training, external delete for Gmail and Notion.
6. Memory tests 4 to 18, harness tests 6 to 12. Fresh proof in `.kineti/evidence.jsonl`.
7. Done when: 83 Rust plus 82 TypeScript still pass plus new tests pass, proof shows FRESH.

## Phase 4 - Multi-tool plus swarm - v4.3

1. Add hosts: Antigravity, Fx.sh, Lovable, Replit, Emergent. Keep first 5: opencode, cursor, claude, codex, gemini. Config only.
2. Native model table by task: code, plan, chat, fix. Ask-first default. Auto only if user turns auto on. Log every switch with reason.
3. Swarm: if more than 1 agent seen, ask in plain words: I see a team here, do you want separate budgets. Save per-agent budgets with audit.
4. Peer mesh: minimum share only, ask-first before sharing.
5. Done when: switch works by plain words, swarm ask shows, audit has entries.

## Phase 5 - Private sync plus ship - v4.4 to v5.0

1. v4.3 hardware signing hidden: user sees confirm $X only. Keys in Secure Enclave or TPM.
2. v4.4 encrypted sync across devices with on and off switch.
3. Delete duplicates listed above in clear-up section.
4. Ship: Homebrew plus npm plus 3 binaries, landing payload under 35 KB, 0 attack escapes in 500.
5. Done when: clean repo, fresh proof, version 5.0 tagged.

---

## Success checks

1. User never types skill, tool, or approve words.
2. Spend, proof, and undo always run, always logged, cannot be edited, can be seen.
3. No data leak between users. Forget truly deletes.
4. One switch for on and off everywhere.
