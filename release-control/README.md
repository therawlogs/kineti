# Internal release-control prototype

This directory implements local adversarial fixtures for the enforced production
release contract. It is not shipped as a `kineti` public command or a new package.
It uses Python 3.9+ standard-library code on POSIX systems. No dependency download,
network operation, credential discovery or paid resource is needed.

Run actual assertions from the project root:

```sh
python3 release-control/run_tests.py
```

Test workspaces live under `.kineti/production-release-implementation/fixtures/`.
Every identity, approval, cost and publication in them is synthetic. The printed
summary is derived from unittest results. A signature on this output would still
not prove an independent test runner or meaningful assertions.

## Interfaces

`controller.py` supplies strict JSON parsing, manifest validation, immutable byte
reads, archive safety inspection, a package/version lock, a hash-chained durable
audit and an offline synthetic promotion controller. Structural schemas are in
`schemas/`; the executable validator also enforces identity relationships, policy
bindings, report inventory, expiry and accounting rules that JSON Schema cannot.
Unknown fields and duplicate keys deny. Timestamps use integer UTC Unix seconds.
Digests use SHA-256 of bytes or canonical sorted ASCII JSON with no nonfinite numbers.
These are release-control interfaces, separate from native canonical receipts.

The manifest is immutable. An approval object separately references its digest,
avoiding digest self-reference. A trusted adapter must authenticate approval and
accounting-exception records outside candidate control. `SyntheticAuthority` merely
models these observations for tests. It is not an authenticated production adapter.
Passing JSON with a reviewer name or integrity=true is never real authentication.

The staged folder must contain exactly the two named approved archives. Read bytes
through no-follow descriptors, check lengths and hashes, inspect members without
extraction, and pass the in-memory bytes unchanged. The controller never runs a
package hook, builds an archive, shells out, or selects a token. Read-only host
staging, independent policy ownership and runner isolation remain external controls.

Provision a private audit directory under independent host ownership before use.
Do not share it with candidate jobs. The synthetic controller binds one immutable
manifest to a repository/version/destination key. It records created, validated,
approved, publishing, per-service intent and verified events, then complete. Failure
after possible external effects records uncertain; failed preflight after prior
intent records partial. Denied preflight grants no synthetic write or credential.
Omitting or altering the log/checkpoint blocks replay. `flock` serializes local
processes, not multiple hosts; distributed serialization is not implemented.

An explicit `reconcile=True` revalidates policy, approval, accounting and bytes,
then reads every synthetic sink before writing an untouched remainder. A missing
synthetic value is definitely absent because the sink is in-process. A real adapter
must distinguish proven absence from network uncertainty and retain state across
hosts. Never treat this synthetic behavior as a registry absence check.

## Disabled production boundary

`ProductionAuthority` always denies. The controller also requires the exact
synthetic adapter type, synthetic mode and synthetic destinations before writing.
There is no OIDC exchange, token fallback, Cargo/npm upload, GitHub mutation or
workflow trigger. Do not replace the disabled adapter with an unchecked command.

The following require further implementation and independent operator evidence:

1. Named restricted coding identity, policy owner, reviewer and settings operator;
   isolated host and independently protected policy repository.
2. Authenticated GitHub jobs/settings/reviewer observations, provider claim binding
   and exact-byte OIDC upload adapters with pinned tooling.
3. Cross-host durable locks/audit retention, cancellation enforcement and incident
   reconciliation against actual registries, not this in-memory sink.
4. Separately approved canary configuration/writes, new-version acceptance on all
   four native targets, production settings and exact release approval.

Normal Cargo publish creates a package from source, so it is not an implemented
exact-byte promotion adapter here. Provider restrictions, archive-byte preservation
and account eligibility need demonstration before enabling a real transport.

See [implementation status](../docs/ENFORCED-PRODUCTION-RELEASE-IMPLEMENTATION.md)
for evidence and the full remaining acceptance boundary. No local pass establishes
P01–P27 production acceptance.
