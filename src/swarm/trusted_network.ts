// src/swarm/trusted_network.ts
// Kineti OS — Trusted Person Network & Inter-Agent Mesh Coordinator

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { projectKdir, readJson, writeJson, ensureDir } from "../../bin/lib.ts";

export type TrustTier = "inner_circle" | "colleague" | "service_agent";
export type RequestStatus = "pending" | "approved" | "rejected" | "blocked";

export interface TrustedPeer {
  peer_agent_id: string;
  display_name: string;
  handle: string;
  tier: TrustTier;
  can_propose_schedules: boolean;
  can_query_availability: boolean;
  can_coordinate_dining: boolean;
  added_at: number;
}

export interface ConnectionRequest {
  request_id: string;
  requester_agent_id: string;
  requester_name: string;
  requester_handle: string;
  requested_tier: TrustTier;
  note: string;
  created_at: number;
  status: RequestStatus;
}

export interface TimeSlot {
  start_epoch: number;
  end_epoch: number;
  is_free: boolean;
}

export interface TrustedNetworkState {
  agent_id: string;
  mesh_paused: boolean;
  peers: Record<string, TrustedPeer>;
  blocked_peers: string[];
  pending_requests: Record<string, ConnectionRequest>;
}

export class TrustedNetworkManager {
  private stateFilePath: string;
  private state: TrustedNetworkState;

  constructor(agentId: string = "agent_praveen", storagePath?: string) {
    this.stateFilePath = storagePath || path.join(projectKdir(), "trusted_network.json");
    this.state = this.loadState(agentId);
  }

  private loadState(agentId: string): TrustedNetworkState {
    ensureDir(path.dirname(this.stateFilePath));
    const disk = readJson<TrustedNetworkState>(this.stateFilePath);
    if (disk && disk.agent_id) {
      return disk;
    }
    return {
      agent_id: agentId,
      mesh_paused: false,
      peers: {},
      blocked_peers: [],
      pending_requests: {},
    };
  }

  private persist(): void {
    ensureDir(path.dirname(this.stateFilePath));
    writeJson(this.stateFilePath, this.state);
  }

  public getAgentId(): string {
    return this.state.agent_id;
  }

  public isMeshPaused(): boolean {
    return this.state.mesh_paused;
  }

  public pauseMesh(): void {
    this.state.mesh_paused = true;
    this.persist();
  }

  public resumeMesh(): void {
    this.state.mesh_paused = false;
    this.persist();
  }

  public isPeerBlocked(peerAgentId: string): boolean {
    return this.state.blocked_peers.includes(peerAgentId);
  }

  public blockPeer(peerAgentId: string): void {
    if (!this.state.blocked_peers.includes(peerAgentId)) {
      this.state.blocked_peers.push(peerAgentId);
    }
    delete this.state.peers[peerAgentId];
    this.persist();
  }

  public unblockPeer(peerAgentId: string): void {
    this.state.blocked_peers = this.state.blocked_peers.filter((id) => id !== peerAgentId);
    this.persist();
  }

  public getBlockedPeers(): string[] {
    return [...this.state.blocked_peers];
  }

  public getPeers(): TrustedPeer[] {
    return Object.values(this.state.peers);
  }

  public getPeer(peerAgentId: string): TrustedPeer | undefined {
    return this.state.peers[peerAgentId];
  }

  public addPeer(peer: TrustedPeer): void {
    if (this.isPeerBlocked(peer.peer_agent_id)) {
      throw new Error(`Cannot add blocked peer: ${peer.peer_agent_id}`);
    }
    this.state.peers[peer.peer_agent_id] = peer;
    this.persist();
  }

  public removePeer(peerAgentId: string): boolean {
    if (this.state.peers[peerAgentId]) {
      delete this.state.peers[peerAgentId];
      this.persist();
      return true;
    }
    return false;
  }

  public getPendingRequests(): ConnectionRequest[] {
    return Object.values(this.state.pending_requests).filter((r) => r.status === "pending");
  }

