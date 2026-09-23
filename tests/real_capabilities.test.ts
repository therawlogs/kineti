import { describe, test, expect, beforeAll, afterAll } from "bun:test";
import { SchedulerEngine } from "../src/scheduler/engine";
import { WatcherManager } from "../src/scheduler/watchers";
import { startServer, AUTH_TOKEN } from "../bin/kineti-companion";
import { existsSync, unlinkSync } from "node:fs";
import { join } from "node:path";

describe("Scheduler & Companion Email Verification Suite", () => {
  // 1. Persistent Scheduler & Watchers Engine
  describe("1. Persistent Scheduler & Watchers Engine", () => {
    const testStorePath = join(process.cwd(), ".kineti", "test_jobs.json");

    afterAll(() => {
      if (existsSync(testStorePath)) unlinkSync(testStorePath);
    });

    test("parses cron expressions and calculates deterministic next run times", () => {
      const scheduler = new SchedulerEngine(testStorePath);
      const base = new Date("2026-10-01T12:00:00Z");

      // Hourly shortcut
      const nextHourly = scheduler.calculateNextCronRun("@hourly", base);
      expect(nextHourly.getMinutes()).toBe(0);
      expect(nextHourly.getHours()).toBe(13);

      // Every 15 minutes step (*/15)
      const next15 = scheduler.calculateNextCronRun("*/15 * * * *", base);
      expect(next15.getMinutes()).toBe(15);
      expect(next15.getHours()).toBe(12);

      // Specific minute and hour
      const nextSpecific = scheduler.calculateNextCronRun("30 9 * * *", base);
      expect(nextSpecific.getMinutes()).toBe(30);
      expect(nextSpecific.getHours()).toBe(9);
    });

    test("schedules, executes, and marks one-shot reminders as completed", async () => {
      const scheduler = new SchedulerEngine(testStorePath);
      // Scheduled 1 second in the past to trigger immediately
      const job = scheduler.addJob({
        name: "Quick Reminder",
        runAt: new Date(Date.now() - 1000),
        action: "send_ping",
        payload: { note: "Time to hydrate" },
      });

      expect(job.status).toBe("active");
      expect(job.runCount).toBe(0);

      let executedAction = "";
      const triggered = await scheduler.checkAndTriggerDueJobs(async (j) => {
        executedAction = j.action;
      });

      expect(triggered.length).toBeGreaterThanOrEqual(1);
      expect(executedAction).toBe("send_ping");

      // Check updated status in store
      const refreshed = scheduler.listJobs().find((j) => j.id === job.id);
      expect(refreshed?.status).toBe("completed");
      expect(refreshed?.runCount).toBe(1);
    });

    test("evaluates price drop watcher and triggers alert when price drops", async () => {
      const scheduler = new SchedulerEngine(testStorePath);
      const watcherMgr = new WatcherManager(scheduler);

      const job = watcherMgr.watchPrice({
        product: "Sony WH-1000XM5",
        targetPriceCents: 30000, // $300.00
      });

      expect(job.action).toBe("watch_price");

      // Simulate price above target -> no trigger
      const resHigh = await watcherMgr.evaluatePriceWatcher(job, async () => 34800);
      expect(resHigh.triggered).toBe(false);

      // Simulate price drop to $289.00 -> triggers!
      const resDrop = await watcherMgr.evaluatePriceWatcher(job, async () => 28900);
      expect(resDrop.triggered).toBe(true);
      expect(resDrop.message).toContain("Price Alert: Sony WH-1000XM5 dropped to $289.00");
    });

    test("evaluates package delivery status changes", () => {
      const scheduler = new SchedulerEngine(testStorePath);
      const watcherMgr = new WatcherManager(scheduler);

      const job = watcherMgr.watchPackage({
        carrier: "USPS",
        trackingNumber: "9400111899562134567890",
        description: "Studio Microphone",
      });

      // Status change to Out for Delivery
      const update1 = watcherMgr.evaluatePackageWatcher(job, "Out for Delivery");
      expect(update1.statusChanged).toBe(true);
      expect(update1.isDelivered).toBe(false);

      // Status change to Delivered -> completes job
      const update2 = watcherMgr.evaluatePackageWatcher(job, "Delivered, Front Door");
      expect(update2.statusChanged).toBe(true);
      expect(update2.isDelivered).toBe(true);
      expect(update2.message).toContain("Package Delivered");
      expect(job.status).toBe("completed");
    });
  });

  // 2. Companion Inbound/Outbound Email API
  describe("2. Companion Inbound & Outbound Email Webhooks", () => {
    let server: any;
    const testPort = 18790;
    const authHeaders = {
      "Authorization": `Bearer ${AUTH_TOKEN}`,
      "Content-Type": "application/json",
    };

    beforeAll(() => {
      server = startServer(testPort);
    });

    afterAll(() => {
      if (server) server.stop(true);
    });

    test("POST /api/email/inbound extracts OTP codes, tracking numbers, and links", async () => {
      const payload = {
        from: "notifications@github.com",
        to: "alex+github_401@mail.kineti.com",
        subject: "Your GitHub verification code",
        text: "Your one-time security code is 849201. Or click https://github.com/auth/verify?id=9988. Package tracking 1Z9999999999999999.",
      };

      const res = await server.fetch(
        new Request("http://localhost/api/email/inbound", {
          method: "POST",
          headers: authHeaders,
          body: JSON.stringify(payload),
        })
      );

      expect(res.status).toBe(200);
      const json = (await res.json()) as any;
      expect(json.success).toBe(true);
      expect(json.email.extracted.otpCode).toBe("849201");
      expect(json.email.extracted.trackingNumber).toBe("1Z9999999999999999");
      expect(json.email.extracted.links[0]).toContain("https://github.com/auth/verify");
    });

    test("POST /api/email/send and GET /api/email/list work end-to-end", async () => {
      const sendRes = await server.fetch(
        new Request("http://localhost/api/email/send", {
          method: "POST",
          headers: authHeaders,
          body: JSON.stringify({
            to: "client@example.com",
            subject: "Project Milestone Approved",
            body: "All acceptance tests passed with zero errors.",
          }),
        })
      );

      expect(sendRes.status).toBe(200);
      const sendJson = (await sendRes.json()) as any;
      expect(sendJson.success).toBe(true);
      expect(sendJson.email.direction).toBe("outbound");

      const listRes = await server.fetch(
        new Request("http://localhost/api/email/list", {
          method: "GET",
          headers: authHeaders,
        })
      );

      expect(listRes.status).toBe(200);
      const listJson = (await listRes.json()) as any;
      expect(Array.isArray(listJson.emails)).toBe(true);
      expect(listJson.emails.some((e: any) => e.subject === "Project Milestone Approved")).toBe(true);
    });
  });
});
