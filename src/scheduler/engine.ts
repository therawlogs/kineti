/**
 * Kineti Persistent Scheduler Engine
 *
 * Provides cron scheduling, one-shot reminders, persistent task state (.kineti/jobs.json),
 * and deterministic trigger evaluation.
 */

import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

export interface ScheduledJob {
  id: string;
  name: string;
  cron?: string;
  runAt?: string;
  action: string;
  payload: Record<string, any>;
  status: "active" | "completed" | "paused" | "failed";
  lastRunAt?: string;
  nextRunAt: string;
  runCount: number;
  maxRuns?: number;
  createdAt: string;
  lastError?: string;
}

export class SchedulerEngine {
  private jobsFile: string;

  constructor(jobsFilePath?: string) {
    this.jobsFile = jobsFilePath || join(process.cwd(), ".kineti", "jobs.json");
    this.ensureStore();
  }

  private ensureStore(): void {
    const dir = join(process.cwd(), ".kineti");
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
    if (!existsSync(this.jobsFile)) {
      writeFileSync(this.jobsFile, JSON.stringify([], null, 2), "utf-8");
    }
  }

  public listJobs(): ScheduledJob[] {
    try {
      this.ensureStore();
      const raw = readFileSync(this.jobsFile, "utf-8");
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }

  private saveJobs(jobs: ScheduledJob[]): void {
    this.ensureStore();
    writeFileSync(this.jobsFile, JSON.stringify(jobs, null, 2), "utf-8");
  }

  /**
   * Schedules a new recurring or one-shot job.
   */
  public addJob(spec: {
    name: string;
    cron?: string;
    runAt?: string | Date;
    action: string;
    payload?: Record<string, any>;
    maxRuns?: number;
  }): ScheduledJob {
    const now = new Date();
    let nextRun: Date;

    if (spec.runAt) {
      nextRun = typeof spec.runAt === "string" ? new Date(spec.runAt) : spec.runAt;
    } else if (spec.cron) {
      nextRun = this.calculateNextCronRun(spec.cron, now);
    } else {
      throw new Error("Must specify either 'cron' or 'runAt' for scheduled job");
    }

    const job: ScheduledJob = {
      id: `job_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
      name: spec.name,
      cron: spec.cron,
      runAt: spec.runAt ? (typeof spec.runAt === "string" ? spec.runAt : spec.runAt.toISOString()) : undefined,
      action: spec.action,
      payload: spec.payload || {},
      status: "active",
      nextRunAt: nextRun.toISOString(),
      runCount: 0,
      maxRuns: spec.maxRuns,
      createdAt: now.toISOString(),
    };

    const jobs = this.listJobs();
    jobs.push(job);
    this.saveJobs(jobs);

    return job;
  }

  /**
   * Adds a simple reminder ("in X minutes" or at specific time).
   */
  public addReminder(name: string, delayMs: number, message: string): ScheduledJob {
    const runAt = new Date(Date.now() + delayMs);
    return this.addJob({
      name,
      runAt,
      action: "reminder",
      payload: { message },
      maxRuns: 1,
    });
  }

  /**
   * Cancels a job by ID.
   */
  public removeJob(id: string): boolean {
    const jobs = this.listJobs();
    const filtered = jobs.filter((j) => j.id !== id);
    if (filtered.length !== jobs.length) {
      this.saveJobs(filtered);
      return true;
    }
    return false;
  }

  /**
   * Pauses a job.
   */
  public pauseJob(id: string): boolean {
    const jobs = this.listJobs();
    const job = jobs.find((j) => j.id === id);
    if (job) {
      job.status = "paused";
      this.saveJobs(jobs);
      return true;
    }
    return false;
  }

  /**
   * Evaluates and triggers any due jobs.
   */
  public async checkAndTriggerDueJobs(
    executor?: (job: ScheduledJob) => Promise<any>
  ): Promise<ScheduledJob[]> {
    const now = new Date();
    const jobs = this.listJobs();
    const triggered: ScheduledJob[] = [];

    for (const job of jobs) {
      if (job.status !== "active") continue;

      const due = new Date(job.nextRunAt);
      if (due <= now) {
        job.lastRunAt = now.toISOString();
        job.runCount += 1;
        triggered.push({ ...job });

        if (executor) {
          try {
            await executor(job);
          } catch (e: any) {
            job.lastError = e.message || String(e);
          }
        }

        // Check if one-shot or reached max runs
        if (job.maxRuns && job.runCount >= job.maxRuns) {
          job.status = "completed";
        } else if (job.cron) {
          // Schedule next run
          const next = this.calculateNextCronRun(job.cron, now);
          job.nextRunAt = next.toISOString();
        } else {
          job.status = "completed";
        }
      }
    }

    if (triggered.length > 0) {
      this.saveJobs(jobs);
    }

    return triggered;
  }

  /**
   * Calculates the next execution Date for a standard 5-part cron pattern or shortcut.
   */
  public calculateNextCronRun(cron: string, fromDate: Date = new Date()): Date {
    let expr = cron.trim();

    // Standard shortcuts
    if (expr === "@hourly") expr = "0 * * * *";
    else if (expr === "@daily" || expr === "@midnight") expr = "0 0 * * *";
    else if (expr === "@weekly") expr = "0 0 * * 0";
    else if (expr === "@monthly") expr = "0 0 1 * *";

    const parts = expr.split(/\s+/);
    if (parts.length !== 5) {
      throw new Error(`Invalid cron format '${cron}'. Expected 5 fields: minute hour dom month dow`);
    }

    const [minPart, hourPart, domPart, monPart, dowPart] = parts;

    // Fast-forward second to 0 of the next minute
    const candidate = new Date(fromDate.getTime() + 60_000);
    candidate.setSeconds(0, 0);

    // Look ahead up to 366 days (527,040 minutes)
    for (let i = 0; i < 527040; i++) {
      const min = candidate.getMinutes();
      const hour = candidate.getHours();
      const dom = candidate.getDate();
      const mon = candidate.getMonth() + 1; // 1-12
      const dow = candidate.getDay(); // 0-6 (0=Sun)

      if (
        this.fieldMatches(minPart, min, 0, 59) &&
        this.fieldMatches(hourPart, hour, 0, 23) &&
        this.fieldMatches(domPart, dom, 1, 31) &&
        this.fieldMatches(monPart, mon, 1, 12) &&
        this.fieldMatches(dowPart, dow, 0, 6)
      ) {
        return candidate;
      }

      // Increment by 1 minute
      candidate.setTime(candidate.getTime() + 60_000);
    }

    throw new Error(`Could not find next matching execution for cron '${cron}'`);
  }

  private fieldMatches(field: string, value: number, min: number, max: number): boolean {
    if (field === "*") return true;

    // Handle step e.g. "*/5" or "10-20/2"
    if (field.includes("/")) {
      const [range, stepStr] = field.split("/");
      const step = parseInt(stepStr, 10);
      if (isNaN(step) || step <= 0) return false;

      let start = min;
      let end = max;
      if (range !== "*") {
        if (range.includes("-")) {
          const [s, e] = range.split("-").map((v) => parseInt(v, 10));
          start = s;
          end = e;
        } else {
          start = parseInt(range, 10);
        }
      }

      if (value < start || value > end) return false;
      return (value - start) % step === 0;
    }

    // Handle lists e.g. "1,3,5"
    if (field.includes(",")) {
      const items = field.split(",");
      return items.some((item) => this.fieldMatches(item.trim(), value, min, max));
    }

    // Handle ranges e.g. "1-5"
    if (field.includes("-")) {
      const [start, end] = field.split("-").map((v) => parseInt(v, 10));
      return value >= start && value <= end;
    }

    // Single number
    const target = parseInt(field, 10);
    return value === target;
  }
}
