# ETHOS.md — Kineti OS v3 Standing Law

Short, absolute rules. Skills propose. Programs enforce. Memory remembers.

## 1. Communication

1.1 Use plain words. Remove jargon and metaphors so anyone can follow without interpretation.
1.2 Present decisions as numbered options.
1.3 Before any generative task, read `WORKFLOWS.md` and the project state. Do not skip gates.

## 2. Locked goal

2.1 The run's goal is written once at stage 1 and can never be edited afterward.
2.2 Any action that cannot be traced to the goal escalates instead of proceeding.

## 3. Money

3.1 Every model and API call is logged with tokens and dollars by the money tracker (`bin/kineti-spend`).
3.2 Crossing a per-stage or global ceiling ($50 default) halts everything immediately.
3.3 Only a human resets a tripped breaker.

## 4. Undo

4.1 Every change to files, databases, or services registers its undo command first (`bin/kineti-saga`).
4.2 On failure, undo steps run newest-first. If one undo fails, log it and continue with the rest.

## 5. Proof

5.1 Test results bind to a fingerprint of the code (`bin/kineti-evidence`). Editing code flips proofs stale.
5.2 Ship refuses STALE or MISSING proofs. Assertions are not evidence.

## 6. Boundaries

6.1 While building, testing, or debugging, edits lock to the active directory (`freeze` rule).
6.2 Sub-agents get the least access their task needs. Escalate beyond it.

## 7. Outbound sends

7.1 Before any data leaves the machine — web research, external APIs — the send is recorded first (`bin/kineti-egress`).
7.2 Send records are hash-chained. Tampering is detectable and reported.

## 8. Clean files

8.1 Committed files contain no personal names, no home paths (use `$HOME`), no real client names, no secrets.
8.2 Verified by search before every commit. Zero matches required.

## 9. Memory

9.1 Every run writes a run-record: goal, stage outcomes, costs, failed approaches.
9.2 Lessons carry expiry dates. Expired lessons stop appearing but are never deleted.
9.3 Cause links stay open-vocabulary, carry proof-backed status, and respect time order.
9.4 Each run-record chains to the previous by fingerprint. History tampering shows.

## 10. Gates

10.1 Feasibility gate (stage 5): any failed check returns the run to diagnosis.
10.2 Spec gate (stage 6): no code exists before explicit human approval.
10.3 Ship gate (stage 11): fresh proofs and a passed security check, both required.
