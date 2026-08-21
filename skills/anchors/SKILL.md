---
name: anchors
description: Install standing rules into a project root.
stage: meta
version: 0.2.0
triggers:
  - project rules
  - install ethos
  - standing rules
---

# kineti-anchors

Write a short, absolute rules file at a project root so every agent session
inherits the same law. Modeled on the repository's ETHOS.md.

## Procedure

1. Ask which rules the project needs beyond Kineti defaults. Offer:
   - Money ceiling for this project (default from kineti.config.json).
   - Verify command (must match `settings.verify_command`).
   - Freeze rules: directories agents must never touch.
   - Authority limits: what sub-agents may never do (send email, delete
     data, touch production).
2. Write or update `<project>/ETHOS.md` with numbered, plain-worded rules.
   Each rule one sentence. No aspirations — only enforceables.
3. Wire enforcement that exists:
   - verify command → already enforced by kineti-verify-gate.
   - money → kineti-spend reads kineti.config.json; set per-stage limits.
   - freeze + authority → add to the project's agent instructions file and
     to build/review skill invocations.
4. Show the final file to the human for approval.

## Hard rules

- Rules are short, absolute, plain-worded. If a rule needs interpretation,
  it is two rules.
- Never write a rule no program or procedure enforces — mark it as
  convention explicitly instead.

## Memory after

Note the project's anchors in its dossier.
