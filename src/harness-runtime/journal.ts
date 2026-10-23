import fs from "node:fs";
import path from "node:path";
import { withFileLock, readStrictLines, checkCheckpoint, commitCheckpoint } from "../../bin/ledger-integrity.ts";
import { writeJson } from "../../bin/lib.ts";
import { sha, uuid, type RunRecord } from "./types.ts";
export class RunJournal {
    readonly dir: string;
    readonly events: string;
    constructor(readonly root: string, readonly runId: string, create = false) {
        if (!uuid(runId))
            throw new Error("invalid run ID");
        this.dir = path.join(root, ".kineti/runs", runId);
        this.events = path.join(this.dir, "events.jsonl");
        if (create) {
            fs.mkdirSync(this.dir, { recursive: false, mode: 0o700 });
            fs.writeFileSync(this.events, "", { flag: "wx", mode: 0o600 });
            commitCheckpoint(this.events);
        }
    }
    append(type: string, data: unknown) {
        return withFileLock(path.join(this.dir, "journal.lock"), () => {
            const records = this.read();
            const prev = records.at(-1)?.hash ?? "GENESIS";
            const row = { seq: records.length, at: new Date().toISOString(), type, data, prev_hash: prev };
            const event = { ...row, hash: sha(JSON.stringify(row)) };
            const fd = fs.openSync(this.events, "a", 0o600);
            try {
                fs.writeSync(fd, JSON.stringify(event) + "\n");
                fs.fsyncSync(fd);
            }
            finally {
                fs.closeSync(fd);
            }
            commitCheckpoint(this.events);
            return event;
        });
    }
    read(): any[] {
        checkCheckpoint(this.events);
        const rows = readStrictLines<any>(this.events);
        let prev = "GENESIS";
        for (let i = 0; i < rows.length; i++) {
            const { hash, ...row } = rows[i];
            if (row.seq !== i || row.prev_hash !== prev || hash !== sha(JSON.stringify(row)))
                throw new Error("run history changed; inspect before recovery");
            prev = hash;
        }
        return rows;
    }
    save(record: RunRecord) { record.updated_at = new Date().toISOString(); this.append("state", record); writeJson(path.join(this.dir, "status.json"), record); }
    status(): RunRecord {
        const records = this.read().filter(r => r.type === "state");
        if (!records.length)
            throw new Error("run has no committed state");
        const r = records.at(-1).data as RunRecord;
        if (r.run_id !== this.runId)
            throw new Error("foreign run state");
        return r;
    }
    cancelled() { return fs.existsSync(path.join(this.dir, "cancel.json")); }
    cancel() { this.read(); writeJson(path.join(this.dir, "cancel.json"), { run_id: this.runId, at: new Date().toISOString() }); this.append("cancel-requested", { run_id: this.runId }); }
}
