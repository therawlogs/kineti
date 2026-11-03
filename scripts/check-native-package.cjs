"use strict";
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");
const root = path.resolve(__dirname, "..");
const pkg = require(path.join(root, "package.json"));
const sums = require(path.join(root, "checksums.json"));
if (pkg.version !== "0.6.1" || sums.version !== pkg.version) throw new Error("fixed release version mismatch");
for (const key of ["linux-x64", "linux-arm64", "darwin-x64", "darwin-arm64"]) {
  const bytes = fs.readFileSync(path.join(root, "native", key, "kineti"));
  if (bytes.length >= 10000000) throw new Error(`${key} binary alone exceeds footprint cap`);
  if (crypto.createHash("sha256").update(bytes).digest("hex") !== sums.targets[key]) throw new Error(`${key} checksum mismatch`);
}
if (!fs.readFileSync(path.join(root, "bin/kineti"), "utf8").startsWith("#!/bin/sh")) throw new Error("package must retain explicit scripts-disabled installation error");
console.log("Four native package payload checksums verified; installed footprints require target acceptance.");
