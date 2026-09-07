import { describe, test, expect, beforeAll, afterAll } from "bun:test";
import { startServer, getHarnessStatus } from "../bin/kineti-companion.ts";

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

  test("GET / returns companion dashboard HTML", async () => {
    const res = await server.fetch(new Request("http://localhost/"));
    expect(res.status).toBe(200);
    const html = await res.text();
    expect(html).toContain("Kineti OS — Visual Companion Canvas");
    expect(html).toContain("Real-Time Spend Circuit Breaker");
    expect(html).toContain("13-Stage Pipeline");
  });

  test("GET /api/status returns JSON status", async () => {
    const res = await server.fetch(new Request("http://localhost/api/status"));
    expect(res.status).toBe(200);
    const json = (await res.json()) as any;
    expect(json.stages).toBeDefined();
    expect(json.stages.length).toBe(13);
    expect(json.spend.total_usd).toBeDefined();
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
