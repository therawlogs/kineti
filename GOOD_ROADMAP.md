# Good Roadmap - Kineti

Date: 2026-09-21.
Version now: v0.3.0, local only, not pushed.
Goal locked: Build universal agent harness with cryptographic verification.
This file holds all agreed plans in one place. No Jev. Kineti native only.

## 1. Product thesis in one line

Kineti does not replace models. It sits around every model call and makes the whole loop faster, cheaper, reliable, and efficient.

1. Faster: skip model calls that are not needed. Reflex triage answers simple chat in microseconds. Snapshots read in nanoseconds under 80 writers. HNSW proposes candidates so search avoids full scans.
2. Cheaper: cut tokens by 38.4 percent with zero-token reactions and a 20-entity kernel. Route each task to the cheapest model that can do it. Real savings run 30 to 40 percent. Hard stop at $50.00, trip at $47.50, per-agent splits under it.
3. Reliable: bind every test run to file state with FRESH or STALE. Block ship on stale. Two keys must sign each result, worker plus reviewer, never the same. Undo stack with hash checks, newest first. 500 attack vectors with 0 escapes.
4. Efficient: graph plus vector search together removes time-order errors of pure vector search. Epistemic tiers stop re-asking settled facts. What you said beats what was guessed. One encrypted snapshot format feeds device sync and cloud mirror.

## 2. What is done - v0.3.0

1. Phase 1 trust base: renamed to `kineti`, write-only audit log at `~/.kineti/audit.log.jsonl` with hash chain, fixed tenant leak so raw search returns global records only, dashboard locked to 127.0.0.1 with token on writes and locked settings keys, install checksum with `KINETI_SHA256`.
2. Phase 2 auto experience: `bin/kineti-router.ts` plain-talk router, no skill names, natural yes and fix, natural ops for spend, undo, proof, status, on and off switch in `.kineti/kineti.json`, `kineti_talk` MCP tool, rewritten tutorials, `GET /api/mini` for menu bar.
3. Phase 3 memory plus safety: new `otd.rs` runtime binder to 20 kernel types with fail-closed checks, new `hnsw.rs` layered graph with deterministic layers, HNSW wired into `vector.rs` with exact re-rank and full-scan fallback, harness 7 to 12 tests plus 5 new checks, forget with proof receipt, risky words warn and ask first.
4. Phase 4 multi-tool plus swarm: 5 new host files, now 10 total, `bin/kineti-models.ts` task table for code, plan, chat, fix with ask-first default, swarm budgets saved from plain words to `.kineti/swarm.json`, mesh add, remove, unblock all audit logged.
5. Phase 5 ship: hardware signer trait plus software fallback in `kineti-harness/src/signer.rs`, encrypted device sync in `bin/kineti-sync.ts` with AES-256-GCM plus scrypt, off by default, import never touches locked goal, deleted duplicate landing page, version 0.2.0 to 0.3.0, tag v0.3.0 local only.
6. Dashboard home tab: Home is default with goal, step, spend, undo count, proof state, on and off, talk box with quick buttons. Connectors face, Vault face, Invite face removed from sight. Endpoints still work and stay tested. Commit `ddcfdb5`, local only.
7. Proof now: 162 bun tests pass, full Rust workspace green, proof `home-tab-full` FRESH, typecheck clean, spend $0 of $50, audit intact, tree clean except this new file.

## 3. Levels - user, device, project, swarm

Use these words everywhere. Do not mix them.

1. L0 User. The human. On cloud they are their GitHub login. Locally they are a display name. One user can own many devices and many projects.
2. L1 Device. One machine. Holds `~/.kineti`: master audit trail, cloud tokens, vault keys. The cloud link binds L0 to L1. Pairing code is made by a device, claimed by a user. One device, one link, revocable alone.
3. L2 Project. One repo folder. Holds `.kineti`: goal, stage, spend, proof, undo, swarm budgets, trust list, invites, pairing intents. Default ceiling $50.00. Everything agents do happens inside exactly one project. Agents never cross projects.
4. L3 Swarm. One run inside one project. Coordinator plus workers plus reviewers. Each agent gets its own Ed25519 key at run start, thrown away at run end. Task envelopes carry the project goal hash so drift is caught. Per-agent budgets come from the project `swarm.json` and can never exceed the project ceiling. A trip at any level stops the run with exit code 3.

