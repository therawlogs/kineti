# Clean-machine setup acceptance

The installer is checked in an isolated home under `.kineti/acceptance`, so
the test does not change the operator's host settings. The check covers all
seven host names in `hosts/*.conf`: Antigravity, Claude Code, Codex, Cursor,
Fx, Gemini, and OpenCode. It checks that each install creates its skill files
and Kineti repository pointer, preserves a user-owned skill, and can be run
again. It also checks uninstall behavior.

Run the installer acceptance test with:

```sh
bun bin/kineti-evidence.ts run \
  --label clean-machine-setup-routing \
  --required-test tests/clean-machine-setup.test.ts \
  -- bun test tests/clean-machine-setup.test.ts
bun bin/kineti-evidence.ts check --label clean-machine-setup-routing
```

The test writes an inspectable status report to
`.kineti/acceptance/clean-machine-setup-routing.json`. Setup is recorded per
host. Natural-language skill routing is marked `unsupported` for every host
until that host can be exercised with an isolated live model runner. Installing
a skill or finding its description is not evidence that a model selected it.

Package-install acceptance is separate: `scripts/clean-install-acceptance.ts`
installs published npm and crate version `0.4.0` as a baseline, then installs
the locally built npm tarball and crate candidate into distinct directories.
The published baseline is not evidence for the candidate. A registry failure
leaves that baseline incomplete; rerun when registry access is available.
