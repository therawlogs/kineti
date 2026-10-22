# GitHub source integration

The repository-root `action.yml` is a composite action that submits immutable candidate bytes to the separately operated Linux evaluator. It is source-only: no consumer GitHub run has yet been accepted, and no new release has been published.

Use an owner-controlled workflow from a trusted branch. Pin `uses: OWNER/REPOSITORY@FULL_REVIEWED_COMMIT_SHA`, and pass that same 40-character SHA as `trusted-commit`. Never use a PR-controlled local action (`uses: ./`) in a job holding service credentials. Do not run candidate install scripts or commands in that job. Candidate code executes only in the separate Linux service.

Provision Bun and an SSH tunnel on the runner through trusted setup. Inputs are `trusted-commit`, `client-project`, `manifest` and `artifact`. `client-project` must be owner-only and outside `GITHUB_WORKSPACE`; its `.kineti/service-client.json` and signing key must be separately provisioned as described in [Linux service](LINUX-SERVICE.md). The owner-signed manifest must bind the exact bounded artifact bytes and approved service pins. Untrusted PR metadata must not choose the endpoint, credential directory, action source or authority pins.

The action verifies its exact source ref and source path, checks candidate bytes against the manifest, then submits signed evaluation. Only an accepted, independently verified result succeeds. There is no automatic export, PR comment or publication. Export remains a distinct explicit protected action.

The action guards cannot prove that the calling workflow itself is trusted. Repository rules, protected branches, workflow ownership, secret provisioning and isolation of runner jobs remain owner responsibilities. Live consumer-run acceptance requires a reviewed commit to be available on GitHub and an explicitly authorized workflow run. Local guard tests and service tests are recorded separately from that outstanding gate.
