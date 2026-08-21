# How-To — Daily and Weekly Operation

## Start a new product run

```sh
cd <project-folder>       # empty or existing repo
# in your agent session:
Load kineti-officehours. My idea: ...
```
The pipeline owns the order from there: diagnose → design → architecture →
feasibility → spec (your approval) → build → review → qa → security → ship.

## Resume after a break

State lives in `<project>/.kineti/state.json`. Ask the agent:
> Read .kineti/state.json and tell me where this run stands and what is next.

## Check money mid-run

```sh
bun "$(cat ~/.kineti/repo)/bin/kineti-spend.ts" status
```
Tripped? Only you may reset it:
`... kineti-spend.ts reset --i-am-human`

## Undo a bad build session

```sh
bun "$(cat ~/.kineti/repo)/bin/kineti-saga.ts" rollback --run-id <id>
```
Undo steps run newest-first; one failing undo does not stop the rest.

## Prove tests before ship

```sh
K=$(cat ~/.kineti/repo)/bin
bun $K/kineti-evidence.ts run --label qa -- -- bun test
bun $K/kineti-evidence.ts check --label qa     # FRESH required by ship
```

## Weekly memory maintenance

```sh
KINETI_PROJECTS="$HOME/projects/a $HOME/projects/b" \
  $(cat ~/.kineti/repo)/scripts/weekly.sh
```
Or put that line in crontab. It expires stale records, verifies every
project's hash chain, checks cause-link time order, and suggests new
vocabulary worth promoting.

## Ask what Kineti remembers

With gbrain connected (opencode sessions have it):
> What do you remember about <topic>?

Without a session:
```sh
gbrain search "<topic>"
```

## Add a new project to memory tracking

Run any pipeline stage once in it — `.kineti/journal.jsonl` appears
automatically and weekly.sh picks it up via KINETI_PROJECTS.

## Retire a machine

```sh
cd <kineti-repo> && ./setup.sh --uninstall   # removes only kineti-* files
```
Memory survives independently in `~/.gbrain` and your brain repository.
