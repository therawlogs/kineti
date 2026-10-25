# Linux evaluator and protected output v1

`kineti linux-service` is a user-operated Linux service. `kineti service-client` and the MCP tool `kineti_linux_service` submit signed requests. This service is separate from the legacy same-account `kineti_protected_write` tool. Legacy receipts do not authorize this service.

The supported candidate is a single file, at most 1 MiB, executed as a BusyBox shell program against an owner-approved CLI inventory. Browser applications, arbitrary Bun packages, network checks and universal interception of host file tools are outside this interface.

## Provisioning

Use a dedicated service OS account or isolated Linux VM. Candidate and caller accounts must have no access to that account's files. The service account must be able to create user/mount/network namespaces, run Bubblewrap, and write a delegated cgroup v2 subtree with memory, pids and CPU controllers. Startup fails if the configured runtime or authority is invalid. Do not grant candidates the service account or root access.

Provision a service directory with mode 0700 and `config.json` with mode 0600. The version 1 configuration is the exported `LinuxServiceConfig` type in `bin/kineti-linux-service.ts`:

- Separate Ed25519 owner public key, evaluator private key and caller public keys. Keep the owner signing key offline from the service and every candidate.
- `pins`: owner-approved project, goal, contract, policy, artifact, inventory, owner and evaluator identities defined by `ApprovedEvaluationPins` in `src/security/evaluation-contract.ts`.
- `contract`: version 1, explicit environment and required checks. Each check has a unique id, argv, base64 stdin, expected exit, exact stdout/stderr SHA-256 and timeout (at most 10 seconds).
- `policy`: version 1, exact relative destination allowlist, caller keys, revocation lists and maximum result age.
- `runtime`: absolute Bubblewrap, prlimit, BusyBox and private library directory paths, cgroup root, and SHA-256 for every executable/library file. Libraries and loader need executable permissions beneath the private service directory; do not mount an authority directory into a candidate.

Use canonical format v2 digests for the contract and policy, and the existing signed manifest v1 format. Have the owner review the inventory, runtime and pins before signing the manifest. `acceptance/linux-service.test.ts` contains executable provisioning examples with disposable keys; those keys and approvals are test inputs, not production authority.

Initialize exactly once, then serve on loopback:

```sh
kineti linux-service init SERVICE_ROOT
kineti linux-service serve SERVICE_ROOT 17889
```

Give each caller an owner-only project containing `.kineti/service-client.json` and its own mode-0600 private key. Client JSON requires version 1, endpoint `http://127.0.0.1:PORT/v1/request`, caller id, relative `private_key_file`, pinned owner/evaluator public keys, and the approved `pins`. Use an SSH local tunnel when the service runs elsewhere. The client logs outbound request digests before transmitting and verifies signed responses against the pinned evaluator.

## Requests and effects

JSON request envelopes use version 1, `operation`, caller, unique request id, expiry, exact arguments and Ed25519 signature. Expiry is bounded to five minutes. The client uses one minute. Operations:

- `evaluate`: arguments `manifest` and `artifact_base64`. The service stores immutable exact bytes and runs every required check in isolation. Only complete accepted observations produce an accepted signed result.
- `export`: arguments are exactly `action: "export-artifact"`, `evaluation_id`, `evaluation_sha256`, `policy_sha256`, `destination` and `digest`. Exact accepted bytes are copied from service storage; the caller cannot replace them. The authorization is consumed once, including across new request ids and restarts.
- `inspect`: query the durable outcome by request id. A failed transport response means unknown; inspect before retrying.

See `src/security/service-request.ts` for envelope and signed-response types. Invoke `kineti service-client evaluate|export|inspect ARGUMENTS.json` from the provisioned caller project. Credentials never belong in command arguments.

Candidates run with UID/GID 65534, no network, capabilities removed, private temporary storage, bounded output, CPU/wall limits, and a real 128 MiB/16-process cgroup. This protects service-owned output only. Host-native writes to other paths remain outside Kineti.

## Recovery and revocation

SQLite WAL with full synchronization records pending before a write and applied after a durable rename. On restart, pending requests become applied only when matching bytes prove the effect; otherwise they become unknown. Unknown authorization is never automatically reused.

Inspect an abandoned lock's process and request/output state before `kineti linux-service recover-lock SERVICE_ROOT --owner-inspected`. A live process prevents recovery. Missing or corrupt initialized state blocks startup; preserve the database, WAL, marker, keys and outputs for owner recovery. Never delete them to retry or initialize a replacement under the old authority.

Owner-only `revocations.json` is read for every request. Revoke callers or evaluations there. Configuration/policy changes require new owner-approved pins and a new signed manifest. Authority replacement requires an explicit separately provisioned instance and caller pin change; there is no automatic key rotation or history reset.

Real Linux acceptance covers isolation, substituted inputs, failed checks, timeout/OOM, replay, concurrency, symlinks, corruption, four crash points, revocation and signed HTTP calls from another UID. See local evidence labels and the completion status for tested versions and remaining host gates.
