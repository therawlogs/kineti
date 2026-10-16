# What the Test Receipt Fingerprint Can and Cannot See

`kineti test -- <command>` saves a receipt with a **workspace fingerprint**: one hash built from the project files. `kineti test check` recomputes it. If the two differ, the receipt is `STALE`. New receipt records are signed by a project-local Ed25519 key and include parsed test output and declared inputs. The signature proves that the matching private key signed those fields; it does not establish an external identity or prove the runner output was honest.

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

## 5. What a passing receipt means

New version 2 receipts bind the recorded command and arguments, declared dependency lockfiles, toolchain and runner/parser versions, required-test inventory, selected non-secret environment values, input digest, final artifact digests, parsed test summary, and a protected-write request when present. Bun, Cargo, and pytest output are accepted only when the known summary parses, at least one test ran, no skips or failures occurred, and every required test is reported. Unknown runner output and an empty required-test inventory cannot authorize the protected gate.

These checks rely on the local runner output and project-local signing key. The signature proves only that the matching local key signed the receipt. It does not establish an external identity or prove that runner output was truthful. A changed parser or changed declared inputs invalidates the receipt. Version 1 receipts remain readable as **limited** history and never authorize the version 2 gate.

Receipt trust properties currently demonstrated: Kineti-issued by its project-local Ed25519 key, bound to declared inputs by hashes, and checkable elsewhere when the receipt and trusted public key are supplied. A verifier on another machine checks declared evidence; it does not assume that a test pass transfers to that machine.

### Path, line-ending, and binary rules

Manifest paths are project-relative, normalized to `/`, and symlinks are represented by their normalized link target rather than followed. Text files are detected by absence of NUL bytes in the first 8 KiB and hash with CRLF normalized to LF. Binary files hash their exact bytes. Files larger than 50 MiB currently use size and modified time rather than full content hashing; such files are not strong content evidence. Secret values are excluded from the environment allowlist because hashing low-entropy secrets can reveal them.

### Protected effect boundary

The supported Cursor action is one exact file write: receipt authorization binds the destination, arguments, and bytes. The gate verifies and atomically writes through the same protected operation. This covers only requests routed through the Kineti MCP tool. Cursor's direct filesystem tools bypass the wrapper and are outside this gate's protection.

### Legacy receipts

Version 1 receipts can be read for limited historical results. They lack the exact-input manifest and signature required by the new gate, so they are never upgraded silently and cannot authorize writes.

## 6. What the receipt is not

A receipt is declared verification evidence. It does not establish that tests are well designed, that an unlisted environment matches, or that an independent evaluator approved the artifact. Workflow 09 requires an immutable artifact and separately inspectable evaluator results; its evaluator must not accept pass/fail evidence controlled by the builder.

## See also

- [HOWTO-daily-loop.md](./HOWTO-daily-loop.md): daily workflow with test receipts
- [UNDO_LIMITATIONS.md](./UNDO_LIMITATIONS.md): what undo cannot reverse
