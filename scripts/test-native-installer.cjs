"use strict";
// Installer failure paths must not replace the explicit disabled-script sentinel.
const assert = require("node:assert/strict");
const vm = require("node:vm");
const fs = require("node:fs");
const crypto = require("node:crypto");
const code = fs.readFileSync("install.cjs", "utf8");
// Constraints from the pinned MCP registry schema, plus the executable's required mode.
const registry = JSON.parse(fs.readFileSync("server.json", "utf8"));
assert.ok(registry.description.length > 0 && registry.description.length <= 100);
assert.equal(registry.version, "0.6.0");
assert.deepEqual(registry.packages[0].packageArguments, [{type: "positional", value: "mcp"}]);
function attempt(platform, arch, valid) {
  let writes = 0;
  const payload = Buffer.from("disposable fixture");
  const manifest = {version: "0.6.0", targets: {[`${platform}-${arch}`]: valid ? crypto.createHash("sha256").update(payload).digest("hex") : "0".repeat(64)}};
  const fakeFs = {readFileSync(file) {return file.endsWith("checksums.json") ? JSON.stringify(manifest) : payload;}, writeFileSync() {writes++;}, renameSync() {}, chmodSync() {}, rmSync() {}, existsSync() {return false;}};
  let error;
  try { vm.runInNewContext(code, {__dirname: "/fixture", process: {platform, arch}, require(name) { return name === "node:fs" ? fakeFs : require(name); }}); } catch (e) {error = e;}
  return {error, writes};
}
let result = attempt("win32", "x64", true);
assert.match(result.error.message, /unsupported platform/);
assert.equal(result.writes, 0);
result = attempt("linux", "x64", false);
assert.match(result.error.message, /checksum mismatch/);
assert.equal(result.writes, 0);
for (const [platform, arch] of [["linux", "x64"], ["linux", "arm64"], ["darwin", "x64"], ["darwin", "arm64"]]) {
  result = attempt(platform, arch, true);
  assert.equal(result.error, undefined);
  assert.equal(result.writes, 1);
}
console.log("6 installer policy cases passed (mocked payloads; native installations are separate)");
