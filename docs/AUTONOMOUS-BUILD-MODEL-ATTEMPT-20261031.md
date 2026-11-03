# Authenticated Linux acceptance: first attempt and runtime repair

Date: 2026-10-31. Status: authenticated inference demonstrated; acceptance incomplete.

## Observed result

1. The owner explicitly approved one memory-only device-code login and at most
   four synthetic model turns. The owner completed the browser ceremony.
2. The pinned Linux Codex 0.166.0 app-server authenticated as ChatGPT `prolite`.
   Before inference, ordinary usage was allowed, the primary window reported 47%
   used, and no purchased-credit balance was available. No API key, paid fallback,
   credit purchase, reset credit, or subscription change was used.
3. One synthetic turn ran using the account's advertised default model,
   `gpt-6.1-sol`, with low reasoning effort. The turn completed, but its local tool
   runtime could not start: `codex-code-mode-host` was missing. The model reported
   that it could not edit the fixture or run its check. The owner's independent
   check still failed. No repair, direct-edit or MCP acceptance is claimed.
4. The fixture controller failed closed after the failed repair check and called
   the native cgroup revocation primitive. It observed an empty cgroup and ended
   the authenticated process. A fresh ephemeral Linux process subsequently
   reported no signed-in account. No `auth.json` cache appeared at the dedicated
   account's standard path. This demonstrates local process/cache cleanup, not
   provider-side session deletion or a universal credential-persistence proof.
5. One of the four authorized turns was consumed. **Three remain.** The fixture
   was a synthetic owner controller using native cgroup control, not a completed
   integration of model repair into the production native grant service.

The missing companion should have been checked before initiating the owner login.
The test controller's stop also destroyed the memory-only session, so further
model execution now needs a fresh owner login; credentials were not recovered,
extracted, copied or persisted to avoid that gate.

## Runtime repair completed without authentication

The earlier downloaded official `codex-aarch64-unknown-linux-musl.tar.gz` contained
only the main executable. Model tools also require the separately shipped
companion. The matching official 0.166.0 helper was downloaded at no monetary
charge and installed only beside the disposable VM fixture's Codex executable,
after preview and an exact guarded inverse were recorded. Original Codex bytes,
the installed desktop host and production source were not changed.

| Artifact | SHA-256 |
| --- | --- |
| Existing Linux Codex executable | `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a` |
| Official helper archive | `ac98d1f878e353a16cba711fd47bfef2f6c814109bdd819ee2b6b6d6a19701e7` |
| Extracted helper executable | `587436a943a59c18cabdbe83f07800d237b60ddd2e0f54d05b23d8b37a0b36f4` |

The archive checksum matched the official release asset metadata. This is not a
claim of independently verified signing identity or installed-package acceptance.

Fresh no-authentication checks of that helper successfully negotiated protocol 1,
opened a session, executed a JavaScript arithmetic assertion without an error,
closed the session and drained its Linux cgroup. An initial diagnostic using a
nonexistent `console` global is preserved as a failed diagnostic; the validated
language-only assertion is the successful execution check. No model allowance
was consumed by these helper tests.

The pinned feature controls also accepted disabling connected apps and plugins.
The next fixture stages those restrictions, pins both executable hashes and
rejects unexpected connected-app tool exposure before starting inference. The
first attempt discovered the host's default connected-app tool catalog but did
not invoke those tools. No claim of complete outbound/credential acceptance is
made from that discovery.

## Prepared next step; not started

A corrected fixture is staged for **three additional turns, four total**:

1. Repair the failing synthetic check and test allowed versus denied direct edits.
2. Invalidate the prior passing input, repair/retest, and exercise restricted native
   MCP discovery/status through the explicitly labelled synthetic gateway.
3. Start a detached synthetic child and revoke the owner cgroup before its delayed
   write. Do not count a killed turn as a normal completed model response.

The prepared controller begins its aggregate counter at one, retains the
four-turn ceiling, one worker, 180-second turn bounds and 15-minute post-login
window, and rechecks ordinary subscription eligibility before each turn. Apps,
plugins and task-command network remain disabled. There is no provider fallback,
API billing, persistent auth cache, token bridge or new sandbox implementation.
The synthetic MCP gateway does not establish production native-service integration.

Because the one approved ephemeral login has ended, **one fresh memory-only
device-code login needs separate owner approval** before this retry. The browser
step still belongs to the owner. No new login was started after the first process
ended. The additional approval would not add model turns, paid spending, broader
credential scope or any release action.

## Evidence and preserved boundaries

Evidence is under `.kineti/autonomy-model-20261031/`: projected eligibility,
synthetic owner requests and results, first-turn outcome, native revocation events,
post-stop account projection, official artifact metadata/hashes, helper protocol
checks, prepared retry scripts and aggregate budget. Authentication responses and
device challenge values were not written to the diagnostic files. The owner saw
the one-time device challenge through the chat UI only.

Production source, locked goal, existing user work, original accounting records,
signatures and unknown historical monetary valuations remain unchanged. New undo
and operational evidence records append to preserved history. Existing native
spending/status blocks on the historical unknown remain in force. No release,
merge, publication, deployment, billing/security setting or additional credential
change was performed.

Official sources:

- [Pinned official release artifacts](https://github.com/openai/codex/releases/tag/rust-v0.166.0)
- [Official app-server protocol](https://learn.chatgpt.com/docs/app-server)
- [Official credential-store configuration](https://learn.chatgpt.com/docs/config-file/config-reference)

Full A01–A14, authenticated repair/retest, direct-edit denial, model-driven detached
revocation, production native-service integration and installed-host acceptance
remain unaccepted until their actual checks run and pass.
