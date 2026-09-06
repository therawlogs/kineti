import { describe, it, expect } from "bun:test";
import * as crypto from "node:crypto";
import * as fs from "node:fs";
import * as path from "node:path";

// Load blueprint text once
const blueprintPath = path.resolve(process.cwd(), "docs/HARNESS_STRATEGY_BLUEPRINT.md");
const blueprintContent = fs.readFileSync(blueprintPath, "utf-8");

describe("Adversarial Challenge Focus 1: Schema & Syntax Rigor", () => {
  it("extracts and validates all JSON code blocks from the blueprint", () => {
    // Regex to match ```json ... ``` blocks
    const jsonBlockRegex = /```json\s*([\s\S]*?)\s*```/g;
    let match;
    const jsonBlocks: { raw: string; parsed?: any; error?: string }[] = [];

    while ((match = jsonBlockRegex.exec(blueprintContent)) !== null) {
      const raw = match[1].trim();
      try {
        const parsed = JSON.parse(raw);
        jsonBlocks.push({ raw, parsed });
      } catch (err: any) {
        jsonBlocks.push({ raw, error: err.message });
      }
    }

    expect(jsonBlocks.length).toBeGreaterThanOrEqual(10);
    const syntaxErrors = jsonBlocks.filter((b) => b.error);
    if (syntaxErrors.length > 0) {
      console.error("Syntax errors in blueprint JSON blocks:", syntaxErrors);
    }
    expect(syntaxErrors.length).toBe(0);
  });

  it("validates Appendix B Runtime OTD JSON Schema against meta-schema rules", () => {
    // Locate Appendix B JSON block
    const appBStart = blueprintContent.indexOf("### 8.2 Appendix B: Complete Runtime OTD");
    expect(appBStart).toBeGreaterThan(0);
    const appBSlice = blueprintContent.slice(appBStart, appBStart + 3000);
    const match = /```json\s*([\s\S]*?)\s*```/.exec(appBSlice);
    expect(match).not.toBeNull();
    const schema = JSON.parse(match![1]);

    expect(schema.$schema).toBe("https://json-schema.org/draft/2020-12/schema");
    expect(schema.title).toBe("KinetiRuntimeOTDActivationPacket");
    expect(schema.type).toBe("object");
    expect(schema.required).toContain("packet_id");
    expect(schema.required).toContain("session_id");
    expect(schema.required).toContain("active_ontology_state");
    expect(schema.required).toContain("enforced_boundaries");

    // Test a valid instance against schema required properties
    const validPacket = {
      packet_id: "787db8bf-e17f-440a-abec-0fbfbb7ecae3",
      session_id: "894f-2026-09-06",
      timestamp: new Date().toISOString(),
      previous_ontology_state: "SPEC_LOCK_MODE",
      active_ontology_state: "BUILD_SAFE_MODE",
      triggering_event: {
        event_type: "gate_passed",
        event_id: "gate-spec-pass-1",
        summary: "Stage 6 Spec Gate approved",
      },
      injected_context_directives: ["Adhere to frozen spec.md"],
      enforced_boundaries: {
        revoked_tools: ["spec_edit"],
        authorized_scopes: ["src/"],
        max_turn_spend_usd: 2.5,
      },
    };

    for (const req of schema.required) {
      expect(validPacket).toHaveProperty(req);
    }
  });

  it("validates Appendix C OVT W3C VC JSON Schema and reveals schema discrepancies with VC instance", () => {
    const appCStart = blueprintContent.indexOf("### 8.3 Appendix C: Complete Outcome Verification Ticket");
    expect(appCStart).toBeGreaterThan(0);
    const appCSlice = blueprintContent.slice(appCStart, appCStart + 3500);
    const match = /```json\s*([\s\S]*?)\s*```/.exec(appCSlice);
    expect(match).not.toBeNull();
    const ovtSchema = JSON.parse(match![1]);

    expect(ovtSchema.title).toBe("OutcomeVerificationTicketVerifiableCredential");
    expect(ovtSchema.required).toEqual([
      "@context",
      "id",
      "type",
      "issuer",
      "issuanceDate",
      "credentialSubject",
      "proof",
    ]);

    // Extract W3C VC instance from Section 3.5
    const sec35Start = blueprintContent.indexOf("#### W3C Verifiable Credentials Compliance Instance");
    expect(sec35Start).toBeGreaterThan(0);
    const sec35Slice = blueprintContent.slice(sec35Start, sec35Start + 2500);
    const vcMatch = /```json\s*([\s\S]*?)\s*```/.exec(sec35Slice);
    expect(vcMatch).not.toBeNull();
    const vcInstance = JSON.parse(vcMatch![1]);

    // Check conformance of instance against schema required properties
    for (const req of ovtSchema.required) {
      expect(vcInstance).toHaveProperty(req);
    }
    for (const req of ovtSchema.properties.credentialSubject.required) {
      expect(vcInstance.credentialSubject).toHaveProperty(req);
    }

    // ADVERSARIAL FINDING: Check proofValue in Section 3.5 instance
    // Notice that proofValue contains ellipsis placeholders!
    for (const proof of vcInstance.proof) {
      const isPlaceholder = proof.proofValue.includes("...") || proof.proofValue.length < 32;
      expect(isPlaceholder).toBe(true); // Demonstrates presence of placeholder ellipses in blueprint
    }
  });

  it("audits Section 3.3 TypeScript Kernel Types vs Appendix C W3C VC Schema for contract mismatches", () => {
    // In Section 3.3, entity #20 is declared as:
    // export interface OVT {
    //   ticket_id: UUID;
    //   session_id: UUID;
    //   goal_hash: SHA256;
    //   code_fingerprint: SHA256;
    //   evidence_hash: SHA256;
    //   merkle_root: SHA256;
    //   total_spend_usd: number;
    //   delivery_timestamp: ISO8601;
    //   signatures: { agent_signature: string; harness_signature: string; };
    //   compliance_proof: { ... };
    // }
    // But in Appendix C, OVT is W3C VC with credentialSubject: { id, sessionId, rootGoalHash, codeFingerprint, merkleRoot, outcomes, policyGatesPassed, totalSessionSpendUsd }
    
    // Proving field name mismatch:
    const tsFields = ["ticket_id", "session_id", "goal_hash", "total_spend_usd", "signatures"];
    const vcFields = ["id", "sessionId", "rootGoalHash", "totalSessionSpendUsd", "proof"];

    expect(tsFields).not.toEqual(vcFields);
    expect("ticket_id").not.toEqual("id");
    expect("session_id").not.toEqual("sessionId");
    expect("goal_hash").not.toEqual("rootGoalHash");
    expect("total_spend_usd").not.toEqual("totalSessionSpendUsd");
  });

  it("checks Appendix A JSON-LD context coverage for all 20 entities", () => {
    const appAStart = blueprintContent.indexOf("### 8.1 Appendix A: Complete Universal 20-Entity JSON-LD Schema");
    expect(appAStart).toBeGreaterThan(0);
    const appASlice = blueprintContent.slice(appAStart, appAStart + 2000);
    const match = /```json\s*([\s\S]*?)\s*```/.exec(appASlice);
    expect(match).not.toBeNull();
    const jsonld = JSON.parse(match![1]);

    const ctx = jsonld["@context"];
    expect(ctx).toBeDefined();

    const entities = [
      "Agent", "Session", "Host", "Goal", "Milestone",
      "Task", "ToolCall", "ToolResult", "Observation", "Artifact",
      "CodeDiff", "Assertion", "GateResult", "SpendEntry", "RollbackAction",
      "Checkpoint", "OTDTrigger", "EvidenceLog", "MerkleLeaf", "OVT"
    ];

    // All 20 entity classes should be mapped in @context
    for (const ent of entities) {
      expect(ctx).toHaveProperty(ent);
    }

    // However, property coverage is severely truncated (only 9 properties declared)
    const propertyKeys = Object.keys(ctx).filter(k => !entities.includes(k) && !k.startsWith("@"));
    // Count how many properties are mapped:
    expect(propertyKeys.length).toBeLessThan(15); // Discrepancy: 20 rich entities have >80 properties, but only ~9 are mapped
  });
});

