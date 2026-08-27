# WORKFLOWS.md — The Kineti v3 Loop

One loop. Thirteen stages. Three gates. Every stage reads memory first and writes it after.

```
 1 officehours ─► 2 diagnose ─► 3 design ─► 4 architecture
        ▲                                          │
        │                                          ▼
        │                              5 FEASIBILITY GATE ◄─┐
        │                              (fail → back to 2)   │ fail
        │                                          │ pass   │
        │                                          ▼        │
        │                                   6 SPEC [STOP]   │
        │                                   human approval  │
        │                                          │ approved
        │                                          ▼
        └── 13 retro + learn ◄─ 12 watch ◄─ 11 ship ◄─ 10 security ◄─ 9 qa ◄─ 8 review ◄─ 7 build
```

## Stage table

| # | Stage | Produces | Hard rule |
|---|---|---|---|
| 1 | officehours | brief.md + locked goal + UX Blueprint (Persona, Journey, Interaction Matrix, 3-Layer Split) | No feature talk before pain is proven with examples; UX blueprint must precede design |
| 2 | diagnose | diagnostics.md (loss table, bottleneck map, cause chains) | Every dollar figure shows its math |
| 3 | design | style brief, HTML screen variants from UX blueprint, winning mockup, tokens file | No component code before HTML screen variants match UX blueprint |
| 4 | architecture | architecture.md, diagrams, contracts, test matrix | Stack chosen by comparison math, never by default |
| 5 | feasibility gate | feasibility.md, pass/fail per check | Any fail returns to stage 2 with reasons |
| 6 | spec | spec.md | HARD STOP: no code before human approval |
| 7 | build | src/, tests, progress commits | Undo step registered = change allowed |
| 8 | review | findings with file:line | No new features during review |
| 9 | qa | report, screenshots, regression tests | 5 self-repair attempts max, then escalate |
| 10 | security | findings with exploit story + fix | Critical finding blocks ship |
| 11 | ship | clean commits, pull request | Fresh proofs + passed security required |
| 12 | watch | alerts versus baseline | Baseline recorded before deploy |
| 13 | retro + learn | expiring lessons, dossier updates | Lessons feed stages 1-5 automatically |

## Harness calls per stage

| Stage | Programs used |
|---|---|
| 1-6 | kineti-state · kineti-spend · kineti-egress (any web research) |
| 7 | + kineti-saga on every change |
| 8-9 | + kineti-evidence run/check |
| 11 | kineti-evidence check (all groups) · verify-gate final run |
| 13 | memory weekly rules apply |

## Memory rules

1. Expiry states: active → warm → cold → archive. Enforced by the weekly job.
2. Cause links: open vocabulary; core words from the 10XE schema (`caused triggers blocks enables requires supports indicates contributes_to remediates contradicts supersedes resolves duplicates`); status candidate → hypothesis → validated → rejected; validation requires a linked proof record; time order checked weekly.
3. Each project's run-records chain by fingerprint.

## Feasibility checks (stage 5)

- Money: profit per user, margin, return vs hurdle rate, breakeven month where relevant.
- Data: completeness × accuracy × freshness ≥ threshold (default 0.8).
- People: power (0-1) × agreement (-1 to +1); powerful opponents become mitigation tasks.