Rules:

1. Mirror consent is per project. Linking a device mirrors nothing by itself. Each project asks separately.
2. Trust is person to person at L0, but every shared item is minimum per L2. Knowing a peer never opens all projects.
3. Money flows down only. Project ceiling splits into swarm budgets splits into agent caps. No level can raise a level above it. Only the human raises the project ceiling, audit logged with who, old, new.
4. Audit has one master: the device log. Project proof stays in the project. Cloud keeps ciphertext mirror plus pairing and revoke events plus nothing else.
5. Swarm death is clean. Run ends, keys discarded, tickets and activity rows remain. A dead swarm spends nothing and signs nothing ever again.

## 4. Repo level and project level - how it works

1. Each repo folder with `.kineti/` is one L2 project with full isolation: goal and task state, spend limit, undo history, proof table, swarm budgets, trust list.
2. Fleet grid and repo switcher across projects exist in docs only. Not verified end to end. Single project now, fleet later.
3. Swarm budgets in `.kineti/swarm.json` are per project. Trust list in `trusted_network.json` is per project. Pairing codes are per project unless stated otherwise.
4. Cloud today is zero. No accounts, no servers, no user IDs in code.

## 5. Dashboard - the differentiator

UI is the differentiator. Chat is primary, dashboard is the second screen for 3 jobs chat does badly: proof during incidents, money per agent, undo and danger actions with a clear yes.

1. Home: dense rows for goal with lock time, stage name plus number, spend used of ceiling, proof label plus FRESH or STALE plus age, undo count with latest labels, on and off, sync state. Talk box on top for plain commands. Every value copies in one click.
2. Activity: one view-only table with time, actor, action, detail, hash. Merges audit log, proof records, spend events, saga runs. Filter by actor and action. This closes our own promise of view-only logs.
3. Team and money: per-agent budgets with used and left bars, shared versus separate switch, model auto-switch state with last 10 switches and reasons, sync state.
4. Settings: name, trusted approve and block in one block, training opt-out, forget with proof receipt, sync export and import, delete data, delete account. One danger zone at the bottom.
5. Cut from sight: Connectors section, Vault tab, Invite entry and popup. Behind-screen endpoints still work and stay tested.
6. Preview: `preview/dashboard.html`, static sample data only, yellow banner, not live, untracked. Delete once approved or rejected.

## 6. Cloud link flow - kineti-dashboard

User says `kineti-dashboard` in Codex, Grok, Cursor, anywhere. Router learns it as a dashboard intent like spend or undo.

1. Kineti answers: Do you want a UI cloud link? Choices: 1. Yes, make one. 2. No, local only.
2. Yes: local companion makes a unique pairing code, for example `KIN-7F2A-91QD`. 10 minute life, single use, saved in `.kineti/pairing.json`, audit logged. Reply shows code plus link `app.getkineti.com/pair`.
3. User opens link, logs in with GitHub, types code.
4. Local machine polls cloud with outbound-only requests until cloud says paired. No inbound ports.
5. Cloud gives back tokens. Local stores in `~/.kineti/cloud.json` with owner-only permissions, audit logs pairing. Local pushes encrypted snapshots on a timer. Web at `app.getkineti.com` shows same Home, Activity, Team and money, Settings.
6. Revoke any time by saying revoke cloud link. Local deletes tokens, tells cloud to drop stored data, audit logs both sides.
7. Then per project, first time: mirror this project to cloud? Yes starts ciphertext push for that project only. File contents and vault secrets never leave device, not even encrypted. Journal note text default excluded, included only if note-sync is turned on per project.

Cloud side is new scope outside this repo: GitHub login, pairing verify API, ciphertext store per user, token issue plus refresh plus revoke, abuse limits per code and per IP.

## 7. Market - honest review

