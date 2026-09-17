/**
 * Kineti Telephony Voice Call Service
 *
 * Provides inbound and outbound telephony calls, TwiML IVR response generation,
 * DTMF input processing, and persistent call tracking.
 */

import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";

export interface TelephonyConfig {
  accountSid?: string;
  authToken?: string;
  fromNumber?: string;
  callsFilePath?: string;
}

export interface CallRecord {
  callSid: string;
  to: string;
  from: string;
  status: "queued" | "ringing" | "in-progress" | "completed" | "busy" | "failed" | "no-answer";
  direction: "outbound-api" | "inbound";
  durationSeconds?: number;
  twiml?: string;
  createdAt: string;
  updatedAt: string;
  notes?: string;
}

export type TwimlVerb =
  | { type: "say"; text: string; voice?: string; language?: string }
  | { type: "gather"; action?: string; numDigits?: number; timeoutSeconds?: number; prompt?: string }
  | { type: "record"; action?: string; maxLengthSeconds?: number; transcribe?: boolean }
  | { type: "dial"; number: string; timeoutSeconds?: number }
  | { type: "hangup" };

export class VoiceCallService {
  private accountSid: string;
  private authToken: string;
  private fromNumber: string;
  private callsFile: string;

  constructor(config: TelephonyConfig = {}) {
    this.accountSid = config.accountSid || process.env.TWILIO_ACCOUNT_SID || "";
    this.authToken = config.authToken || process.env.TWILIO_AUTH_TOKEN || "";
    this.fromNumber = config.fromNumber || process.env.TWILIO_PHONE_NUMBER || "+15005550006";
    this.callsFile = config.callsFilePath || join(process.cwd(), ".kineti", "calls.json");
    this.ensureStore();
  }

  private ensureStore(): void {
    const dir = join(process.cwd(), ".kineti");
    if (!existsSync(dir)) {
      mkdirSync(dir, { recursive: true });
    }
    if (!existsSync(this.callsFile)) {
      writeFileSync(this.callsFile, JSON.stringify([], null, 2), "utf-8");
    }
  }

  public listCalls(): CallRecord[] {
    try {
      this.ensureStore();
      const raw = readFileSync(this.callsFile, "utf-8");
      return JSON.parse(raw);
    } catch {
      return [];
    }
  }

  private saveCalls(calls: CallRecord[]): void {
    this.ensureStore();
    writeFileSync(this.callsFile, JSON.stringify(calls, null, 2), "utf-8");
  }

  /**
   * Generates standard TwiML XML for Twilio voice interactions.
   */
  public generateTwiml(verbs: TwimlVerb[]): string {
    let xml = '<?xml version="1.0" encoding="UTF-8"?>\n<Response>\n';
    for (const verb of verbs) {
      switch (verb.type) {
        case "say": {
          const voiceAttr = verb.voice ? ` voice="${verb.voice}"` : ' voice="Polly.Joanna"';
          const langAttr = verb.language ? ` language="${verb.language}"` : "";
          xml += `  <Say${voiceAttr}${langAttr}>${this.escapeXml(verb.text)}</Say>\n`;
          break;
        }
        case "gather": {
          const actionAttr = verb.action ? ` action="${verb.action}"` : "";
          const digitsAttr = verb.numDigits ? ` numDigits="${verb.numDigits}"` : "";
          const timeoutAttr = verb.timeoutSeconds ? ` timeout="${verb.timeoutSeconds}"` : ' timeout="5"';
          xml += `  <Gather${actionAttr}${digitsAttr}${timeoutAttr}>\n`;
          if (verb.prompt) {
            xml += `    <Say voice="Polly.Joanna">${this.escapeXml(verb.prompt)}</Say>\n`;
          }
          xml += `  </Gather>\n`;
          break;
        }
        case "record": {
          const actionAttr = verb.action ? ` action="${verb.action}"` : "";
          const lenAttr = verb.maxLengthSeconds ? ` maxLength="${verb.maxLengthSeconds}"` : ' maxLength="60"';
          const transAttr = verb.transcribe ? ' transcribe="true"' : "";
          xml += `  <Record${actionAttr}${lenAttr}${transAttr}/>\n`;
          break;
        }
        case "dial": {
          const timeoutAttr = verb.timeoutSeconds ? ` timeout="${verb.timeoutSeconds}"` : "";
          xml += `  <Dial${timeoutAttr}>${this.escapeXml(verb.number)}</Dial>\n`;
          break;
        }
        case "hangup": {
          xml += `  <Hangup/>\n`;
          break;
        }
      }
    }
    xml += "</Response>";
    return xml;
  }

