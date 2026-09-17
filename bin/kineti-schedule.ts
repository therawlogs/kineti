#!/usr/bin/env bun
/**
 * Kineti Persistent Scheduler CLI
 *
 * Command-line interface for managing cron jobs, reminders, price drop monitors,
 * and background tracking loops.
 */

import { SchedulerEngine } from "../src/scheduler/engine";
import { WatcherManager } from "../src/scheduler/watchers";

function printUsage(): void {
  console.log(`
Kineti Scheduler CLI — Autonomous Scheduling & Background Watchers

Usage:
  bun bin/kineti-schedule.ts list
  bun bin/kineti-schedule.ts add --name <name> --cron <cron> --action <action> [--payload <json>]
  bun bin/kineti-schedule.ts remind --name <name> --in <duration> --message <text>
  bun bin/kineti-schedule.ts watch-price --product <query> --target <cents>
  bun bin/kineti-schedule.ts watch-package --carrier <carrier> --tracking <number> [--desc <text>]
  bun bin/kineti-schedule.ts cancel --id <job_id>
  bun bin/kineti-schedule.ts trigger
  bun bin/kineti-schedule.ts daemon [--interval <seconds>]

Examples:
  bun bin/kineti-schedule.ts remind --name "Coffee" --in 15m --message "Check on espresso"
  bun bin/kineti-schedule.ts watch-price --product "Sony WH-1000XM5" --target 29900
  bun bin/kineti-schedule.ts add --name "Morning Briefing" --cron "0 9 * * 1-5" --action "briefing"
`);
}

