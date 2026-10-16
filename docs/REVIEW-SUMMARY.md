# Deterministic reviewer summary

`kineti-review-summary` turns a separate evaluator report into a canonical JSON summary bound to the exact artifact bytes and receipt bytes. The evaluator writes its report separately from the immutable artifact. The builder may package the inputs and run the formatter, but the evaluator owns the per-case results.

```sh
bun bin/kineti-review-summary.ts \
  --artifact artifact.tar \
  --receipt receipt.json \
  --evaluation evaluator-result.json \
  --out review-summary.json
bun bin/kineti-review-summary.ts --verify review-summary.json --expected-digest <digest-retained-by-review-coordinator>
```

The evaluator report is JSON with `evaluator.id`, `evaluator.version`, and a non-empty `results` array. Each result has a unique `id`, a `status` of `pass`, `fail`, or `unsupported`, and a non-empty `reason`. Every `pass` or `fail` result must include both `evidence_path` and `evidence_digest`; unsupported results must include neither. The path is a normalized relative path from the evaluator report's directory, with `/` separators and no absolute path, `.` or `..` component. It must resolve to a regular file within that directory (symlink escapes are rejected). The digest is SHA-256 over the exact file bytes, including its original line endings; the CLI recomputes and compares it before writing the summary. Keep the actual output, log, or other review evidence file alongside the evaluator report and include it in the handoff so another person can inspect and re-hash it. Cases are sorted by ID and object keys use canonical order, so the same inputs produce byte-identical summary JSON.

The summary includes SHA-256 digests of the artifact and receipt, evaluator identity/version, all results, an overall status, and a digest of its canonical contents. Exit codes are stable: `0` pass, `1` at least one failed case, `2` invalid invocation, `4` at least one unsupported case and no failures, `7` invalid input or summary. A separate coordinator should retain the summary digest; `--expected-digest` checks against that independent anchor. The embedded digest alone detects accidental inconsistency, not an attacker who can rewrite both the summary and its digest.

This is deterministic formatting and digest binding. It does not authenticate a real-world reviewer, establish that the evaluator was independent, or prove that evaluator commands ran. Workflow 09 must enforce read-only artifact handoff and retain evaluator results separately.