  private escapeXml(unsafe: string): string {
    return unsafe
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&apos;");
  }

  /**
   * Initiates an outbound telephone call via Twilio REST API.
   */
  public async initiateCall(options: {
    to: string;
    from?: string;
    twiml?: string;
    url?: string;
    statusCallback?: string;
  }): Promise<CallRecord> {
    const callerId = options.from || this.fromNumber;
    const now = new Date().toISOString();

    if (!this.accountSid || !this.authToken) {
      throw new Error("Twilio credentials not configured (TWILIO_ACCOUNT_SID and TWILIO_AUTH_TOKEN required)");
    }

    const endpoint = `https://api.twilio.com/2010-04-01/Accounts/${this.accountSid}/Calls.json`;
    const formParams = new URLSearchParams();
    formParams.set("To", options.to);
    formParams.set("From", callerId);

    if (options.twiml) {
      formParams.set("Twiml", options.twiml);
    } else if (options.url) {
      formParams.set("Url", options.url);
    } else {
      const defaultTwiml = this.generateTwiml([
        { type: "say", text: "Hello, this is a call from your Kineti autonomous agent." },
        { type: "hangup" },
      ]);
      formParams.set("Twiml", defaultTwiml);
    }

    if (options.statusCallback) {
      formParams.set("StatusCallback", options.statusCallback);
    }

    const authHeader = "Basic " + Buffer.from(`${this.accountSid}:${this.authToken}`).toString("base64");
    const response = await fetch(endpoint, {
      method: "POST",
      headers: {
        "Authorization": authHeader,
        "Content-Type": "application/x-www-form-urlencoded",
        "User-Agent": "OpenAI File Downloader, XaiImageApiFetch/1.0",
      },
      body: formParams.toString(),
    });

    if (!response.ok) {
      const errText = await response.text();
      throw new Error(`Twilio Call API failed [${response.status}]: ${errText}`);
    }

    const data = (await response.json()) as any;
    const record: CallRecord = {
      callSid: data.sid || `call_${Date.now()}`,
      to: options.to,
      from: callerId,
      status: (data.status as any) || "queued",
      direction: "outbound-api",
      twiml: options.twiml,
      createdAt: now,
      updatedAt: now,
    };

    const calls = this.listCalls();
    calls.unshift(record);
    this.saveCalls(calls.slice(0, 100));

    return record;
  }

  /**
   * Processes a status callback webhook from Twilio.
   */
  public handleStatusWebhook(payload: Record<string, any>): CallRecord | null {
    const callSid = payload.CallSid || payload.callSid;
    if (!callSid) return null;

    const calls = this.listCalls();
    const existingIndex = calls.findIndex((c) => c.callSid === callSid);

    const now = new Date().toISOString();
    const duration = payload.CallDuration ? parseInt(payload.CallDuration, 10) : undefined;
    const status = (payload.CallStatus || payload.status || "completed") as CallRecord["status"];

    if (existingIndex >= 0) {
      calls[existingIndex].status = status;
      calls[existingIndex].updatedAt = now;
      if (duration !== undefined) {
        calls[existingIndex].durationSeconds = duration;
      }
      this.saveCalls(calls);
      return calls[existingIndex];
    } else {
      const newRecord: CallRecord = {
        callSid,
        to: payload.To || payload.to || "",
        from: payload.From || payload.from || "",
        status,
        direction: (payload.Direction || "inbound") as any,
        durationSeconds: duration,
        createdAt: now,
        updatedAt: now,
      };
      calls.unshift(newRecord);
      this.saveCalls(calls.slice(0, 100));
      return newRecord;
    }
  }
}
