/**
 * Kineti Multimodal Vision & OCR Engine
 *
 * Extracts structured data from receipts, invoices, screenshots, and documents.
 * Supports offline heuristic document parsing and cloud multimodal vision inference.
 */

export interface ReceiptItem {
  name: string;
  quantity: number;
  priceCents: number;
}

export interface ReceiptData {
  merchant: string;
  date?: string;
  items: ReceiptItem[];
  subtotalCents?: number;
  taxCents?: number;
  tipCents?: number;
  totalCents: number;
  paymentMethod?: string;
  confidence: number;
}

export interface DocumentAnalysis {
  title?: string;
  keyValues: Record<string, string>;
  lines: string[];
  tables: string[][][];
}

export interface ScreenshotAnalysis {
  detectedHeadings: string[];
  detectedButtons: string[];
  detectedInputs: string[];
  errorMessages: string[];
  rawText: string;
}

export class MultimodalVisionEngine {
  private openaiApiKey?: string;

  constructor(config: { openaiApiKey?: string } = {}) {
    this.openaiApiKey = config.openaiApiKey || process.env.OPENAI_API_KEY;
  }

  /**
   * Parses raw OCR text or receipt buffer into structured accounting fields.
   */
  public parseReceipt(input: string | Buffer): ReceiptData {
    const text = typeof input === "string" ? input : input.toString("utf-8");
    const lines = text.split(/\r?\n/).map((l) => l.trim()).filter((l) => l.length > 0);

    let merchant = "Unknown Merchant";
    let date: string | undefined;
    const items: ReceiptItem[] = [];
    let subtotalCents: number | undefined;
    let taxCents: number | undefined;
    let tipCents: number | undefined;
    let totalCents = 0;
    let paymentMethod: string | undefined;

    // 1. Merchant name: first non-empty line that isn't a date or address
    for (const line of lines.slice(0, 5)) {
      if (!/^\d/.test(line) && !/receipt|invoice|welcome|order|#|ticket/i.test(line)) {
        merchant = line;
        break;
      }
    }

    // 2. Date extraction (e.g. MM/DD/YYYY, YYYY-MM-DD, Month DD, YYYY)
    const dateRegex = /\b(\d{1,2}[\/\-\.]\d{1,2}[\/\-\.]\d{2,4}|\d{4}[\/\-\.]\d{1,2}[\/\-\.]\d{1,2})\b/;
    for (const line of lines) {
      const match = line.match(dateRegex);
      if (match) {
        date = match[1];
        break;
      }
    }

    // 3. Line items and totals
    const priceRegex = /\$?(\d+\.\d{2})\b/;
    for (const line of lines) {
      const lower = line.toLowerCase();

      // Check for Subtotal
      if (lower.includes("subtotal") || lower.includes("sub-total") || lower.includes("net amount")) {
        const p = this.extractPriceCents(line);
        if (p !== undefined) subtotalCents = p;
        continue;
      }

      // Check for Tax
      if (lower.includes("tax") || lower.includes("vat") || lower.includes("gst")) {
        const p = this.extractPriceCents(line);
        if (p !== undefined) taxCents = p;
        continue;
      }

      // Check for Tip / Gratuity
      if (lower.includes("tip") || lower.includes("gratuity")) {
        const p = this.extractPriceCents(line);
        if (p !== undefined) tipCents = p;
        continue;
      }

      // Check for Total
      if (lower.includes("total") || lower.includes("amount due") || lower.includes("balance")) {
        const p = this.extractPriceCents(line);
        if (p !== undefined) totalCents = p;
        continue;
      }

      // Payment method
      if (/visa|mastercard|amex|discover|apple pay|google pay|cash/i.test(line)) {
        paymentMethod = line;
      }

      // Regular line item check
      const match = line.match(priceRegex);
      if (match && !/total|subtotal|tax|balance|change|cash|card/i.test(lower)) {
        const price = Math.round(parseFloat(match[1]) * 100);
        let itemName = line.replace(match[0], "").trim();
        itemName = itemName.replace(/^[-*•\d\.\s]+/, "").trim();

        // Check for quantity (e.g. "2x Coffee" or "Coffee x2" or "2 Coffee")
        let quantity = 1;
        const qtyMatch = line.match(/\b(\d+)\s*[xX]\b/) || line.match(/^[xX]?(\d+)\s+/);
        if (qtyMatch) {
          quantity = parseInt(qtyMatch[1], 10) || 1;
        }

        if (itemName.length > 0) {
          items.push({ name: itemName, quantity, priceCents: price });
        }
      }
    }

    // Fallback calculation for total
    if (totalCents === 0 && items.length > 0) {
      const sum = items.reduce((acc, item) => acc + item.priceCents * item.quantity, 0);
      totalCents = sum + (taxCents || 0) + (tipCents || 0);
    }

    return {
      merchant,
      date,
      items,
      subtotalCents,
      taxCents,
      tipCents,
      totalCents,
      paymentMethod,
      confidence: items.length > 0 && totalCents > 0 ? 0.92 : 0.65,
    };
  }