1. Coding agents consolidated around 4 giants. Copilot 4.7M paid at 42 percent share. Cursor $2B ARR with 7M users. Claude Code 28 percent share with top satisfaction and $2.5B run rate. Codex open source with 93,000 stars plus managed cloud products coming from OpenAI and Anthropic. Cognition buying Windsurf with talks at $25 to $48B. Three to five winners, rest absorbed.
2. Money moved against buyers. Seats are the floor, usage is the bill. Real spend $100 to $200 per head monthly, heavy users $500 to $2000. Budgets now look like cloud-compute budgets. Our breaker and per-agent budgets speak to this pain.
3. Trust fell while use rose. 84 percent use AI tools, trust in accuracy 40 to 29 percent. 30 percent of solved benchmark patches behave differently from ground truth. Seniors measured 19 percent slower with AI help. Developers spend 11.4 hours a week reviewing versus 9.8 writing. Job flipped from producer to reviewer.
4. Industry answer is the harness layer. Managed agents with worktree isolation, sub-agents, hooks, skills, audit logs. Report line: whoever owns the harness captures most value above the model layer.
5. Personal agents are hype plus red ink. Muse launched with secure VMs and subscriptions. Instinct jumped $50M to $10B talks in 6 weeks with 100,000 free users, no reported revenue, dozens of calls per request, already at capacity.

Where Kineti wins: pre-ship enforcement nobody owns like us, local-first with ciphertext-only cloud, neutral across models in a lock-in war, spend control timed to the billing pain, worktree isolation matching where Claude Code went.

Where Kineti loses: no model, no cloud runtime, no distribution, harness owners are competitors not customers, models still cap us so the messenger gets blamed, one developer against funded armies, no revenue proof yet.

Winnable game: neutral governance layer for teams running many models. Proof they can audit, budgets they can enforce, undo they can trust. Price per verified outcome, never per seat.

## 8. Rivals - LangChain, Jev, others

1. LangChain plus LangGraph plus Deep Agents plus LangSmith: number 1 framework, 100M downloads monthly, 139,000 stars, $1.25B value, 6,000 paying LangSmith customers with 5 of Fortune 10. LangChain builds, LangGraph runs stateful graphs with checkpoints and human-in-the-loop, Deep Agents is their long-running harness, LangSmith observes, evaluates, deploys, auto-suggests fixes. We lead on pre-ship blocking and spend ceilings. They lead on 1,000 integrations, eval scale, and distribution. Their traces live on their servers, which fights our local-first story.
2. Jev by TypeSafe: early access decision model, $40M seed, $0.042 per M input with free output, 70 to 500 ms, on OpenRouter, Cloudflare, Vercel, and since Sept 17 inside LangChain as classifier plus model router plus auto-mode guard. Vendor speed claims have admitted bias, no ground-truth scoring. Zero text errors is real and narrow, a typed wrong answer is still wrong. It cannot remember, undo, prove, budget, sign, or explain. Our call to exclude it holds. Revisit only with customer-scale evidence and strict acceptance numbers.
3. Others short: CrewAI for role teams, Microsoft Agent Framework for Microsoft shops, OpenAI Agents SDK needing Temporal or DBOS for durable runs, Google ADK for GCP teams, LlamaIndex for document work, Mastra as closest TypeScript shape to watch, TrueForge-style runtimes on the same street as our harness with less verification math.
4. Winning shape is wrap, not replace: Kineti guards graph nodes, records trails locally like LangSmith but private, later plugs a validated classifier where our native router sits. Seats for all three are already built.

## 9. On hold - two-pass schema standardizer

User research idea: input in any form goes to a schema checker that drops waste, checker picks best model, second run uses even less because memory is better.

Search result: full loop is not built. Only parts exist.

