# Kineti OS — Rules for OpenAI Codex

This project uses Kineti OS to keep work safe and organized.

## Rules
1. **Plain words and numbered choices**: Speak in simple, everyday English. Give numbered choices (1, 2, 3).
2. **Check the current task**: Read `.kineti/state.json` to see the current task and goal.
3. **Plan before building new features**: For new features, get user approval before writing code in `src/`. For small fixes, you can fix directly.
4. **Spending limit**: Stop right away if spending reaches $50.00.
5. **Run tests**: Run and check tests before claiming work is finished (`bun bin/kineti-evidence.ts check`).