function parseDurationMs(duration: string): number {
  const match = duration.match(/^(\d+)([smhd])$/i);
  if (!match) {
    throw new Error(`Invalid duration format '${duration}'. Use e.g. 30s, 15m, 2h, 1d`);
  }
  const val = parseInt(match[1], 10);
  const unit = match[2].toLowerCase();
  switch (unit) {
    case "s":
      return val * 1000;
    case "m":
      return val * 60 * 1000;
    case "h":
      return val * 60 * 60 * 1000;
    case "d":
      return val * 24 * 60 * 60 * 1000;
    default:
      return val * 60 * 1000;
  }
}

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  const command = args[0];

  const scheduler = new SchedulerEngine();
  const watcherManager = new WatcherManager(scheduler);

  if (!command || command === "help" || command === "--help") {
    printUsage();
    process.exit(0);
  }

  if (command === "list") {
    const jobs = scheduler.listJobs();
    if (jobs.length === 0) {
      console.log("No scheduled jobs found.");
      return;
    }
    console.log(`\nFound ${jobs.length} scheduled job(s):\n`);
    for (const j of jobs) {
      const scheduleDesc = j.cron ? `Cron: ${j.cron}` : `Run at: ${j.runAt || j.nextRunAt}`;
      console.log(`• [${j.status.toUpperCase()}] ${j.name} (${j.id})`);
      console.log(`  Action: ${j.action} | Next: ${j.nextRunAt} | ${scheduleDesc}`);
      if (j.payload && Object.keys(j.payload).length > 0) {
        console.log(`  Payload: ${JSON.stringify(j.payload)}`);
      }
      console.log();
    }
    return;
  }

  if (command === "add") {
    const nameIdx = args.indexOf("--name");
    const cronIdx = args.indexOf("--cron");
    const actionIdx = args.indexOf("--action");
    const payloadIdx = args.indexOf("--payload");

    if (nameIdx === -1 || cronIdx === -1 || actionIdx === -1) {
      console.error("Error: --name, --cron, and --action are required.");
      process.exit(1);
    }

    const name = args[nameIdx + 1];
    const cron = args[cronIdx + 1];
    const action = args[actionIdx + 1];
    let payload = {};
    if (payloadIdx !== -1 && args[payloadIdx + 1]) {
      try {
        payload = JSON.parse(args[payloadIdx + 1]);
      } catch (e) {
        console.error("Error parsing --payload JSON");
        process.exit(1);
      }
    }

    const job = scheduler.addJob({ name, cron, action, payload });
    console.log(`✓ Scheduled job '${job.name}' (${job.id})`);
    console.log(`  Next execution: ${job.nextRunAt}`);
    return;
  }

  if (command === "remind") {
    const nameIdx = args.indexOf("--name");
    const inIdx = args.indexOf("--in");
    const msgIdx = args.indexOf("--message");

    if (nameIdx === -1 || inIdx === -1 || msgIdx === -1) {
      console.error("Error: --name, --in, and --message are required.");
      process.exit(1);
    }

    const name = args[nameIdx + 1];
    const duration = args[inIdx + 1];
    const message = args[msgIdx + 1];
    const delayMs = parseDurationMs(duration);

    const job = scheduler.addReminder(name, delayMs, message);
    console.log(`✓ Reminder scheduled: '${name}' in ${duration}`);
    console.log(`  Execution time: ${job.nextRunAt}`);
    return;
  }

  if (command === "watch-price") {
    const prodIdx = args.indexOf("--product");
    const targetIdx = args.indexOf("--target");

    if (prodIdx === -1 || targetIdx === -1) {
      console.error("Error: --product and --target are required.");
      process.exit(1);
    }

    const product = args[prodIdx + 1];
    const targetPriceCents = parseInt(args[targetIdx + 1], 10);

    const job = watcherManager.watchPrice({ product, targetPriceCents });
    console.log(`✓ Price watcher created for '${product}' (Target: $${(targetPriceCents / 100).toFixed(2)})`);
    console.log(`  Job ID: ${job.id}`);
    return;
  }

  if (command === "watch-package") {
    const carrierIdx = args.indexOf("--carrier");
    const trackIdx = args.indexOf("--tracking");
    const descIdx = args.indexOf("--desc");

    if (trackIdx === -1) {
      console.error("Error: --tracking is required.");
      process.exit(1);
    }

    const carrier = (carrierIdx !== -1 ? args[carrierIdx + 1] : "Auto") as any;
    const trackingNumber = args[trackIdx + 1];
    const description = descIdx !== -1 ? args[descIdx + 1] : "Package";

    const job = watcherManager.watchPackage({ carrier, trackingNumber, description });
    console.log(`✓ Package watcher created for ${carrier} #${trackingNumber}`);
    console.log(`  Job ID: ${job.id}`);
    return;
  }

  if (command === "cancel") {
    const idIdx = args.indexOf("--id");
    if (idIdx === -1 || !args[idIdx + 1]) {
      console.error("Error: --id <job_id> required.");
      process.exit(1);
    }
    const id = args[idIdx + 1];
    const removed = scheduler.removeJob(id);
    if (removed) {
      console.log(`✓ Cancelled job '${id}'`);
    } else {
      console.log(`Job '${id}' not found.`);
    }
    return;
  }

  if (command === "trigger") {
    console.log("Checking due jobs...");
    const triggered = await scheduler.checkAndTriggerDueJobs(async (job) => {
      console.log(`⚡ Triggered job: ${job.name} (${job.action})`);
    });
    console.log(`Completed check. Triggered ${triggered.length} job(s).`);
    return;
  }

  if (command === "daemon") {
    const intervalIdx = args.indexOf("--interval");
    const intervalSec = intervalIdx !== -1 ? parseInt(args[intervalIdx + 1], 10) : 30;
    console.log(`Starting Kineti Scheduler Daemon (checking every ${intervalSec}s)...`);

    setInterval(async () => {
      try {
        await scheduler.checkAndTriggerDueJobs(async (job) => {
          console.log(`[${new Date().toISOString()}] ⚡ Executed: ${job.name} (${job.action})`);
        });
      } catch (err) {
        console.error("Daemon evaluation error:", err);
      }
    }, intervalSec * 1000);
    return;
  }

  console.error(`Unknown command: ${command}`);
  printUsage();
  process.exit(1);
}

if (import.meta.main) {
  main().catch((err) => {
    console.error("Fatal error:", err);
    process.exit(1);
  });
}
