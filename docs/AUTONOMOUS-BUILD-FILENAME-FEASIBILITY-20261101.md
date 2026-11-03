# Filename exclusions: supported-control feasibility

Date: 2026-11-01. Status: native macOS enforcement verified offline; no compatible drop-in solution established for the current Linux owner service. Production source and grants were not changed.

## Finding

A supported mechanism exists in the stock macOS Codex permission profile, enforced by Seatbelt. In the tested cached macOS arm64 Codex 0.164.0, a writable work root with deny globs blocked new matching filenames before creation while ordinary new files and atomic replacement remained usable. No custom sandbox, syscall supervisor, filesystem implementation, watcher or post-write rollback was used.

This does not make the Linux grant service accepted. Its pinned host remains Linux Codex 0.166.0, and its independently tested owner lifecycle uses Linux cgroups. The macOS tests establish the command-permission primitive only, not a macOS owner service, descendant supervision, authenticated model task, direct-model file tool, installed product or release. No runtime was switched and no Linux grant was widened or migrated.

## Documented support and limits

The official permission documentation describes named filesystem profiles and native platform enforcement. It explicitly says Linux deny-glob matches are collected at each sandboxed process startup; later matching files are not added to that running process. A fresh process rescans, but does not prevent a process from creating and reading a new match during its own lifetime. The macOS backend uses Seatbelt. These facts explain the observed platform difference without implying that every future host or configuration has been tested. Source: [OpenAI permissions](https://learn.chatgpt.com/docs/permissions), accessed 2026-11-01.

The tested macOS shape is:

```toml
[permissions.filename-test.filesystem]
":minimal" = "read"
"/approved/work" = "write"
"/approved/work/**/.env*" = "deny"
"/approved/work/**/*.key" = "deny"
"/approved/work/**/*.pem" = "deny"
"/approved/work/**/*.private" = "deny"

[permissions.filename-test.network]
enabled = false
```

This is a reviewable example, not an installed configuration or authorization. The saved actual argv/profile binds the synthetic paths and cached executable. The tested CLI syntax is `codex sandbox -P filename-test -C <fixture> ... -- <command>`; newer documentation examples were not assumed to match this older cached CLI.

## Offline observations

| Candidate | Result |
| --- | --- |
| Linux 0.166.0, writable root plus deny globs | Newly created `.env.new`, `new.key` and `nested/.env.new` could be written and read within the same process. Ordinary file creation and replacement worked. The exclusion requirement failed. |
| Linux read-only parent plus exact existing-file write grants | Sandbox startup failed with a synthetic `.git`/`.aws` mount target beneath a regular file, including a sandbox-worker panic. No command success was counted and no broader-write fallback was tried. |
| Linux mixed existing/absent exact-file grants | The existing-file startup error masked the absent target; this was not counted as an independent absent-target test. |
| Linux absent-file-only exact write grant | The command started, but creation of the approved target returned EROFS along with other writes. It therefore did not provide usable approved-file creation. |
| macOS 0.164.0, writable root plus deny globs | New root/nested matching filenames were blocked; an existing excluded file could not be read. Ordinary new files, directories and atomic edits remained allowed. |
| macOS read-only parent plus existing-file write grants | In-place writes to the two approved files worked, while other creations and atomic replacement were blocked. This narrower mode is not general autonomous file creation. |
| Extended macOS native case | Eight meaningful denials: excluded-name rename, hard link, excluded-name symlink, write through an allowed symlink to an excluded target, child write in a newly created directory, child private-name write, parent-alias PEM write and an existing excluded read. Two positive controls passed afterward: ordinary atomic edit and ordinary file creation. |

The excluded creation attempts in the macOS cases failed before the files appeared. Reads of files that were never created were not counted as separate read-enforcement evidence. The existing excluded read was independently meaningful because its fixture existed before launch. All data was synthetic; no actual credential files were probed.

The cached macOS binary SHA-256 is `d8990e3ee7acfbc26f91265b12a1a83f0fee40d0e0f8f1f40a37d4ec21f499cf` and matches its preserved cached archive member. It was not newly downloaded or substituted for the pinned Linux host. Linux remained `f2e6c980c937e734644bed1b4a9a018d3e63a0ccbe71aafc85c86a8c6e4b462a`; native cleanup used the existing protected `7aa562a1064a55d38137d743099edad728cc9df0d9b4a051340d4b64e7eceec8` executable.

Earlier diagnostic errors are retained: nested macOS sandbox execution was unavailable inside the enclosing managed sandbox; the older CLI does not use the attempted `sandbox macos` spelling; Python's developer-tool runtime was not readable under the minimal test profile. Those were not accepted results. The successful macOS cases used system shell tools under the stock profile with isolated project-local configuration. No extra user-data read permission was added to resolve Python startup.

## Preservation and next boundary

Evidence is under `.kineti/autonomy-filename-policy-20261101/`: exact programs and argv/profiles, `linux-*.json`, `mac-shell-*.json`, `mac-extended.json`, artifact provenance, preservation checks, spending assessments and the native operational verifier/receipt. Linux probes used only initialize and command execution, not authentication or inference, and were drained with the existing native cgroup primitive. The prior protected owner records were hash-checked. The probe groups are absent, no task UID processes or auth cache remain, and the disposable VM is stopped with its disk retained. Mac tests used separate project-local Codex configuration and temp files; installed settings and existing user work were not changed.

Accounting integrity remains verified, monetary reservations clear and all thirteen historical valuation unknowns and reported amounts unchanged. Original signatures and earlier ledger/evidence/undo prefixes are preserved. New paid spending, fresh logins, acceptance model turns, credential expansion, external publication, merge and deployment were zero. No new reviewer agent or Cargo regression run was needed for this research-only pass; production Rust code did not change.

The current Linux service remains blocked on strict dynamic filename exclusions. Using the verified macOS mechanism in production requires a separate compatible owner-adapter/lifecycle feasibility and acceptance plan; existing Linux revocation guarantees cannot simply be presumed on macOS. A supported Linux host change or an explicitly reviewed contract/scope amendment is another decision path, not authority supplied by this report. Full A01–A14 acceptance, current-platform direct-model tests, four-target installed acceptance and all release/accounting approvals remain pending.
