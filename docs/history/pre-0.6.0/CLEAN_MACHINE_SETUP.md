# Clean-machine setup acceptance

Use isolated folders to check that the published packages and local candidates install correctly. Keep the results separate from host or model behavior, which these install checks cannot prove.

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
requires `KINETI_BASELINE_VERSION`, `KINETI_NPM_TARBALL` and `KINETI_CRATE_PATH` as explicit inputs, then installs
the locally built npm tarball and crate candidate into distinct directories.
The published baseline is not evidence for the candidate. A registry failure
leaves that baseline incomplete; rerun when registry access is available.