  private extractPriceCents(text: string): number | undefined {
    const match = text.match(/\$?(\d+\.\d{2})\b/);
    if (match) {
      return Math.round(parseFloat(match[1]) * 100);
    }
    return undefined;
  }

  /**
   * Extracts structured key-value pairs and sections from documents.
   */
  public extractDocumentText(input: string | Buffer): DocumentAnalysis {
    const text = typeof input === "string" ? input : input.toString("utf-8");
    const rawLines = text.split(/\r?\n/).map((l) => l.trim()).filter((l) => l.length > 0);

    const keyValues: Record<string, string> = {};
    const tables: string[][][] = [];
    let title: string | undefined;

    if (rawLines.length > 0) {
      title = rawLines[0];
    }

    for (const line of rawLines) {
      // Key-Value pattern: "Key: Value" or "Key - Value"
      const kvMatch = line.match(/^([A-Za-z0-9\s_-]+)[:]\s*(.+)$/);
      if (kvMatch) {
        const key = kvMatch[1].trim();
        const val = kvMatch[2].trim();
        if (key.length > 0 && val.length > 0) {
          keyValues[key] = val;
        }
      }
    }

    return {
      title,
      keyValues,
      lines: rawLines,
      tables,
    };
  }

  /**
   * Analyzes a screenshot text / layout to discover interactive UI controls.
   */
  public analyzeScreenshot(input: string | Buffer): ScreenshotAnalysis {
    const text = typeof input === "string" ? input : input.toString("utf-8");
    const lines = text.split(/\r?\n/).map((l) => l.trim()).filter((l) => l.length > 0);

    const detectedHeadings: string[] = [];
    const detectedButtons: string[] = [];
    const detectedInputs: string[] = [];
    const errorMessages: string[] = [];

    for (const line of lines) {
      // Buttons e.g. [Submit], <OK>, (Continue)
      const btnMatch = line.match(/[\[\(<]([A-Za-z0-9\s_-]{2,20})[\]\)>]/);
      if (btnMatch) {
        detectedButtons.push(btnMatch[1]);
      } else if (/^(submit|save|cancel|continue|sign in|login|next|confirm|checkout)$/i.test(line)) {
        detectedButtons.push(line);
      }

      // Input field indicators e.g. "Email Address: ____", "Search..."
      if (/email|password|username|search|enter|type here|first name|last name/i.test(line)) {
        detectedInputs.push(line);
      }

      // Errors
      if (/error|failed|invalid|exception|required|warning/i.test(line)) {
        errorMessages.push(line);
      }

      // Headings
      if (line.length < 50 && (/^[A-Z\s]{4,}$/.test(line) || /^[#]+/.test(line))) {
        detectedHeadings.push(line.replace(/^[#\s]+/, ""));
      }
    }

    return {
      detectedHeadings,
      detectedButtons,
      detectedInputs,
      errorMessages,
      rawText: text,
    };
  }

  /**
   * Dispatches an image buffer to a cloud multimodal LLM endpoint.
   */
  public async analyzeWithLLM(
    imageBuffer: Buffer,
    mimeType: string,
    prompt: string,
    apiKey?: string
  ): Promise<string> {
    const key = apiKey || this.openaiApiKey;
    if (!key) {
      throw new Error("Multimodal API key not configured (OPENAI_API_KEY required for cloud vision)");
    }

    const base64Data = imageBuffer.toString("base64");
    const dataUri = `data:${mimeType};base64,${base64Data}`;

    const body = {
      model: "gpt-4o",
      messages: [
        {
          role: "user",
          content: [
            { type: "text", text: prompt },
            { type: "image_url", image_url: { url: dataUri } },
          ],
        },
      ],
      max_tokens: 1000,
    };

    const res = await fetch("https://api.openai.com/v1/chat/completions", {
      method: "POST",
      headers: {
        "Authorization": `Bearer ${key}`,
        "Content-Type": "application/json",
        "User-Agent": "OpenAI File Downloader, XaiImageApiFetch/1.0",
      },
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      const err = await res.text();
      throw new Error(`Cloud vision request failed [${res.status}]: ${err}`);
    }

    const json = (await res.json()) as any;
    return json.choices?.[0]?.message?.content || "";
  }
}
