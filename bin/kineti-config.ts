import fs from "node:fs";
import path from "node:path";

export function writeCursorConfig(file: string, nextServer: unknown, hooks: { beforeRename?: () => void; at?: number } = {}): string {
  const dir = path.dirname(file);
  fs.mkdirSync(dir, { recursive: true });
  let current: Record<string, any> = {};
  if (fs.existsSync(file)) {
    current = JSON.parse(fs.readFileSync(file, "utf8"));
    if (!current || typeof current !== "object" || Array.isArray(current)) throw new Error("Cursor MCP config must be a JSON object");
  }
  const servers = current.mcpServers ?? {};
  if (!servers || typeof servers !== "object" || Array.isArray(servers)) throw new Error("Cursor mcpServers must be an object");
  const backup = `${file}.bak.${hooks.at ?? Date.now()}`;
  if (fs.existsSync(file)) fs.copyFileSync(file, backup, fs.constants.COPYFILE_EXCL);
  const merged = { ...current, mcpServers: { ...servers, kineti: nextServer } };
  const temp = path.join(dir, `.${path.basename(file)}.${process.pid}.${Math.random().toString(36).slice(2)}.tmp`);
  try {
    fs.writeFileSync(temp, JSON.stringify(merged, null, 2) + "\n", { flag: "wx", mode: 0o600 });
    hooks.beforeRename?.();
    fs.renameSync(temp, file);
  } catch (error) {
    try { fs.rmSync(temp, { force: true }); } catch {}
    throw error;
  }
  return backup;
}
