---
name: skillify
description: Package a proven workflow into a new skill.
stage: meta
version: 0.2.0
---

# kineti-skillify

Turn something that worked into a repeatable procedure the agent can run
again. One skill does one job.

## Procedure

1. **Prove it worked twice** before packaging: same steps, similar result,
   in this project or another. One lucky run is not a skill.
2. **Extract the skeleton**: trigger (when to run), inputs, numbered steps,
   outputs, hard rules, harness calls (spend/evidence/saga as relevant).
3. **Write `skills/<name>/SKILL.md`** in this repository following the
   exact frontmatter format of existing skills
   (name, description, stage, version 0.1.0).
4. **Plain-word pass**: remove every word a newcomer would misread.
5. **Add the memory hooks**: what to recall before, what to record after.
6. **Register it**: add to WORKFLOWS.md if it belongs to the pipeline, or
   list under meta skills; re-run `./setup.sh` so all hosts receive it.
7. **First real use is a trial**: annotate anything that needed human
   rescue; fold fixes back within 24 hours.

## Hard rules

- One skill does one job. If the description says "and", split it.
- No skill ships without its hard rules section.

## Memory after

Record the new skill and its first-run notes in the journal.
