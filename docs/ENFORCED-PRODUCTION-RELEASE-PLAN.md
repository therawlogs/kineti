# Enforced production release path: implementation plan

Date: 2026-10-25. Status: proposed sequence; no implementation authorized.

Read with the [design-review packet](ENFORCED-PRODUCTION-RELEASE-DESIGN.md) and
[governing contract](ENFORCED-PRODUCTION-RELEASE-CONTRACT.md). All P01–P27 start
not run for this milestone. Existing 0.6.0 evidence is regression guidance only.

## 1. Finish design review before implementation

Resolve the packet's named-role, credential inventory, complete accounting
checkpoint, provider feasibility, immutable tool pins and $0 eligibility blockers.
Recheck live settings with allowlisted read-only requests and local outbound audit;
never copy credentials into evidence. Inventory API destinations before requests:
GitHub API for therawlogs/kineti and the approved controller repository; official
registry endpoints for exact approved packages; official provider documentation.
Redirects and upload hosts require review before use. No public upload is implied.

Deliver exact schema/approval binding, protected-policy topology, tool versions,
settings diffs and resource confirmation. Owner approves the exact local work and
new implementation-only subscription exception with all 13 unknowns preserved.
Do not begin implementation because this plan was merely prepared or discussed.

## 2. Establish controlled definitions and isolated development

After exact approval, preview file additions and save undo/backups before edits.
Proposed additions: internal `release-control/` schemas, validator, transport
adapters, audit state machine and local adversarial fixtures. No public CLI/MCP or
receipt format change; propose an amendment first if one becomes necessary.
Retain native gate/accounting/undo behavior and frozen 0.6.0 workflows/contracts.

An authorized operator establishes the separately controlled policy repository
and restricted coding identity. The independent policy owner reviews assertions
and pins, command maps and protection policy. Settings and credential changes
require their own exact approval, even if implementation has been authorized.
Local prototypes can be reviewed before activation, but cannot certify independence.

## 3. Implement and test local fail-closed control

Build strict manifest parsing and identity/freshness validation, immutable staging,
settings/approval revalidation, an authenticated approval adapter and digest binding.
Then implement append-only durable audit, package/version locking and explicit
partial/uncertain reconciliation. Use a synthetic sink with a write counter and
credential-release counter. Every denial must leave both at zero. Inject failures
before and after every persisted intent, token exchange, upload and outcome write.
Test duplicate keys, unsafe archives, altered hashes, exact empty argv, replay,
future/stale evidence, wrong issuer and all non-success job conclusions.

Keep synthetic ledgers physically separate and labeled. Demonstrate scoped
exception matching without changing native failures or real history. Never import
synthetic approvals into real promotion state. Prove exact-byte archive publishing
using reviewed adapters without privileges first. No source build/package hooks in
the privileged context. Unit/mock success alone cannot pass host/provider rows.

## 4. Owner-approved isolated canary and operator configuration

Present exact account/repository/namespace/version/actions/destinations and $0 bound
for separate canary approval. Recheck automatic deployment integration before any
branch push; use only an explicitly approved isolated branch/environment when side
effects cannot be excluded. No tests write real main, 0.6.0, website or production
registry packages. Operator applies only separately approved protection/trust changes.

Verify effective rules and issuer restrictions, identity isolation with harmless
canaries, provider token-exchange denial, independent review and clean sink promotion.
Exercise publisher revocation and exact partial-state recovery. Prefer safe
non-mutating provider authorization checks; when a real canary write is necessary,
obtain its exact separate approval first. Unavailable provider evidence leaves a row
blocked. Never substitute local token absence or CLI exit for provider enforcement.

## 5. Native regression and candidate acceptance

Choose exact future version only with owner review; preflight both registries and
tag, never auto-bump. Freeze source/tree, schemas, tools and artifacts. Reuse and
adapt reviewed assertions in new next-release workflows. Do not rewrite historical
published-060 verification. Run Cargo fmt, Clippy and retained suites in UTC and
Asia/Kolkata, using native `kineti test run/check` receipts with exact argv and
required-test declarations in isolated fixtures. Bind executable and policy digests
independently; receipts alone are insufficient.

Run actual Linux musl x64/arm64 and macOS x64/arm64 jobs. Install both routes outside
the checkout on all four targets, run self-test and actual MCP discovery/call,
verify disabled-script/unsupported-platform errors, Node/Bun absence in runtime
PATH and footprints below 10,000,000 bytes. Save download/unpacked sizes separately.
Use the packet's bounded batch only after verified quota/storage eligibility.
Save exact command arrays, times, runner identities, source/artifact/tool digests,
assertion inventory and real results. Changed inputs invalidate affected evidence.

## 6. Independent acceptance and production approvals

An independently authenticated person/service outside coding authority reviews
P01–P26 results, incident drill, effective settings and remaining blockers.
No same-session helper is independent review. Any missing required evidence blocks.
Present a redacted public audit plus private immutable evidence, exact version,
archive hashes/sizes, settings/controller/policy digests, run/attempt/jobs and
approval expiry. Resolve real accounting through separate reviewed recovery or
obtain a new exact artifact-scoped accounting exception. Never reuse 0.6.0 approval.

Obtain separate permission for live production protections/registry trust and then
exact release artifacts/version/accounting approval. Evidence is at most 240 minutes
old; approval expires at the earlier evidence expiry or 24-hour approval limit.
No approval bypasses corruption, uncertain accounting writes or reservations.

## 7. Separately authorized publication and completion

