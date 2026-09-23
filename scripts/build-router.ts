#!/usr/bin/env bun
// scripts/build-router.ts
// Single-source router codegen: bin/kineti.js is GENERATED from bin/kineti.ts.
// Never hand-edit bin/kineti.js. Regenerate with: bun run build:router
// CI rebuilds and fails on any diff so the two can never drift.

import fs from "node:fs";

const SRC = "bin/kineti.ts";
const OUT = "bin/kineti.js";

const result = await Bun.build({
  entrypoints: [SRC],
  target: "node",
  minify: false,
});

if (!result.success) {
  for (const log of result.logs) console.error(log);
  process.exit(1);
}

const built = result.outputs[0];
const text = await built.text();
const lines = text.split("\n");
lines[0] = "#!/usr/bin/env node";
fs.writeFileSync(OUT, lines.join("\n"));
fs.chmodSync(OUT, 0o755);
console.log(`router codegen: ${SRC} -> ${OUT} (${lines.join("\n").length} bytes)`);