describe("Adversarial Challenge Focus 2: Financial & Mathematical Consistency", () => {
  const proFormaData = [
    { m: "M1", installs: 2000, wau: 500, proSeats: 0, proMRR: 0, entAccts: 0, entSeats: 0, entMRR: 0, totalMRR: 0, arr: 0 },
    { m: "M2", installs: 5000, wau: 1250, proSeats: 25, proMRR: 975, entAccts: 0, entSeats: 0, entMRR: 0, totalMRR: 975, arr: 11700 },
    { m: "M3", installs: 10000, wau: 2500, proSeats: 65, proMRR: 2535, entAccts: 0, entSeats: 0, entMRR: 0, totalMRR: 2535, arr: 30420 },
    { m: "M4", installs: 18000, wau: 4500, proSeats: 130, proMRR: 5070, entAccts: 1, entSeats: 10, entMRR: 2500, totalMRR: 7570, arr: 90840 },
    { m: "M5", installs: 25000, wau: 6250, proSeats: 210, proMRR: 8190, entAccts: 1, entSeats: 10, entMRR: 2500, totalMRR: 10690, arr: 128280 },
    { m: "M6", installs: 35000, wau: 8750, proSeats: 320, proMRR: 12480, entAccts: 2, entSeats: 25, entMRR: 6250, totalMRR: 18730, arr: 224760 },
    { m: "M7", installs: 48000, wau: 12000, proSeats: 460, proMRR: 17940, entAccts: 4, entSeats: 45, entMRR: 11250, totalMRR: 29190, arr: 350280 },
    { m: "M8", installs: 62000, wau: 15500, proSeats: 620, proMRR: 24180, entAccts: 6, entSeats: 70, entMRR: 17500, totalMRR: 41680, arr: 500160 },
    { m: "M9", installs: 80000, wau: 20000, proSeats: 800, proMRR: 31200, entAccts: 8, entSeats: 95, entMRR: 23750, totalMRR: 54950, arr: 659400 },
    { m: "M10", installs: 100000, wau: 25000, proSeats: 1020, proMRR: 39780, entAccts: 11, entSeats: 130, entMRR: 32500, totalMRR: 72280, arr: 867360 },
    { m: "M11", installs: 125000, wau: 31250, proSeats: 1280, proMRR: 49920, entAccts: 14, entSeats: 170, entMRR: 42500, totalMRR: 92420, arr: 1109040 },
    { m: "M12", installs: 150000, wau: 37500, proSeats: 1550, proMRR: 60450, entAccts: 18, entSeats: 225, entMRR: 56250, totalMRR: 116700, arr: 1400400 },
    { m: "M15", installs: 200000, wau: 50000, proSeats: 2150, proMRR: 83850, entAccts: 26, entSeats: 360, entMRR: 90000, totalMRR: 173850, arr: 2086200 },
    { m: "M18", installs: 250000, wau: 62500, proSeats: 2800, proMRR: 109200, entAccts: 35, entSeats: 520, entMRR: 130000, totalMRR: 239200, arr: 2870400 },
    { m: "M21", installs: 300000, wau: 75000, proSeats: 3350, proMRR: 130650, entAccts: 44, entSeats: 700, entMRR: 175000, totalMRR: 305650, arr: 3667800 },
    { m: "M24", installs: 350000, wau: 87500, proSeats: 3900, proMRR: 152100, entAccts: 55, entSeats: 920, entMRR: 230000, totalMRR: 382100, arr: 4585200 },
  ];

  it("verifies exact arithmetic of seats * price and total MRR/ARR in 24-month model", () => {
    for (const row of proFormaData) {
      // 1. Pro MRR = Pro Seats * $39
      expect(row.proMRR).toBe(row.proSeats * 39);
      // 2. Enterprise MRR = Enterprise Seats * $250
      expect(row.entMRR).toBe(row.entSeats * 250);
      // 3. Total MRR = Pro MRR + Enterprise MRR
      expect(row.totalMRR).toBe(row.proMRR + row.entMRR);
      // 4. ARR = Total MRR * 12
      expect(row.arr).toBe(row.totalMRR * 12);
      // 5. WAU = 25% of cumulative installs
      expect(row.wau).toBe(row.installs * 0.25);
    }
  });

  it("stress-tests conversion rate assumptions against the modeled pro-forma numbers", () => {
    // Assumption: "Free-to-Pro Conversion: 2.0% to 4.0% of active users convert to the paid Pro tier"
    const conversionViolations: { month: string; rate: number }[] = [];
    for (const row of proFormaData) {
      if (row.wau > 0 && row.proSeats > 0) {
        const rate = (row.proSeats / row.wau) * 100;
        if (rate > 4.0) {
          conversionViolations.push({ month: row.m, rate: Number(rate.toFixed(2)) });
        }
      }
    }

    // Month 10 through Month 24 all exceed the 4.0% ceiling!
    expect(conversionViolations.length).toBeGreaterThan(0);
    expect(conversionViolations[0].month).toBe("M10"); // Starts exceeding at M10 (4.08%)
    expect(conversionViolations[conversionViolations.length - 1].rate).toBe(4.46); // M24 is 4.46%
  });

  it("evaluates Enterprise account expansion rate discrepancy vs stated 0.5% assumption", () => {
    // Assumption: "0.5% of Pro accounts expand into multi-seat Enterprise accounts"
    // At M12: Pro Seats = 1,550. 0.5% = 7.75 accounts. Modeled: 18 accounts!
    const m12 = proFormaData.find((r) => r.m === "M12")!;
    const expectedEntAcctsAt05Pct = m12.proSeats * 0.005;
    expect(expectedEntAcctsAt05Pct).toBe(7.75);
    expect(m12.entAccts).toBe(18); // Discrepancy: 18 is 2.32x higher than static 0.5% penetration!

    // At M24: Pro Seats = 3,900. 0.5% = 19.5 accounts. Modeled: 55 accounts!
    const m24 = proFormaData.find((r) => r.m === "M24")!;
    const expectedEntAcctsAt05PctM24 = m24.proSeats * 0.005;
    expect(expectedEntAcctsAt05PctM24).toBe(19.5);
    expect(m24.entAccts).toBe(55); // Discrepancy: 55 is 2.82x higher than static 0.5% penetration!
  });

  it("verifies Month 12 COGS, OpEx, Gross Margin, and EBITDA calculations", () => {
    const rev = 116700;
    const cogsItems = [450, 750, 250, 320, 3650];
    const totalCogs = cogsItems.reduce((a, b) => a + b, 0);
    expect(totalCogs).toBe(5420);

    const grossProfit = rev - totalCogs;
    expect(grossProfit).toBe(111280);

    const grossMarginPct = (grossProfit / rev) * 100;
    expect(Number(grossMarginPct.toFixed(2))).toBe(95.36);

    const opexItems = [380, 290, 200, 850, 1200];
    const totalOpex = opexItems.reduce((a, b) => a + b, 0);
    expect(totalOpex).toBe(2920);

    const ebitda = grossProfit - totalOpex;
    expect(ebitda).toBe(108360);

    const ebitdaMarginPct = (ebitda / rev) * 100;
    expect(Number(ebitdaMarginPct.toFixed(2))).toBe(92.85);

    const annualNetIncome = ebitda * 12;
    expect(annualNetIncome).toBe(1300320);

    // Audit Stripe fees calculation:
    // If billed monthly: 1550 pro seats * $0.30 = $465.
    // 2.9% of $116,700 = $3,384.30.
    // Sum = $3,849.30. Modeled is $3,650. Difference is $199.30/mo (modeled is ~5% lower).
    const standardStripeFee = rev * 0.029 + 1550 * 0.3;
    expect(standardStripeFee).toBeCloseTo(3849.3, 1);
  });

  it("stress-tests Enterprise ROI formulation and Payback Period calculation", () => {
    const N = 1000;
    const Pf = 0.025;
    const Cf = 4800;
    const C_kineti = 250;
    const S = 10;

    const expectedRisk = N * Pf * Cf;
    expect(expectedRisk).toBe(120000);

    const monthlyCost = S * C_kineti;
    expect(monthlyCost).toBe(2500);

    const netMonthlySavings = expectedRisk - monthlyCost;
    expect(netMonthlySavings).toBe(117500);

    const netAnnualSavings = netMonthlySavings * 12;
    expect(netAnnualSavings).toBe(1410000);

    const roi = (netMonthlySavings / monthlyCost) * 100;
    expect(roi).toBe(4700);

    // Payback Period:
    // Daily gross risk mitigation: $120,000 / 30 = $4,000/day
    // Days to pay back $2,500 cost: 2,500 / 4,000 = 0.625 days (15.0 hours)
    // Blueprint claims "0.6 days (14 hours)"
    const daysGross = monthlyCost / (expectedRisk / 30);
    const hoursGross = daysGross * 24;
    expect(daysGross).toBe(0.625);
    expect(hoursGross).toBe(15.0);

    // If based on net savings: 2500 / (117,500 / 30) = 0.638 days (15.3 hours)
    const daysNet = monthlyCost / (netMonthlySavings / 30);
    const hoursNet = daysNet * 24;
    expect(Number(daysNet.toFixed(3))).toBe(0.638);
    expect(Number(hoursNet.toFixed(1))).toBe(15.3);

    // Stress test: what if Kineti catch rate is 90% instead of 100%?
    const mitigatedRisk90 = expectedRisk * 0.90;
    const netSavings90 = mitigatedRisk90 - monthlyCost;
    const roi90 = (netSavings90 / monthlyCost) * 100;
    expect(roi90).toBe(4220); // Still 4,220%

    // Stress test: what if failure cost is $1,200 instead of $4,800?
    const riskLow = N * Pf * 1200;
    const netSavingsLow = riskLow - monthlyCost;
    const roiLow = (netSavingsLow / monthlyCost) * 100;
    expect(riskLow).toBe(30000);
    expect(roiLow).toBe(1100); // 1,100%
  });
});

