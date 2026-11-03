# Cloud continuation checkpoint

Date: 2026-11-03. The owner requested saving all current source to the existing private checkpoint branch, then retiring local VMs and build resources. This is a workspace handoff, not a completed owner service or a release.

## Source and remaining work

Continue from `codex/owner-service-checkpoint-20261103`. The implementation checkpoint before retirement is commit `75e31015ab9795fe4a86416a4018fffe00b57aa4`. Source, Git history, original signatures and dated reports remain. See `AUTONOMOUS-BUILD-STOP-EXPORT-20261103.md` for exact tested artifacts, successful stop/export components and individual A01–A14 gaps. No acceptance row is promoted by deleting local resources.

The production macOS owner backend remains disabled. It still needs integration with the shared grant/audit/repair/recovery engine, explicit guest/VM/worker grants, SDK execution and MCP callbacks, bounded workspace/home/runtime placement, cancellation/revocation/expiry/crash behavior, runtime fingerprints, limits, delivery and recovery acceptance. Linux's tested stock profile still permits newly created excluded filenames. No custom sandbox fallback is authorized. Cold checkpoints destroy memory-only guest model authentication; unattended authenticated continuation is not established.

## Retired local resources

Both test VMs, `kmac3` and `kineti-autonomy`, are retired under the owner's deletion request. Their OS disks are not retained as full backups. Local Cargo build outputs and dependency caches, downloaded SDK/tool caches, installed test binaries, obsolete VM firmware backups and Python/Node caches are removed. The old global CLI and Cursor integration remain uninstalled. Reprovisioning is not byte-for-byte undo and must not be presented as restoration of the same VM identity or old grants.

Before retirement, private work/audit records were exported and hash-verified: 11,645 Linux files and 1,408 macOS archive members. The macOS record archive preserves bounded task-volume images as work/evidence; these are not bootable OS VM backups. A nested incomplete export was excluded from the canonical archive. Login credentials and downloaded SDK/build caches were excluded from guest record exports. Exact recently tested native artifacts are retained in a compact private archive for historical evidence, not installed or registered as a product.

Local private preservation material is under `.kineti/cloud-retirement-20261103/`, alongside earlier `.kineti` evidence, original records and the exact old-package/configuration undo backups. The retirement preview, guest archive/member hashes, source baseline, native recovery proposal, dependency-source comparison and deletion results identify what was retained and removed. Any locally changed or unverifiable cached dependency source is preserved separately before cache removal. The recovery proposal only restores exported regular records into a new inspection directory; it never recreates a VM, restores credentials, makes programs executable or replays interrupted operations.

## Before using a cloud checkout

1. A Git clone does **not** contain `.kineti` accounting, private evidence, original issuer material or guest record archives. Those remain local; no private runtime data is uploaded by this handoff. Review and approve the exact secure transfer scope/destination separately. Do not upload raw `.kineti` to GitHub or transfer credential/signing material by assumption.
2. Preserve the locked goal, original history/signatures, durable counters and all thirteen unresolved monetary unknowns. Do not initialize a fresh ledger, invent missing values, reset unknown costs to zero, or blindly replay historical undo/recovery/VM commands. Missing private state blocks continuity claims.
3. Verify actual cloud OS/architecture, supported isolation, existing entitlement and no-additional-charge eligibility before running anything. Previous local VM bindings and approval identities are retired. Do not assume GitHub Actions minutes, hosted macOS/x64 capacity, storage, model usage or networking are free. No paid resource is authorized.
4. Rebuild reviewed project-local tools from the exact source and locked dependencies only after resource setup is approved. Preserve request audit records and record fresh source/artifact/environment digests. Prior passing results are historical, not evidence that a new cloud environment passed.
5. Keep fresh provider/admin logins, bounded model turns, credential expansion, new independent review, reinstallation, merging, publication and deployment separately gated. No automatic VM recreation, SDK download, provider login, model run, package install or release is initiated by this document.

New paid-spending authority remains zero. Ordinary accounting/publication gates remain blocked on the thirteen historical monetary unknowns. Automatic deployments remain disabled. No cloud service was provisioned and no private handoff archive was uploaded.
