# Authenticated Linux acceptance: readiness and credential gate

Date: 2026-10-31. Status: prepared; authenticated acceptance not run.

The owner authorized continued implementation using existing subscription access,
with $0 new paid spending and separate credential/release gates. The locked goal,
existing work, accounting history and unresolved monetary valuations are preserved.

## Fresh observations

1. The existing macOS host reports ChatGPT subscription authentication, ordinary
   usage allowed, 46% of its reported primary window used, and no purchased credit
   balance. This is current resource metadata, not a valuation of historical use.
2. The pinned Linux Codex 0.166.0 executable has SHA-256
   `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`.
   Both the dedicated acceptance account and the normal guest account report no
   signed-in account. No Linux model request was made.
3. The pinned CLI advertises device-code sign-in. Its generated Linux app-server
   schema includes `chatgptDeviceCode`. Its external-token login alternative is
   explicitly marked internal-only; this candidate will not use token extraction,
   forwarding or that interface as an authentication bridge.
4. The pinned Linux app-server accepted
   `cli_auth_credentials_store="ephemeral"` and returned a successful projected
   account read. Official documentation defines this backend as memory-only for
   the current process. Actual authenticated storage behavior remains a live
   acceptance check; no sign-in was started to test it without owner approval.
5. The existing native grant schema intentionally accepts no credential handles
   and only owned local resources. It has not been silently widened to admit a
   subscription credential or model transport. The model adapter and its explicit
   host-auth binding still require implementation and testing after this gate.

Evidence: `.kineti/autonomy-auth-20261031/`, including sanitized host/Linux account
projections, actual pinned login help/schema, executable identity and exact probe
scripts. Raw account identities and credential values are not retained.

## Exact proposal requiring separate owner approval

Authorize one owner-completed ChatGPT device-code sign-in to the **existing
subscription**, inside the disposable `kineti-autonomy` Linux VM, for the isolated
`kineti-test-one` acceptance account and pinned app-server process only.

1. Use the supported `account/login/start` device-code flow. The owner completes
   authentication in their browser. Do not read or copy host auth files, extract
   tokens, use API keys, create service accounts, or use an external-token bridge.
2. Require the ephemeral credential backend. Request no persistent credential
   cache or host keychain changes. Keep authentication values out of argv,
   grants, snapshots, receipts and diagnostic logs. If authenticated operation
   needs file storage, a different backend, broader scopes or additional
   account/security changes, stop and present that separate change first.
3. Do not enable device-code login in security settings automatically. If the
   existing account policy disables it, stop; neither settings changes nor a
   token-copy fallback are included in this proposal.
4. Credential availability is limited to the pinned Linux host process. Task
   shell commands and direct editing retain the tested filesystem/network
   boundary; they receive no credential handle or general outbound permission.
   Inspect any authentication URL before presenting it to the owner. Only the
   official Codex-managed ChatGPT sign-in/inference transport is in scope;
   provider overrides, third-party endpoints and private-source uploads are not.
5. After implementing the explicit subscription-host binding, run at most four
   synthetic model turns, one worker, up to 15 minutes after authentication and
   at most 180 seconds per turn. Use synthetic fixture code only. Demonstrate a
   failing test, model repair, fresh retest, direct-edit denials, restricted MCP
   and an owner revocation/time-bound stop. Unrun rows remain pending.
6. Use only ordinary included subscription allowance. Check account type,
   ordinary-usage eligibility and reported quota before every turn. Stop if
   eligibility is missing, ambiguous, exhausted or at the 95% stop. Do not consume
   purchased credits, reset credits, paid fallback, API billing or upgrades.
7. Approval does not resolve unknown historical costs or authorize any monetary
   reservation/recovery, merging, publication, deployment or external data change.
8. Keep the session in the revocable Linux process group of authority (cgroup v2),
   with the owner controller outside it. Terminating that process ends local
   credential availability; it does not promise deletion of a provider-side login
   session or change the owner's broader ChatGPT account. Preserve audit evidence
   and stop the disposable VM at the end. No source merge/release follows.

This proposal permits a narrowly bounded authentication ceremony and acceptance
run. It does not itself approve or execute either. The browser step still needs
the owner; no login code has been generated or redeemed.

## Why approval is required

The current task explicitly keeps credential expansion separately gated.
`docs/AUTONOMOUS-BUILD-CONTRACT.md` section 3 requires separate exact approval for
broader credentials, and section 9 says implementation approval does not authorize
credential or scope changes. Giving a Linux process new subscription credentials,
even in memory and on the same subscription, crosses that boundary.

## Official references

- [Authentication and headless device login](https://learn.chatgpt.com/docs/auth?surface=cli)
- [App-server authentication protocol](https://learn.chatgpt.com/docs/app-server)
- [Credential-store configuration](https://learn.chatgpt.com/docs/config-file/config-reference)

Live pinned behavior/schema takes precedence over assuming newer documentation
features are supported by the installed executable. The OpenAI Docs skill was
used for official documentation verification; it did not impose this approval
request. The owner's instruction and approved contract impose the credential gate.
