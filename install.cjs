"use strict";
// Node is used only during npm installation, never to execute the installed gate.
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const root = __dirname;
const key = `${process.platform}-${process.arch}`;
const supported = ["linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64"];
if (!supported.includes(key)) throw new Error(`kineti: unsupported platform ${key}`);
const manifest = JSON.parse(fs.readFileSync(path.join(root, "checksums.json"), "utf8"));
if (manifest.version !== "0.6.0") throw new Error("kineti: checksum version mismatch");
const bytes = fs.readFileSync(path.join(root, "native", key, "kineti"));
const sha = crypto.createHash("sha256").update(bytes).digest("hex");
if (!/^[a-f0-9]{64}$/.test(manifest.targets[key]) || sha !== manifest.targets[key]) throw new Error("kineti: native payload checksum mismatch; nothing installed");
const target = path.join(root, "bin", "kineti");
const temporary = target + ".selected";
try {
  fs.writeFileSync(temporary, bytes, { flag: "wx", mode: 0o755 });
  fs.renameSync(temporary, target);
  fs.chmodSync(target, 0o755);
  fs.rmSync(path.join(root, "native"), { recursive: true });
} catch (error) {
  if (fs.existsSync(temporary)) fs.unlinkSync(temporary);
  throw error;
}
