# Trusted Linux fake-publish evaluator

This service evaluates one shell artifact against four service-owned integer-addition cases (`sum-v1`). For arguments `a b`, the artifact must print only their sum and exit zero. The supervisor launches each case and compares the returned data itself. Candidate test files, printed passing summaries, local test receipts, and swarm tickets never authorize an export. This is a small acceptance fixture, not a general package publisher or a guarantee about arbitrary programs.

## Automated acceptance — no personal server required

From the source checkout, activate Bun and Rust, then run:

```sh
bash scripts/evaluator/build.sh
bun bin/kineti-evidence.ts run --label safety-evaluator --required-test scripts/evaluator-acceptance.test.ts --timeout 120 -- bun test scripts/evaluator-acceptance.test.ts
bun bin/kineti-evidence.ts check --label safety-evaluator
```

The build sends only the native binary and reviewed runtime scripts to Docker. It does not send workspace keys, credentials or other runtime records. Standard package signatures and TLS verification remain enabled. The build uses configured proxy variables; the cloud runner's `proxy` hostname is resolved explicitly when present because the Docker builder has a separate DNS context.

The trusted driver creates a disposable container and volume, evaluator UID 10001, candidate worker UID 10002, an untrusted submitter UID 10003, and private service state. The submitter is separate so worker cleanup cannot kill the requesting client. The dedicated worker UID is exclusive to this serial service; its helper kills detached descendants after each case, and Docker init reaps orphaned processes. Candidate subprocesses run without supplementary groups or privilege escalation, with CPU, file-size, address-space and wall-time limits. The container has no network, a read-only root filesystem, a process limit and a memory limit. Candidate code cannot access the Docker socket. The service's narrowly allowed root helper changes identity and applies limits; it does not accept an arbitrary executable or command.

The test driver controls Docker and is trusted. An administrator or coding agent with the Docker socket, root privileges, service-account access or permission to replace the trusted image can bypass this boundary. Acceptance concerns the restricted candidate process, not that administrator. Tests in this environment do not certify another deployment.

## Service protocol

The Linux-only native CLI supports:

```text
kineti-cli evaluator init STATE
kineti-cli evaluator serve STATE SOCKET
kineti-cli evaluator submit SOCKET REQUEST.json
kineti-cli evaluator revoke STATE operation|run|issuer ID
kineti-cli evaluator reconcile STATE OPERATION_ID
```

Initialization refuses to overwrite existing state. The private signing key stays in `STATE/signing.key`; do not copy it into the candidate workspace. The public issuer is printed at initialization. Clients communicate using one bounded JSON object per Unix-socket connection. A request contains exactly `operation_id`, `run_id`, `session_id` (UUIDs), `artifact_hex`, `artifact_sha256`, and `expires_at` (Unix seconds, at most four hours ahead). The decoded artifact is limited to 64 KiB. Identifiers supplied by clients establish correlation, not authority.

The service retains and hashes bytes before executing fixed checks. The signed result binds the request, bytes, runtime/helper/supervisor digests, inventory, policy, destination, expiry and identities. Its private export directory is fixed by the service. Clients cannot choose paths or commands. `applied` is returned only after a durable export. CLI submit exits zero for applied, one for denied/unknown, and two for invalid invocation or transport failure; inspect the JSON status when distinguishing denial from uncertainty.

Ordinary local receipts remain useful development evidence but are not service authorizations. The legacy writer cannot reach the private service destination under the candidate account.

## Crash recovery and revocation

Operation records and exports must use a local persistent filesystem with working file locks, atomic rename and fsync. The service synchronizes intent before exporting and synchronizes exported bytes and directory metadata before recording completion. The applied/reconciled record retains the original signed authorization under `previous`; old signatures are not rewritten. Same-operation requests return the original outcome; different bytes or identities under that operation ID are rejected.

A crash after rename can reconcile the existing digest into an applied result without writing again. A crash before export remains unknown and blocks new exports. The service owner must inspect the original operation and invoke `reconcile`: an existing matching destination is recorded as applied; a confirmed absent destination consumes the authorization as denied and removes any staged bytes. A differing destination requires investigation. Never delete the operation ledger or issue a new operation simply to retry an unknown effect.

Use the owner-only `revoke` command to serialize policy changes with exports. Direct policy-file edits while the service runs are outside those ordering guarantees. Revocation prevents new effects; an already applied operation remains historical evidence and is not undone.

## Later deployment

The current runtime manifest targets x86-64 Ubuntu 24.04. Other platforms require a reviewed runtime manifest and their own acceptance results. Deploy on a Linux host controlled by the evaluator owner, outside the candidate's administrative control. Reserve distinct evaluator/candidate UIDs 10001/10002, and keep normal clients under a different account; fail installation if those IDs belong to other users. Install the reviewed native binary, `candidate` helper and systemd unit from `scripts/evaluator/`. Review the helper and configure sudo to permit only evaluator's execution of `/usr/local/libexec/kineti-candidate`; candidate must have no sudo access.

Create `/var/lib/kineti` owned by evaluator, `/var/lib/kineti-jobs` owned by evaluator with mode 0711, and a runtime socket directory owned by evaluator. Initialize `/var/lib/kineti/state` once as evaluator. Keep state private (0700). The supplied systemd unit disables networking and restricts writable paths. On restart, remove an abandoned socket only after confirming the prior daemon is stopped; never remove durable state. Service provisioning requires administrator permission on that host. Rerun the full acceptance suite against that deployment before making a deployment-level claim.

The automated test uses injected service failpoints after intent and rename, and a completion-record error after export. An error after durable intent returns `unknown`; a denial cannot conceal an export that may already have happened. These are owner-controlled process settings, never client request fields.
