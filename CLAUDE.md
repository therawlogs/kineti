# Kineti OS — Rules for Claude Code

This project uses Kineti OS to keep work safe and organized. Follow these simple rules:

## Rules
1. **Plain words and numbered choices**: Speak in plain English. Never use metaphors or jargon. When asking the user to choose, give numbered options (1, 2, 3).
2. **Check the current task**: Before doing anything, read `.kineti/state.json` to see the current goal and task.
3. **Plan before building new features**: For new features, show a clear plan and get user approval before writing code in `src/`. For simple bug fixes and small edits, you can fix the issue directly.
4. **Run tests**: When running tests, save the results using `bun bin/kineti-evidence.ts run --label <name> -- <command>`. Never say tests passed without running them.
5. **Spending limit**: Check with `bun bin/kineti-spend.ts check`. If total spending reaches $50.00 ($10 per stage), stop right away and tell the user.
6. **Save undo steps**: Before making big changes, save an undo command using `bun bin/kineti-saga.ts push "<undo-command>"`.
7. **Keep files inside the project**: Save all work inside this project folder. Never write to `/tmp` or outside folders.

## Useful Commands
- Check status: `bun bin/kineti-state.ts get`
- Check costs: `bun bin/kineti-spend.ts check` (`status` is view-only, `check` enforces the cap)
- Check tests: `bun bin/kineti-evidence.ts check --label <name>`
- View dashboard: `http://127.0.0.1:8788` (`bun run companion`, override with `KINETI_COMPANION_PORT`)
