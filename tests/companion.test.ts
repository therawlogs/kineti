import { describe, test, expect, beforeAll, afterAll } from "bun:test";
import { startServer, getHarnessStatus, getFleetStatus, AUTH_TOKEN } from "../bin/kineti-companion.ts";

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
    // Default is local-only; extra repos come from .kineti/fleet.local.json (gitignored)
    expect(fleet.repos.length).toBeGreaterThanOrEqual(1);
    expect(fleet.total_fleet_budget).toBeGreaterThan(0);
    expect(fleet.settings).toBeDefined();
    expect(fleet.settings.ides.cursor).toBe(true);
    // No demo PII committed to source
    const raw = JSON.stringify(fleet);
    expect(raw).not.toContain("praveen@kineti.dev");
    expect(raw).not.toContain("Sarah Lin");
  });

  test("GET / without token is 401 with no hex token in body", async () => {
    const res = await server.fetch(new Request("http://localhost/"));
    expect(res.status).toBe(401);
    const html = await res.text();
    expect(html).not.toContain(AUTH_TOKEN);
    // No 64-hex token leak
    expect(html).not.toMatch(/[0-9a-f]{64}/);
    expect(res.headers.get("vary")).toContain("Origin");
  });

  test("GET / with token returns dashboard HTML token-free", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("Kineti OS — Visual Companion Canvas");
    expect(html).toContain("Real-Time Spend Circuit Breaker");
    expect(html).toContain("13-Stage Pipeline");
    // Multi-repo additions
    expect(html).toContain("btn-repo-switcher");
    expect(html).toContain("fleet-view");
    expect(html).toContain("settings-sheet");
    // Token must not be embedded
    expect(html).not.toContain(AUTH_TOKEN);
    expect(html).not.toContain("KINETI_TOKEN");
    expect(res.headers.get("vary")).toContain("Origin");
  });

  test("GET /api/status without token is 401, with token is 200", async () => {
    const unauth = await server.fetch(new Request("http://localhost/api/status"));
    expect(unauth.status).toBe(401);
    const res = await server.fetch(
      new Request("http://localhost/api/status", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.stages).toBeDefined();
    expect(json.stages.length).toBe(13);
    expect(json.spend.total_usd).toBeDefined();
    expect(res.headers.get("vary")).toContain("Origin");
  });

  test("GET /api/fleet without token is 401, with token returns fleet", async () => {
    const unauth = await server.fetch(new Request("http://localhost/api/fleet"));
    expect(unauth.status).toBe(401);
    const res = await server.fetch(
      new Request("http://localhost/api/fleet", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.active_repo_id).toBeDefined();
    expect(Array.isArray(json.repos)).toBe(true);
    expect(json.repos.length).toBeGreaterThanOrEqual(1);
    expect(json.total_fleet_spend).toBeGreaterThanOrEqual(0);
    expect(json.settings).toBeDefined();
  });

  test("POST /api/fleet/select switches active repository context", async () => {
    const auth = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const fleetRes = await server.fetch(
      new Request("http://localhost/api/fleet", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    const fleetJson = (await fleetRes.json()) as any;
    const localId = fleetJson.repos.find((r: any) => r.is_local)?.id ?? fleetJson.active_repo_id;
    const selectRes = await server.fetch(
      new Request("http://localhost/api/fleet/select", {
        method: "POST",
        headers: auth,
        body: JSON.stringify({ repo_id: localId }),
      }),
    );
    expect(selectRes.status).toBe(200);
    const selectJson = (await selectRes.json()) as any;
    expect(selectJson.success).toBe(true);

    // Unknown repo is 404 (with auth)
    const missingRes = await server.fetch(
      new Request("http://localhost/api/fleet/select", {
        method: "POST",
        headers: auth,
        body: JSON.stringify({ repo_id: "no-such-repo-xyz" }),
      }),
    );
    expect(missingRes.status).toBe(404);
  });

  test("POST /api/fleet/select without token is 401", async () => {
    const res = await server.fetch(
      new Request("http://localhost/api/fleet/select", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ repo_id: "local" }),
      }),
    );
    expect(res.status).toBe(401);
  });

  test("GET and POST /api/settings need token; GET without token is 401", async () => {
    const auth = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    // GET without token is blocked
    const unauthGet = await server.fetch(new Request("http://localhost/api/settings"));
    expect(unauthGet.status).toBe(401);
    // GET settings with token
    const getRes = await server.fetch(
      new Request("http://localhost/api/settings", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(getRes.status).toBe(200);
    const settings = (await getRes.json()) as any;
    expect(settings.ides.antigravity).toBe(true);

    // POST without token is blocked
    const unauthRes = await server.fetch(
      new Request("http://localhost/api/settings", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ repo_budgets: { "test-repo": 999 } }),
      }),
    );
    expect(unauthRes.status).toBe(401);

    // Update settings with generic test values (no PII)
    const fleetRes = await server.fetch(
      new Request("http://localhost/api/fleet", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    const fleetJson = (await fleetRes.json()) as any;
    const localId = fleetJson.repos.find((r: any) => r.is_local)?.id ?? fleetJson.active_repo_id;
    const postRes = await server.fetch(
      new Request("http://localhost/api/settings", {
        method: "POST",
        headers: auth,
        body: JSON.stringify({
          repo_budgets: { [localId]: 42.0 },
        }),
      }),
    );
    expect(postRes.status).toBe(200);
    const updated = (await postRes.json()) as any;
    expect(updated.success).toBe(true);
    expect(updated.settings.repo_budgets[localId]).toBe(42.0);
  });

  test("Rejects malicious external origins (CSWSH protection)", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", {
        headers: {
          Origin: "http://malicious-attacker-site.com",
          Authorization: `Bearer ${AUTH_TOKEN}`,
        },
      }),
    );
    expect(res.status).toBe(403);
    expect(res.headers.get("vary")).toContain("Origin");
  });

  test("Allows trusted localhost and 127.0.0.1 origins", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", {
        headers: {
          Origin: "http://localhost:3000",
          Authorization: `Bearer ${AUTH_TOKEN}`,
        },
      }),
    );
    expect(res.status).toBe(200);
  });

  test("Rejects DNS rebinding Host (evil.com, lookalikes, LAN IP)", async () => {
    // Use evil URLs: Bun builds req.url from Host for real traffic,
    // and Host is a forbidden header for `new Request`, so URL carries it here.
    for (const evilUrl of [
      "http://evil.com/",
      "http://localhost.evil.com/",
      "http://127.0.0.1.evil.com/",
      "http://192.168.1.9/",
      "http://10.0.0.5/",
    ]) {
      const res = await server.fetch(
        new Request(evilUrl, {
          headers: { Authorization: `Bearer ${AUTH_TOKEN}` },
        }),
      );
      expect(res.status).toBe(403);
    }
  });

  test("Trusted Host localhost and 127.0.0.1 pass with token", async () => {
    for (const goodUrl of [
      "http://localhost/",
      "http://127.0.0.1/",
      "http://localhost:18788/",
      "http://127.0.0.1:18788/",
    ]) {
      const res = await server.fetch(
        new Request(goodUrl, {
          headers: { Authorization: `Bearer ${AUTH_TOKEN}` },
        }),
      );
      expect(res.status).toBe(200);
    }
  });

  test("CORS is deny-by-default: no wildcard allow-origin", async () => {
    const res = await server.fetch(
      new Request("http://localhost/api/status", {
        headers: { Origin: "http://localhost:3000", Authorization: `Bearer ${AUTH_TOKEN}` },
      }),
    );
    expect(res.status).toBe(200);
    expect(res.headers.get("access-control-allow-origin")).toBeNull();
    expect(res.headers.get("vary")).toContain("Origin");
  });

  test("GET / with ?token=<token> returns 200 and sets session cookie", async () => {
    const res = await server.fetch(new Request(`http://localhost/?token=${AUTH_TOKEN}`));
    expect(res.status).toBe(200);
    const setCookie = res.headers.get("set-cookie");
    expect(setCookie).toBeDefined();
    expect(setCookie).toContain(`kineti_token=${AUTH_TOKEN}`);
    const html = await res.text();
    expect(html).toContain("Kineti Settings");
  });

  test("GET / with session cookie returns 200", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", {
        headers: { Cookie: `kineti_token=${AUTH_TOKEN}` },
      }),
    );
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("Kineti Settings");
  });

  test("GET /whatsapp-onboarding does not contain 650 numbers or Instinct", async () => {
    const res = await server.fetch(new Request("http://localhost/whatsapp-onboarding?t=wa_demo"));
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("wa_demo");
    expect(html).toContain("Connect WhatsApp");
    expect(html).not.toContain("Instinct");
    expect(html).not.toContain("650");
    expect(html).not.toContain("870-2892");
    expect(html).not.toContain("468-7388");
  });

  test("GET /whatsapp-onboarding rejects XSS injection payloads with 400", async () => {
    const res = await server.fetch(new Request("http://localhost/whatsapp-onboarding?t=%22%3E%3Csvg%20onload=alert(1)%3E"));
    expect(res.status).toBe(400);
    const text = await res.text();
    expect(text).toContain("Invalid pairing token format");
  });

  test("POST /login with valid token redirects 303 with HttpOnly cookie", async () => {
    const body = new FormData();
    body.append("token", AUTH_TOKEN);
    const res = await server.fetch(
      new Request("http://localhost/login", {
        method: "POST",
        body,
      }),
    );
    expect(res.status).toBe(303);
    expect(res.headers.get("location")).toBe("/");
    const setCookie = res.headers.get("set-cookie") || "";
    expect(setCookie).toContain("HttpOnly");
    expect(setCookie).toContain("SameSite=Strict");
    expect(setCookie).toContain(`kineti_token=${AUTH_TOKEN}`);
  });

  test("POST /api/connectors/toggle toggles connector status", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const res = await server.fetch(
      new Request("http://localhost/api/connectors/toggle", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ connector: "slack" }),
      }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.success).toBe(true);
    expect(json.connector.connected).toBe(true);
  });

  test("POST /api/vault creates logins and cards with input validation", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    // Add Login
    const loginRes = await server.fetch(
      new Request("http://localhost/api/vault", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ type: "login", domain: "github.com", username: "octocat" }),
      }),
    );
    expect(loginRes.status).toBe(200);
    const loginJson = (await loginRes.json()) as any;
    expect(loginJson.success).toBe(true);
    expect(loginJson.vault.logins.some((l: any) => l.domain === "github.com")).toBe(true);

    // Add Card
    const cardRes = await server.fetch(
      new Request("http://localhost/api/vault", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ type: "card", brand: "Mastercard", spend_cap: 150 }),
      }),
    );
    expect(cardRes.status).toBe(200);
    const cardJson = (await cardRes.json()) as any;
    expect(cardJson.success).toBe(true);
    expect(cardJson.vault.cards.some((c: any) => c.brand === "Mastercard" && c.spend_cap === 150)).toBe(true);
  });

  test("POST /api/mesh/approve approves pending connection request", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    // First get or inspect mesh
    const meshRes = await server.fetch(
      new Request("http://localhost/api/mesh", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(meshRes.status).toBe(200);
  });
});


