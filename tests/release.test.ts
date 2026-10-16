import { describe, expect, test } from "bun:test";
import fs from "node:fs";
import { runInNewContext } from "node:vm";
import { crateNames, registryPreflight } from "../scripts/release-preflight.ts";
const response = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status });
describe("release preflight", () => {
  test("checks the requested version across npm and all eight crates", async () => {
    const seen: string[] = [];
    const fetcher = (async (url: string | URL | Request) => { seen.push(String(url)); return response(String(url).includes("npmjs") ? { versions: { "0.4.0": {} } } : { versions: [{ num: "0.4.0" }] }); }) as typeof fetch;
    await registryPreflight("0.5.0", fetcher);
    expect(seen).toHaveLength(9);
    for (const name of crateNames) expect(seen.some((url) => url.endsWith("/" + name))).toBe(true);
  });
  test("blocks an already published npm version before checking crates", async () => {
    let calls = 0;
    const fetcher = (async () => { calls++; return response({ versions: { "0.5.0": {} } }); }) as unknown as typeof fetch;
    await expect(registryPreflight("0.5.0", fetcher)).rejects.toThrow("already exists");
    expect(calls).toBe(1);
  });
  test("blocks any already published crate version", async () => {
    const fetcher = (async (url: string | URL | Request) => response(String(url).includes("npmjs") ? { versions: {} } : { versions: [{ num: "0.5.0" }] })) as typeof fetch;
    await expect(registryPreflight("0.5.0", fetcher)).rejects.toThrow("kineti-core@0.5.0 already exists");
  });
  test("does not treat failed registry requests as unused versions", async () => {
    const fetcher = (async () => response({ error: "unavailable" }, 503)) as unknown as typeof fetch;
    await expect(registryPreflight("0.5.0", fetcher)).rejects.toThrow("HTTP 503");
  });
  test("publication uses the verified tarball and does not cancel an upload in progress", () => {
    const workflow = fs.readFileSync(".github/workflows/release.yml", "utf8");
    expect(workflow).toContain("cancel-in-progress: false");
    expect(workflow).toContain("Check publishing credentials before any upload");
    expect(workflow).toContain("npm-package.json");
    expect(workflow).toContain("m.sha256");
    expect(workflow).toContain("include-hidden-files: true");
    expect(workflow).not.toContain("--no-verify");
    expect(workflow).not.toContain("build-binaries:");
  });
  test("the website gives one plain paragraph and keeps accessibility controls", () => {
    const page = fs.readFileSync("website/index.html", "utf8");
    const main = page.split("<main>")[1].split("</main>")[0];
    expect(main.match(/<p[ >]/g)).toHaveLength(1);
    expect(main).toContain("Protected writes are experimental");
    expect(main).toContain("does not control every agent action");
    expect(page).toContain("prefers-reduced-motion");
    expect(page).toContain('id="bg-toggle"');
    expect(main).toContain("github.com/therawlogs/kineti");
  });
  test("background script keeps the poster for reduced motion, mobile and data saving", () => {
    const page = fs.readFileSync("website/index.html", "utf8");
    const script = page.split("<script>")[1].split("</script>")[0];
    for (const mode of ["reduced", "mobile", "save-data", "desktop"]) {
      let appended = 0, paused = 0;
      const video = { paused: true, playbackRate: 1, pause() { paused++; this.paused = true; }, play() { this.paused = false; return Promise.resolve(); }, load() {}, removeAttribute() {}, querySelector() { return null; }, appendChild() { appended++; }, addEventListener() {} };
      const toggle = { hidden: true, setAttribute() {}, addEventListener() {} };
      const bg = { classList: { add() {}, remove() {} } };
      const document = { hidden: false, querySelector(selector: string) { return selector.includes("video") ? video : bg; }, getElementById() { return toggle; }, addEventListener() {}, createElement() { return {}; } };
      const window = { matchMedia(query: string) { return { matches: query.includes("reduced-motion") ? mode === "reduced" : mode === "mobile", addEventListener() {} }; } };
      runInNewContext(script, { document, window, navigator: { connection: { saveData: mode === "save-data" } } });
      expect(appended, mode).toBe(mode === "desktop" ? 1 : 0);
      if (mode !== "desktop") { expect(paused, mode).toBeGreaterThan(0); expect(toggle.hidden, mode).toBe(true); }
    }
    expect(page).toMatch(/@media \(prefers-reduced-motion: reduce\)\s*\{\s*\.cosmic-bg video \{ animation: none;/);
  });

});
