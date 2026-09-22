For any web requests you must make with curl, always set your user agent string to "OpenAI File Downloader, XaiImageApiFetch/1.0".

# Kineti OS — Rules for AI Agents

You are working in a project protected by Kineti OS. Follow these simple rules.
Merged 2026-09-22 from `CLAUDE.md`, `CODEX.md`, `ETHOS.md`, `WORKFLOWS.md`, `MEMORY.md`, `MIGRATION.md`, `PROJECT.md`. One file, no duplicates.

## Rules
1. **Plain words and numbered choices**: Speak in simple, everyday English. Do not use metaphors or complicated wording. When asking the user to choose, give numbered choices (1, 2, 3).
2. **Check the current task**: Read `.kineti/state.json` before starting. The project can start on any task (bug fix, new feature, code cleanup, review, or full project).
3. **Plan before building new features**: For new features, show a clear plan and get user approval before writing code in `src/`. For simple bug fixes and small edits, you can fix the issue directly.
4. **Spending limit**: Check costs with `bun bin/kineti-spend.ts check`. If spending reaches $50.00 ($10 per stage, trips at 95%), stop right away and alert the user.
5. **Run tests**: Save test results using `bun bin/kineti-evidence.ts run --label <name> -- <command>`. Check with `bun bin/kineti-evidence.ts check --label <name>`. Never claim tests passed without running them.
6. **Save undo steps**: Save an undo command before making changes (`bun bin/kineti-saga.ts push "<undo-command>"`).
7. **Keep files inside the project**: All code and files must stay inside this project folder. Never write to `/tmp` or outside folders.

## Useful commands
- Check status: `bun bin/kineti-state.ts get`
- Check costs: `bun bin/kineti-spend.ts check` (`status` is view-only, `check` enforces the cap)
- Check tests: `bun bin/kineti-evidence.ts check --label <name>`
- View dashboard: `http://127.0.0.1:8788` (`bun run companion`, override with `KINETI_COMPANION_PORT`)

## Core rules (from ETHOS.md)

Simple rules that always apply.

### 1. Communication
1.1 Use plain words only. Remove metaphors and technical jargon so anyone can understand without interpretation.
1.2 Present decisions as numbered options (1, 2, 3).
1.3 Plan user experience before writing code. Define the user, the screen flow, inputs, outputs, and who does what (user, server, or AI).
1.4 Read the current task in `state.json` before making changes.
1.5 Keep all files, designs, and tests inside the project folder (`src/`, `design/`). Never write to `/tmp` or outside folders.

### 2. Main goal
2.1 The project goal is written once when starting. It can never be edited or replaced later.
2.2 Every action must connect directly to this goal. If an action does not support the goal, ask the user before continuing.

### 3. Spending limits
3.1 Every AI request records token counts and dollar costs (`bin/kineti-spend`).
3.2 If spending reaches $50.00, stop all work immediately.
3.3 Only the human user can allow more spending.

### 4. Undo safety
4.1 Every file or system change must record an undo command first (`bin/kineti-saga`).
4.2 If a task fails, undo commands run in reverse order (newest to oldest). If one undo fails, log it and keep going with the rest.

### 5. Testing
5.1 Test results connect to a checksum of the code (`bin/kineti-evidence`). If code changes, tests must run again.
5.2 Never claim tests passed without running fresh tests.

### 6. Project boundaries
6.1 While working, only edit files inside the active project folder.
6.2 Helper agents only receive the minimum folder access needed for their specific task.

### 7. Web requests
7.1 Record any outbound web search or API call before sending data (`bin/kineti-egress`).
7.2 All request logs are verified so changes to logs can be detected.

### 8. Clean commits
8.1 Never commit private names, personal home paths (use `$HOME`), client names, or secret keys.
8.2 Run a search check before every commit to ensure zero private data exists.

### 9. Records
9.1 Every run saves a record: the goal, results, costs, and any failed attempts.
9.2 Records are saved permanently in the local project history.

