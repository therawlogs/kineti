import { describe, test, expect, afterAll } from "bun:test";
import { SchedulerEngine } from "../src/scheduler/engine";
import { WatcherManager } from "../src/scheduler/watchers";
import { existsSync, unlinkSync } from "node:fs";
import { join } from "node:path";

describe("Scheduler utility suite", () => {
  // 1. Persistent Scheduler & Watchers Engine
  describe("1. Persistent Scheduler & Watchers Engine", () => {
    const testStorePath = join(process.cwd(), ".kineti", "test_jobs.json");

    afterAll(() => {
      if (existsSync(testStorePath)) unlinkSync(testStorePath);
    });

    test("uses local calendar time and returns exact cron instants", () => {
      const scheduler = new SchedulerEngine(testStorePath);
      const base = new Date("2026-10-01T12:00:00.000Z");
      const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;
      const expected: Record<string, [string, string, string]> = {
        UTC: ["2026-10-01T13:00:00.000Z", "2026-10-01T12:15:00.000Z", "2026-10-02T09:30:00.000Z"],
        "Asia/Kolkata": ["2026-10-01T12:30:00.000Z", "2026-10-01T12:15:00.000Z", "2026-10-02T04:00:00.000Z"],
        "Asia/Calcutta": ["2026-10-01T12:30:00.000Z", "2026-10-01T12:15:00.000Z", "2026-10-02T04:00:00.000Z"],
        "America/New_York": ["2026-10-01T13:00:00.000Z", "2026-10-01T12:15:00.000Z", "2026-10-01T13:30:00.000Z"],
      };
      expect(expected[zone], `exact cron instants are specified for ${zone}`).toBeDefined();

      // Hourly shortcut
      const nextHourly = scheduler.calculateNextCronRun("@hourly", base);
      expect(nextHourly.toISOString()).toBe(expected[zone][0]);

      // Every 15 minutes step (*/15)
      const next15 = scheduler.calculateNextCronRun("*/15 * * * *", base);
      expect(next15.toISOString()).toBe(expected[zone][1]);

      // Specific minute and hour
      const nextSpecific = scheduler.calculateNextCronRun("30 9 * * *", base);
      expect(nextSpecific.toISOString()).toBe(expected[zone][2]);
    });

    test("skips nonexistent local times and visits both repeated local times", () => {
      if (Intl.DateTimeFormat().resolvedOptions().timeZone !== "America/New_York") return;
      const scheduler = new SchedulerEngine(testStorePath);

      const beforeSpringForward = new Date("2026-03-08T06:59:00.000Z");
      expect(scheduler.calculateNextCronRun("30 2 * * *", beforeSpringForward).toISOString())
        .toBe("2026-03-09T06:30:00.000Z");

      const beforeFirstFallBackMatch = new Date("2026-11-01T04:59:00.000Z");
      expect(scheduler.calculateNextCronRun("30 1 * * *", beforeFirstFallBackMatch).toISOString())
        .toBe("2026-11-01T05:30:00.000Z");

      const afterFirstFallBackMatch = new Date("2026-11-01T05:31:00.000Z");
      expect(scheduler.calculateNextCronRun("30 1 * * *", afterFirstFallBackMatch).toISOString())
        .toBe("2026-11-01T06:30:00.000Z");
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

});
