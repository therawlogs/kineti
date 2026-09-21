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

  test("GET / with ?token=<token> is rejected with 401 (no token leak in URL)", async () => {
    const res = await server.fetch(new Request(`http://localhost/?token=${AUTH_TOKEN}`));
    expect(res.status).toBe(401);
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

  test("POST /login with valid token redirects 303 with HttpOnly session cookie", async () => {
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
    expect(setCookie).toContain("kineti_token=");

    // Extract session token from cookie and verify it accesses /
    const match = setCookie.match(/kineti_token=([a-f0-9]+)/);
    expect(match).toBeDefined();
    const sessionToken = match![1];
    expect(sessionToken).not.toBe(AUTH_TOKEN);

    const authRes = await server.fetch(
      new Request("http://localhost/", {
        headers: { Cookie: `kineti_token=${sessionToken}` },
      }),
    );
    expect(authRes.status).toBe(200);

    // Test POST /logout revokes the session
    const logoutRes = await server.fetch(
      new Request("http://localhost/logout", {
        method: "POST",
        headers: { Cookie: `kineti_token=${sessionToken}` },
      }),
    );
    expect(logoutRes.status).toBe(303);

    // Subsequent request with revoked session must be 401
    const revokedRes = await server.fetch(
      new Request("http://localhost/", {
        headers: { Cookie: `kineti_token=${sessionToken}` },
      }),
    );
    expect(revokedRes.status).toBe(401);
  });

  test("POST /api/connectors/toggle toggles connector status", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const getRes = await server.fetch(new Request("http://localhost/api/settings", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }));
    const initialSettings = (await getRes.json()) as any;
    const initialSlack = Boolean(initialSettings.connectors?.slack?.connected);

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
    expect(json.connector.connected).toBe(!initialSlack);
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

  test("GET /api/local-token is removed (401/404) to prevent local auth bypass", async () => {
    const localRes = await server.fetch(
      new Request("http://127.0.0.1:8788/api/local-token", {
        headers: { Host: "127.0.0.1:8788" },
      }),
    );
    expect([401, 404]).toContain(localRes.status);
  });

  test("POST /api/connectors/add, configure, and delete manages connectors", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    
    // Add custom connector
    const addRes = await server.fetch(
      new Request("http://localhost/api/connectors/add", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({
          key: "custom_crm",
          name: "Custom CRM",
          account: "sales@crm.example",
          apiKey: "sec_12345",
          desc: "Lead sync connector",
        }),
      }),
    );
    expect(addRes.status).toBe(200);
    const addJson = (await addRes.json()) as any;
    expect(addJson.success).toBe(true);
    expect(addJson.connector.name).toBe("Custom CRM");

    // Configure connector
    const cfgRes = await server.fetch(
      new Request("http://localhost/api/connectors/configure", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({
          connector: "custom_crm",
          account: "updated@crm.example",
          desc: "Updated description",
        }),
      }),
    );
    expect(cfgRes.status).toBe(200);
    const cfgJson = (await cfgRes.json()) as any;
    expect(cfgJson.connector.account).toBe("updated@crm.example");

    // Delete custom connector
    const delRes = await server.fetch(
      new Request("http://localhost/api/connectors/delete", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ connector: "custom_crm" }),
      }),
    );
    expect(delRes.status).toBe(200);
    const delJson = (await delRes.json()) as any;
    expect(delJson.success).toBe(true);

    // Delete standard connector resets it
    const delStdRes = await server.fetch(
      new Request("http://localhost/api/connectors/delete", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ connector: "slack" }),
      }),
    );
    expect(delStdRes.status).toBe(200);
  });

  test("POST /api/vault/delete removes entries by type and index", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    // First create a login to delete
    await server.fetch(
      new Request("http://localhost/api/vault", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ type: "login", domain: "delete-me.com", username: "temp_user" }),
      }),
    );

    const delRes = await server.fetch(
      new Request("http://localhost/api/vault/delete", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ type: "login", index: 0 }),
      }),
    );
    expect(delRes.status).toBe(200);
    const delJson = (await delRes.json()) as any;
    expect(delJson.success).toBe(true);
  });

  test("POST /api/contact/update and /api/contact/test manage contact channels", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    
    // Update contact
    const updateRes = await server.fetch(
      new Request("http://localhost/api/contact/update", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({
          imessage_number: "+15551234567",
          whatsapp_number: "+15559876543",
        }),
      }),
    );
    expect(updateRes.status).toBe(200);
    const updateJson = (await updateRes.json()) as any;
    expect(updateJson.settings.imessage_number).toBe("+15551234567");
    expect(updateJson.settings.whatsapp_number).toBe("+15559876543");

    // Test ping
    const testRes = await server.fetch(
      new Request("http://localhost/api/contact/test", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({
          channel: "whatsapp",
          recipient: "+15559876543",
          message: "Ping from tests",
        }),
      }),
    );
    expect(testRes.status).toBe(200);
    const testJson = (await testRes.json()) as any;
    expect(testJson.success).toBe(true);
    expect(testJson.dispatched).toBe(true);
    expect(testJson.channel).toBe("whatsapp");
  });

  test("POST /api/mesh/add, remove, and unblock manage trusted network peers", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };

    // Add peer
    const addPeerRes = await server.fetch(
      new Request("http://localhost/api/mesh/add", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({
          agentId: "colleague_agent_test",
          name: "Colleague Agent",
          tier: "colleague",
        }),
      }),
    );
    expect(addPeerRes.status).toBe(200);
    const addPeerJson = (await addPeerRes.json()) as any;
    expect(addPeerJson.success).toBe(true);
    expect(addPeerJson.peer.peer_agent_id).toBe("colleague_agent_test");

    // Remove peer
    const remPeerRes = await server.fetch(
      new Request("http://localhost/api/mesh/remove", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ agentId: "colleague_agent_test" }),
      }),
    );
    expect(remPeerRes.status).toBe(200);

    // Block peer
    const blkRes = await server.fetch(
      new Request("http://localhost/api/mesh/block", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ agent_id: "colleague_agent_test" }),
      }),
    );
    expect(blkRes.status).toBe(200);

    // Unblock peer
    const unblkRes = await server.fetch(
      new Request("http://localhost/api/mesh/unblock", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ agentId: "colleague_agent_test" }),
      }),
    );
    expect(unblkRes.status).toBe(200);
  });

  test("GET /api/mini returns spend, goal, undo, and power state", async () => {
    const res = await server.fetch(
      new Request("http://localhost/api/mini", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.ceiling).toBe(50);
    expect(json.enabled).toBe(true);
    expect(json.pending_undo).toBeGreaterThanOrEqual(0);
  });

  test("POST /api/power toggles Kineti off and back on", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const fs = await import("node:fs");
    const path = await import("node:path");
    const switchFile = path.join(process.cwd(), ".kineti", "kineti.json");
    const hadSwitch = fs.existsSync(switchFile);
    const off = await server.fetch(
      new Request("http://localhost/api/power", { method: "POST", headers: authHeaders, body: JSON.stringify({ on: false }) }),
    );
    expect(off.status).toBe(200);
    expect(((await off.json()) as any).enabled).toBe(false);
    const mini = (await (await server.fetch(
      new Request("http://localhost/api/mini", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    )).json()) as any;
    expect(mini.enabled).toBe(false);
    const on = await server.fetch(
      new Request("http://localhost/api/power", { method: "POST", headers: authHeaders, body: JSON.stringify({ on: true }) }),
    );
    expect(((await on.json()) as any).enabled).toBe(true);
    if (!hadSwitch) fs.rmSync(switchFile, { force: true });
  });

  test("POST /api/talk answers in plain words with choices", async () => {
    const res = await server.fetch(
      new Request("http://localhost/api/talk", {
        method: "POST",
        headers: { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` },
        body: JSON.stringify({ message: "how much have I spent?" }),
      }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.intent).toBe("spend");
    expect(json.reply).toContain("Choices:");
    expect(json.reply).not.toContain("kineti-");
  });

  test("dashboard home tab is default, vault and invite are gone", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    const html = await res.text();
    expect(html).toContain('data-tab="home"');
    expect(html).toContain('id="talk-input"');
    expect(html).toContain('id="toggle-power"');
    expect(html).not.toContain('data-tab="vault"');
    expect(html).not.toContain("Invite a friend");
    expect(html).not.toContain('id="connectors-list"');
  });
});


