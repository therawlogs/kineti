# Local workflow, memory, and incident commands

These commands add local product behavior described by the remaining-work contract. They do not establish a remote evaluator, host security boundary, live model integration, or user identity provider.

## Project workflow

Select a mode when activating a new project, then lock its goal:

```sh
kineti state init --project sample --mode harness --goal "Build the agreed sample"
```

Legacy state with no mode remains pending. Do not select a mode on behalf of its owner.

For a new project, save the approved artifacts in the project and record them in order:

```sh
kineti workflow record --step discovery --file docs/brief.md
kineti workflow approve discovery
kineti workflow record --step design --file docs/storyboard.md
kineti workflow approve design
kineti workflow record --step no-data --file docs/no-data.md
kineti workflow approve no-data
kineti workflow record --step architecture --file docs/architecture.md
kineti workflow approve architecture
kineti workflow generate-outcome --file docs/outcome-contract.json
# Complete and review the draft, then record its exact bytes.
kineti workflow record --step outcome --file docs/outcome-contract.json
kineti workflow approve outcome
kineti workflow status
```

Use `data-intake` instead of `no-data` when the project needs inputs. The design artifact must be approved first. The terminal approval command prints the artifact’s SHA-256 and requires the operator to type `APPROVE <digest>` in a TTY. The MCP workflow tool can inspect state and record artifacts, but it cannot create an owner approval.

Each artifact record stores its relative source path, raw-byte digest, time, and parent artifact digests. `status`, later steps, and approvals re-read source bytes; edits or missing files make the recorded approval stale until that artifact is recorded and approved again. Re-recording an earlier artifact makes approvals that depend on the older version inapplicable. The ledger is local; it is not a cryptographic identity proof for the person at the terminal.

The generated outcome file is a draft. Before it can be recorded or approved, it must link the current discovery, design, data/no-data, and architecture digests and include the locked goal, personas, storyboard and mock-data shape, data decision, features linked to personas/outcomes/tests, architecture choices, tool review, executable checks, roadmap, explicitly confirmed priorities, decision graph, and reviewed unknowns. Real inputs need a purpose approval reference in the form `data-intake:<approved-artifact-sha256>`. Every decision-graph evidence reference must use `<step>:<current-artifact-sha256>` and match one of the linked sources; a causal edge must include such evidence. Each roadmap item's `priority_id` must match a confirmed priority item. Empty tool choices, graph edges, or unknown lists must be explicit. Approval remains a separate human terminal action tied to the exact completed bytes.

To record a conflict that affects only named work, provide at least two choices. Name affected workflow steps using `discovery`, `design`, `data-intake`, `no-data`, `architecture`, or `outcome`; `data` names both data paths and `all` names every step. A conflict pauses its named step and later dependent steps, but it does not pause earlier or unrelated work. Status reports `blocked_by` for each affected step.

```sh
kineti workflow conflict add --id scope-change-1 --request "Change the approved export" --affected "csv export" --option "Keep current scope" --option "Start a new run"
kineti workflow conflict list
kineti workflow conflict resolve --id scope-change-1
```

Resolution asks for a numbered choice in a TTY. A pending conflict record does not automatically detect conflicts in natural-language requests; the host must identify and record them. It also does not infer arbitrary feature names from artifact text, so name a workflow step when that work should be blocked by the ledger.

## Contextual memory

The command and MCP API derive scope from the current project state and the local OS user id. They do not accept a caller-selected user scope. The MCP `prepare` action returns the exact bounded `{schema_version, user_prompt, contextual_memory}` payload selected for the connected host; it omits scope identifiers, corrected versions, deleted versions, and other users' or projects' records. This is a local adapter seam, not a claim that a live model was called or that every host uses this payload. The CLI accepts content in an argument, so use MCP when process arguments might expose sensitive text.

```sh
kineti context store --id mem-sample-001 --content "The test import needs a token refresh." --tags '["import"]' --sources '[]'
kineti context retrieve --query "test import" --limit 8
kineti context prepare --prompt "Continue the import task." --query "test import" --limit 8
kineti context correct --id mem-sample-001 --new-id mem-sample-002 --content "Correction: refresh the token before importing."
kineti context delete --id mem-sample-002 --confirm mem-sample-002
kineti memory verify-chain --dir .
```

Each memory body, tags, and source references are AES-256-GCM encrypted with a per-record key. Keys and ciphertext files live under `.kineti/memory/`; the hash-linked journal stores scope, content and ciphertext digests, an encrypted-blob path, and correction/deletion metadata. Retrieval filters by both user and project, excludes superseded and tombstoned entries, checks ciphertext and plaintext digests, and resolves source digests. Retrieval is bounded lexical matching; there is no vector database or graph retrieval path in this interface.

Deletion appends a tombstone, then removes the per-record key and ciphertext file. The tombstone prevents retrieval after restart even if cleanup is interrupted. This is cryptographic key removal, not a promise of physical overwrite: filesystem snapshots or backups outside this interface may retain prior files. The local UID scope does not separate people sharing one OS account. The live-model request payload has not been audited by this implementation.

## Incident records

Incidents are append-only records in the existing `.kineti/journal.jsonl` hash chain. Use `kineti incident add --record '<JSON>'` with a record containing:

- `id` using an `ir-` prefix and `scope` with both `user` and `project`.
- `title`, observed `events` with time, actor, task id and observations, plus exact `evidence` references (`source_id`, relative `path`, and SHA-256).
- `reported_causes` attributed to a reporter, separate `hypotheses`, `dependencies`, missing evidence, and conflicting explanations.
- For a correction, a new id and `correction_of` pointing to an earlier same-scope incident.

Inspect or list only within explicit scope:

```sh
kineti incident show --id ir-sample-001 --user local-operator --project sample
kineti incident list --user local-operator --project sample
```

The writer verifies every source digest before appending. Reports resolve sources again and show missing or changed files. Event order and dependency links are displayed as observations; neither proves causation. The incident API checks declared scope values but the current local CLI has no multi-user authentication service.
