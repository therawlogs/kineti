import { describe, test, expect, beforeAll, afterAll } from "bun:test";
import { startServer, getHarnessStatus, getFleetStatus } from "../bin/kineti-companion.ts";

describe("Kineti Visual Companion Server (kineti-companion.ts)", () => {
  let server: any;
  const testPort = 18788;

  beforeAll(() => {
    server = startServer(testPort);
  });

  afterAll(() => {
    if (server) server.stop(true);
  });

  test("getHarnessStatus returns structured pipeline and spend data", () => {
    const status = getHarnessStatus();
    expect(status).toBeDefined();
    expect(status.stages.length).toBe(13);
    expect(status.spend).toBeDefined();
    expect(status.spend.ceiling_usd).toBe(50.0);
    expect(status.stages[0].name).toBe("officehours");
    expect(status.stages[12].name).toBe("retro");
  });

  test("getFleetStatus returns fleet repositories and budget totals", () => {
    const fleet = getFleetStatus();
    expect(fleet).toBeDefined();
    expect(fleet.repos.length).toBeGreaterThanOrEqual(4);
    expect(fleet.total_fleet_budget).toBeGreaterThan(0);
    expect(fleet.settings).toBeDefined();
    expect(fleet.settings.ides.cursor).toBe(true);
  });

  test("GET / returns companion dashboard HTML with multi-repo components", async () => {
    const res = await server.fetch(new Request("http://localhost/"));
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("Kineti OS — Visual Companion Canvas");
    expect(html).toContain("Real-Time Spend Circuit Breaker");
    expect(html).toContain("13-Stage Pipeline");
    // Multi-repo additions
    expect(html).toContain("btn-repo-switcher");
    expect(html).toContain("fleet-view");
    expect(html).toContain("settings-sheet");
  });

  test("GET /api/status returns JSON status", async () => {
    const res = await server.fetch(new Request("http://localhost/api/status"));
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.stages).toBeDefined();
    expect(json.stages.length).toBe(13);
    expect(json.spend.total_usd).toBeDefined();
  });

  test("GET /api/fleet returns registered fleet repositories and summary", async () => {
    const res = await server.fetch(new Request("http://localhost/api/fleet"));
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.active_repo_id).toBeDefined();
    expect(Array.isArray(json.repos)).toBe(true);
    expect(json.repos.length).toBeGreaterThanOrEqual(4);
    expect(json.total_fleet_spend).toBeGreaterThanOrEqual(0);
    expect(json.settings).toBeDefined();
  });

  test("POST /api/fleet/select switches active repository context", async () => {
    const selectRes = await server.fetch(
      new Request("http://localhost/api/fleet/select", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ repo_id: "payment-service" }),
      }),
    );
    expect(selectRes.status).toBe(200);
    const selectJson = (await selectRes.json()) as any;
    expect(selectJson.success).toBe(true);
    expect(selectJson.active_repo_id).toBe("payment-service");

    // Status now reflects selected remote repository
    const statusRes = await server.fetch(new Request("http://localhost/api/status"));
    const statusJson = (await statusRes.json()) as any;
    expect(statusJson.project).toBe("payment-service");
    expect(statusJson.task.name).toContain("webhook signatures");

    // Switch back to local repository
    const resetRes = await server.fetch(
      new Request("http://localhost/api/fleet/select", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ repo_id: "kineti-local-harness" }),
      }),
    );
    expect(resetRes.status).toBe(200);
  });

  test("GET and POST /api/settings manages auto-latch and repo allocations", async () => {
    // GET settings
    const getRes = await server.fetch(new Request("http://localhost/api/settings"));
    expect(getRes.status).toBe(200);
    const settings = (await getRes.json()) as any;
    expect(settings.github.connected).toBe(true);
    expect(settings.ides.antigravity).toBe(true);

    // Update settings
    const postRes = await server.fetch(
      new Request("http://localhost/api/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          repo_budgets: { "auth-api": 45.0 },
          repo_owners: { "auth-api": "Sarah Lin" },
        }),
      }),
    );
    expect(postRes.status).toBe(200);
    const updated = (await postRes.json()) as any;
    expect(updated.success).toBe(true);
    expect(updated.settings.repo_budgets["auth-api"]).toBe(45.0);
    expect(updated.settings.repo_owners["auth-api"]).toBe("Sarah Lin");
  });

  test("Rejects malicious external origins (CSWSH protection)", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", {
        headers: {
          Origin: "http://malicious-attacker-site.com",
        },
      }),
    );
    expect(res.status).toBe(403);
  });

  test("Allows trusted localhost and 127.0.0.1 origins", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", {
        headers: {
          Origin: "http://localhost:3000",
        },
      }),
    );
    expect(res.status).toBe(200);
  });
});
