import { afterEach, describe, expect, test } from "bun:test";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { ContextMemoryStore, localContextScope } from "../bin/kineti-memory-job.ts";
import { buildMemoryModelRequest } from "../bin/kineti-context.ts";

const REPO = path.resolve(import.meta.dir, "..");
const roots: string[] = [];
function makeProject() {
  const base = path.join(REPO, ".kineti", "context-memory-tests");
  fs.mkdirSync(base, { recursive: true });
  const dir = fs.mkdtempSync(path.join(base, "project-"));
  fs.mkdirSync(path.join(dir, ".kineti"), { recursive: true });
  fs.writeFileSync(path.join(dir,".kineti/state.json"),JSON.stringify({project:"memory-test"}),{mode:0o600});
  roots.push(dir);
  return dir;
}
afterEach(() => { for (const root of roots.splice(0)) fs.rmSync(root, { recursive: true, force: true }); });

describe("encrypted contextual memory", () => {
  test("persists across store instances, isolates both scopes, and only retrieves the corrected live version", () => {
    const dir = makeProject(), otherDir=makeProject(), scopeA = localContextScope(dir), scopeB = { ...scopeA,user: "local-user-b" }, scopeC=localContextScope(otherDir);
    const first = new ContextMemoryStore(dir, REPO);
    const original = first.append({ id: "mem-context-001", scope: scopeA, content: "The blue import needs a token refresh.", tags: ["import"], sources: [] });
    expect(()=>first.append({ id: "mem-context-002", scope: scopeB, content: "Different user's private note about billing.", tags: ["billing"], sources: [] })).toThrow("actual OS user");
    new ContextMemoryStore(otherDir,REPO).append({ id: "mem-context-003", scope: scopeC, content: "A separate project note.", tags: [], sources: [] });

    const restarted = new ContextMemoryStore(dir, REPO);
    expect(restarted.retrieve(scopeA, "blue import").entries.map((entry) => entry.id)).toEqual([original.id]);
    expect(()=>restarted.retrieve(scopeB)).toThrow("actual OS user");
    expect(()=>restarted.retrieve(scopeC)).toThrow("actual OS user");
    expect(new ContextMemoryStore(otherDir,REPO).retrieve(scopeC).entries.map(x=>x.id)).toEqual(["mem-context-003"]);

    restarted.append({ id: "mem-context-004", scope: scopeA, content: "Correction: refresh the token before importing the blue file.", tags: ["import"], sources: [], correction_of: original.id });
    const afterCorrection = restarted.retrieve(scopeA, "blue file");
    expect(afterCorrection.entries.map((entry) => entry.id)).toEqual(["mem-context-004"]);
    expect(afterCorrection.entries[0].content).toContain("Correction:");
    const capturedOutgoingPayload = buildMemoryModelRequest(restarted, scopeA, "Continue the import task.", "blue file");
    expect(capturedOutgoingPayload).toEqual({
      schema_version: 1,
      user_prompt: "Continue the import task.",
      contextual_memory: [{ id: "mem-context-004", content: "Correction: refresh the token before importing the blue file.", tags: ["import"], sources: [] }],
    });
    expect(JSON.stringify(capturedOutgoingPayload)).not.toContain("The blue import needs a token refresh.");
    expect(JSON.stringify(capturedOutgoingPayload)).not.toContain("Different user's private note");
    expect(() => restarted.delete("mem-context-002", scopeA)).toThrow("not found in this user and project scope");

    restarted.delete("mem-context-004", scopeA);
    expect(new ContextMemoryStore(dir, REPO).retrieve(scopeA).entries).toEqual([]);
    expect(buildMemoryModelRequest(new ContextMemoryStore(dir, REPO), scopeA, "Continue the import task.").contextual_memory).toEqual([]);
    expect(fs.existsSync(path.join(dir, ".kineti", "memory", "keys", "mem-context-004.key"))).toBe(false);
    expect(fs.existsSync(path.join(dir,".kineti/memory/keys/mem-context-001.key"))).toBe(false);
    expect(fs.existsSync(path.join(dir,".kineti/memory/blobs/mem-context-001.enc"))).toBe(false);
    expect(fs.existsSync(path.join(dir, ".kineti", "memory", "blobs", "mem-context-004.enc"))).toBe(false);
    const journal = fs.readFileSync(path.join(dir, ".kineti", "journal.jsonl"), "utf8");
    expect(journal).not.toContain("token before importing");
    expect(journal).not.toContain("Different user's private note");
    expect(journal).not.toContain("import");
  });

  test("requires exact inspectable source bytes and reports source changes during retrieval", () => {
    const dir = makeProject(), scope = localContextScope(dir);
    const source = path.join(dir, "source.log"); fs.writeFileSync(source, "source line\n");
    const digest = crypto.createHash("sha256").update(fs.readFileSync(source)).digest("hex");
    const store = new ContextMemoryStore(dir, REPO);
    expect(() => store.append({ id: "mem-context-010", scope, content: "A note", tags: [], sources: [{ source_id: "src-one", path: "source.log", sha256: "0".repeat(64) }] })).toThrow("exact verified project files");
    store.append({ id: "mem-context-011", scope, content: "A note connected to source", tags: [], sources: [{ source_id: "src-one", path: "source.log", sha256: digest }] });
    expect(store.retrieve(scope).entries[0].sources[0].status).toBe("verified");
    const journal = fs.readFileSync(path.join(dir, ".kineti", "journal.jsonl"), "utf8");
    expect(journal).not.toContain("source.log");
    fs.writeFileSync(source, "modified source line\n");
    expect(store.retrieve(scope).entries[0].sources[0].status).toBe("changed");
  });
});