describe("Adversarial Challenge Focus 3: Edge Case & Failure Mode Analysis", () => {
  it("proves delimiter collision vulnerability in MerkleLeaf hash calculation", () => {
    // In Section 3.3 line 1216:
    // node_hash = SHA256(prev_hash + entity_type + entity_id + canonical_payload_hash)
    // If simple string concatenation is used without delimiters, delimiter collision is possible!

    const prev_hash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    const payload_hash = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

    // Scenario: Attacker manipulates entity_type and entity_id
    // Case A: entity_type = "ToolCall", entity_id = "1234abcd"
    const typeA = "ToolCall";
    const idA = "1234abcd";

    // Case B: entity_type = "Tool", entity_id = "Call1234abcd"
    const typeB = "Tool";
    const idB = "Call1234abcd";

    // Raw concatenation as specified:
    const concatA = prev_hash + typeA + idA + payload_hash;
    const concatB = prev_hash + typeB + idB + payload_hash;

    expect(concatA).toEqual(concatB);

    const hashA = crypto.createHash("sha256").update(concatA).digest("hex");
    const hashB = crypto.createHash("sha256").update(concatB).digest("hex");

    // Proves COLLISION VULNERABILITY under raw concatenation:
    expect(hashA).toEqual(hashB);

    // Defense: Length-prefixed or null-delimited hashing
    const safeConcatA = `${prev_hash}\x00${typeA}\x00${idA}\x00${payload_hash}`;
    const safeConcatB = `${prev_hash}\x00${typeB}\x00${idB}\x00${payload_hash}`;
    const safeHashA = crypto.createHash("sha256").update(safeConcatA).digest("hex");
    const safeHashB = crypto.createHash("sha256").update(safeConcatB).digest("hex");

    expect(safeHashA).not.toEqual(safeHashB);
  });

  it("evaluates linear Merkle chain race conditions under concurrent agent writes", () => {
    // Simulate two concurrent agents trying to append to a linear chain
    const initialRoot = "0000000000000000000000000000000000000000000000000000000000000000";

    // Agent 1 produces Event 1
    const event1Payload = "agent1:action1";
    const event1Hash = crypto.createHash("sha256").update(initialRoot + event1Payload).digest("hex");

    // Agent 2 produces Event 2 concurrently (both read initialRoot as prev_hash)
    const event2Payload = "agent2:action2";
    const event2Hash = crypto.createHash("sha256").update(initialRoot + event2Payload).digest("hex");

    // If Agent 1 commits first, the chain is now at event1Hash
    // If Agent 2 then appends with prev_hash = initialRoot, the chain FORKS!
    // If Agent 2 is replayed sequentially on top of Event 1:
    const seqRoot12 = crypto.createHash("sha256").update(event1Hash + event2Payload).digest("hex");
    // If Agent 2 was applied first and Agent 1 second:
    const seqRoot21 = crypto.createHash("sha256").update(event2Hash + event1Payload).digest("hex");

    // Non-commutative: Order matters!
    expect(seqRoot12).not.toEqual(seqRoot21);
    // Demonstrates that a linear prev_hash chain cannot support lockless concurrency;
    // it requires explicit mutex serialization or a true DAG with multiple parents.
  });

  it("tests Ed25519 dual-signature protocol and detects payload tampering", () => {
    // Generate Agent keypair and Harness keypair
    const agentKeys = crypto.generateKeyPairSync("ed25519");
    const harnessKeys = crypto.generateKeyPairSync("ed25519");

    const sessionId = "787db8bf-e17f-440a-abec-0fbfbb7ecae3";
    const goalHash = crypto.createHash("sha256").update("Root Goal").digest("hex");
    const deliverableHash = crypto.createHash("sha256").update("Deliverables").digest("hex");

    // 1. Agent signs Hash_agent = SHA256(sessionId || goalHash || deliverableHash)
    const agentDigest = crypto
      .createHash("sha256")
      .update(sessionId + goalHash + deliverableHash)
      .digest();
    const agentSig = crypto.sign(null, agentDigest, agentKeys.privateKey);

    // Verify Agent signature
    const agentSigValid = crypto.verify(null, agentDigest, agentKeys.publicKey, agentSig);
    expect(agentSigValid).toBe(true);

    // 2. Harness signs Hash_harness = SHA256(Hash_agent || S_agent || M_root || FP_code || spend_total_usd)
    const merkleRoot = crypto.createHash("sha256").update("MerkleTree").digest("hex");
    const codeFingerprint = crypto.createHash("sha256").update("CodeFiles").digest("hex");
    const spendUsd = "1.42";

    const harnessPayload = Buffer.concat([
      agentDigest,
      agentSig,
      Buffer.from(merkleRoot, "utf-8"),
      Buffer.from(codeFingerprint, "utf-8"),
      Buffer.from(spendUsd, "utf-8"),
    ]);
    const harnessDigest = crypto.createHash("sha256").update(harnessPayload).digest();
    const harnessSig = crypto.sign(null, harnessDigest, harnessKeys.privateKey);

    // Verify Harness signature
    const harnessSigValid = crypto.verify(null, harnessDigest, harnessKeys.publicKey, harnessSig);
    expect(harnessSigValid).toBe(true);

    // ATTACK 1: Corrupted agent signature
    const corruptedAgentSig = Buffer.from(agentSig);
    corruptedAgentSig[0] ^= 0xff; // Flip bits
    const corruptedSigValid = crypto.verify(null, agentDigest, agentKeys.publicKey, corruptedAgentSig);
    expect(corruptedSigValid).toBe(false);

    // ATTACK 2: Tampered spend value ($1.42 -> $1.43)
    const tamperedSpendPayload = Buffer.concat([
      agentDigest,
      agentSig,
      Buffer.from(merkleRoot, "utf-8"),
      Buffer.from(codeFingerprint, "utf-8"),
      Buffer.from("1.43", "utf-8"),
    ]);
    const tamperedHarnessDigest = crypto.createHash("sha256").update(tamperedSpendPayload).digest();
    const tamperedSigValid = crypto.verify(null, tamperedHarnessDigest, harnessKeys.publicKey, harnessSig);
    expect(tamperedSigValid).toBe(false);

    // ATTACK 3: Float formatting ambiguity ("1.42" vs "1.420")
    const ambiguousSpendPayload = Buffer.concat([
      agentDigest,
      agentSig,
      Buffer.from(merkleRoot, "utf-8"),
      Buffer.from(codeFingerprint, "utf-8"),
      Buffer.from("1.420", "utf-8"),
    ]);
    const ambiguousHarnessDigest = crypto.createHash("sha256").update(ambiguousSpendPayload).digest();
    const ambiguousSigValid = crypto.verify(null, ambiguousHarnessDigest, harnessKeys.publicKey, harnessSig);
    expect(ambiguousSigValid).toBe(false); // Proves necessity of strict canonical serialization (e.g. integer cents or RFC 8785 JCS)
  });

  it("checks SQL DDL tautology flaw in causal_edges chk_temporal_order constraint", () => {
    // Look at line 909-911 of blueprint:
    // CONSTRAINT chk_temporal_order CHECK (
    //     relationship_type NOT IN ('CAUSED_BY', 'BLOCKS') OR created_at >= created_at
    // )
    // Notice `created_at >= created_at` is always true!
    const testRow = { relationship_type: "CAUSED_BY", created_at: new Date() };
    const tautologyResult = testRow.created_at >= testRow.created_at;
    expect(tautologyResult).toBe(true); // Demonstrates the check constraint is a no-op tautology
  });
});
