#!/usr/bin/env bun
import fs from "node:fs";
export const crateNames = ["kineti-core", "kineti-reflex", "kineti-memory", "kineti-connectors", "kineti-actions", "kineti-gateway", "kineti-harness", "kineti-cli"];
export async function registryPreflight(version: string, fetcher: typeof fetch = fetch) {
  const npm = await fetcher("https://registry.npmjs.org/kineti");
  if (!npm.ok) throw new Error(`npm registry check failed: HTTP ${npm.status}`);
  const data = await npm.json() as { versions: { [version: string]: unknown } };
  if (data.versions[version]) throw new Error(`kineti@${version} already exists; inspect publication before retrying`);
  for (const name of crateNames) {
    const response = await fetcher(`https://crates.io/api/v1/crates/${name}`, { headers: { "User-Agent": "Kineti release preflight (github.com/therawlogs/kineti)" } });
    if (!response.ok) throw new Error(`${name} registry check failed: HTTP ${response.status}`);
    const info = await response.json() as { versions: Array<{ num: string }> };
    if (info.versions.some((v) => v.num === version)) throw new Error(`${name}@${version} already exists; inspect publication before retrying`);
  }
}
if (import.meta.main) {
  const version = JSON.parse(fs.readFileSync("package.json", "utf8")).version;
  const tag = process.argv[2];
  if (tag !== `v${version}` || !/^\d+\.\d+\.\d+$/.test(version)) throw new Error("release tag does not match the stable package version");
  for (const file of ["kineti.config.json", "server.json"]) if (JSON.parse(fs.readFileSync(file, "utf8")).version !== version) throw new Error(`version mismatch: ${file}`);
  for (const file of ["Cargo.toml", "core-native/Cargo.toml"]) if (fs.readFileSync(file, "utf8").match(/\[workspace\.package\][^\[]*?version\s*=\s*"([^"]+)"/s)?.[1] !== version) throw new Error(`version mismatch: ${file}`);
  await registryPreflight(version);
  console.log(`Unused release version checked: ${version}; npm and ${crateNames.length} Rust crates`);
}
