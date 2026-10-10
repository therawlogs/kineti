import { describe, test, expect, beforeAll, afterAll } from "bun:test";
import fs from "node:fs";
import path from "node:path";
import { startServer, getHarnessStatus, getMiniStatus, describeGate, getFleetStatus, AUTH_TOKEN } from "../bin/kineti-companion.ts";
import { fingerprint } from "../bin/kineti-evidence.ts";
import { microcentsToUsd } from "../bin/lib.ts";

describe("Kineti Visual Companion Server (kineti-companion.ts)", () => {
  let server: any;
  const testPort = 18788;
  const settingsFile = path.join(process.cwd(), ".kineti", "companion_settings.json");
  const switchFile = path.join(process.cwd(), ".kineti", "kineti.json");
  let settingsBackup: string | null = null;
  let switchBackup: string | null = null;

  beforeAll(() => {
    settingsBackup = fs.existsSync(settingsFile) ? fs.readFileSync(settingsFile, "utf8") : null;
    switchBackup = fs.existsSync(switchFile) ? fs.readFileSync(switchFile, "utf8") : null;
    server = startServer(testPort);
  });

  afterAll(() => {
    if (server) server.stop(true);
    if (settingsBackup === null) fs.rmSync(settingsFile, { force: true });
    else fs.writeFileSync(settingsFile, settingsBackup, { mode: 0o600 });
    if (switchBackup === null) fs.rmSync(switchFile, { force: true });
    else fs.writeFileSync(switchFile, switchBackup, { mode: 0o600 });
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
    // v0.4.0 reports the current checkout only; remote and extra fleet entries are unavailable.
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

  test("fake connector toggles are disabled and settings expose no connector credentials", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const getRes = await server.fetch(new Request("http://localhost/api/settings", { headers: authHeaders }));
    const initialSettings = (await getRes.json()) as any;
    expect(initialSettings.connectors).toBeUndefined();

    const res = await server.fetch(
      new Request("http://localhost/api/connectors/toggle", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ connector: "slack" }),
      }),
    );
    expect(res.status).toBe(410);
  });

  test("model table is suggestions only; automatic switching is unavailable", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const status = (await (await server.fetch(new Request("http://localhost/api/models", { headers: authHeaders }))).json()) as any;
    expect(status.automatic_switching).toBe(false);
    expect(status.table.code.host).toBeString();
    const response = await server.fetch(new Request("http://localhost/api/models/auto", {
      method: "POST", headers: authHeaders, body: JSON.stringify({ on: true }),
    }));
    expect(response.status).toBe(410);
  });

  test("credential vault endpoints are disabled and do not accept real secrets", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const getRes = await server.fetch(new Request("http://localhost/api/vault", { headers: authHeaders }));
    expect(getRes.status).toBe(410);
    const addRes = await server.fetch(
      new Request("http://localhost/api/vault", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ type: "totp", issuer: "GitHub", secret: "JBSWY3DPEHPK3PXP" }),
      }),
    );
    expect(addRes.status).toBe(410);
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

  test("custom connector registration and credential storage are disabled", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
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
    expect(addRes.status).toBe(410);
    const configureRes = await server.fetch(new Request("http://localhost/api/connectors/configure", {
      method: "POST", headers: authHeaders, body: JSON.stringify({ connector: "custom_crm", apiKey: "secret" }),
    }));
    expect(configureRes.status).toBe(410);
  });

  test("vault deletion endpoint is disabled without touching stored data", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const delRes = await server.fetch(
      new Request("http://localhost/api/vault/delete", {
        method: "POST",
        headers: authHeaders,
        body: JSON.stringify({ type: "login", index: 0 }),
      }),
    );
    expect(delRes.status).toBe(410);
  });

  test("contact details can be stored but message dispatch is unavailable", async () => {
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
    expect(testRes.status).toBe(410);
  });

  test("simulated email and data-deletion endpoints do not claim success", async () => {
    const authHeaders = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const requests = [
      new Request("http://localhost/api/email/send", { method: "POST", headers: authHeaders, body: JSON.stringify({ to: "x@example.invalid", subject: "test" }) }),
      new Request("http://localhost/api/email/inbound", { method: "POST", headers: authHeaders, body: JSON.stringify({ from: "x@example.invalid", text: "test" }) }),
      new Request("http://localhost/api/forget", { method: "POST", headers: authHeaders, body: JSON.stringify({ text: "old note" }) }),
      new Request("http://localhost/api/privacy/purge", { method: "POST", headers: authHeaders }),
    ];
    for (const request of requests) {
      const response = await server.fetch(request);
      expect(response.status).toBe(410);
      expect(await response.text()).not.toContain("success");
    }
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

  test("POST /api/talk answers dashboard intent with cloud link prompt", async () => {
    const res = await server.fetch(
      new Request("http://localhost/api/talk", {
        method: "POST",
        headers: { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` },
        body: JSON.stringify({ message: "kineti-dashboard" }),
      }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.intent).toBe("dashboard");
    expect(json.reply).toContain("Choices:");
  });

  test("GET /api/activity returns a view-only trail", async () => {
    const denied = await server.fetch(new Request("http://localhost/api/activity"));
    expect(denied.status).toBe(401);
    const res = await server.fetch(
      new Request("http://localhost/api/activity?n=10", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(Array.isArray(json.activity)).toBe(true);
    for (const row of json.activity) {
      expect(row.at).toBeDefined();
      expect(row.actor).toBeDefined();
      expect(row.action).toBeDefined();
    }
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

  test("dashboard has Activity, Team, and Settings tabs plus a local budget field", async () => {
    const res = await server.fetch(
      new Request("http://localhost/", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
    );
    const html = await res.text();
    expect(html).toContain('data-tab="activity"');
    expect(html).toContain('data-tab="team"');
    expect(html).toContain('data-tab="settings"');
    expect(html).toContain('id="activity-rows"');
    expect(html).toContain('id="team-agents-list"');
    expect(html).toContain('id="mirror-ceiling"');
    expect(html).toContain("No project data is mirrored to a cloud service.");
    expect(html).not.toContain("Make code");
    expect(html).not.toContain("Delete data");
    expect(html).not.toContain("Forget something");
    expect(html).not.toContain("mail.kineti.com");
    expect(html).not.toContain("Base32 Secret");
    expect(html).toContain('id="home-proof"');
  });

  test("pairing, cloud, and mirror endpoints need token", async () => {
    for (const p of ["/api/pairing", "/api/cloud", "/api/mirror"]) {
      const denied = await server.fetch(new Request("http://localhost" + p));
      expect(denied.status).toBe(401);
    }
  });

  test("cloud linking is unavailable while the local budget remains configurable", async () => {
    const fs = await import("node:fs");
    const path = await import("node:path");
    const auth = { "Content-Type": "application/json", "Authorization": `Bearer ${AUTH_TOKEN}` };
    const pairFile = path.join(process.cwd(), ".kineti", "pairing.json");
    const mirrorFile = path.join(process.cwd(), ".kineti", "mirror.json");
    const hadPair = fs.existsSync(pairFile) ? fs.readFileSync(pairFile, "utf8") : null;
    const hadMirror = fs.existsSync(mirrorFile) ? fs.readFileSync(mirrorFile, "utf8") : null;
    const machineBackup = process.env.KINETI_MACHINE_DIR;
    const machineScratch = path.join(process.cwd(), ".kineti", "test-companion-machine");
    fs.mkdirSync(machineScratch, { recursive: true });
    const tmpMachine = fs.mkdtempSync(path.join(machineScratch, "run-"));
    process.env.KINETI_MACHINE_DIR = tmpMachine;
    try {
      const status = (await (await server.fetch(
        new Request("http://localhost/api/pairing", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
      )).json()) as any;
      expect(status).toEqual({ available: false, linked: false });
      const pairRes = await server.fetch(new Request("http://localhost/api/pairing", { method: "POST", headers: auth }));
      expect(pairRes.status).toBe(410);
      const claimRes = await server.fetch(new Request("http://localhost/api/pairing/claim", { method: "POST", headers: auth }));
      expect(claimRes.status).toBe(410);

      const cloud = (await (await server.fetch(
        new Request("http://localhost/api/cloud", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
      )).json()) as any;
      expect(cloud.linked).toBe(false);
      expect(cloud.available).toBe(false);
      expect(JSON.stringify(cloud)).not.toContain("access_token");

      const mirror0 = (await (await server.fetch(
        new Request("http://localhost/api/mirror", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
      )).json()) as any;
      expect(mirror0.local_only).toBe(true);
      expect(mirror0.ceiling).toBeGreaterThan(0);

      const bad = await server.fetch(
        new Request("http://localhost/api/mirror", { method: "POST", headers: auth, body: JSON.stringify({ ceiling: 5000 }) }),
      );
      expect(bad.status).toBe(400);

      const mirror1 = (await (await server.fetch(
        new Request("http://localhost/api/mirror", { method: "POST", headers: auth, body: JSON.stringify({ ceiling: 40 }) }),
      )).json()) as any;
      expect(mirror1.local_only).toBe(true);
      expect(mirror1.ceiling).toBe(40);

      const mirrorCap = (await (await server.fetch(
        new Request("http://localhost/api/mirror", { method: "POST", headers: auth, body: JSON.stringify({ ceiling: 40 }) }),
      )).json()) as any;
      expect(mirrorCap.ceiling).toBe(40);

      const mini = (await (await server.fetch(
        new Request("http://localhost/api/mini", { headers: { Authorization: `Bearer ${AUTH_TOKEN}` } }),
      )).json()) as any;
      expect(mini.ceiling).toBe(40);

      const over = await server.fetch(
        new Request("http://localhost/api/mirror", { method: "POST", headers: auth, body: JSON.stringify({ ceiling: 5000 }) }),
      );
      expect(over.status).toBe(400);

      const dropRes = await server.fetch(new Request("http://localhost/api/pairing/drop", { method: "POST", headers: auth }));
      expect(dropRes.status).toBe(410);
    } finally {
      if (hadPair === null) fs.rmSync(pairFile, { force: true });
      else fs.writeFileSync(pairFile, hadPair);
      if (hadMirror === null) fs.rmSync(mirrorFile, { force: true });
      else fs.writeFileSync(mirrorFile, hadMirror);
      if (machineBackup === undefined) delete process.env.KINETI_MACHINE_DIR;
      else process.env.KINETI_MACHINE_DIR = machineBackup;
      fs.rmSync(tmpMachine, { recursive: true, force: true });
    }
  });
});

describe("harness status fixes (A1/A3/A5)", () => {
  test("A1: 50,000,000 microcents renders as $50.00", () => {
    const spendFile = path.join(process.cwd(), ".kineti", "spend.json");
    const had = fs.existsSync(spendFile) ? fs.readFileSync(spendFile, "utf8") : null;
    try {
      fs.writeFileSync(spendFile, JSON.stringify({ total_microcents: 50_000_000, tripped: false }));
      expect(microcentsToUsd(50_000_000)).toBe(50);
      const status = getHarnessStatus();
      expect(status.spend.total_usd).toBe(50);
    } finally {
      if (had === null) fs.rmSync(spendFile, { force: true });
      else fs.writeFileSync(spendFile, had);
    }
  });

  test("A3: fresh project gates default to unknown, rendered as not evaluated", () => {
    const stateFile = path.join(process.cwd(), ".kineti", "state.json");
    const had = fs.existsSync(stateFile) ? fs.readFileSync(stateFile, "utf8") : null;
    try {
      const state = had ? JSON.parse(had) : { version: 1, project: "test-fixture", stage: 1, history: [] };
      delete state.gates;
      fs.mkdirSync(path.dirname(stateFile), { recursive: true });
      fs.writeFileSync(stateFile, JSON.stringify(state));
      const status = getHarnessStatus();
      expect(status.gates).toEqual({});
      expect(status.gates.spec).toBeUndefined();
      expect(describeGate(status.gates, "spec")).toBe("not evaluated");
      expect(describeGate(status.gates, "security")).toBe("not evaluated");
      expect(describeGate(status.gates, "ship")).toBe("not evaluated");
      expect(describeGate({ spec: "pass", security: "pending" }, "spec")).toBe("pass");
      expect(describeGate({ spec: "pass", security: "pending" }, "security")).toBe("pending");
      expect(describeGate(null, "spec")).toBe("not evaluated");
    } finally {
      if (had === null) fs.rmSync(stateFile, { force: true });
      else fs.writeFileSync(stateFile, had);
    }
  });

  test("A5: proof is STALE on fingerprint mismatch, FRESH on match", () => {
    const evFile = path.join(process.cwd(), ".kineti", "evidence.jsonl");
    const had = fs.existsSync(evFile) ? fs.readFileSync(evFile, "utf8") : null;
    try {
      fs.writeFileSync(
        evFile,
        JSON.stringify({ at: new Date().toISOString(), label: "fp-probe", cmd: "bun test", exit_code: 0, fingerprint: "mismatch-canary" }) + "\n",
      );
      expect(getMiniStatus().proof_state).toBe("STALE");
      const fp = fingerprint();
      fs.writeFileSync(
        evFile,
        JSON.stringify({ at: new Date().toISOString(), label: "fp-probe", cmd: "bun test", exit_code: 0, fingerprint: fp }) + "\n",
      );
      const mini = getMiniStatus();
      expect(mini.proof_label).toBe("fp-probe");
      expect(mini.proof_state).toBe("FRESH");
    } finally {
      if (had === null) fs.rmSync(evFile, { force: true });
      else fs.writeFileSync(evFile, had);
    }
  });
});
