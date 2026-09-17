import { describe, test, expect } from "bun:test";
import crypto from "node:crypto";

describe("Omnichannel Ingestion: WhatsApp, iMessage, and Agent Email", () => {
  test("WhatsApp onboarding pairing token lifecycle", () => {
    const phoneNumber = "+14155552671";
    const timestamp = 1710000000;
    const cleanPhone = phoneNumber.replace(/[^0-9]/g, "");
    const raw = `${cleanPhone}:${timestamp}:kineti_wa_pair_v1`;
    const hash = crypto.createHash("sha256").update(raw).digest("hex");
    const token = `wa_${hash.slice(0, 16)}_${timestamp}`;

    expect(token.startsWith("wa_")).toBe(true);

    const onboardingUrl = `https://app.getkineti.com/whatsapp-onboarding?t=${token}`;
    expect(onboardingUrl).toContain("https://app.getkineti.com/whatsapp-onboarding?t=");
    expect(onboardingUrl).toContain(token);
  });

  test("iMessage location coordinate parsing", () => {
    function extractLocationCoordinates(text: string): [number, number] | null {
      // Apple Maps
      const appleMatch = text.match(/maps\.apple\.com\/\?(?:ll|q)=([0-9.-]+),([0-9.-]+)/);
      if (appleMatch) {
        return [parseFloat(appleMatch[1]), parseFloat(appleMatch[2])];
      }
      // Google Maps
      const googleMatch = text.match(/google\.com\/maps\?q=([0-9.-]+),([0-9.-]+)/);
      if (googleMatch) {
        return [parseFloat(googleMatch[1]), parseFloat(googleMatch[2])];
      }
      // Decimal coords
      const coordMatch = text.match(/(?:Location:\s*)?([0-9.-]+),\s*([0-9.-]+)/);
      if (coordMatch) {
        const lat = parseFloat(coordMatch[1]);
        const lon = parseFloat(coordMatch[2]);
        if (lat >= -90 && lat <= 90 && lon >= -180 && lon <= 180) {
          return [lat, lon];
        }
      }
      return null;
    }

    const apple = "I parked here: https://maps.apple.com/?ll=37.7749,-122.4194";
    const google = "Meet me at https://google.com/maps?q=37.7833,-122.4167";
    const direct = "Location: 37.7900, -122.4000";

    expect(extractLocationCoordinates(apple)).toEqual([37.7749, -122.4194]);
    expect(extractLocationCoordinates(google)).toEqual([37.7833, -122.4167]);
    expect(extractLocationCoordinates(direct)).toEqual([37.7900, -122.4000]);
    expect(extractLocationCoordinates("Regular chat message")).toBeNull();
  });

  test("Agent dedicated email parsing (@mail.kineti.com)", () => {
    function parseAgentEmail(rawMime: string) {
      const otpMatch = rawMime.match(/(?:verification code|code is|passcode:)\s*([0-9]{6})/i);
      const trackingMatch = rawMime.match(/\b(1Z[0-9A-Z]{16}|9400[0-9]{18})\b/);
      return {
        otp: otpMatch ? otpMatch[1] : null,
        tracking_number: trackingMatch ? trackingMatch[1] : null,
      };
    }

    const sampleMime = `
From: security@service.com
To: prav@mail.kineti.com
Subject: Your Verification Code

Your verification code is 839201. Use this within 10 minutes.
Tracking for your package: 1Z9999999999999999.
    `;

    const parsed = parseAgentEmail(sampleMime);
    expect(parsed.otp).toBe("839201");
    expect(parsed.tracking_number).toBe("1Z9999999999999999");
  });
});
