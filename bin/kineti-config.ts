import fs from "node:fs";
import path from "node:path";

function stableJson(value: unknown): string {
  const normalize = (item: any): any => {
    if (Array.isArray(item)) return item.map(normalize);
    if (item && typeof item === "object") {
      return Object.fromEntries(Object.keys(item).sort().map((key) => [key, normalize(item[key])]));
    }
    return item;
  };
  return JSON.stringify(normalize(value));
}

export function writeCursorConfig(file: string, nextServer: unknown, hooks: {
  beforeWrite?: () => void;
  beforeRename?: () => void;
  rename?: typeof fs.renameSync;
  at?: number;
  serverName?: string;
  migrateLegacy?: boolean;
} = {}): string {
  const dir = path.dirname(file);
  fs.mkdirSync(dir, { recursive: true });
  let current: Record<string, any> = {};
  if (fs.existsSync(file)) {
    current = JSON.parse(fs.readFileSync(file, "utf8"));
    if (!current || typeof current !== "object" || Array.isArray(current)) throw new Error("Cursor MCP config must be a JSON object");
  }
  const servers = current.mcpServers ?? {};
  if (!servers || typeof servers !== "object" || Array.isArray(servers)) throw new Error("Cursor mcpServers must be an object");
  const name=hooks.serverName ?? "kineti";
  if(!/^kineti(?:_[a-f0-9]{12})?$/.test(name))throw new Error("invalid Kineti server name");
  const mergedServers={...servers};
  const legacy=hooks.migrateLegacy && name!=="kineti" && Object.hasOwn(servers,"kineti");
  if(legacy){if(stableJson(servers.kineti)!==stableJson(nextServer))throw new Error("Legacy kineti entry differs; configuration preserved for owner review");delete mergedServers.kineti;}
  if (Object.hasOwn(servers, name)) {
    if (stableJson(servers[name]) !== stableJson(nextServer)) throw new Error("Cursor MCP config already has a different kineti entry; resolve the conflict before running init");
    if(!legacy)return "";
  }
  mergedServers[name]=nextServer;
  const backup = file + ".bak." + (hooks.at ?? Date.now());
  if (fs.existsSync(file)) fs.copyFileSync(file, backup, fs.constants.COPYFILE_EXCL);
  const merged = { ...current, mcpServers: mergedServers };
  const temp = path.join(dir, "." + path.basename(file) + "." + process.pid + "." + Math.random().toString(36).slice(2) + ".tmp");
  try {
    hooks.beforeWrite?.();
    fs.writeFileSync(temp, JSON.stringify(merged, null, 2) + "\n", { flag: "wx", mode: 0o600 });
    hooks.beforeRename?.();
    (hooks.rename ?? fs.renameSync)(temp, file);
  } catch (error) {
    try { fs.rmSync(temp, { force: true }); } catch {}
    throw error;
  }
  return backup;
}
