# Kineti 1.0.0 — Open-Core & Founding Team Pilot Playbook

**Date:** November 23, 2026  
**Product:** Kineti `1.0.0` — AI-PR Verification & Anti-Tampering Gate  
**License / Delivery Posture:** **Open-Core (MIT CLI & GitHub Action) + Paid B2B Founding Team Pilot** (CLI, `action.yml`, and offline demo are MIT-licensed; paid pilot covers hands-on Week 1 org rollout, policy design, custom rule/parser tuning, and direct founder support)  
**Contact:** `hello@getkineti.com`

---

## 1. One-Page Founding Team Pilot Offer

### Who This Is For
Engineering organizations (10–150 engineers) where developers actively open pull requests authored or co-authored by AI coding agents (Claude Code, Codex CLI, Cursor, GitHub Copilot Workspace) and need a tamper-resistant merge gate on **Linux GitHub Actions**.

### The Concrete Failure Modes Kineti Stops
Standard CI trusts the pull request branch’s own test files and runner scripts. When an AI coding agent hits a stubborn test failure, standard CI turns green if the agent:
1. **Weakens policy or workflows** (edits `.kineti/gate.json`, `.github/workflows/**`, `pytest.ini`, or `jest.config.*`).
2. **Silently skips or deletes tests** (adds `@pytest.mark.skip`, `@pytest.mark.xfail`, `it.skip`, `test.only`, `#[ignore]`, `t.Skip`, deletes a test function, or appends `--passWithNoTests`).
3. **Claims tests passed locally before a final edit** (even a 1-byte edit inside `src/legacy/` invalidates prior evidence with exit `4`).

Kineti runs offline on your Linux GitHub Actions runner (`permissions: contents: read`), loads `.kineti/gate.json` strictly from the **protected base branch**, scans the PR diff across Rust, Python, JS/TS, and Go, runs exact base-branch allowlisted test commands under POSIX process-group supervision, and emits an Ed25519-signed `kineti-pr-audit-v1` JSON compliance bundle plus a formatted GitHub Step Summary table.

### Open-Core & Founding Team Pilot Pricing

| Option | Price / License | Scope | Best For |
| --- | --- | --- | --- |
| **Open-Core CLI & Action** | **Free (MIT License)** | Full native Rust `kineti` binary, `kineti pr-check`, `action.yml`, polyglot diff scanner, Ed25519 audit packs, offline 4-scene demo | Individual engineers and self-serve teams evaluating or rolling out on their own |
| **Founding Team Setup** | **$1,500 one-time** | Fixed-scope Week 1 hands-on rollout across your GitHub org (unlimited repos during pilot), base-branch `.kineti/gate.json` + CODEOWNERS policy design, audit-pack wiring, + 30 days of direct founder tuning | Teams wanting a fast, zero-disruption org rollout without burning internal staff-engineer cycles |
| **Design-Partner Pilot** | **$299 / month per org** | Ongoing org support across unlimited repos during pilot, prebuilt Linux `x86_64` / `aarch64` binaries, rule/parser tuning, priority CI hotfixes, cancel anytime with 30 days’ notice | Teams wanting continuous pilot tuning and direct founder engineering access |

**Zero-Risk Execution Guarantee:** The MIT binary runs 100% offline inside your own Linux GitHub Actions runners. Zero source code leaves your CI environment, zero SaaS dependencies, and removing the composite action step immediately restores pre-Kineti CI behavior.

---

## 2. Week 1 Onboarding Plan

| Day | Milestone | Concrete Deliverable |
| --- | --- | --- |
| **Day 1** | 30-min architecture kickoff | Select 1–3 high-velocity repos; inventory exact test commands (`cargo test`, `bun test`, `pytest`, `jest`, `vitest`, `go test`, `node --test`, `make test`), `required_tests`, and `protected_paths`. |
| **Day 2** | Base-branch policy + advisory CI | Commit `.kineti/gate.json` on protected `main`; add `action.yml` workflow in non-blocking advisory mode to observe real PR traffic with zero merge disruption. |
| **Day 3** | Tamper drill & demo validation | Run offline pilot demo (`examples/pilot-demo/run-demo.sh`) plus 4 shadow PRs (policy edit, skip marker, deleted test, clean fix) to verify zero false positives. |
| **Day 4** | Enforce branch protection & audit archive | Flip GitHub branch protection rule to require the Kineti status check; archive `kineti-pr-audit-v1` JSON artifacts and verify `$GITHUB_STEP_SUMMARY` tables. |
| **Day 5** | Multi-repo rollout & override runbook | Roll out to remaining pilot repos and hand over the CODEOWNERS override runbook (`--allow-policy-change` / `--allow-test-reduction` only on explicit human sign-off). |

---

## 3. Realistic Solo-Founder Support SLA

Kineti is founder-built and founder-supported during the pilot. We set commitments we can actually keep:

- **Support hours:** Monday–Friday business hours via `hello@getkineti.com` and an optional shared Slack Connect / async thread.
- **Blocking CI regressions / false positives:** Initial response within **1 business day**; priority hotfix binary or config workaround when an active merge queue is affected.
- **Policy design / parser requests:** Response within **2 business days**.
- **Private security disclosures:** Acknowledgement within **5 business days** via `security@getkineti.com` (aligned with `SECURITY.md`).
- **No fabricated 24/7 enterprise SLA:** Because Kineti runs offline on your own GitHub Actions runners with zero external runtime dependency, Kineti itself cannot experience a hosted SaaS outage.

---

## 4. Buyer Objection-Handling FAQ (Honest Answers Only)