1. Paper 2 line 108: 30M micro-router in SRAM, 45 to 65 microseconds. Research only, no code.
2. `core-native/crates/kineti-reflex/src/sensor.rs:9,52,121`: finds type and intent in <1ms. Only picks emoji versus model call. No schema cleaning, no model pick.
3. `core-native/crates/kineti-core/src/kernel.rs:276`: `canonicalize_json`. Only for hashing sameness. Not for cutting context.
4. `core-native/crates/kineti-gateway/src/cortex.rs:195,234`: builds prompt from memory. Adds context. No waste drop, no token measure.
5. `core-native/crates/kineti-memory/src/otd.rs`: binds payload to 20 types, fail-closed. Closest to checker. Only for memory payloads, not all input forms.
6. `bin/kineti-models.ts`: picks host by keywords code, plan, chat, fix. No schema input, no learning.
7. Second run with less context: no file measures run 1 versus run 2 tokens. No test.

Held plan, not started:

1. Add input checker that takes text, image ref, audio ref, tool output and returns fixed fields plus drops the rest.
2. Feed cleaned fields to model picker.
3. Save what worked to memory.
4. On next same task, load memory first and send smaller input. Log token count both times as proof.

## 10. Cleanup - files not needed

Safe to remove, all verified:

1. `target/` 446 MB: build cache, gitignored. `cargo clean` frees it. Rebuild takes time, nothing else lost.
2. `node_modules/` 32 MB: gitignored. Reinstall with bun.
3. `.agents/` 3 MB: old agent scratch notes, untracked and gitignored. No code reads it. Delete freely.
4. `preview/dashboard.html` 12 KB: preview only. Delete once approved or rejected.
5. `src/browser/playwright_agent.ts`, `src/documents/pdf_generator.ts`, `src/telephony/voice_call.ts`, `src/multimodal/vision.ts`: only used by extra non-core tests, nothing in `bin` calls them. Delete with their tests.
6. Extra 8 test files: after item 5 loses callers, merge rest into core 10.
7. Docs merges: audit 3 into 2, instruction files 7 into 2, archive zip.
8. `dist/` 4 KB: gitignored stub. Leave it.

Keep: `web/privacy.html` plus `terms.html` for legal, `skills/` 68 KB backend only, `hosts/` all 10, `.kineti/` runtime state never committed.

Removed already: `web/index.html` byte-identical duplicate of audited `public/waitlist.html`. Left small host instruction files alone, each targets a different editor.

## 11. Decisions (locked 2026-09-22)

1. Journal note text: excluded by default, opt in per project. Done in mirror toggle.
2. Vault secrets and file contents: never leave device, even encrypted. No exception.
3. Swarm keys: fresh per run, thrown away at run end. Dead swarm spends and signs nothing.
4. Project ceiling: $50.00 default, set per project at mirror time. Mirror flow asks once.
5. Fleet grid: single project first, fleet later.
6. Pairing code: typed code like `KIN-7F2A-91QD`, 10 minute life kept. No QR.
7. GitHub OAuth scopes: read user email and profile only, nothing on repos.
8. No means no: local dashboard forever, never nudged, user can ask any time.
9. `app.getkineti.com`: owned already.
10. First sales lead: reliability first. Proof you can audit, budgets you can enforce.

## 12. Success checks

1. User never types skill, tool, or approve words.
2. Spend, proof, and undo always run, always logged, cannot be edited, can be seen.
3. No data leak between users. Forget truly deletes with proof.
4. One switch for on and off everywhere.
5. Same screens local and cloud. Same plain-talk router. Same audit trail.
6. Price per verified outcome, never per seat.

## 13. Ship log (0.3.x, 2026-09-23)

Stay on 0.3.x, no 5.0 jump. Homebrew skipped. Hardware enclave bridge open, software fallback ships.
- Version 0.3.1, tag `v0.3.1` local. npm name `kineti` confirmed, publish by human with `npm publish`.
- Pack 336.7 kB, 167 files. `files` allowlist crashes npm 11, so `.npmignore` denylist instead.
- Standalone `bun build --compile` binaries cannot run in this environment (exit 137 even on hello world). Ship runs on bun plus node via committed `bin/kineti.js`.
- Landing 6,702 bytes gzipped (limit 35,000). Attack suite 500 plus 50 vectors, 0 escapes. Proof `ship-0.3.1` FRESH.
