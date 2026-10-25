# Protected write policy and limits

Protected writes are experimental and cover only requests sent through Kineti’s MCP tool. Direct editor writes bypass the tool. Forged test output and the gap between checking and writing remain unresolved, so this is not a secure sandbox.

## Local trust policy

Protected writes require .kineti/protected-write-policy.json. A missing, unreadable, malformed, or unsupported policy blocks authorization with result code 2. The policy format is:

    {
      "version": 1,
      "trusted_issuer_key": "-----BEGIN PUBLIC KEY-----\nPASTE THE PUBLIC KEY FROM .kineti/evidence-issuer.pem HERE\n-----END PUBLIC KEY-----\n",
      "revoked_run_ids": [],
      "revoked_issuer_keys": []
    }

Create the file only after you have reviewed the issuer key. Kineti never enrolls a receipt's key while checking that receipt. Keep the file mode at 0600. The local account that can edit this policy can also edit the project files and the private signing key in .kineti/evidence-issuer.pem.private; this policy does not defend against that account or an adversarial builder. It provides a separate, explicit trust choice for ordinary receipt checks.

To revoke a proof, add its run_id to revoked_run_ids. To revoke an issuer, add its exact public-key PEM string to revoked_issuer_keys. The writer reads this policy for every authorization and repeats it under the local write lock. The JSON file is limited to 1 MiB, at most 10,000 revoked run IDs and 1,000 revoked Ed25519 issuer keys. Invalid entries block with code 2. Keep a reviewed backup with restricted access. If the file is lost or malformed, restore a reviewed backup or have the project owner provision it again; there is no automatic recovery or remote revocation service. A key rotation requires an intentional policy update. Receipts signed by a key that is no longer trusted will be blocked.

## Receipt checks

The writer accepts only version 2 receipts with a valid signature from the pinned issuer, parser identity, passing nonzero evidence, no failure or skip counts, a valid timestamp no more than 240 minutes old, and no timestamp more than 60 seconds ahead of the local clock. It rechecks the current toolchain, declared environment hashes, workspace fingerprint, dependencies, required test files, and artifacts.

For protected writes, set settings.verify_command_argv in kineti.config.json to the exact JSON string array used after the evidence command's --. This array is compared element by element. The older settings.verify_command string remains supported through quote-aware parsing but cannot represent every possible argument boundary. The cmd field in a receipt is display text and is not used to reconstruct argv. The --expect-cmd check still has its documented substring behavior.

The test observation boundary is unresolved. The parser recognizes captured Bun, Cargo, and pytest summary text. Required Bun file names are checked in that same untrusted output. This is not a validated result channel or a required-case inventory. A test file can print passing summaries and required file names, then exit before the cases run. The same-command regression reproduces this through receipt creation and the actual writer; both the forged run and a genuine control are currently accepted. Matching argv and executable hashes do not establish case execution.

Acceptance of a gate that rejects forged success is blocked. The smallest next design decision is who controls an independent evaluator and its required-case inventory, and how it authenticates results to the authorizer outside builder-controlled output. This contract does not implement that evaluator or a sandbox.

Declared inventory, dependencies, pre-existing artifacts, environment values, toolchain and parser identity are captured before execution and compared afterward. The checker and writer share strict receipt and current-input checks. Normal receipt checks read the existing local public key and never generate one. Unset, empty, and the literal string `<unset>` have different hashes. Environment declarations whose names look secret-like are rejected; raw environment values are not stored or logged.

## Write and recovery behavior

The writer takes a local exclusive lock, rechecks policy and workspace, saves the run ID as used, writes a temporary file, rechecks the fingerprint and destination parent identities, then renames the temporary file to the destination. Replay state is a local array in `.kineti/protected-write-runs.json`, limited to 16 MiB; malformed or unreadable state blocks with code 2. IDs are never removed automatically. A missing replay file starts an empty store, so deleting it removes single-use history; keep a restricted reviewed backup and do not delete it to retry. A proof is consumed before the rename. If a write fails after consumption or the process crashes, the destination may be absent or its final state may be unknown; inspect the destination and use a new receipt before retrying. Kineti does not retry automatically.

The lock coordinates Kineti protected writers only. Authorization, path checks, workspace rechecks, temporary-file creation, and rename are separate operations. Another process can change inputs or replace a destination parent between those operations. Atomic rename protects the final replacement from partial file contents; it does not make authorization and effect one atomic action. The no-gap authorization/effect guarantee is unresolved.

Tests exercise a failure immediately before rename, competition while the lock is held, a changed checked input and a destination parent replaced by a symlink. The tested platform is macOS with a local filesystem supporting exclusive file creation, POSIX permissions, symlinks and same-filesystem rename. No Windows, network-filesystem or real Cursor host acceptance is established. These finite tests cannot establish that no race exists. There is no durability barrier for power loss; inspect both destination and replay state after a crash and require a new proof. This contract does not add a general sandbox, OS-level policy, or protection from direct editor tools.

## Protected-write result codes

- 0: write applied
- 1: authorization blocked
- 2: invalid input or required policy/state could not be read
- 3: execution or filesystem failure
