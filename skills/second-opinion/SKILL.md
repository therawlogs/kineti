---
name: second-opinion
description: Independent bug review from a different model, when one is available.
stage: verify
version: 0.1.0
triggers:
  - second opinion
  - another model review
  - cross-model check
---

# kineti-second-opinion

A different vendor's model reads the same diff looking only for bugs.
Agreement between models raises severity; disagreement gets recorded.

## Find a reviewer

Check in order; use the first that exists:

```sh
command -v codex   && REVIEWER="codex exec"          # OpenAI Codex CLI
command -v gemini  && REVIEWER="gemini -p"           # Gemini CLI
command -v grok    && REVIEWER="grok"                # xAI Grok Build
```

None found → say so plainly and stop. Never fake a second opinion.

## Procedure

1. Collect the same diff the review stage saw:
   `git diff <base>...HEAD` (or the review chunk).
2. Send it with this exact instruction appended:
   "Find bugs only. No style comments. For each finding: file, line,
   what breaks, and the input that triggers it. If you find none, say
   'no findings'."
3. Record egress before sending: code leaves the machine.
   `bun "$K/kineti-egress.ts" record --host <vendor> --description "diff review by second model"`
4. Merge with the in-house review findings:
   - Found by both → severity up one level.
   - Only here → verify by hand before adding; hallucinated findings
     from either reviewer are discarded with a note.
5. Append a section to `review.md`: reviewer identity, findings kept,
   findings discarded and why.

## Hard rules

- Bugs only. Style, naming, and structure comments are out of scope.
- The diff never ships to a vendor without an egress receipt.
- No reviewer available is a normal outcome — report and continue.

## Memory after

Note which reviewer caught what; repeated unique catches justify keeping
that reviewer in the loop.
