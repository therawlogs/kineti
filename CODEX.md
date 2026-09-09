# Kineti OS — Rules for OpenAI Codex

This project uses Kineti OS to keep work safe and organized.

## Rules
1. **Plain words and numbered choices**: Speak in simple, everyday English. Give numbered choices (1, 2, 3).
2. **Check the current task**: Read `.kineti/state.json` to see the current task and goal.
3. **Plan before building new features**: For new features, get user approval before writing code in `src/`. For small fixes, you can fix directly.
4. **Spending limit**: Check costs with `bun bin/kineti-spend.ts check`. Stop right away if spending reaches $50.00 ($10 per stage).
5. **Run tests**: Save proof with `bun bin/kineti-evidence.ts run --label <name> -- <command>` and check with `bun bin/kineti-evidence.ts check --label <name>`.
6. **Save undo steps**: Before big changes, save an undo with `bun bin/kineti-saga.ts push "<undo-command>"`.
7. **Keep files inside the project**: Save all work inside this project folder. Never write to `/tmp` or outside folders.
