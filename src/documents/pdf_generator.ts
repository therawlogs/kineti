/**
 * Kineti PDF Document Generator
 *
 * Generates valid PDF 1.4 documents natively in TypeScript without external dependencies.
 * Creates clean receipts, flight/travel itineraries, and structured reports.
 */

import { writeFileSync } from "node:fs";

export interface PdfDocumentSpec {
  title: string;
  subtitle?: string;
  metadata?: Record<string, string>;
  sections?: { heading: string; body: string }[];
  tables?: {
    headers: string[];
    rows: string[][];
  }[];
  footer?: string;
}

export interface ReceiptPdfSpec {
  merchant: string;
  receiptNumber: string;
  date: string;
  items: { name: string; quantity: number; priceUsd: number }[];
  subtotalUsd: number;
  taxUsd: number;
  tipUsd?: number;
  totalUsd: number;
  paymentMethod: string;
}

export interface ItineraryPdfSpec {
  passengerName: string;
  bookingReference: string;
  flights: {
    flightNumber: string;
    carrier: string;
    origin: string;
    destination: string;
    departureTime: string;
    arrivalTime: string;
    seat: string;
  }[];
  hotelReservations?: {
    hotelName: string;
    checkIn: string;
    checkOut: string;
    confirmationCode: string;
  }[];
}

export class PdfGenerator {
  /**
   * Builds a standard PDF 1.4 binary buffer from text commands.
   */
  public generateDocument(spec: PdfDocumentSpec): Uint8Array {
    const textCommands: string[] = [];
    let y = 750; // Start near top of standard US Letter page (612 x 792 pt)

    // Title
    textCommands.push(this.formatText(spec.title, 54, y, "F2", 20));
    y -= 28;

    // Subtitle
    if (spec.subtitle) {
      textCommands.push(this.formatText(spec.subtitle, 54, y, "F1", 12));
      y -= 24;
    }

    // Divider line
    textCommands.push(`54 ${y} m 558 ${y} l S`);
    y -= 20;

    // Metadata Key-Values
    if (spec.metadata) {
      for (const [k, v] of Object.entries(spec.metadata)) {
        textCommands.push(this.formatText(`${k}: ${v}`, 54, y, "F1", 10));
        y -= 15;
      }
      y -= 10;
    }

    // Sections
    if (spec.sections) {
      for (const sec of spec.sections) {
        textCommands.push(this.formatText(sec.heading, 54, y, "F2", 14));
        y -= 18;

        const bodyLines = this.wrapText(sec.body, 75);
        for (const line of bodyLines) {
          textCommands.push(this.formatText(line, 54, y, "F1", 10));
          y -= 14;
        }
        y -= 12;
      }
    }

    // Tables
    if (spec.tables) {
      for (const tbl of spec.tables) {
        // Table Header
        let x = 54;
        const colWidth = 504 / Math.max(tbl.headers.length, 1);
        for (const h of tbl.headers) {
          textCommands.push(this.formatText(h, x, y, "F2", 10));
          x += colWidth;
        }
        y -= 14;
        textCommands.push(`54 ${y} m 558 ${y} l S`);
        y -= 14;

        // Table Rows
        for (const row of tbl.rows) {
          x = 54;
          for (const cell of row) {
            textCommands.push(this.formatText(cell, x, y, "F1", 9));
            x += colWidth;
          }
          y -= 14;
        }
        y -= 10;
      }
    }

    // Footer
    const footerText = spec.footer || "Generated autonomously by Kineti OS";
    textCommands.push(this.formatText(footerText, 54, 40, "F1", 8));
    textCommands.push(`54 50 m 558 50 l S`);

    return this.buildPdfFromStream(textCommands.join("\n"));
  }

