# Kineti native gate — agent rules

1. Read `.kineti/state.json` before work. Never replace its locked goal.
2. Use plain words and numbered choices. Follow `docs/GATE-0.6.0-CONTRACT.md`.
3. Preview changes and record undo before editing. Preserve user modifications,
   original signatures, historical records, unresolved costs and interrupted work.
4. Keep generated files, tests, configurations and backups inside this project.
5. Use the native executable: `kineti spend check`, `kineti test run/check`,
   `kineti undo push/rollback`, `kineti status`. From source, build with Cargo
   using `core-native/Cargo.toml`; the executable is `kineti`.
6. Never claim tests passed unless freshly run. Save exact commands, results,
   source/artifact digests and environments. Distinguish test trust from signatures.
7. The approved local implementation exception allows existing subscription
   usage only, with $0 new paid spending. Historical monetary unknowns remain
   unresolved. Never bypass integrity failures or unresolved monetary reservations.
   Publication is separately blocked on accounting review and artifact approval.
8. Reported spending uses integer amounts, total/stage limits and the 95% stop.
   Confirmation flags do not prove human identity. Recovery needs an exact
   reviewed proposal and evidence; never reset unknown costs to zero.
9. Check outbound-request destinations before transmitting project information.
   Preserve local request audit records. Do not commit secrets or personal paths.
10. Package publication, merging and deployment require separate approval.
    Native runner feasibility is not installed-package acceptance. Never claim
    automatic interception or protected writes. Cursor/Antigravity are unverified.

The old harness and Bun tooling remain in Git history, not the active product.
Dated release history is preserved. The crate is `kineti-cli` and the npm package
is `kineti`; both ship the native gate. Current code is in
`core-native/crates/kineti-cli/`; runtime state in `.kineti/` is never committed.
