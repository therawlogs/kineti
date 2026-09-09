# Security Report — v3.1 ship gate (2026-09-09)

Scope: commit `70bbc1d` plus working tree. Method: OWASP walk with file:line
evidence, HTTP boundary threat walk, secret scan of tracked files and history,
`bun audit`, agent-specific checks. No external scanners were downloaded.

## Findings fixed in this pass

### F1. HTML escaper missed quotes — attribute breakout (low)
- Where: `bin/kineti-companion.ts` (`escapeHtml`, dashboard script).
- Story: as attacker I write a team member name containing `"` into the
  local fleet file, open the dashboard, and the injected text breaks out of
  `<option value="...">` and runs script in the page.
- Limits: the fleet file is local-only and gitignored, so this is self-XSS
  at worst (writing that file already equals running code).
- Fix: `escapeHtml` now also escapes `"` and `'`.
- Status: fixed, tests pass.

### F2. Repository id written raw into onclick handler (low)
- Where: `bin/kineti-companion.ts` (fleet card button).
- Story: as attacker I put a crafted repo id with a quote into the local
  fleet file, open the dashboard, and script runs when the card renders.
- Limits: same as F1 — local file write already equals code execution.
- Fix: backslashes and single quotes in the id are escaped before rendering.
- Status: fixed, tests pass.

### F3. Origin check passed lookalike hosts (low)
- Where: `bin/kineti-companion.ts` (`startServer` fetch guard).
- Story: as attacker I lure the user to `http://localhost.evil.com`, which
  passed the old starts-with string check.
- Limits: changing anything still needs the Bearer token, which a foreign
  site cannot read or guess, so this alone changed nothing.
- Fix: the guard now compares the exact hostname (`localhost`, `127.0.0.1`).
- Status: fixed, tests pass.

### F4. Dashboard listened on all network interfaces (medium)
- Where: `bin/kineti-companion.ts` (`Bun.serve` with no hostname).
- Story: as attacker on the same Wi-Fi I open `http://<victim-ip>:8788`,
  read the page token, then call the settings, gate, and spend-reset
  endpoints, which trusted anyone holding the token.
- Fix: the server now binds to `127.0.0.1` only. Documented use
  (`http://127.0.0.1:8788` in the browser) is unchanged.
- Status: fixed, tests pass.

### F5. Budget ceiling accepted zero and negative numbers (low)
- Where: `bin/kineti-companion.ts` (`POST /api/settings`).
- Story: as holder of the local token I set a ceiling of `0`, and the spend
  meter divides by zero and renders `NaN`.
- Fix: the server only accepts finite numbers above zero.
- Status: fixed, tests pass.

## Checked and clean

- Injection: no database calls. Every process launch uses the argument-list
  form, never a shell string (`bin/kineti-companion.ts` gate handler,
  `bin/kineti-mcp.ts` `runBin`). MCP tool names map to fixed script files.
- Login protection: every state-changing endpoint (`fleet/select`,
  `settings`, `gate`, `spend/reset`) needs `Authorization: Bearer <token>`.
  The token is 256-bit random, stored with owner-only permissions in
  `.kineti/auth_token`, which is gitignored and untracked.
- Secrets: no keys, passwords, or tokens in tracked files, configs, or
  history. No `.env` files exist; `.env`, keys, and certificates are
  gitignored. `auth_token` and `fleet.local.json` are untracked.
- Unsafe data parsing: the fleet override file is read field by field
  (id must be a string, the local repo entry cannot be overridden).
- Dependencies: `bun audit` reports no issues (5 packages). The package has
  zero runtime dependencies — only type and compiler dev tools.
- Records: spend log, test-evidence log, undo log, and outbound-request
  ledger all exist and are written on every run.

## Threat walk (HTTP boundary, 7 endpoints)

No numbered architecture drawing exists in this repo, so the walk covered
the dashboard boundary instead: `GET /`, `GET /api/status`, `GET /api/fleet`,
`POST /api/fleet/select`, `GET+POST /api/settings`, `POST /api/gate`,
`POST /api/spend/reset`. Result: read endpoints are open but now
loopback-only (see accepted risks); all write endpoints need the token;
`gate` only accepts pass/fail/pending values; `spend/reset` reuses the
human-confirmed CLI path.

## Agent-specific checks

- MCP tools call fixed local scripts; there is no extra tool allowlist, by
  design — the MCP client is the user's own coding agent, which already has
  shell access. Outbound requests are recorded in the egress ledger.
- Swarm roles and dual-signed outcome tickets are simulation-level
  protections; the Ed25519 identity code was not in the changed set and was
  reviewed by observation only.

## Proposed accepted risks (need human sign-off)

1. Read endpoints (`/`, `/api/status`, `/api/fleet`, `GET /api/settings`)
   need no token. Fine on loopback; anyone with local machine access can
   already read `.kineti/` files directly.
2. Authenticated `gate.*` and settings writes accept any key names with
   checked value types. The same user can already edit `.kineti/state.json`
   by hand, so this adds no new power.
3. `install.sh` pins HTTPS and TLS 1.2 but ships no checksum (noted in the
   script itself). Confirm release notes over the same connection.
4. GitHub Actions are pinned to major tags, not exact hashes. Consider
   hash-pinning later.

## Verdict

No critical findings. Five low/medium findings, all fixed and verified:
typecheck clean, full suite plus installer smoke test pass with fresh
evidence (`security-fix-verify`). Gate recommendation: `security: pass`.
