import { describe, test, expect, beforeEach } from "bun:test";
import { TrustedNetworkManager } from "../src/swarm/trusted_network.ts";
import { PrivacyGovernanceManager } from "../src/privacy/governance.ts";
import { ViralInviteEngine } from "../src/growth/viral_invites.ts";
import path from "node:path";
import fs from "node:fs";

describe("Kineti Autonomous Capabilities & Governance Layer", () => {
  const tmpDir = path.join(process.cwd(), ".kineti", "test_scratch");

  beforeEach(() => {
    fs.mkdirSync(tmpDir, { recursive: true });
  });

  test("TrustedNetworkManager enforces whitelist, blocking, pause, and minimum-disclosure coordination", () => {
    const meshPath = path.join(tmpDir, "test_mesh.json");
    if (fs.existsSync(meshPath)) fs.unlinkSync(meshPath);

    const manager = new TrustedNetworkManager("agent_user", meshPath);
    expect(manager.isMeshPaused()).toBe(false);

    // 1. Inbound request from unknown peer
    const reqRes = manager.receiveConnectionRequest(
      "agent_alex",
      "Alex Kim",
      "@alexkim",
      "inner_circle",
      "Group dinner planning",
    );
    expect(reqRes.status).toBe("queued");
    expect(reqRes.requestId).toBeDefined();

    // 2. Approve request
    const peer = manager.approveRequest(reqRes.requestId!, "inner_circle");
    expect(peer.peer_agent_id).toBe("agent_alex");
    expect(peer.can_coordinate_dining).toBe(true);

    // 3. Minimum-disclosure schedule alignment
    const mySlots = [
      { start_epoch: 1000, end_epoch: 2000, is_free: true },
      { start_epoch: 3000, end_epoch: 4500, is_free: true },
    ];
    const peerSlots = [
      { start_epoch: 3000, end_epoch: 5000, is_free: true },
    ];

    const alignment = manager.alignSchedules("agent_alex", mySlots, peerSlots, 15);
    expect(alignment.status).toBe("matched");
    expect(alignment.mutual_slot?.start_epoch).toBe(3000);
    expect(alignment.mutual_slot?.end_epoch).toBe(3900); // 15 mins = 900s

    // 4. Dining coordination
    const dining = manager.coordinateDining(
      "agent_alex",
      ["Chez Panisse", "Shizen"],
      ["Shizen", "State Bird"],
      ["Pork"],
    );
    expect(dining).toBe("Shizen");

    // 5. Global pause toggle
    manager.pauseMesh();
    expect(manager.isMeshPaused()).toBe(true);
    const blockedByPause = manager.receiveConnectionRequest("agent_bob", "Bob", "@bob", "colleague", "Hi");
    expect(blockedByPause.status).toBe("paused");

    // 6. Block peer
    manager.blockPeer("agent_spammer");
    expect(manager.isPeerBlocked("agent_spammer")).toBe(true);
    const spamRes = manager.receiveConnectionRequest("agent_spammer", "Spam", "@spam", "service_agent", "Ad");
    expect(spamRes.status).toBe("blocked");
  });

  test("PrivacyGovernanceManager toggles model training and coordinates self-serve external purge", () => {
    const privPath = path.join(tmpDir, "test_privacy.json");
    if (fs.existsSync(privPath)) fs.unlinkSync(privPath);

    const privacy = new PrivacyGovernanceManager("alex", privPath);
    expect(privacy.isImproveKinetiEnabled()).toBe(false);
    expect(privacy.canEgressTelemetry()).toBe(false);

    // Toggle opt-in / opt-out
    privacy.setImproveKineti(true);
    expect(privacy.isImproveKinetiEnabled()).toBe(true);
    expect(privacy.canEgressTelemetry()).toBe(true);

    privacy.setImproveKineti(false);
    expect(privacy.isImproveKinetiEnabled()).toBe(false);
    expect(privacy.canEgressTelemetry()).toBe(false);

    // Trigger external data purge
    const purge = privacy.executeExternalDataPurge();
    expect(purge.success).toBe(true);
    expect(purge.sources_cleared).toContain("google_workspace");
    expect(purge.sources_cleared).toContain("microsoft_graph");
    expect(purge.sources_cleared).toContain("notion");
    expect(purge.receipt_digest.length).toBe(64);
  });

  test("ViralInviteEngine enforces tier-based quotas and vanity link redemption", () => {
    const invPath = path.join(tmpDir, "test_invites.json");
    if (fs.existsSync(invPath)) fs.unlinkSync(invPath);

    const engine = new ViralInviteEngine("alex", "free", invPath);
    expect(engine.getMaxQuota()).toBe(3);
    expect(engine.getRemainingQuota()).toBe(3);

    const inv1 = engine.createInvite("friend1@example.com");
    expect(inv1.vanity_url).toContain("getkineti.com/join/alex?code=");
    expect(engine.getRemainingQuota()).toBe(2);

    const inv2 = engine.createInvite("friend2@example.com");
    const inv3 = engine.createInvite("friend3@example.com");
    expect(engine.getRemainingQuota()).toBe(0);

    // 4th invite must throw
    expect(() => engine.createInvite("friend4@example.com")).toThrow("quota exceeded");

    // Upgrade tier to pro -> quota expands to 10
    engine.setTier("pro");
    expect(engine.getMaxQuota()).toBe(10);
    expect(engine.getRemainingQuota()).toBe(7);

    // Claim invite
    expect(engine.claimInvite(inv1.code)).toBe(true);
    expect(engine.claimInvite(inv1.code)).toBe(false); // cannot reclaim
  });
});
