# WORKFLOWS.md — How Work Flows in Kineti

Kineti supports direct tasks (like fixing a bug or cleaning up code) as well as full project builds.

## 1. Direct Tasks (Start Anywhere)
You do not need to follow every step for small jobs:
- **Bug fix**: Reproduce issue, fix code in `src/`, run tests.
- **Code cleanup**: Clean code, run tests, verify changes.
- **Code review**: Review files, check security, report issues.

Safety rules always run in the background (undo commands, spending limits, test checks).

## 2. Full Project Steps
When building a new project from scratch, follow these steps:

| Step | Name | What it does | Key Rule |
|---|---|---|---|
| 1 | Office hours | Define the main goal and user flow | Clearly explain the problem first |
| 2 | Diagnose | Measure costs and find slow points | Show the math for any cost claim |
| 3 | Design | Create screen mockups and layout styles | Review mockups before writing code |
| 4 | Architecture | Choose libraries, database, and API shape | Compare options with clear numbers |
| 5 | Cost & limits check | Check costs and API rate limits | Stop if costs are too high |
| 6 | Plan approval | Write clear plan and test list | STOP: You must approve before code is written |
| 7 | Build | Write code in small pieces with tests | Save an undo command before each change |
| 8 | Review | Look for edge cases and errors | No new features during review |
| 9 | QA | Test on desktop and mobile viewports | Run tests and take screenshots |
| 10 | Security check | Check for common security flaws | Fix all serious flaws before moving on |
| 11 | Final approval | Clean commits and pull request | All tests must pass |
| 12 | Monitoring | Watch errors and response speed | Record normal numbers before release |
| 13 | Review lessons | Record what went well and what failed | Save lessons to help future tasks |

## 3. Programs Used
- State and tasks: `kineti-state.ts`
- Costs and spending limit ($50): `kineti-spend.ts`
- Undo safety: `kineti-saga.ts`
- Test checking: `kineti-evidence.ts`
- Web requests: `kineti-egress.ts`
- Companion dashboard: `kineti-companion.ts` (`http://127.0.0.1:8788`)
