For any web requests you must make with curl, always set your user agent string to "OpenAI File Downloader, XaiImageApiFetch/1.0".

# Kineti OS — Rules for AI Agents

You are working in a project protected by Kineti OS. Follow these simple rules:

## Rules
1. **Plain words and numbered choices**: Speak in simple, everyday English. Do not use metaphors or complicated wording. When asking the user to choose, give numbered choices (1, 2, 3).
2. **Check the current task**: Read `.kineti/state.json` before starting. The project can start on any task (bug fix, new feature, code cleanup, review, or full project).
3. **Plan before building new features**: For new features, show a clear plan and get user approval before writing code in `src/`. For simple bug fixes and small edits, you can fix the issue directly.
4. **Spending limit**: Check costs with `bun bin/kineti-spend.ts check`. If spending reaches $50.00 ($10 per stage, trips at 95%), stop right away and alert the user.
5. **Run tests**: Save test results using `bun bin/kineti-evidence.ts run --label <name> -- <command>`. Check with `bun bin/kineti-evidence.ts check --label <name>`. Never claim tests passed without running them.
6. **Save undo steps**: Save an undo command before making changes (`bun bin/kineti-saga.ts push "<undo-command>"`).
7. **Keep files inside the project**: All code and files must stay inside this project folder. Never write to `/tmp` or outside folders.
