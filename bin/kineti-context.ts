#!/usr/bin/env bun
/** Project-scoped contextual memory API; user identity comes from the local OS account. */
import { bindLegacyMemory, ContextMemoryStore, localContextScope, type ContextMemoryInput } from "./kineti-memory-job.ts";

export interface MemoryModelRequest {
  schema_version: 1;
  user_prompt: string;
  unavailable_memory?: string[];
  contextual_memory: Array<{ id: string; content: string; tags: string[]; sources: ReturnType<ContextMemoryStore["retrieve"]>["entries"][number]["sources"] }>;
}

/** Build the exact bounded payload handed to a model-capable host adapter. */
export function buildMemoryModelRequest(
  store: ContextMemoryStore,
  scope: { user: string; project: string },
  prompt: string,
  query = "",
  limit = 8,
): MemoryModelRequest {
  if (typeof prompt !== "string" || !prompt.trim() || Buffer.byteLength(prompt, "utf8") > 32_768) {
    throw new Error("model request needs a non-empty prompt up to 32 KiB");
  }
  if (typeof query !== "string") throw new Error("memory query must be text");
  const retrieved = store.retrieve(scope, query, limit);
  return {
    schema_version: 1,
    user_prompt: prompt,
    ...(retrieved.unavailable.length?{unavailable_memory:retrieved.unavailable}:{}),
    contextual_memory: retrieved.entries.map((entry) => ({
      id: entry.id,
      content: entry.content,
      tags: entry.tags,
      sources: entry.sources,
    })),
  };
}

function value(args: string[], name: string): string | undefined {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : undefined;
}
function usage(): void {
  console.log(`kineti context - encrypted, project-scoped memory

Commands:
  store --id <mem-id> --content <text> [--tags <json-array>] [--sources <json-array>]
  retrieve [--query <text>] [--limit <1-20>]
  prepare --prompt <text> [--query <text>] [--limit <1-20>]
  correct --id <existing-mem-id> --new-id <new-mem-id> --content <text>
  delete --id <mem-id> --confirm <same-mem-id>
  bind-legacy --owner-inspected   Explicit human-terminal binding of reviewed legacy OS-user records

The local OS user id and current project id define scope. Stored text is encrypted. Retrieval excludes corrected and deleted versions. Prepare prints the exact bounded context payload returned to a model-capable host; it does not call an external model.`);
}

async function main(args: string[] = process.argv.slice(2)): Promise<void> {
  const [action] = args;
  if (!action || action === "--help" || action === "-h") { usage(); return; }
  const scope = localContextScope();
  const store = new ContextMemoryStore(process.cwd());
  if(action==="bind-legacy") {
    if(!process.stdin.isTTY||!args.includes("--owner-inspected"))throw new Error("legacy binding requires owner inspection in a human terminal");
    const {createInterface}=await import("node:readline/promises");const rl=createInterface({input:process.stdin,output:process.stdout});
    try {const answer=await rl.question(`Legacy memory can enter this current project only after review. Type BIND ${scope.project}: `);console.log(JSON.stringify(bindLegacyMemory(process.cwd(),answer)));}finally{rl.close();}return;
  }
  if (action === "store" || action === "correct") {
    const id = action === "store" ? value(args, "--id") : value(args, "--new-id");
    const content = value(args, "--content");
    let tags: string[] = [], sources: ContextMemoryInput["sources"] = [];
    try { if (value(args, "--tags")) tags = JSON.parse(value(args, "--tags")!) as string[]; }
    catch { throw new Error("--tags must be a JSON array of strings"); }
    try { if (value(args, "--sources")) sources = JSON.parse(value(args, "--sources")!) as ContextMemoryInput["sources"]; }
    catch { throw new Error("--sources must be a JSON array of evidence references"); }
    const correction = action === "correct" ? value(args, "--id") : undefined;
    if (!id || !content) throw new Error(`${action} requires ${action === "store" ? "--id" : "--new-id"} and --content`);
    const record = store.append({ id, content, tags, sources, scope, ...(correction ? { correction_of: correction } : {}) });
    console.log(`STORED ${record.id}: ${record.data.content_sha256}`);
    return;
  }
  if (action === "retrieve") {
    const limitRaw = value(args, "--limit");
    const result = store.retrieve(scope, value(args, "--query") ?? "", limitRaw === undefined ? 8 : Number(limitRaw));
    console.log(JSON.stringify(result, null, 2));
    return;
  }
  if (action === "prepare") {
    const prompt = value(args, "--prompt") ?? "";
    const limitRaw = value(args, "--limit");
    const request = buildMemoryModelRequest(store, scope, prompt, value(args, "--query") ?? "", limitRaw === undefined ? 8 : Number(limitRaw));
    console.log(JSON.stringify(request, null, 2));
    return;
  }
  if (action === "delete") {
    const id = value(args, "--id"), confirmation = value(args, "--confirm");
    if (!id || confirmation !== id) throw new Error("delete requires --id and --confirm with the same id");
    store.delete(id, scope);
    console.log(`DELETED ${id}: encrypted content and key removed; only journal audit metadata remains`);
    return;
  }
  usage();
  throw new Error(`unknown context command: ${action}`);
}

if (import.meta.main) main().catch((error) => { console.error(`kineti context: ${error instanceof Error ? error.message : String(error)}`); process.exitCode = 2; });