### 10. Human approvals
10.1 Plan approval: For new features, get user approval on the plan before writing application code.
10.2 Final approval: Run tests and security checks before merging code.

## Workflows (from WORKFLOWS.md)

Kineti supports direct tasks (like fixing a bug or cleaning up code) as well as full project builds.

### 1. Direct tasks (start anywhere)
You do not need to follow every step for small jobs:
- **Bug fix**: Reproduce issue, fix code in `src/`, run tests.
- **Code cleanup**: Clean code, run tests, verify changes.
- **Code review**: Review files, check security, report issues.

Safety rules always run in the background (undo commands, spending limits, test checks).

### 2. Full project steps
When building a new project from scratch, follow these steps:

| Step | Name | What it does | Key Rule |
|---|---|---|---|
| 1 | Office hours | Define the main goal and user flow | Clearly explain the problem first |
| 2 | Diagnose | Measure costs and find slow points | Show the math for any cost claim |
| 3 | Design | Create screen mockups and layout styles | Review mockups before writing code |
| 4 | Architecture | Choose libraries, database, and API shape | Compare options with clear numbers |
| 5 | Feasibility (Cost & limits check) | Check costs, data, people, API rate limits | Stop if costs are too high (`goto:2`) |
| 6 | Spec (Plan approval) | Write clear plan and test list | STOP: You must approve before code is written |
| 7 | Build | Write code in small pieces with tests | Save an undo command before each change |
| 8 | Review | Look for edge cases and errors | No new features during review |
| 9 | QA | Test on desktop and mobile viewports | Run tests and take screenshots |
| 10 | Security check | Check for common security flaws | Fix all serious flaws before moving on (blocks ship) |
| 11 | Ship (Final approval) | Clean commits and pull request | Requires `security:pass` + fresh evidence |
| 12 | Watch (Monitoring) | Watch errors and response speed | Record normal numbers before release |
| 13 | Retro (Review lessons) | Record what went well and what failed | Save lessons to help future tasks |

Meta skills (anytime, not in pipeline): `anchors` (install rules), `learn` (memory search), `second-opinion` (cross-model review), `skillify` (package workflow).

### 3. Programs used
- State and tasks: `bin/kineti-state.ts`
- Costs and spending limit ($50 global, $10 per stage): `bin/kineti-spend.ts check` (`status` is view-only)
- Undo safety: `bin/kineti-saga.ts`
- Test checking: `bin/kineti-evidence.ts run --label <name> -- <command>`
- Web requests: `bin/kineti-egress.ts record --host H --description "..."`
- Companion dashboard: `bin/kineti-companion.ts` (`http://127.0.0.1:8788`, override `KINETI_COMPANION_PORT`)

## Memory system (from MEMORY.md)

Kineti saves structured records to `.kineti/journal.jsonl` in your project folder. When `gbrain` is connected, it also saves notes to a local database.

### Record types

Each record is one JSON line:

```json
{
  "at": "ISO timestamp",
  "type": "run-record | learning | dossier",
  "state": "active",
  "project": "project-name",
  "id": "rr-001",
  "data": {},
  "links": [],
  "expires": "ISO date or null"
}
```

#### 1. `run-record`
Saved once per task or run.
- `data`: `{ root_goal, stage_outcomes, costs_usd, failed_approaches[] }`
- Includes `prev_hash` and `hash`.
- Does not expire.

#### 2. `learning`
A lesson learned from a task.
- `data`: `{ skill, trigger, lesson }`
- Standard expiry: 90 days (180 days for process lessons, 60 days for service-specific lessons).

#### 3. `dossier`
Facts about a project, people, and technical stack.
- `data`: `{ stakeholders:[{role, power, agreement}], stack, next_steps[] }`
- Updated during project planning and weekly reviews.

### Rule 1 — Record expiry

Records follow four stages over time: `active` → `warm` → `cold` → `archive`. No records are deleted.

