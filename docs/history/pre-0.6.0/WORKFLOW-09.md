# Workflow 09: Independent evidence review

Use a separate evaluator and keep its findings tied to the exact artifact reviewed. The builder must not write or edit the evaluator’s pass/fail report. This guide describes the handoff; it does not supply an enforced independent evaluator.

## Freeze and hand off

```sh
RUN_ID="<receipt-run-uuid-from-the-candidate-test>"
RUN_DIR=".kineti/workflow-09/$RUN_ID"
mkdir -p "$RUN_DIR"
git ls-files -co --exclude-standard -z | tar -cf "$RUN_DIR/artifact.tar" --null -T -
chmod 0444 "$RUN_DIR/artifact.tar"
bun -e 'const [ledger,id,out]=process.argv.slice(1); const rows=(await Bun.file(ledger).text()).split(/\n/).filter(Boolean).map(JSON.parse); const row=rows.find((r)=>r.run_id===id); if(!row) throw new Error("receipt run id not found"); await Bun.write(out,JSON.stringify(row)+"\n")' \
  .kineti/evidence.jsonl "$RUN_ID" "$RUN_DIR/receipt.json"
shasum -a 256 "$RUN_DIR/artifact.tar" "$RUN_DIR/receipt.json" > "$RUN_DIR/handoff.sha256"
```

Use the run ID from the candidate test receipt; do not substitute a newly generated review ID.

Record SHA-256 values for `artifact.tar` and `receipt.json` in the generated `handoff.sha256` manifest. Give the evaluator the archive, manifest, receipt, acceptance checklist, and commands. Provide read-only access to the archive and receipt; the evaluator writes its report in a separate location. The evaluator recomputes both input digests before review. A changed input invalidates the run.

## Evaluator report and summary

The evaluator writes its own JSON report with `evaluator.id`, `evaluator.version`, and a non-empty `results` array. Each case has a unique ID, status `pass`, `fail`, or `unsupported`, and a reason. Include inspectable command output or an evidence digest for each result. The evaluator must not accept builder-supplied pass/fail values.

After receiving the evaluator report, generate the canonical summary from the same immutable files:

```sh
bun bin/kineti-review-summary.ts \
  --artifact "$RUN_DIR/artifact.tar" \
  --receipt "$RUN_DIR/receipt.json" \
  --evaluation /path/to/evaluator-result.json \
  --out "$RUN_DIR/review-summary.json"
bun bin/kineti-review-summary.ts --verify "$RUN_DIR/review-summary.json" \
  --expected-digest <digest-retained-by-independent-coordinator>
```

The coordinator retains the summary digest separately from the builder's workspace. The canonical summary binds the artifact and receipt byte digests, evaluator ID/version, and sorted per-case results. A failed case produces `fail`; an unrun or unsupported check remains `unsupported`. Neither is a passing review. If any input changes, freeze a new artifact and repeat the review.

The summary is structured review evidence. It does not authenticate a person's identity or prove that a different model or window was used; the handoff process and coordinator record provide that separation. If the evaluator cannot run a host, revocation, replay, or containment case, it must mark the case unsupported.
