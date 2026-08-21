# bin/ — Harness programs

All six are implemented and tested (tests/harness.test.ts). Run any of them with `bun bin/<name>.ts`.

| Program | Job |
|---|---|
| kineti-state | Keep the run's position and locked goal |
| kineti-spend | Track tokens and dollars; trip ceilings |
| kineti-saga | Register undo steps; unwind newest-first on failure |
| kineti-evidence | Bind test results to code fingerprints |
| kineti-verify-gate | Block session end while verify fails |
| kineti-egress | Hash-chained log of outbound sends |