Revalidate settings, authorizations, accounting, archive hashes and version/tag
availability. Acquire durable package/version lock, persist intent before the first
write. Publish Cargo exact bytes and verify registry download; then npm exact bytes
and verify; then approved tag/GitHub release and approved assets. Store each outcome
before the next write. On ambiguity stop in uncertain; on partial publication record
exact completed/untouched services and stop. Reconcile before any retry; changed or
expired authorization needs review. Never auto-unpublish, overwrite or change version.

After publication, repeat registry-downloaded installations/self-test/MCP/runtime
PATH/footprint checks on all four actual targets. P27 and retained audit are required
before completion. A canary-only pass is candidate readiness, not production proof.

## 8. P01–P27 traceability and current status

Phase numbers identify where the final demonstration belongs. Local fixtures come
first for rows marked local + external; all still require the contract's real
boundary evidence. Every case records actors, timestamp, exact commands, run/attempt,
policy/source/manifest/archive digests, result, authority/sink counters and redacted
audit references. Do not infer denial from missing access or a failed CLI alone.

| ID | Scenario | Final phase | Execution scope | Status |
| --- | --- | --- | --- | --- |
| P01 | Agent directly pushes main, edits rules/owners or self-approves policy | 4 | protected host/provider | Not run |
| P02 | Agent invokes Cargo/npm/GitHub publication outside the approved controller | 4 | protected host/provider | Not run |
| P03 | Agent reads owner home/config/keychain, CI secrets or signer material | 4 | protected host/provider | Not run |
| P04 | Delete/rename/skip gate, forge a same-name success, submit neutral/skipped/cancelled/old-run status | 3 | local + external | Not run |
| P05 | PR modifies workflow, trusted tests, command map or verifier | 4 | protected host/provider | Not run |
| P06 | Replace gate binary or issuer key; re-sign a receipt locally | 3 | local + external | Not run |
| P07 | Replace tests with `printf` passing output or an exit-zero stub | 3 | local + external | Not run |
| P08 | Delete tests, run no assertions, use skipped/truncated/failed test output | 3 | local + external | Not run |
| P09 | Replay receipt from another commit/repository/session/run attempt or change argv including empty arguments | 3 | local + external | Not run |
| P10 | Change input/dependency/environment/runtime after checking; expire evidence or use future timestamps | 3 | local + external | Not run |
| P11 | Swap archive, rebuild after approval, mutate staged bytes or replace a same-name artifact | 3 | local + external | Not run |
| P12 | Poison cache/artifact, inject path traversal/symlink or run package hooks in publisher | 3 | local + external | Not run |
| P13 | Use wrong OIDC audience/repository/workflow/ref/environment/package | 4 | protected host/provider | Not run |
| P14 | Use fork PR or privileged trigger to gain secrets/OIDC/release rights | 4 | protected host/provider | Not run |
| P15 | Change effective protections, revoke reviewer/publisher or use expired approval after approval | 3 | local + external | Not run |
| P16 | Use unknown costs, unresolved monetary reservations, corrupt/missing history, interrupted write/recovery or manipulated snapshot | 3 | local + external | Not run |
| P17 | Concurrent accounting updates, stage/total 95% stops and reset without headroom | 3 | local + external | Not run |
| P18 | Simulate approved exception versus omitted, stale or wrong-artifact exception | 3 | local + external | Not run |
| P19 | Race two promotions, cancel a publishing job, timeout or lose network after an upload | 3 | local + external | Not run |
| P20 | Fail Cargo, npm, tag/release or audit persistence at each boundary | 3 | local + external | Not run |
| P21 | Prioritize wrong/occupied version or moved tag; use artifact signed by a valid but disallowed workflow | 3 | local + external | Not run |
| P22 | Clean approved canary promotion through the protected boundary | 4 | protected host/provider | Not run |
| P23 | Four native target Cargo/npm candidate installation, self-test, MCP and footprint acceptance | 5 | native runners | Not run |
| P24 | Retained native/legacy regression suites under UTC and Asia/Kolkata | 5 | native runners | Not run |
| P25 | GitHub/registry API outage, exhausted included quota or possible additional charge | 3 | local + external | Not run |
| P26 | Incident drill: revoke publisher, pause promotion and resume a known partial state | 4 | protected host/provider | Not run |
| P27 | Separately authorized real publication and four-platform published-package check | 7 | native runners | Not run |

Exact acceptance criteria remain in the contract; this table does not relax them.
P16/P18 must retain actual native accounting denials and distinguish approved
exceptions from unresolved reservations/integrity. P19 must cover manual cancellation
and runner death, not only two competing workflow dispatches. P22 must show approved
bytes at the canary sink and zero writes/credential release for denied cases. P27
requires separate real-publication approval and actual registry installations.

## 9. Evidence retention, undo and stop conditions

Retain append-only private local evidence and source/artifact digests inside this
project. Export remote artifacts before their one-day expiry and verify hashes.
No arbitrary deletion to meet quota; stop when retention cannot fit verified $0
resources. Redact secrets, personal paths, private account and ledger data from any
public packet and independently review it before publication.

Documentation undo is registered as run `production-release-design-20261025`;
preview with native `kineti undo rollback --run-id production-release-design-20261025`.
Its inverse removes only these two new documents after checking recorded hashes.
It preserves evidence and undo history. Do not run rollback after user edits without
inspection. Future code/settings changes need their own scoped undo plans. Registry
uploads cannot be undone by this documentation action; incident handling preserves
partial outcomes and never silently unpublishes.

Stop on unknown provider enforcement, absent independent roles, unresolved inventory,
integrity failure, uncertain accounting writes, monetary reservations, possible
additional charge, quota exhaustion, required feature unavailability, missing audit,
expired approval or uncontrolled deployment side effects. Report exact remaining
work; do not label prepared or implemented work accepted without its evidence.
