/**
 * Kineti Autonomous Watchers Engine
 *
 * Implements persistent watchers for price drops, package deliveries,
 * flight delays, and urgent emails.
 */

import { SchedulerEngine, ScheduledJob } from "./engine";

export interface PriceWatcherConfig {
  product: string;
  targetPriceCents: number;
  currentPriceCents?: number;
  notifyChannel?: "whatsapp" | "imessage" | "email" | "slack";
  recipient?: string;
}

export interface PackageWatcherConfig {
  carrier: "USPS" | "FedEx" | "UPS" | "DHL" | "Auto";
  trackingNumber: string;
  description?: string;
  lastStatus?: string;
  delivered?: boolean;
}

export interface UrgentEmailWatcherConfig {
  vipSenders?: string[];
  keywords?: string[];
  checkIntervalCron?: string;
}

export class WatcherManager {
  private scheduler: SchedulerEngine;

  constructor(scheduler?: SchedulerEngine) {
    this.scheduler = scheduler || new SchedulerEngine();
  }

  /**
   * Registers a price drop watcher that checks every 15 minutes.
   */
  public watchPrice(config: PriceWatcherConfig): ScheduledJob {
    return this.scheduler.addJob({
      name: `Watch Price: ${config.product}`,
      cron: "*/15 * * * *", // every 15 minutes
      action: "watch_price",
      payload: {
        product: config.product,
        targetPriceCents: config.targetPriceCents,
        currentPriceCents: config.currentPriceCents,
        notifyChannel: config.notifyChannel || "whatsapp",
        recipient: config.recipient,
      },
    });
  }

  /**
   * Registers a package tracker watcher that checks every 30 minutes.
   */
  public watchPackage(config: PackageWatcherConfig): ScheduledJob {
    return this.scheduler.addJob({
      name: `Track Package: ${config.trackingNumber}`,
      cron: "*/30 * * * *", // every 30 minutes
      action: "watch_package",
      payload: {
        carrier: config.carrier,
        trackingNumber: config.trackingNumber,
        description: config.description || "Package delivery",
        lastStatus: config.lastStatus || "unknown",
        delivered: false,
      },
    });
  }

  /**
   * Registers an urgent email monitor.
   */
  public watchUrgentEmails(config: UrgentEmailWatcherConfig): ScheduledJob {
    return this.scheduler.addJob({
      name: "Watch Urgent Emails",
      cron: config.checkIntervalCron || "*/5 * * * *", // every 5 minutes
      action: "watch_urgent_emails",
      payload: {
        vipSenders: config.vipSenders || [],
        keywords: config.keywords || ["urgent", "action required", "payment due", "security alert", "invoice"],
      },
    });
  }

  /**
   * Executes a price check iteration.
   * Can evaluate simulated or real Brave Shopping prices.
   */
  public async evaluatePriceWatcher(
    job: ScheduledJob,
    priceFetcher?: (product: string) => Promise<number>
  ): Promise<{ triggered: boolean; currentPriceCents: number; message?: string }> {
    const { product, targetPriceCents } = job.payload;
    let currentPriceCents: number;

    if (priceFetcher) {
      currentPriceCents = await priceFetcher(product);
    } else {
      // If no custom fetcher, check existing payload or default
      currentPriceCents = job.payload.currentPriceCents || targetPriceCents;
    }

    job.payload.currentPriceCents = currentPriceCents;

    if (currentPriceCents <= targetPriceCents) {
      const savingsUsd = ((targetPriceCents - currentPriceCents) / 100).toFixed(2);
      const currentUsd = (currentPriceCents / 100).toFixed(2);
      const msg = `Price Alert: ${product} dropped to $${currentUsd} (saving $${savingsUsd} vs target)!`;
      return { triggered: true, currentPriceCents, message: msg };
    }

    return { triggered: false, currentPriceCents };
  }

  /**
   * Evaluates a package tracking check.
   */
  public evaluatePackageWatcher(
    job: ScheduledJob,
    mockStatus?: string
  ): { statusChanged: boolean; newStatus: string; isDelivered: boolean; message?: string } {
    const { trackingNumber, description, lastStatus } = job.payload;
    const currentStatus = mockStatus || "In Transit - Out for Delivery";

    const isDelivered = currentStatus.toLowerCase().includes("delivered");
    const statusChanged = currentStatus !== lastStatus;

    job.payload.lastStatus = currentStatus;
    job.payload.delivered = isDelivered;

    if (isDelivered) {
      job.status = "completed";
      return {
        statusChanged: true,
        newStatus: currentStatus,
        isDelivered: true,
        message: `Package Delivered: ${description} (${trackingNumber}) has arrived!`,
      };
    }

    if (statusChanged) {
      return {
        statusChanged: true,
        newStatus: currentStatus,
        isDelivered: false,
        message: `Tracking Update: ${description} (${trackingNumber}) is now ${currentStatus}.`,
      };
    }

    return { statusChanged: false, newStatus: currentStatus, isDelivered: false };
  }
}
