# Autonomous build delivery reset

Date: 2026-11-03. Execution approach changed at the owner's request. This does not amend the contract, remove acceptance requirements, or authorize credentials, model turns, spending or release actions.

The previous approach accumulated successful boundary primitives and disposable fixtures without completing the macOS task service. That is the central implementation gap. `task.rs` still enables owner approval, serving and model serving only on Linux; `task_grant.rs` pins the Linux 0.166.0 host. A macOS authenticated channel and stock sandbox are components, not a completed adapter.

## Fixed implementation path

1. Reuse the existing grant, durable counters, repair and recovery logic. Separate its platform operations into explicit backend interfaces. Preserve the Linux behavior with regression evidence before connecting macOS. Do not create another independent repair-loop implementation inside a fixture.
2. Implement the macOS backend as one installed, root-controlled guest service and one host controller retaining the verified whole-VM lease. Stop rebuilding VM provisioning for each test. Keep owner approvals and agent requests separate. Bind the actual macOS SDK version, protected configuration, dedicated task identity, bounded writable storage and lifecycle before enabling execution. Unsupported controls must fail closed; no custom sandbox fallback.
3. Use one offline acceptance entry point and one deterministic project. It must exercise grant validation, an edit, a failing check, repair, independent owner retest, verified diff, active revocation, cold restart and conflict-safe recovery. Test the product entry points. Existing fixture successes remain component evidence until that entry point uses the production service.
4. Fix failures in that implementation and rerun the same acceptance entry point. After two unsuccessful fixes for the same blocker, stop that investigation, record the exact failing boundary and supported alternatives, and make a feasibility decision. Do not replace a failed case with a narrower success or start another unbounded sequence of VM probes.
5. Once that flow passes, run the remaining contract matrix against the same candidate: escape routes, stale evidence, exhaustion, interruption and concurrency. Then prepare one exact candidate/digest/evidence packet for independent review, gated real-model acceptance and native website-package acceptance. Preserve all required targets and acceptance rows.

The next implementation deliverable is the owner-service/backend boundary with a regression-preserved Linux implementation and explicit macOS admission checks. The next runtime deliverable is the single offline flow above. Another isolated IPC or VM probe is not a completion milestone.

## Evidence retained at reset

- Saved macOS arm64 native checks: 31 library and 62 integration tests passed under UTC and Asia/Kolkata; formatting, Clippy and release build passed. The native source hashes were rechecked on 2026-11-03 and still match those runs. These checks were not rerun during this reset.
- `runtime-final-tools/result.json` records a successful offline app-server MCP catalog/status call, argv-override rejection, allowed direct patch/terminal writes, and denied new excluded filenames through both paths. Responses came from a deterministic loopback fixture, with zero real model calls. This is not authenticated model acceptance.
- The later build fixture observed a failing shell check, repair and passing retest. Its independent owner launch failed from UID-switch ordering. The corrected replay did not complete provisioning; a malformed YAML attempt was rejected and later boot attempts did not produce the required fixture. Independent retest and cold-boot recovery therefore remain unaccepted.
- Latest read-only inventory verified all four project VMs stopped. The macOS VM configuration exactly matches its preserved original. Guest data, disabled service identity, failed attempts and undo records remain preserved.
- Native account assessment verifies integrity and clear monetary reservations. Thirteen historical valuations remain unknown; none were reset or assigned a price.

Detailed artifacts remain under `.kineti/autonomy-macos-task-service-20261102/`. Native source/artifact digest manifests are `final-source-digests.json` and `final-artifact.json`. The successful SDK fixture is `runtime-final-tools/`; failed combined attempts remain separately labeled. The local website now removes the retired npm install promotion and accurately states that no approved website download is available. Nothing was deployed.

## Boundaries that remain

Full A01–A14 acceptance is still open. The shared service extraction and complete macOS backend are implementation work, not approval gates. Fresh logins and real acceptance model turns, any additional reviewer beyond the previously approved review, credential expansion and release actions retain their separate gates. Historical accounting clearance remains required for publication. New paid-spending authority remains zero. No merge, publication, deployment, paid runner or provider API fallback is authorized by this plan.


Owner-service implementation update, 2026-11-03: see `AUTONOMOUS-BUILD-OWNER-CHECKPOINT-20261103.md`. The shared engine/platform extraction, read-only interrupted-service inspection, durable guest boot retry and VM startup lease fence now have offline evidence. The complete Linux owner-service regression entry point passed, including active and owner-crash descendants. macOS task-service admission still fails closed: its persistent guest worker and sealed storage/resource backend are not integrated. These results promote no A01–A14 row and authorize no login, model turn or release action. Earlier stopped-VM inventories remain dated observations; use the checkpoint's final inventory for this batch.