### Q1: "Why not just use standard GitHub Actions CI?"
Standard CI executes whatever test files and test scripts are on the PR branch. If an AI coding agent adds `@pytest.mark.skip`, deletes three failing tests, or appends `--passWithNoTests`, standard CI happily turns green. Kineti loads `.kineti/gate.json` strictly from the protected **base branch** (`git show <base>:.kineti/gate.json`) and blocks PR diffs that tamper with policy or test coverage.

### Q2: "Why not enforce this with Claude Code / Codex local hooks or `CLAUDE.md` rules?"
Prompt rules and local hooks run inside the same developer account and agent context—an agent or developer with repo write access can ignore prompts, edit a file after tests run, or bypass local hooks. Kineti’s real trust boundary is your **protected Linux CI runner**, outside the agent’s write access.  
*(Honesty note: Kineti does not claim automatic local interception, and Cursor/Antigravity MCP integrations remain unverified.)*

### Q3: "Does Kineti sandbox our AI agents on developer laptops?"
No. Kineti is not a VM, container, or kernel sandbox, and it provides no protected writes or same-account adversary boundary. Local `kineti test run` / `test check` commands help catch accidental staleness before pushing, while real merge enforcement happens on your protected CI runner.

### Q4: "Does the signed `kineti-pr-audit-v1` bundle prove our tests are high quality or grant SOC 2 compliance?"
No—and we explicitly encode those limits in the JSON schema itself:
- Every audit bundle carries `"independent_assertion_quality_proven": false`. An Ed25519 signature proves **record integrity relative to the signing key**, not that test assertions are meaningful or cannot fabricate passing output.
- Count-aware verification (`--parser auto`) applies to `cargo test` and multi-suite `bun test`. Allowlisted polyglot runners (`pytest`, `jest`, `vitest`, `go test`, `node --test`, `make test`) run with `--parser exit-code` and explicitly record `"count_observed": false` alongside exit status, exact `argv`, required test path presence, and tree freshness.
- What `kineti-pr-audit-v1` *does* provide for SOC 2 / change-control reviews is a tamper-evident, machine-verifiable record binding base/head SHAs, base-branch policy SHA-256, PR-diff anti-tampering findings, and supervised test receipts.

### Q5: "Does our source code or test output leave our GitHub Actions runner?"
No. The Rust binary is 100% offline, performs zero telemetry or outbound network calls, and runs with `permissions: contents: read` inside your own Linux runner.

---

## 5. Outreach Templates (Non-Spammy, Technical, Direct)

### Template 1 — Cold Email (CTO / VP or Head of Engineering)

**Subject:** Stopping AI PRs from adding `it.skip` / weakening CI

Hi [Name],

Quick engineering question: as your team ships more PRs with Claude Code, Codex, or Cursor, how are you stopping agents from "fixing" red builds by adding `@pytest.mark.skip` / `it.skip` / `#[ignore]`, deleting tests, or tweaking workflow configs?

Standard CI turns green on all of those because it trusts the PR branch.

I built **Kineti** (open-core MIT), a single offline Rust binary + Linux GitHub Action (`permissions: contents: read`) that:
- Loads `.kineti/gate.json` strictly from your protected base branch (blocks PR edits to gate rules or `.github/workflows/**`).
- Scans PR diffs across Rust, Python, JS/TS, and Go for added skip/ignore markers, deleted tests, and weakening runner flags.
- Runs your exact allowlisted test commands and outputs an Ed25519-signed `kineti-pr-audit-v1` JSON bundle + GitHub Step Summary.

Nothing leaves your CI runner, the core CLI/Action is MIT-licensed, and we’re explicit about limits (signatures prove record integrity, not assertion quality).

I’m opening 3 **Founding Team Pilots** this month (**$1,500 one-time Week 1 hands-on org rollout** or **$299/mo per org** ongoing pilot support, unlimited repos during pilot). Happy to send a 15-second offline terminal demo or walk through it on one of your repos—worth a look?

Best,  
[Your Name]  
`hello@getkineti.com` · `https://getkineti.com`

---

### Template 2 — LinkedIn / X Direct Message

Hey [Name] — saw you’re scaling AI-assisted PRs at [Company]. Curious if you’ve run into agents slipping `it.skip` / `@pytest.mark.skip` / deleted tests or workflow tweaks past standard CI yet?

I built **Kineti**, an open-core (MIT) offline Linux GitHub Action in Rust that enforces base-branch test policy, catches polyglot skip/deletion tampering in PR diffs, and emits signed `kineti-pr-audit-v1` bundles with zero code egress. The CLI is MIT-licensed, and I’m offering Founding Team Pilots (`$1,500` hands-on Week 1 rollout or `$299/mo` org support). Happy to share a quick 15-second CLI demo if useful—no deck.

---

### Template 3 — Warm Intro (Forwardable Blurb)

**Subject:** Intro: [Your Name] (Kineti — tamper-resistant CI merge gate for AI PRs)

Hey [Mutual Contact], thanks for offering to connect me with [Target Name]! Here’s a quick forwardable blurb:

> [Target Name] — [Your Name] built **Kineti**, an open-core (MIT) offline Rust CLI + Linux GitHub Action for engineering teams using Claude Code / Codex / Cursor. It fixes a blind spot in standard CI: locking test policy to the protected base branch, blocking AI PRs that add `@pytest.mark.skip` / `it.skip` / `#[ignore]`, delete tests, or weaken `.github/workflows/**`, and outputting signed `kineti-pr-audit-v1` audit summaries inside your own runner with zero SaaS telemetry.
>
> The core CLI/Action is MIT-licensed, and they’re onboarding Founding Team Pilots for hands-on Week 1 org rollout (`$1,500` fixed setup or `$299/mo` per org, unlimited repos during pilot) with a 15-second offline demo. Worth a quick intro?