  /**
   * Generates a formal merchant receipt PDF.
   */
  public generateReceipt(spec: ReceiptPdfSpec): Uint8Array {
    const tableRows = spec.items.map((it) => [
      it.name,
      it.quantity.toString(),
      `$${it.priceUsd.toFixed(2)}`,
      `$${(it.quantity * it.priceUsd).toFixed(2)}`,
    ]);

    tableRows.push(["Subtotal", "", "", `$${spec.subtotalUsd.toFixed(2)}`]);
    tableRows.push(["Sales Tax", "", "", `$${spec.taxUsd.toFixed(2)}`]);
    if (spec.tipUsd) {
      tableRows.push(["Tip / Gratuity", "", "", `$${spec.tipUsd.toFixed(2)}`]);
    }
    tableRows.push(["TOTAL PAID", "", "", `$${spec.totalUsd.toFixed(2)}`]);

    return this.generateDocument({
      title: spec.merchant,
      subtitle: `Official Purchase Receipt #${spec.receiptNumber}`,
      metadata: {
        "Date": spec.date,
        "Payment Method": spec.paymentMethod,
        "Status": "CONFIRMED & SETTLED",
      },
      tables: [
        {
          headers: ["Item Description", "Qty", "Unit Price", "Total"],
          rows: tableRows,
        },
      ],
      footer: "Kineti OS Autonomous Purchasing Agent — Proof of Settlement",
    });
  }

  /**
   * Generates a travel / flight itinerary PDF.
   */
  public generateItinerary(spec: ItineraryPdfSpec): Uint8Array {
    const flightRows = spec.flights.map((f) => [
      `${f.carrier} ${f.flightNumber}`,
      `${f.origin} → ${f.destination}`,
      f.departureTime,
      f.arrivalTime,
      f.seat,
    ]);

    const docSpec: PdfDocumentSpec = {
      title: "Travel Itinerary",
      subtitle: `Passenger: ${spec.passengerName}`,
      metadata: {
        "Booking Confirmation": spec.bookingReference,
        "Ticketing Status": "ISSUED",
      },
      tables: [
        {
          headers: ["Flight", "Route", "Departure", "Arrival", "Seat"],
          rows: flightRows,
        },
      ],
      footer: "Kineti Concierge — Live Flight & Travel Companion",
    };

    if (spec.hotelReservations && spec.hotelReservations.length > 0) {
      const hotelRows = spec.hotelReservations.map((h) => [
        h.hotelName,
        h.checkIn,
        h.checkOut,
        h.confirmationCode,
      ]);
      docSpec.tables!.push({
        headers: ["Hotel / Accommodation", "Check-in", "Check-out", "Confirmation"],
        rows: hotelRows,
      });
    }

    return this.generateDocument(docSpec);
  }

  /**
   * Saves generated PDF bytes to file.
   */
  public saveToFile(pdfBytes: Uint8Array, targetPath: string): void {
    writeFileSync(targetPath, Buffer.from(pdfBytes));
  }

  private formatText(text: string, x: number, y: number, font: string, size: number): string {
    const escaped = text.replace(/\\/g, "\\\\").replace(/\(/g, "\\(").replace(/\)/g, "\\)");
    return `BT /${font} ${size} Tf ${x.toFixed(1)} ${y.toFixed(1)} Td (${escaped}) Tj ET`;
  }

  private wrapText(text: string, maxLen: number): string[] {
    const words = text.split(/\s+/);
    const lines: string[] = [];
    let cur = "";
    for (const w of words) {
      if (cur.length + w.length + 1 > maxLen) {
        lines.push(cur);
        cur = w;
      } else {
        cur = cur.length === 0 ? w : `${cur} ${w}`;
      }
    }
    if (cur.length > 0) lines.push(cur);
    return lines;
  }

  private buildPdfFromStream(contentStream: string): Uint8Array {
    const streamBytes = Buffer.from(contentStream, "utf-8");
    const streamLen = streamBytes.length;

    const objects: string[] = [];
    objects.push("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj");
    objects.push("2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj");
    objects.push(
      "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R /F2 6 0 R >> >> >>\nendobj"
    );
    objects.push(`4 0 obj\n<< /Length ${streamLen} >>\nstream\n${contentStream}\nendstream\nendobj`);
    objects.push("5 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj");
    objects.push("6 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj");

    let pdf = "%PDF-1.4\n";
    const offsets: number[] = [0];

    for (let i = 0; i < objects.length; i++) {
      offsets.push(pdf.length);
      pdf += objects[i] + "\n";
    }

    const startXref = pdf.length;
    pdf += `xref\n0 ${objects.length + 1}\n0000000000 65535 f \n`;
    for (let i = 1; i <= objects.length; i++) {
      const off = offsets[i].toString().padStart(10, "0");
      pdf += `${off} 00000 n \n`;
    }

    pdf += `trailer\n<< /Size ${objects.length + 1} /Root 1 0 R >>\nstartxref\n${startXref}\n%%EOF\n`;
    return new Uint8Array(Buffer.from(pdf, "utf-8"));
  }
}
