---
name: security
description: Run the OWASP checklist and threat walk over every boundary. Stage 10 gate.
stage: verify
version: 0.2.0
---

# kineti-security

The security officer pass. Every finding carries a concrete exploit story
and a fix. A critical finding blocks ship.

## Harness

```sh
KIN="$(cat "$HOME/.kineti/repo")"; K="$KIN/bin"
bun "$K/kineti-spend.ts" check || exit 3
```
Log model calls with `--stage security`. Record egress before downloading
any scanner or advisory data.

## Procedure

1. **OWASP walk** over the codebase, one item at a time, with file:line
   evidence for every claim (no generic advice):
   injection · broken auth · sensitive data exposure · broken access
   control · misconfiguration · XSS · insecure deserialization · known
   vulnerable dependencies · insufficient logging.
2. **Threat walk of the architecture drawing**: for each numbered arrow,
   ask who could abuse it — spoofed caller, tampered payload, replayed
   message, over-privileged agent. Each answered threat gets a control or
   an accepted-risk note signed by the human.
3. **Agent-specific checks**: tool allowlists enforced in code (not just
   prompts); untrusted text wrapped as data before reaching models;
   outbound sends logged; authority tiers respected by sub-agents.
4. **Secret scan**: no keys, tokens, or credentials in code, config, or
   history being shipped. `.env` files gitignored and absent from commits.
5. **Dependency audit**: run the project's audit command; every finding
   triaged fix/accept with a reason.
6. **Write `security-report.md`**: findings with severity
   (critical/high/medium/low), exploit story ("as attacker I do X, then Y
   happens"), fix, and status.
7. **Verdict**:
   - No criticals open → `bun "$K/kineti-state.ts" set gate.security pass`
   - Any critical → `gate.security fail`, fix now, re-run this skill.
8. Pass → `bun "$K/kineti-state.ts" set stage 11`.

## Outputs

`security-report.md`, gate.security verdict, fixes committed separately
(`security-fix:` prefix).

## Hard rules

- A critical finding blocks ship. No exceptions, no "temporary" accepts.
- Every finding shows the exploit story; vague findings are not findings.
- Security fixes commit separately from feature work.

## Memory after

Record vulnerability classes found; they become review-stage checklists.
