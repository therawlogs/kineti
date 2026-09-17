import { describe, test, expect, beforeAll, afterAll } from "bun:test";
import { VoiceCallService } from "../src/telephony/voice_call";
import { MultimodalVisionEngine } from "../src/multimodal/vision";
import { PdfGenerator } from "../src/documents/pdf_generator";
import { SchedulerEngine } from "../src/scheduler/engine";
import { WatcherManager } from "../src/scheduler/watchers";
import { AutonomousBrowserAgent } from "../src/browser/playwright_agent";
import { startServer, AUTH_TOKEN } from "../bin/kineti-companion";
import { existsSync, unlinkSync } from "node:fs";
import { join } from "node:path";

describe("Autonomous Capabilities & Dual Engine Verification Suite", () => {
  // 1. Telephony Voice Call Service
  describe("1. Telephony Voice Call Service", () => {
    test("generates valid TwiML XML with custom voice and verbs", () => {
      const telephony = new VoiceCallService();
      const twiml = telephony.generateTwiml([
        { type: "say", text: "Hello from Kineti! Press 1 to confirm.", voice: "Polly.Joanna" },
        { type: "gather", numDigits: 1, timeoutSeconds: 5, prompt: "Please press a key" },
        { type: "hangup" },
      ]);

      expect(twiml).toContain('<?xml version="1.0" encoding="UTF-8"?>');
      expect(twiml).toContain("<Response>");
      expect(twiml).toContain('<Say voice="Polly.Joanna">Hello from Kineti! Press 1 to confirm.</Say>');
      expect(twiml).toContain('<Gather numDigits="1" timeout="5">');
      expect(twiml).toContain("<Hangup/>");
      expect(twiml).toContain("</Response>");
    });

    test("escapes special characters safely in TwiML", () => {
      const telephony = new VoiceCallService();
      const twiml = telephony.generateTwiml([
        { type: "say", text: "Bill & Ted's \"Big Adventure\" <2026>" },
      ]);
      expect(twiml).toContain("Bill &amp; Ted&apos;s &quot;Big Adventure&quot; &lt;2026&gt;");
    });

    test("fails gracefully with clear error when Twilio credentials missing", async () => {
      const telephony = new VoiceCallService({ accountSid: "", authToken: "" });
      expect(
        telephony.initiateCall({ to: "+1234567890" })
      ).rejects.toThrow("Twilio credentials not configured");
    });

    test("processes incoming status webhooks and persists call history", () => {
      const telephony = new VoiceCallService();
      const record = telephony.handleStatusWebhook({
        CallSid: "CA_test_sid_123",
        From: "+15550001",
        To: "+15550002",
        CallStatus: "completed",
        CallDuration: "45",
      });

      expect(record).not.toBeNull();
      expect(record?.callSid).toBe("CA_test_sid_123");
      expect(record?.status).toBe("completed");
      expect(record?.durationSeconds).toBe(45);

      const allCalls = telephony.listCalls();
      const found = allCalls.find((c) => c.callSid === "CA_test_sid_123");
      expect(found).toBeDefined();
      expect(found?.status).toBe("completed");
    });
  });

  // 2. Multimodal Vision & OCR
  describe("2. Multimodal Vision & OCR Engine", () => {
    const vision = new MultimodalVisionEngine();

    test("parses raw receipt text into structured financial fields", () => {
      const receiptSample = `
Trader Joe's
Store #123 - San Francisco, CA
Date: 10/14/2026

Organic Avocados      3.99
Almond Milk           2.49
2x Dark Chocolate Bar 5.98
Subtotal             12.46
Tax                   1.12
Total Paid           13.58
Visa ending in 4242
Thank you for shopping!
      `;

      const parsed = vision.parseReceipt(receiptSample);
      expect(parsed.merchant).toBe("Trader Joe's");
      expect(parsed.date).toBe("10/14/2026");
      expect(parsed.items.length).toBeGreaterThanOrEqual(2);
      expect(parsed.subtotalCents).toBe(1246);
      expect(parsed.taxCents).toBe(112);
      expect(parsed.totalCents).toBe(1358);
      expect(parsed.paymentMethod).toContain("Visa");
      expect(parsed.confidence).toBeGreaterThan(0.8);
    });

    test("extracts key-values and structured sections from documents", () => {
      const docSample = `
Quarterly Cloud Infrastructure Summary
Invoice Number: INV-2026-9081
Due Date: 2026-11-01
Account: Production-West
Cluster: us-west-2a
      `;

      const result = vision.extractDocumentText(docSample);
      expect(result.title).toBe("Quarterly Cloud Infrastructure Summary");
      expect(result.keyValues["Invoice Number"]).toBe("INV-2026-9081");
      expect(result.keyValues["Due Date"]).toBe("2026-11-01");
      expect(result.keyValues["Account"]).toBe("Production-West");
    });

    test("analyzes screenshots to detect interactive buttons and UI elements", () => {
      const screenshotText = `
SETTINGS & ACCESS
[Sign in with Google]
(Submit Order)
[Cancel]
Enter Verification Code: ______
Error: Invalid two-factor token provided
      `;

      const analysis = vision.analyzeScreenshot(screenshotText);
      expect(analysis.detectedButtons).toContain("Sign in with Google");
      expect(analysis.detectedButtons).toContain("Submit Order");
      expect(analysis.detectedButtons).toContain("Cancel");
      expect(analysis.detectedInputs.some((i) => i.includes("Verification Code"))).toBe(true);
      expect(analysis.errorMessages.some((e) => e.includes("Invalid two-factor token"))).toBe(true);
    });
  });

  // 3. Structured PDF Generator
  describe("3. Native PDF Document Generator", () => {
    const pdfGen = new PdfGenerator();

    test("generates compliant PDF 1.4 binary stream with xref and EOF", () => {
      const pdfBytes = pdfGen.generateDocument({
        title: "Kineti Autonomous Task Report",
        subtitle: "Weekly Executive Summary",
        metadata: {
          "Author": "Kineti Core Agent",
          "Verification Level": "Deterministic Level-3",
        },
        sections: [
          {
            heading: "Infrastructure Health",
            body: "All background watchers, reflexes, and memory stores verified operational with 0 errors.",
          },
        ],
        footer: "Kineti OS Autonomous Agent",
      });

      expect(pdfBytes).toBeInstanceOf(Uint8Array);
      const text = Buffer.from(pdfBytes).toString("latin1");
      expect(text.startsWith("%PDF-1.4")).toBe(true);
      expect(text).toContain("/Type /Catalog");
      expect(text).toContain("/Type /Pages");
      expect(text).toContain("/Type /Font");
      expect(text).toContain("xref");
      expect(text).toContain("trailer");
      expect(text).toContain("%%EOF");
    });

    test("generates structured merchant receipt PDF", () => {
      const receiptPdf = pdfGen.generateReceipt({
        merchant: "Acme Cloud Services",
        receiptNumber: "RC-77890",
        date: "2026-09-17",
        items: [
          { name: "GPU Compute Hours (A100)", quantity: 10, priceUsd: 2.5 },
          { name: "Persistent Storage NVMe (1TB)", quantity: 1, priceUsd: 45.0 },
        ],
        subtotalUsd: 70.0,
        taxUsd: 5.6,
        totalUsd: 75.6,
        paymentMethod: "Corporate Card •••• 9811",
      });

      expect(receiptPdf.length).toBeGreaterThan(500);
      const str = Buffer.from(receiptPdf).toString("latin1");
      expect(str).toContain("Acme Cloud Services");
      expect(str).toContain("RC-77890");
    });

    test("generates flight travel itinerary PDF", () => {
      const itineraryPdf = pdfGen.generateItinerary({
        passengerName: "Dr. Praveen Kumar",
        bookingReference: "KINETI9X",
        flights: [
          {
            flightNumber: "UA 882",
            carrier: "United Airlines",
            origin: "SFO",
            destination: "NRT",
            departureTime: "11:30 AM",
            arrivalTime: "3:00 PM (+1)",
            seat: "12A",
          },
        ],
        hotelReservations: [
          {
            hotelName: "Park Hyatt Tokyo",
            checkIn: "2026-10-18",
            checkOut: "2026-10-24",
            confirmationCode: "HYATT-4091",
          },
        ],
      });

      expect(itineraryPdf.length).toBeGreaterThan(500);
      const str = Buffer.from(itineraryPdf).toString("latin1");
      expect(str).toContain("Travel Itinerary");
      expect(str).toContain("KINETI9X");
    });
  });

  // 4. Persistent Scheduler & Watchers Engine
  describe("4. Persistent Scheduler & Watchers Engine", () => {
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

  // 5. Autonomous Browser Agent
  describe("5. Autonomous Browser Agent", () => {
    test("blocks untrusted protocols (e.g. file://, ftp://)", async () => {
      const browser = new AutonomousBrowserAgent();
      expect(browser.navigate("file:///etc/passwd")).rejects.toThrow("Security violation");
      expect(browser.navigate("ftp://example.com/file")).rejects.toThrow("Security violation");
    });

    test("executes autonomous checkout and returns verifiable order summary", async () => {
      const browser = new AutonomousBrowserAgent();
      const checkout = await browser.autoCheckout({
        fullName: "Praveen",
        email: "praveen@example.com",
        phone: "+14155550199",
        addressLine1: "100 Market St",
        city: "San Francisco",
        state: "CA",
        postalCode: "94105",
      });

      expect(checkout.success).toBe(true);
      expect(checkout.orderConfirmation).toMatch(/^ord_/);
      expect(checkout.stepReached).toBe("completed_confirmation");
      expect(checkout.receiptSummary).toContain("Praveen");
      expect(checkout.receiptSummary).toContain("100 Market St");
    });
  });

  // 6. Companion Inbound/Outbound Email API
  describe("6. Companion Inbound & Outbound Email Webhooks", () => {
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
        to: "prav+github_401@mail.kineti.com",
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