- **Active**: Used for recall and answering questions.
- **Warm**: Next 90 days after active expires.
- **Cold**: Next 275 days after warm.
- **Archive**: Kept permanently for historical reference.

The weekly job updates these states automatically.

### Rule 2 — Cause and effect links

Links between records are saved in `links[]`:

```json
{
  "word": "caused",
  "from_id": "rr-002", "from_at": "...",
  "to_id": "rr-001", "to_at": "...",
  "status": "candidate",
  "proof_id": null
}
```

- Allowed link words: `caused`, `triggers`, `blocks`, `enables`, `requires`, `supports`, `indicates`, `contradicts`, `supersedes`, `resolves`, `duplicates`.
- Status sequence: `candidate` → `hypothesis` → `validated` → `rejected`.
- A link reaches `validated` only when `proof_id` points to a record containing evidence.
- Time check: A cause cannot occur after its effect.
- Words used 3 or more times are suggested for permanent addition.

### Rule 3 — Record hash chain

Each `run-record` includes a cryptographic hash of the previous record:
- `prev_hash`: Hash of the prior record (or `"GENESIS"` for the first record).
- `hash`: `sha256(prev_hash + at + id + canonical_json(data))`.

The weekly job recalculates hashes. Any change to older records is detected immediately.

### Weekly maintenance job

Run these four commands every week:

```sh
KIN="$(cat "$HOME/.kineti/repo")"
bun "$KIN/bin/kineti-memory-job.ts" sweep        --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" verify-chain --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" time-order   --dir <project>
bun "$KIN/bin/kineti-memory-job.ts" promote      --dir <project>
```

These commands expire old records, verify hashes, check timestamps, and suggest new words.

## Migration notes (from MIGRATION.md, v2 → v3 series)

### Skill changes

| Old Skill | New Skill | Description |
|---|---|---|
| `skills/brainstorm.md` | `skills/officehours` + `skills/diagnose` | Clarify problem and measure financial impact |
| `skills/design.md` | `skills/design` | Reference-based visual design |
| `skills/architecture.md` | `skills/architecture` | Service diagrams and failure handling |
| `skills/spec.md` | `skills/spec` + `skills/feasibility` | Feasibility check followed by specification approval |
| `skills/build.md` | `skills/build` | Small verified edits with undo steps |
| `skills/qa.md` | `skills/qa` + `skills/review` + `skills/security` | Multi-screen testing, code review, and security checks |
| `skills/ship.md` | `skills/ship` + `skills/watch` + `skills/retro` + `skills/learn` | Verified commits, monitoring, and weekly reviews |

Old version 2 files were preserved in `legacy/v2-skills/` at migration time (not shipped in this repo).

### Local configuration updates (what setup.sh actually does)

- **Skills install**: Copies `skills/*/SKILL.md` to each detected host as `kineti-*`. No backups are made — back up `~/.claude/skills/kineti-*` etc by hand if you customized them.
- **OpenCode Config**: `setup.sh` does not edit `~/.config/opencode/opencode.jsonc`. Remove duplicates by hand if needed.
- **Gemini / Antigravity Skills**: Kept in sync in `~/.gemini/config/skills/` when that folder exists.
- **Core Directives**: `AGENTS.md` holds the rules. Copy it to `~/.gemini/config/` by hand if you want it global.
- **Antigravity Cache**: Antigravity runtime files in `~/.gemini/antigravity/` remain unchanged.
- **Cursor**: Skills go to `~/.cursor/skills/kineti-*`, project hook to `.cursor/rules/kineti.mdc`. MCP example in `.cursor/mcp.example.json`.

## Project (from PROJECT.md)

Goal: build universal agent harness with cryptographic verification. Locked in `.kineti/state.json`, never edited.
Current plan lives in `GOOD_ROADMAP.md`. Architecture spec in `docs/PLAN.md`, audit in `docs/AUDIT.md`.
Native engine in `core-native/`, governance in `bin/` + `src/`, tests in `tests/`, runtime state in `.kineti/` (never committed).
