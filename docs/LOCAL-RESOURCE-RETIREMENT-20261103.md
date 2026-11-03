# Local resource retirement and source checkpoint

Date: 2026-11-03. The owner explicitly authorized deletion of unused VMs, the macOS installer and obsolete backups, while preserving source, work and accounting, and authorized a GitHub commit. This does not authorize merging, a package release or deployment.

Removed resources:

1. Obsolete stopped Linux guests `kineti` and `kineti-stop-20261102`, using official Lima deletion after preservation checks.
2. The 19.77 GB downloaded macOS installer. The installed guest disk, identity and configuration remain intact. A fresh cold boot of `kmac3` without the installer succeeded, reached stock boot completion and stopped through the native owner channel.
3. The obsolete full-checkout archive and discarded-build-cache recovery archive. These resources were explicitly retired permanently; their bytes cannot be restored automatically. Existing historical undo entries were not rewritten. Compact original historical records and resource hashes are retained locally, with this retirement recorded alongside them.
4. Obsolete Rust compile caches/libraries from the default and previous macOS task-service targets, and the unused Node dependency directory. The npm wrapper declares no third-party runtime or development dependencies. Current owner-service build/dependency caches and required historical executables remain.

The project measured approximately 68.1 GB allocated before retirement and 37.0 GB afterward, a reduction of about 31.1 GB. Figures use whole-project `du` measurements; summing file sizes or sparse-disk capacities gives different numbers. Final small logs and Git objects can change these figures slightly.

Kept resources:

1. `kmac3`, the provisioned macOS guest needed for the unfinished worker/storage/resource integration, and `kineti-autonomy`, the Linux regression guest. Both are stopped after this batch. Their disk allocations account for about 29.1 GB of the remaining project space.
2. Current source and work, original accounting records and signatures, the locked goal, Git history, compact historical work/evidence, required SDK/native artifacts and offline dependency sources.
3. All 4,909 protected owner-store files from the obsolete clone were verified byte-identical in the retained Linux guest. The two old guests contained the same 240 reviewed synthetic source/record files, preserved separately. Before removing the old full-checkout archive, 298 unique historical files were preserved in a verified compact archive; 8,248 additional files matched current bytes. No credential values were exported from the guests. A broad export was rejected by automatic approval review; a reviewed, hash-bound source/record list was used instead.

Evidence is private project-local runtime state under `.kineti/retirement-20261103/` and is not committed. It includes exact removal plans and hashes, compact archive verification, owner-store preservation, VM lifecycle commands, disk measurements, native accounting assessment and command logs. Authoritative work/accounting preservation is checked independently of the retired caches and guest disks. Historical valuations remain unresolved; none were reset or priced.

Fresh local verification: 36 Rust library and 62 integration tests passed in UTC and Asia/Kolkata; formatting and Clippy passed. The release-controller prototype passed 44 tests. The mocked installer policy suite initially failed because registry-retirement metadata and its old test expectations disagreed. The description was shortened to the existing schema limit and the test now requires an empty retired package list and private package status, retaining the installer checksum/sentinel cases. Mocked payload checks do not establish installed-package acceptance.

The source checkpoint includes the existing unfinished owner-service work and website-only distribution restrictions. The macOS task backend still fails closed; this cleanup is not full contract acceptance. Root and website `vercel.json` files disable automatic Git deployments, preserving the separate release gate when this checkpoint branch is pushed. The native validation workflows are manual or target other branch names; no workflow is dispatched as part of this checkpoint. No new provider login, real model turn, paid resource, merge, publication or deployment is authorized or performed by this batch.
