# What the Test Receipt Fingerprint Can and Cannot See

`kineti test -- <command>` saves a receipt with a **workspace fingerprint**: one hash built from the project files. `kineti test check` recomputes it. If the two differ, the receipt is `STALE`.

The fingerprint can be wrong in two directions:

1. **False alarm**: the receipt goes `STALE` when nothing meaningful changed. This is safe. You re-run the tests and lose a minute.
2. **Blind spot**: the receipt stays `FRESH` when something did change. This is unsafe. The receipt says "nothing changed" when that is not true.

The code is in `bin/kineti-evidence.ts` (`fingerprint`).

---

## 1. How the fingerprint is built

Kineti walks the project folder in name order. Each entry adds its relative path and one value:

| Entry | Value added |
|-------|-------------|
| File up to 50 MiB | SHA-256 (CRLF normalized to LF for text files; raw bytes for binary) |
| File over 50 MiB | `large:<size>:<modified time>` |
| Symlink | `symlink:<target path>` (the link is not followed) |
| File or folder Kineti cannot read | `unreadable` |
| Excluded folder or file | nothing (skipped) |

Excluded folders: `.git`, `.kineti`, `.agents`, `node_modules`, `dist`, `build`, `.next`, `coverage`, `tmp`, `.cache`, `legacy`, `target`. Excluded files: `.DS_Store`.

---

## 2. False alarms (safe)

| Change | Why the receipt goes STALE |
|--------|----------------------------|
| Rename or move a file, same contents | The relative path is part of the hash |
| New untracked file, such as a scratch note | Every non-excluded file is included, tracked by git or not |
| File over 50 MiB is touched but not edited | Its modified time is part of the hash |

Text files (files without null bytes in their first 8 KB) normalize `\r\n` to `\n` during chunked hashing. A line-ending flip on Windows between CRLF and LF will not alter the fingerprint. Binary files continue to be hashed as exact raw bytes.

---

## 3. Blind spots (unsafe)

| Change | Seen? | Notes |
|--------|-------|-------|
| Edit a file up to 50 MiB | ✅ Yes | Full hash |
| Edit a file over 50 MiB that changes its size or modified time | ✅ Yes | |
| Edit a file over 50 MiB that keeps the same size **and** modified time | ❌ No | Only size and time are checked above 50 MiB. Normal edits update the time; tools that reset it on purpose are not caught |
| Point a symlink somewhere else | ✅ Yes | Target path is hashed |
| Edit the file a symlink points to, **outside** the project | ❌ No | Links are not followed |
| Edit the file a symlink points to, **inside** the project | ✅ Yes | The real file is hashed at its own path |
| A file becomes unreadable, or a new unreadable file appears | ✅ Yes | Recorded as `unreadable` |
| Edit a file that stays unreadable to Kineti | ❌ No | Kineti cannot read it, so it cannot hash it |
| Anything inside an excluded folder | ❌ No | Skipped by design |
| Sockets, pipes, device files | ❌ No | Not regular files |

In v0.4.0 and earlier, files over 4 MiB, symlinks, and unreadable files were skipped completely.

---

## 4. Good habits

1. Keep generated files and large data in an excluded folder, or out of the project.
2. If a test reads a file over 50 MiB, re-run the test after changing that file, even if the receipt says `FRESH`.
3. Do not keep test inputs behind symlinks that point outside the project.
4. Fix permissions so Kineti can read every file the tests use.
5. Delete scratch files before running `kineti test`, or put them in `tmp/`, which is excluded.

---

## 5. What exit code 0 does not prove

Kineti does not inspect individual tests inside your test runner. It runs one command (for example `bun test` or `npm test`) and writes down three things: the label, the process exit code, and a workspace fingerprint of your files. Later, CI checks that record: exit code `0` = pass.

Here is what that means in practice:

### Skipped tests
Say your test suite has 100 tests. Someone marks 40 of them as skipped (`test.skip`, `#[ignore]`, `@pytest.mark.skip`). Most test runners print `60 passed, 40 skipped` and exit with `0`. Kineti sees only the exit code `0`. The receipt is marked `FRESH` and passing.

### Zero tests (vacuous pass)
If a config error, wrong glob, or renamed file makes the runner find zero tests, many runners still exit with `0`. Kineti records a pass for an empty run.

### Why this matters
Kineti's promise is: *"the agent says tests passed, CI checks it is true."*
- What Kineti **does verify**: the test command was run recently, the files are unchanged since the run, and the process exited 0.
- What Kineti **does not verify**: whether tests actually executed, how many tests ran, or whether difficult test cases were skipped.

An agent that skips difficult tests can produce an exit code of 0.

### Where Kineti is safe
1. **Stale-result checking**: This is where agent claims usually fall apart. If an agent runs tests, passes, and then edits code (e.g. removes a test assertion or changes implementation files), the workspace fingerprint changes immediately and marks the receipt `STALE`.
2. **Kineti runner guards**: `kineti test` and `kineti-evidence.ts run` inspect runner outputs:
   - **Zero-tests guard (default ON)**: Checks that known runners executed tests. If an empty file or empty workspace is detected (e.g. Bun `Ran 0 tests across 1 file` or Cargo workspace with 0 passed tests), Kineti flips the result to exit code 1 and records `guard_failure: "zero-tests"`. Pass `--allow-zero-tests` to permit zero tests.
   - **Skipped-tests guard (opt-in with `--forbid-skipped`)**: Checks for non-zero skip counts (`1 skip`, `1 skipped`, `1 ignored`). If found, Kineti flips the result to exit code 1 and records `guard_failure: "skipped-tests"`.
   - **Receipt preservation**: Guard failures are saved in `.kineti/evidence.jsonl`. When checked, `kineti test check` reports `STALE (guard failed: zero-tests)` or `STALE (guard failed: skipped-tests)`.
3. **Strict runner flags**: You can also configure runners directly to exit non-zero:

| Runner | Zero Tests Default | Skipped Tests Default | Strict flags to force failure |
|--------|-------------------|-----------------------|------------------------------|
| **Jest** | Exits 1 (unless `--passWithNoTests`) | Exits 0 | `--passWithNoTests=false` |
| **Vitest** | Configurable | Exits 0 | `passWithNoTests: false` |
| **Pytest** | Exits 5 | Exits 0 | `--fail-on-empty-test-suite` |
| **Bun** | Exits 0 on empty files | Exits 0 | Kineti zero-test guard or test assertions |
| **Cargo** | Exits 0 if no tests in crate | Exits 0 | Kineti zero-test guard or `-- --ignored` |

*Note on runner guards*: Guards inspect standard output for known runners (Bun, Jest, Vitest, Pytest, Cargo). Runners outside this list, or custom test logs, are not guaranteed to be caught.

---

## 6. What the receipt is not

The receipt is a local record. It is not a signed certificate, and it does not prove the tests are good tests. It only ties "this command exited 0" to "these files, as fingerprinted above".

---

## See also

- [HOWTO-daily-loop.md](./HOWTO-daily-loop.md): daily workflow with test receipts
- [UNDO_LIMITATIONS.md](./UNDO_LIMITATIONS.md): what undo cannot reverse
