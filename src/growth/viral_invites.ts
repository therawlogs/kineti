// src/growth/viral_invites.ts
// Kineti OS — Tier-based Viral Invite Engine & Vanity Referrals

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { projectKdir, readJson, writeJson, ensureDir } from "../../bin/lib.ts";

export type UserSubscriptionTier = "free" | "pro" | "admin";

export interface InviteRecord {
  code: string;
  vanity_url: string;
  created_at: number;
  recipient?: string;
  status: "pending" | "claimed" | "revoked";
  claimed_at?: number;
}

export interface InvitesState {
  owner_handle: string;
  tier: UserSubscriptionTier;
  invites: Record<string, InviteRecord>;
}

export class ViralInviteEngine {
  private filePath: string;
  private state: InvitesState;

  constructor(ownerHandle: string = "user", tier: UserSubscriptionTier = "free", customPath?: string) {
    this.filePath = customPath || path.join(projectKdir(), "invites.json");
    this.state = this.loadState(ownerHandle, tier);
  }

  private loadState(ownerHandle: string, tier: UserSubscriptionTier): InvitesState {
    ensureDir(path.dirname(this.filePath));
    const disk = readJson<InvitesState>(this.filePath);
    if (disk && disk.owner_handle) {
      return disk;
    }
    return {
      owner_handle: ownerHandle,
      tier,
      invites: {},
    };
  }

  private persist(): void {
    ensureDir(path.dirname(this.filePath));
    writeJson(this.filePath, this.state);
  }

  public getTier(): UserSubscriptionTier {
    return this.state.tier;
  }

  public setTier(tier: UserSubscriptionTier): void {
    this.state.tier = tier;
    this.persist();
  }

  public getMaxQuota(): number {
    switch (this.state.tier) {
      case "free":
        return 3;
      case "pro":
        return 10;
      case "admin":
        return -1; // Unlimited
    }
  }

  public getUsedQuota(): number {
    return Object.values(this.state.invites).filter((i) => i.status !== "revoked").length;
  }

  public getRemainingQuota(): number {
    const max = this.getMaxQuota();
    if (max === -1) return 999999;
    const remaining = max - this.getUsedQuota();
    return Math.max(0, remaining);
  }

  public generateVanityUrl(code: string): string {
    const safeHandle = (this.state.owner_handle || "user").replace(/[^\w-]/g, "");
    return `https://getkineti.com/join/${safeHandle}?code=${encodeURIComponent(code)}`;
  }

  public createInvite(recipient?: string): InviteRecord {
    if (this.getRemainingQuota() <= 0) {
      throw new Error(`Invite quota exceeded for tier '${this.state.tier}'. Upgrade for more invites.`);
    }

    const cleanRecipient = recipient
      ? recipient.trim().slice(0, 100).replace(/[^\w@.-]/g, "")
      : undefined;

    const code = `kineti_${crypto.randomBytes(4).toString("hex")}`;
    const record: InviteRecord = {
      code,
      vanity_url: this.generateVanityUrl(code),
      created_at: Date.now(),
      recipient: cleanRecipient,
      status: "pending",
    };

    this.state.invites[code] = record;
    this.persist();
    return record;
  }

  public listInvites(): InviteRecord[] {
    return Object.values(this.state.invites);
  }

  public revokeInvite(code: string): boolean {
    const inv = this.state.invites[code];
    if (!inv || inv.status === "claimed") return false;
    inv.status = "revoked";
    this.persist();
    return true;
  }

  public claimInvite(code: string): boolean {
    const inv = this.state.invites[code];
    if (!inv || inv.status !== "pending") return false;
    inv.status = "claimed";
    inv.claimed_at = Date.now();
    this.persist();
    return true;
  }
}