  public receiveConnectionRequest(
    requesterAgentId: string,
    requesterName: string,
    requesterHandle: string,
    requestedTier: TrustTier,
    note: string,
  ): { status: "queued" | "blocked" | "paused"; requestId?: string } {
    const cleanAgentId = (requesterAgentId || "").trim().slice(0, 64).replace(/[^\w-]/g, "");
    if (!cleanAgentId) {
      throw new Error("Invalid requesterAgentId: must be alphanumeric/dash/underscore");
    }
    const validTiers: TrustTier[] = ["inner_circle", "colleague", "service_agent"];
    if (!validTiers.includes(requestedTier)) {
      throw new Error(`Invalid requested tier: '${requestedTier}'`);
    }
    const cleanName = (requesterName || "").trim().slice(0, 100);
    const cleanHandle = (requesterHandle || "").trim().slice(0, 50).replace(/[^\w@.-]/g, "");
    const cleanNote = (note || "").trim().slice(0, 500);

    if (this.isPeerBlocked(cleanAgentId)) {
      return { status: "blocked" };
    }
    if (this.state.mesh_paused) {
      return { status: "paused" };
    }

    const requestId = `req_${cleanAgentId}_${Date.now()}`;
    const req: ConnectionRequest = {
      request_id: requestId,
      requester_agent_id: cleanAgentId,
      requester_name: cleanName,
      requester_handle: cleanHandle,
      requested_tier: requestedTier,
      note: cleanNote,
      created_at: Date.now(),
      status: "pending",
    };

    this.state.pending_requests[requestId] = req;
    this.persist();
    return { status: "queued", requestId };
  }

  public approveRequest(requestId: string, tier: TrustTier): TrustedPeer {
    const req = this.state.pending_requests[requestId];
    if (!req) {
      throw new Error(`Request not found: ${requestId}`);
    }

    req.status = "approved";
    const peer: TrustedPeer = {
      peer_agent_id: req.requester_agent_id,
      display_name: req.requester_name,
      handle: req.requester_handle,
      tier,
      can_propose_schedules: true,
      can_query_availability: true,
      can_coordinate_dining: tier === "inner_circle",
      added_at: Date.now(),
    };

    this.state.peers[peer.peer_agent_id] = peer;
    this.persist();
    return peer;
  }

  public rejectRequest(requestId: string): boolean {
    const req = this.state.pending_requests[requestId];
    if (!req) return false;
    req.status = "rejected";
    this.persist();
    return true;
  }

  /**
   * Aligns schedules with minimum-necessary disclosure (free/busy slots only).
   * Calendar event details, titles, and private notes are never disclosed.
   */
  public alignSchedules(
    peerAgentId: string,
    mySlots: TimeSlot[],
    peerSlots: TimeSlot[],
    durationMinutes: number,
  ): { mutual_slot?: TimeSlot; status: "matched" | "no_match" } {
    const peer = this.state.peers[peerAgentId];
    if (!peer) {
      throw new Error(`Peer not in trusted network: ${peerAgentId}`);
    }
    if (!peer.can_query_availability) {
      throw new Error(`Peer lacks availability query permission`);
    }

    const durationSec = durationMinutes * 60;
    for (const mySlot of mySlots.filter((s) => s.is_free)) {
      for (const peerSlot of peerSlots.filter((s) => s.is_free)) {
        const overlapStart = Math.max(mySlot.start_epoch, peerSlot.start_epoch);
        const overlapEnd = Math.min(mySlot.end_epoch, peerSlot.end_epoch);
        if (overlapEnd - overlapStart >= durationSec) {
          return {
            mutual_slot: {
              start_epoch: overlapStart,
              end_epoch: overlapStart + durationSec,
              is_free: true,
            },
            status: "matched",
          };
        }
      }
    }

    return { status: "no_match" };
  }

  /**
   * Coordinates group dining preferences between inner-circle peers.
   */
  public coordinateDining(
    peerAgentId: string,
    myFavorites: string[],
    peerFavorites: string[],
    dietaryBlocklist: string[],
  ): string | null {
    const peer = this.state.peers[peerAgentId];
    if (!peer || peer.tier !== "inner_circle" || !peer.can_coordinate_dining) {
      throw new Error(`Peer not authorized for dining coordination: ${peerAgentId}`);
    }

    for (const fav of myFavorites) {
      const peerWants = peerFavorites.some((p) => p.toLowerCase() === fav.toLowerCase());
      const violatesDiet = dietaryBlocklist.some((d) => fav.toLowerCase().includes(d.toLowerCase()));
      if (peerWants && !violatesDiet) {
        return fav;
      }
    }

    return null;
  }
}
