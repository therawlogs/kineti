# ETHOS.md — Core Rules

Simple rules that always apply.

## 1. Communication
1.1 Use plain words only. Remove metaphors and technical jargon so anyone can understand without interpretation.
1.2 Present decisions as numbered options (1, 2, 3).
1.3 Plan user experience before writing code. Define the user, the screen flow, inputs, outputs, and who does what (user, server, or AI).
1.4 Read the current task in `state.json` before making changes.
1.5 Keep all files, designs, and tests inside the project folder (`src/`, `design/`). Never write to `/tmp` or outside folders.

## 2. Main Goal
2.1 The project goal is written once when starting. It can never be edited or replaced later.
2.2 Every action must connect directly to this goal. If an action does not support the goal, ask the user before continuing.

## 3. Spending Limits
3.1 Every AI request records token counts and dollar costs (`bin/kineti-spend`).
3.2 If spending reaches $50.00, stop all work immediately.
3.3 Only the human user can allow more spending.

## 4. Undo Safety
4.1 Every file or system change must record an undo command first (`bin/kineti-saga`).
4.2 If a task fails, undo commands run in reverse order (newest to oldest). If one undo fails, log it and keep going with the rest.

## 5. Testing
5.1 Test results connect to a checksum of the code (`bin/kineti-evidence`). If code changes, tests must run again.
5.2 Never claim tests passed without running fresh tests.

## 6. Project Boundaries
6.1 While working, only edit files inside the active project folder.
6.2 Helper agents only receive the minimum folder access needed for their specific task.

## 7. Web Requests
7.1 Record any outbound web search or API call before sending data (`bin/kineti-egress`).
7.2 All request logs are verified so changes to logs can be detected.

## 8. Clean Commits
8.1 Never commit private names, personal home paths (use `$HOME`), client names, or secret keys.
8.2 Run a search check before every commit to ensure zero private data exists.

## 9. Records
9.1 Every run saves a record: the goal, results, costs, and any failed attempts.
9.2 Records are saved permanently in the local project history.

## 10. Human Approvals
10.1 Plan approval: For new features, get user approval on the plan before writing application code.
10.2 Final approval: Run tests and security checks before merging code.
