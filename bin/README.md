# bin/ — Harness programs (Phase 1)

Six Bun + TypeScript programs will live here:

| Program | Job |
|---|---|
| kineti-state | Keep the run's position and locked goal |
| kineti-spend | Track tokens and dollars; trip ceilings |
| kineti-saga | Register undo steps; unwind newest-first on failure |
| kineti-evidence | Bind test results to code fingerprints |
| kineti-verify-gate | Block session end while verify fails |
| kineti-egress | Hash-chained log of outbound sends |

Empty in Phase 0 by design: the installer must stay dependency-free.
Bun gets installed at the start of Phase 1.
