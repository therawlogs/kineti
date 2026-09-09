# Daily and Weekly Guide

This guide shows how to run and maintain Kineti.

## Start a New Task or Project

1. Open your project folder:
   ```sh
   cd <project-folder>
   ```
2. In your agent session, tell the agent what you want to do:
   - For a direct task: `Fix the login bug in auth.ts` or `Refactor database queries`
   - For a full project: `Load kineti-officehours. My idea: ...`

You can start at any step or follow the full sequence:
`officehours` → `diagnose` → `design` → `architecture` → `feasibility` → `spec` → `build` → `review` → `qa` → `security` → `ship` → `watch` → `retro`.

## Resume Saved Work

The current state is saved in `<project>/.kineti/state.json`.

Ask the agent:
> Read .kineti/state.json and tell me the current state and next step.

## Check Money Spent

Check the current cost (use `check` to enforce the cap; `status` is view-only):
```sh
bun "$(cat "$HOME/.kineti/repo")/bin/kineti-spend.ts" check
```

Limits: $50 global, $10 per stage, breaker trips at 95% (~$47.50). If tripped, the run stops. Only a human can reset it:
```sh
bun "$(cat "$HOME/.kineti/repo")/bin/kineti-spend.ts" reset --i-am-human
```

## Undo Changes

To undo changes from a run:
```sh
bun "$(cat ~/.kineti/repo)/bin/kineti-saga.ts" rollback --run-id <id>
```

Undo steps run newest first. If one undo step fails, the remaining steps still run.

## Verify Tests Before Shipping

Run tests and save proof:
```sh
KIN="$(cat "$HOME/.kineti/repo")/bin"
bun "$KIN/kineti-evidence.ts" run --label qa -- bun test
bun "$KIN/kineti-evidence.ts" check --label qa
```

Shipping requires recent passing test proof. Ship is blocked until `security` gate is `pass` and evidence is fresh.

## Weekly Maintenance

Run the weekly check (colon-separated list, quoted for spaces in paths):
```sh
KINETI_PROJECTS="$HOME/projects/a:$HOME/projects/b" \
  "$(cat "$HOME/.kineti/repo")/scripts/weekly.sh"
```

This script:
1. Marks old records as expired.
2. Checks that history records were not changed.
3. Checks that timestamps follow real order.
4. Lists frequently used words to save.

## Search Saved Memory

With gbrain active:
> What do you remember about <topic>?

In the terminal:
```sh
gbrain search "<topic>"
```

## Track a New Project

Run any task in the project. Kineti creates `.kineti/journal.jsonl` automatically.

## Uninstall Kineti

To remove Kineti from your computer:
```sh
cd <kineti-repo> && ./setup.sh --uninstall
```

This removes only Kineti files. Saved notes in `~/.gbrain` stay intact.
