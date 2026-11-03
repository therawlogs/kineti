# Authenticated Linux model acceptance: four-turn result

Date: 2026-10-31. Status: four authorized turns consumed; acceptance incomplete.

The owner approved a memory-only device login, then separately approved a fresh
login after the first session ended. The four-turn ceiling was not reset: the
second controller started its counter at one and attempted only three more turns.
Both sessions used the existing ChatGPT `prolite` subscription, ordinary allowance
was checked before each turn, and observed quota stayed below the 95% stop. No API
key, purchased credits, reset credits, paid fallback or new paid spending was used.
Historical subscription allocations remain unvalued where previously unknown.

## What actually ran

| Turn | Observed result | Acceptance |
| --- | --- | --- |
| 1 | Authenticated inference succeeded with the account-advertised default `gpt-6.1-sol`. Local tools could not start because the separately shipped code-mode helper was absent. Independent check still failed. The controller ended the memory-only session. | Authentication/inference only; repair failed |
| 2 | After the official helper repair and approved fresh login, direct model patches changed the allowed value and allowed-text files. A protected edit attempt was denied. The owner's independent check passed, but the model terminal returned exit 2 without a confirmed test pass. | Direct edits observed; model-terminal verification failed |
| 3 | The owner changed the previously passing value to stale and freshly observed failure. The model repaired it; the owner's independent retest passed. The native MCP server was discovered on the actual thread, but the MCP status call triggered `mcpServer/elicitation/request`, which the conservative controller declined. | Stale repair observed; model-terminal and MCP status acceptance failed |
| 4 | The model attempted the exact detached-child script, but terminal startup again returned exit 2. The child-ready marker never appeared. The owner stopped the session and drained the cgroup. | No model-driven detached-child revocation pass; the intended child never started |

Both authenticated app-server processes ended through the native cgroup control.
The owner audit records successful cgroup revocation with no remaining population.
Fresh ephemeral account reads afterward report no signed-in Linux account, and
no standard-path `auth.json` appeared. These are local process/cache observations;
they do not prove provider-side session deletion or universal absence of secrets
from every possible operating-system storage mechanism.

Allowed file changes are synthetic fixture changes only. Protected fixture bytes
remained unchanged. The second thread discovered only the intended `kineti_task`
MCP server after apps/plugins were disabled. The MCP status rejection happened
before its intended status operation succeeded; discovery is not acceptance.

## Runtime repair and remaining diagnosis

The original Codex executable remains byte-for-byte unchanged. Its official
single-executable archive did not include the separately distributed
`codex-code-mode-host`. The exact matching 0.166.0 official helper was added only
to the disposable fixture after guarded undo registration:

- Helper archive SHA-256: `ac98d1f878e353a16cba711fd47bfef2f6c814109bdd819ee2b6b6d6a19701e7`.
- Helper executable SHA-256: `587436a943a59c18cabdbe83f07800d237b60ddd2e0f54d05b23d8b37a0b36f4`.
- Existing Codex executable SHA-256: `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`.

The helper checksum matched official release metadata. A no-auth protocol test
negotiated version 1, opened a session, executed an arithmetic assertion without
an error, closed the session and drained its cgroup. That repair was verified
again by the model's successful direct edits in turns 2 and 3.

The terminal failure is not yet repaired. `/bin/bash` is a pre-existing 29-byte
wrapper around `/bin/ash`, not genuine Bash. It lacks Bash-specific builtins, but
normal `-lc` execution works. A new no-login, zero-model probe ran the same simple
check successfully through both `/bin/bash -lc` and `/bin/sh -c` under the named
filesystem/network profile. Therefore the wrapper is a compatibility concern,
**not a proven sole cause** of the model's unified-terminal exit 2. It was not
replaced, and no shell package or custom sandbox fallback was installed.

The MCP failure is separately identified: the owner controller declined the
host's elicitation request. It did not auto-approve unknown protocol requests or
suppress prompts globally. A follow-up needs a supported, narrowly declared
policy for the exact task-status operation and an actual live retest. A broad
approval switch is not a fix. Current documentation describes per-tool MCP
approval controls, but their correct pinned behavior has not been accepted here.

## Limits and next work

1. Exactly four subscription turns have been attempted. No model-turn allowance
   remains from this batch. No further login or inference will start without a
   separate owner-approved scope/budget.
2. Diagnose and repair the model unified-terminal path without weakening the
   stock host sandbox, while preserving the original shell/runtime files.
3. Prepare the exact MCP status policy and test its pinned elicitation behavior.
4. Only after those preparations, propose a bounded new authenticated test batch.
   A fresh memory-only session would need another owner browser step.
5. Complete production native-service/model integration separately. The test
   used a labelled synthetic root controller and task-status gateway together
   with native cgroup revocation; it was not the production grant service's full
   autonomous model-repair implementation. Full A01–A14 and installed-package
   acceptance remain pending.

## Evidence and preservation

Evidence is under `.kineti/autonomy-model-20261031/`: projected eligibility,
owner RPC requests, both attempt summaries, final fourth-turn events, final guest
observations, account cleanup projections, official runtime hashes, no-auth
helper/terminal probes and guarded undo registrations. The first failed diagnostic
using an unavailable JavaScript `console` global is retained separately from the
successful language-only arithmetic assertion. Failed or unrun cases are not
counted as passing tests.

Device challenge values were shown to the owner through the chat UI, not written
to diagnostic files. No credential values were copied into argv, grants, snapshots
or receipts. Two compiler-cache files created by an initial Python syntax check
were verified as task-generated and moved byte-for-byte into this project's
ignored evidence area; later syntax checks used in-memory compilation.

Production source and existing user work, the locked goal, original financial
records, signatures and historical monetary unknowns are preserved. New operational
receipts and undo registrations append to the original histories. Ordinary native
spend/status blocks remain intact. No merge, publication, deployment, provider,
billing/security setting or additional credential action was performed.

References:

- [Official pinned runtime release](https://github.com/openai/codex/releases/tag/rust-v0.166.0)
- [Official app-server protocol](https://learn.chatgpt.com/docs/app-server)
- [Official configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference)
