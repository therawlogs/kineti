# Durable local native memory v1

`kineti-cli memory` provides graph/vector retrieval from the same encrypted current-memory view as `kineti context`. It requires Bun and the trusted `bin/kineti-native-memory.ts` bridge. Set operator-controlled absolute `KINETI_BUN` and `KINETI_NATIVE_BRIDGE` paths and run from the intended project. These paths are executable authority, not candidate-selected settings.

The native CLI accepts version 1 JSON through stdin. Supported operations are `ingest`, `correct`, `delete`, `graph`, and `vector`. The bridge additionally exposes `snapshot`. Ingest/correct payloads contain category, key, value and 1–256 finite embedding values. Corrections require an existing record id. Delete requires `confirm` matching the id. See `acceptance/native-memory.test.ts` for complete executable request examples. Do not put private memory content in command arguments or logs.

Identity comes from the actual OS UID and SHA-256 of the canonical project root. Display names, HOME overrides and caller `user`/`scope` fields do not grant access. Two folders with the same project label remain separate. Project storage paths must be owned by the current UID; symlinked storage paths and writable shared components are rejected. Use separate OS accounts for mutually untrusted users.

The bridge keeps AES-GCM content and keys in project-local `.kineti` storage, with a checked append-only journal. Native graph/vector indexes rebuild from the authorized current view on each request, including after restart. Corrections mask superseded content; deletion removes content keys/blobs for the correction ancestry and retains permitted journal metadata. Missing/corrupt encrypted records block native reconstruction rather than silently accepting an incomplete index.

Existing label-based memory is unavailable until a human explicitly inspects and binds exact historical records using `kineti context bind-legacy --owner-inspected` in a real terminal. This appends a versioned scope binding and preserves original history. Do not rewrite old identities or signatures.

Deletion applies to the live project store and subsequent current/historical context retrieval. Independent backups, filesystem snapshots and copies are outside this deletion operation and need their own retention policy. Encryption does not protect against an adversary controlling the same OS account or the running process.

The old `MemoryEngine`, SQL schema strings and direct in-memory library indexes remain prototypes. Their caller labels are not authenticated identity and their SQL strings do not provide durable SQLite storage. Product persistence uses `PersistentLocalMemory` and the bridge above. No distributed identity/account platform is supplied.

Linux acceptance uses two real OS accounts and two projects each through ingest, native graph/vector retrieval, restart, correction and deletion. Host-delivered context is a separate acceptance item; a passing storage test alone does not prove delivery to a model.
