//! K2K (Kineti-to-Kineti) Inter-Agent Mesh Protocol (`kineti-gateway::k2k`).
//!
//! Provides a secure, peer-to-peer inter-agent coordination protocol:
//! - Strict whitelist policy: Unknown external agents are held in pending queue
//! - Global pause toggle: Instantly stop all inbound agent-to-agent chatter
//! - Three-tier trust model: `InnerCircle`, `Colleague`, `ServiceAgent`
//! - Minimum-necessary disclosure: Free/busy availability without leaking calendar titles or private notes
//! - Autonomous dining & group schedule coordination

use kineti_connectors::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use std::collections::{BTreeMap, BTreeSet};

/// Trust tier assigned to a connected external peer agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TrustTier {
    /// Full schedule alignment, dining coordination, and standing session synchronization.
    InnerCircle,
    /// Availability querying and proposed time slots only; no dining or deep coordination.
    Colleague,
    /// Transactional external queries only (e.g. restaurant concierge, reservation verification).
    ServiceAgent,
}

impl TrustTier {
    /// Returns the string representation of the tier.
    pub fn as_str(&self) -> &'static str {
        match self {
            TrustTier::InnerCircle => "inner_circle",
            TrustTier::Colleague => "colleague",
            TrustTier::ServiceAgent => "service_agent",
        }
    }

    /// Parses string representation into TrustTier.
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "inner_circle" => Some(TrustTier::InnerCircle),
            "colleague" => Some(TrustTier::Colleague),
            "service_agent" => Some(TrustTier::ServiceAgent),
            _ => None,
        }
    }
}

/// Lifecycle status of an inbound connection request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    /// Awaiting user review and approval.
    Pending,
    /// Approved and added to the allowlist.
    Approved,
    /// Rejected by user.
    Rejected,
    /// Blocked to prevent future contact.
    Blocked,
}

/// An inbound connection request from an external agent.
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectionRequest {
    /// Unique identifier for this connection request.
    pub request_id: String,
    /// Unique agent identifier of the sender.
    pub requester_agent_id: String,
    /// Display name of the human behind the requester agent.
    pub requester_name: String,
    /// Human-readable handle or identifier (e.g. @maya, phone, or email).
    pub requester_handle: String,
    /// Requested trust level.
    pub requested_tier: TrustTier,
    /// Optional introductory note.
    pub note: String,
    /// Unix timestamp in milliseconds when request was received.
    pub created_at: u64,
    /// Current status.
    pub status: RequestStatus,
}

/// An approved trusted peer in the agent's allowlist.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustedPeer {
    /// Unique agent identifier of the trusted peer.
    pub peer_agent_id: String,
    /// Display name of the trusted person.
    pub display_name: String,
    /// User handle or phone/email.
    pub handle: String,
    /// Assigned trust tier.
    pub tier: TrustTier,
    /// Whether this peer is allowed to propose schedule slots.
    pub can_propose_schedules: bool,
    /// Whether this peer can query free/busy availability.
    pub can_query_availability: bool,
    /// Whether this peer can coordinate group dining reservations.
    pub can_coordinate_dining: bool,
    /// Timestamp when added to trusted network.
    pub added_at: u64,
}

/// Time slot representation for minimum-disclosure schedule alignment.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeSlot {
    /// Start epoch in seconds.
    pub start_epoch: u64,
    /// End epoch in seconds.
    pub end_epoch: u64,
    /// Whether this slot is open and free.
    pub is_free: bool,
}

/// Outcome of schedule alignment.
#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleStatus {
    /// A mutual free slot was identified without conflicts.
    MutualSlotFound(TimeSlot),
    /// No mutually free window within the requested boundary.
    NoMutualSlot,
    /// Candidate time slots offered for review.
    ProposedTimes(Vec<TimeSlot>),
}

/// Discrete intent communicated via K2K protocol message.
#[derive(Debug, Clone, PartialEq)]
pub enum K2KIntent {
    /// Inbound request to establish peer connection.
    ConnectRequest {
        /// Requester human name.
        requester_name: String,
        /// Requester handle.
        requester_handle: String,
        /// Proposed trust tier.
        proposed_tier: TrustTier,
        /// Introductory message.
        note: String,
    },
    /// Notification that connection request was approved.
    ConnectApproved {
        /// Granted trust tier.
        assigned_tier: TrustTier,
    },
    /// Notification that connection request was declined.
    ConnectRejected,
    /// Request to align schedules for a meeting or session.
    ScheduleAlignmentRequest {
        /// Start of search window (epoch sec).
        window_start: u64,
        /// End of search window (epoch sec).
        window_end: u64,
        /// Duration of requested meeting in minutes.
        duration_minutes: u32,
        /// High-level context hint (e.g. "Catch-up", never confidential agenda).
        context_hint: String,
    },
    /// Response providing candidate slots and status.
    ScheduleAlignmentResponse {
        /// Available mutual slots.
        available_slots: Vec<TimeSlot>,
        /// Status outcome.
        status: ScheduleStatus,
    },
    /// Request to coordinate group dining reservations.
    DiningCoordinationRequest {
        /// Number of guests.
        party_size: u32,
        /// Desired cuisine styles (e.g. ["Japanese", "Italian"]).
        cuisine_preferences: Vec<String>,
        /// Hard dietary ceiling restrictions (e.g. ["vegan", "peanut allergy"]).
        dietary_restrictions: Vec<String>,
        /// Date / time window string.
        date_hint: String,
    },
    /// Response with mutually viable venue recommendation.
    DiningCoordinationResponse {
        /// Viable restaurant agreed upon.
        agreed_restaurant: Option<String>,
        /// Reserved or proposed time.
        time_slot: Option<String>,
        /// Whether reservation is finalized.
        confirmed: bool,
    },
    /// Sync for standing recurring sync sessions.
    StandingSessionSync {
        /// Cadence string (e.g. "weekly_tuesday").
        cadence: String,
        /// Current recurring time slot.
        current_slot: String,
        /// Next scheduled occurrence.
        next_slot: Option<String>,
    },
}

/// Standard envelope for Kineti-to-Kineti protocol communication.
#[derive(Debug, Clone, PartialEq)]
pub struct K2KProtocolMessage {
    /// Unique message identifier.
    pub message_id: String,
    /// Sender agent ID.
    pub sender_agent_id: String,
    /// Target recipient agent ID.
    pub recipient_agent_id: String,
    /// Timestamp in epoch milliseconds.
    pub timestamp: u64,
    /// Optional cryptographic signature of payload.
    pub signature: Option<String>,
    /// Contained protocol intent.
    pub intent: K2KIntent,
}

/// Result of evaluating an inbound K2K message against mesh policies.
#[derive(Debug, Clone, PartialEq)]
pub enum K2KDeliveryResult {
    /// Accepted and dispatched to local agent reasoning loop.
    Accepted(K2KIntent),
    /// Sender is unknown; connection request was quarantined to pending queue.
    QueuedPendingApproval(String),
    /// Sender is blocked; message dropped silently.
    Blocked,
    /// Mesh communication is globally paused.
    MeshPaused,
    /// Sender tier does not possess permissions for this intent.
    UnauthorizedTier,
}

/// K2K Agent-to-Agent Mesh Coordinator.
#[derive(Debug, Clone)]
pub struct K2KCoordinator {
    agent_id: String,
    mesh_paused: bool,
    peers: BTreeMap<String, TrustedPeer>,
    blocked_peers: BTreeSet<String>,
    pending_requests: BTreeMap<String, ConnectionRequest>,
}

impl K2KCoordinator {
    /// Creates a new K2K coordinator for a local agent.
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            mesh_paused: false,
            peers: BTreeMap::new(),
            blocked_peers: BTreeSet::new(),
            pending_requests: BTreeMap::new(),
        }
    }

    /// Local agent identifier.
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Globally pauses all inbound mesh traffic.
    pub fn pause_mesh(&mut self) {
        self.mesh_paused = true;
    }

    /// Resumes inbound mesh traffic.
    pub fn resume_mesh(&mut self) {
        self.mesh_paused = false;
    }

    /// Returns whether mesh communication is paused.
    pub fn is_paused(&self) -> bool {
        self.mesh_paused
    }

    /// Checks if a peer is explicitly blocked.
    pub fn is_peer_blocked(&self, peer_id: &str) -> bool {
        self.blocked_peers.contains(peer_id)
    }

    /// Blocks a peer by agent ID. Removes them from allowlist if present.
    pub fn block_peer(&mut self, peer_id: &str) {
        self.blocked_peers.insert(peer_id.to_string());
        self.peers.remove(peer_id);
    }

    /// Unblocks a peer.
    pub fn unblock_peer(&mut self, peer_id: &str) {
        self.blocked_peers.remove(peer_id);
    }

    /// Adds or updates an approved trusted peer in the allowlist.
    pub fn add_peer(&mut self, peer: TrustedPeer) {
        if !self.blocked_peers.contains(&peer.peer_agent_id) {
            self.peers.insert(peer.peer_agent_id.clone(), peer);
        }
    }

    /// Removes a peer from the allowlist.
    pub fn remove_peer(&mut self, peer_id: &str) -> Option<TrustedPeer> {
        self.peers.remove(peer_id)
    }

    /// Fetches a trusted peer by agent ID.
    pub fn get_peer(&self, peer_id: &str) -> Option<&TrustedPeer> {
        self.peers.get(peer_id)
    }

    /// Returns list of all approved trusted peers.
    pub fn list_peers(&self) -> Vec<&TrustedPeer> {
        self.peers.values().collect()
    }

    /// Returns list of pending inbound connection requests awaiting user review.
    pub fn list_pending_requests(&self) -> Vec<&ConnectionRequest> {
        self.pending_requests
            .values()
            .filter(|r| r.status == RequestStatus::Pending)
            .collect()
    }

    /// Approves a pending connection request and adds the peer to allowlist.
    pub fn approve_request(&mut self, request_id: &str, tier: TrustTier) -> Option<TrustedPeer> {
        if let Some(req) = self.pending_requests.get_mut(request_id) {
            req.status = RequestStatus::Approved;
            let peer = TrustedPeer {
                peer_agent_id: req.requester_agent_id.clone(),
                display_name: req.requester_name.clone(),
                handle: req.requester_handle.clone(),
                tier,
                can_propose_schedules: true,
                can_query_availability: true,
                can_coordinate_dining: tier == TrustTier::InnerCircle,
                added_at: 1710000000,
            };
            self.add_peer(peer.clone());
            Some(peer)
        } else {
            None
        }
    }

    /// Rejects a pending connection request.
    pub fn reject_request(&mut self, request_id: &str) -> bool {
        if let Some(req) = self.pending_requests.get_mut(request_id) {
            req.status = RequestStatus::Rejected;
            true
        } else {
            false
        }
    }

    /// Processes an inbound K2K protocol message according to zero-trust whitelist rules.
    pub fn receive_message(&mut self, message: K2KProtocolMessage) -> K2KDeliveryResult {
        // 1. Check if sender is blocked
        if self.blocked_peers.contains(&message.sender_agent_id) {
            return K2KDeliveryResult::Blocked;
        }

        // 2. Check if global mesh pause is active
        if self.mesh_paused {
            return K2KDeliveryResult::MeshPaused;
        }

        // 3. Check if this is an introductory connection request
        if let K2KIntent::ConnectRequest {
            ref requester_name,
            ref requester_handle,
            proposed_tier,
            ref note,
        } = message.intent
        {
            let req_id = format!("req_{}_{}", message.sender_agent_id, message.timestamp);
            let req = ConnectionRequest {
                request_id: req_id.clone(),
                requester_agent_id: message.sender_agent_id,
                requester_name: requester_name.clone(),
                requester_handle: requester_handle.clone(),
                requested_tier: proposed_tier,
                note: note.clone(),
                created_at: message.timestamp,
                status: RequestStatus::Pending,
            };
            self.pending_requests.insert(req_id.clone(), req);
            return K2KDeliveryResult::QueuedPendingApproval(req_id);
        }

        // 4. Strict Whitelist Enforcement: Sender must already be on allowlist
        let peer = match self.peers.get(&message.sender_agent_id) {
            Some(p) => p,
            None => {
                // Unknown sender attempting direct operational intent: quarantine to pending
                let req_id = format!("req_{}_{}", message.sender_agent_id, message.timestamp);
                let req = ConnectionRequest {
                    request_id: req_id.clone(),
                    requester_agent_id: message.sender_agent_id,
                    requester_name: "Unknown Agent".to_string(),
                    requester_handle: "Unknown".to_string(),
                    requested_tier: TrustTier::Colleague,
                    note: "Attempted unverified direct communication".to_string(),
                    created_at: message.timestamp,
                    status: RequestStatus::Pending,
                };
                self.pending_requests.insert(req_id.clone(), req);
                return K2KDeliveryResult::QueuedPendingApproval(req_id);
            }
        };

        // 5. Tier & Permission Verification
        match &message.intent {
            K2KIntent::ScheduleAlignmentRequest { .. } | K2KIntent::ScheduleAlignmentResponse { .. } => {
                if !peer.can_query_availability {
                    return K2KDeliveryResult::UnauthorizedTier;
                }
            }
            K2KIntent::DiningCoordinationRequest { .. } | K2KIntent::DiningCoordinationResponse { .. } => {
                if !peer.can_coordinate_dining || peer.tier != TrustTier::InnerCircle {
                    return K2KDeliveryResult::UnauthorizedTier;
                }
            }
            K2KIntent::StandingSessionSync { .. } => {
                if peer.tier != TrustTier::InnerCircle {
                    return K2KDeliveryResult::UnauthorizedTier;
                }
            }
            _ => {}
        }

        K2KDeliveryResult::Accepted(message.intent)
    }

    /// Aligns schedules with minimum-necessary disclosure (free/busy slots only).
    /// Private calendar details and event titles are never shared across agents.
    pub fn align_schedule(
        &self,
        peer_id: &str,
        my_slots: &[TimeSlot],
        peer_slots: &[TimeSlot],
        duration_minutes: u32,
    ) -> Result<ScheduleStatus, &'static str> {
        let peer = self.peers.get(peer_id).ok_or("Peer not in trusted network")?;
        if !peer.can_query_availability {
            return Err("Peer lacks availability query permission");
        }

        let duration_secs = (duration_minutes as u64) * 60;

        // Find intersecting free slots
        let mut mutual_slots = Vec::new();

        for my_slot in my_slots.iter().filter(|s| s.is_free) {
            for peer_slot in peer_slots.iter().filter(|s| s.is_free) {
                let overlap_start = my_slot.start_epoch.max(peer_slot.start_epoch);
                let overlap_end = my_slot.end_epoch.min(peer_slot.end_epoch);

                if overlap_end > overlap_start && (overlap_end - overlap_start) >= duration_secs {
                    mutual_slots.push(TimeSlot {
                        start_epoch: overlap_start,
                        end_epoch: overlap_start + duration_secs,
                        is_free: true,
                    });
                }
            }
        }

        if let Some(first) = mutual_slots.first() {
            Ok(ScheduleStatus::MutualSlotFound(first.clone()))
        } else if !mutual_slots.is_empty() {
            Ok(ScheduleStatus::ProposedTimes(mutual_slots))
        } else {
            Ok(ScheduleStatus::NoMutualSlot)
        }
    }

    /// Coordinates dining preferences between two inner-circle agents.
    pub fn coordinate_dining(
        &self,
        peer_id: &str,
        local_favorites: &[&str],
        peer_favorites: &[&str],
        dietary_blocklist: &[&str],
    ) -> Result<Option<String>, &'static str> {
        let peer = self.peers.get(peer_id).ok_or("Peer not in trusted network")?;
        if peer.tier != TrustTier::InnerCircle || !peer.can_coordinate_dining {
            return Err("Peer not authorized for dining coordination");
        }

        // Find intersection of favorites that do not violate dietary ceiling
        for &fav in local_favorites {
            let matches_peer = peer_favorites.iter().any(|&p| p.eq_ignore_ascii_case(fav));
            let violates_diet = dietary_blocklist.iter().any(|&d| fav.to_lowercase().contains(&d.to_lowercase()));

            if matches_peer && !violates_diet {
                return Ok(Some(fav.to_string()));
            }
        }

        Ok(None)
    }
}

impl KinetiConnectorProtocol for K2KCoordinator {
    fn connector_name(&self) -> &'static str {
        "k2k_mesh"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "pause_mesh",
            "resume_mesh",
            "approve_request",
            "reject_request",
            "block_peer",
            "unblock_peer",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "pause_mesh" | "resume_mesh" => ConsequenceLevel::Operational,
            "approve_request" | "block_peer" => ConsequenceLevel::HighConsequence,
            "reject_request" | "unblock_peer" => ConsequenceLevel::Operational,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let mut map = BTreeMap::new();
        match action {
            "pause_mesh" => {
                map.insert("status".to_string(), Value::String("mesh_paused".to_string()));
                Ok(Value::Object(map))
            }
            "resume_mesh" => {
                map.insert("status".to_string(), Value::String("mesh_active".to_string()));
                Ok(Value::Object(map))
            }
            "approve_request" => {
                let req_id = get_str_property(payload, "request_id").unwrap_or("");
                map.insert("approved".to_string(), Value::Bool(!req_id.is_empty()));
                map.insert("request_id".to_string(), Value::String(req_id.to_string()));
                Ok(Value::Object(map))
            }
            "block_peer" => {
                let peer_id = get_str_property(payload, "peer_id").unwrap_or("");
                map.insert("blocked".to_string(), Value::Bool(!peer_id.is_empty()));
                map.insert("peer_id".to_string(), Value::String(peer_id.to_string()));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strict_whitelist_quarantines_unknown_agent() {
        let mut coord = K2KCoordinator::new("agent_user");

        let msg = K2KProtocolMessage {
            message_id: "msg_001".to_string(),
            sender_agent_id: "agent_unknown".to_string(),
            recipient_agent_id: "agent_user".to_string(),
            timestamp: 1710000100,
            signature: None,
            intent: K2KIntent::ConnectRequest {
                requester_name: "Alice Smith".to_string(),
                requester_handle: "@alicesmith".to_string(),
                proposed_tier: TrustTier::InnerCircle,
                note: "Let's connect our calendars!".to_string(),
            },
        };

        // Unknown agent should be quarantined to pending requests
        let result = coord.receive_message(msg);
        match result {
            K2KDeliveryResult::QueuedPendingApproval(req_id) => {
                assert!(req_id.contains("agent_unknown"));
            }
            other => panic!("Expected QueuedPendingApproval, got {:?}", other),
        }

        assert_eq!(coord.list_pending_requests().len(), 1);
        assert_eq!(coord.list_peers().len(), 0);
    }

    #[test]
    fn test_approve_request_and_allowlist_delivery() {
        let mut coord = K2KCoordinator::new("agent_user");

        let msg = K2KProtocolMessage {
            message_id: "msg_002".to_string(),
            sender_agent_id: "agent_alex".to_string(),
            recipient_agent_id: "agent_user".to_string(),
            timestamp: 1710000200,
            signature: None,
            intent: K2KIntent::ConnectRequest {
                requester_name: "Alex Kim".to_string(),
                requester_handle: "@alexkim".to_string(),
                proposed_tier: TrustTier::InnerCircle,
                note: "Team dinner planning".to_string(),
            },
        };

        let res = coord.receive_message(msg);
        let req_id = match res {
            K2KDeliveryResult::QueuedPendingApproval(id) => id,
            _ => panic!("Expected queued"),
        };

        // User approves request
        let approved = coord.approve_request(&req_id, TrustTier::InnerCircle);
        assert!(approved.is_some());
        assert_eq!(coord.list_peers().len(), 1);
        assert_eq!(coord.list_pending_requests().len(), 0);

        // Now Maya can send dining coordination request
        let dining_msg = K2KProtocolMessage {
            message_id: "msg_003".to_string(),
            sender_agent_id: "agent_alex".to_string(),
            recipient_agent_id: "agent_user".to_string(),
            timestamp: 1710000300,
            signature: None,
            intent: K2KIntent::DiningCoordinationRequest {
                party_size: 4,
                cuisine_preferences: vec!["Italian".to_string()],
                dietary_restrictions: vec!["gluten-free".to_string()],
                date_hint: "Friday 7 PM".to_string(),
            },
        };

        let dining_res = coord.receive_message(dining_msg);
        match dining_res {
            K2KDeliveryResult::Accepted(K2KIntent::DiningCoordinationRequest { party_size, .. }) => {
                assert_eq!(party_size, 4);
            }
            other => panic!("Expected Accepted dining request, got {:?}", other),
        }
    }

    #[test]
    fn test_global_pause_toggle() {
        let mut coord = K2KCoordinator::new("agent_user");

        coord.add_peer(TrustedPeer {
            peer_agent_id: "agent_friend".to_string(),
            display_name: "Friend".to_string(),
            handle: "@friend".to_string(),
            tier: TrustTier::InnerCircle,
            can_propose_schedules: true,
            can_query_availability: true,
            can_coordinate_dining: true,
            added_at: 1710000000,
        });

        // Toggle mesh paused
        coord.pause_mesh();
        assert!(coord.is_paused());

        let msg = K2KProtocolMessage {
            message_id: "msg_pause_test".to_string(),
            sender_agent_id: "agent_friend".to_string(),
            recipient_agent_id: "agent_user".to_string(),
            timestamp: 1710000400,
            signature: None,
            intent: K2KIntent::ScheduleAlignmentRequest {
                window_start: 1710000000,
                window_end: 1710003600,
                duration_minutes: 30,
                context_hint: "Quick sync".to_string(),
            },
        };

        assert_eq!(coord.receive_message(msg), K2KDeliveryResult::MeshPaused);

        // Resume mesh
        coord.resume_mesh();
        assert!(!coord.is_paused());
    }

    #[test]
    fn test_block_peer_drops_messages() {
        let mut coord = K2KCoordinator::new("agent_user");

        coord.block_peer("agent_spammer");
        assert!(coord.is_peer_blocked("agent_spammer"));

        let msg = K2KProtocolMessage {
            message_id: "msg_spam".to_string(),
            sender_agent_id: "agent_spammer".to_string(),
            recipient_agent_id: "agent_user".to_string(),
            timestamp: 1710000500,
            signature: None,
            intent: K2KIntent::ConnectRequest {
                requester_name: "Spam".to_string(),
                requester_handle: "@spam".to_string(),
                proposed_tier: TrustTier::ServiceAgent,
                note: "Buy now".to_string(),
            },
        };

        assert_eq!(coord.receive_message(msg), K2KDeliveryResult::Blocked);
    }

    #[test]
    fn test_minimum_disclosure_schedule_alignment() {
        let mut coord = K2KCoordinator::new("agent_user");

        coord.add_peer(TrustedPeer {
            peer_agent_id: "agent_partner".to_string(),
            display_name: "Partner".to_string(),
            handle: "@partner".to_string(),
            tier: TrustTier::InnerCircle,
            can_propose_schedules: true,
            can_query_availability: true,
            can_coordinate_dining: true,
            added_at: 1710000000,
        });

        let my_slots = vec![
            TimeSlot { start_epoch: 1000, end_epoch: 2000, is_free: true },
            TimeSlot { start_epoch: 2000, end_epoch: 3000, is_free: false },
            TimeSlot { start_epoch: 3000, end_epoch: 4000, is_free: true },
        ];

        let peer_slots = vec![
            TimeSlot { start_epoch: 1500, end_epoch: 2500, is_free: true }, // overlap 1500..2000 (500s < 1800s)
            TimeSlot { start_epoch: 3000, end_epoch: 5000, is_free: true }, // overlap 3000..4000 (1000s > 600s for 10 min)
        ];

        let alignment = coord.align_schedule("agent_partner", &my_slots, &peer_slots, 10).unwrap();
        match alignment {
            ScheduleStatus::MutualSlotFound(slot) => {
                assert_eq!(slot.start_epoch, 3000);
                assert_eq!(slot.end_epoch, 3600); // 10 minutes = 600s
            }
            _ => panic!("Expected MutualSlotFound"),
        }
    }

    #[test]
    fn test_dining_coordination_safe_match() {
        let mut coord = K2KCoordinator::new("agent_user");

        coord.add_peer(TrustedPeer {
            peer_agent_id: "agent_colleague".to_string(),
            display_name: "Colleague".to_string(),
            handle: "@colleague".to_string(),
            tier: TrustTier::InnerCircle,
            can_propose_schedules: true,
            can_query_availability: true,
            can_coordinate_dining: true,
            added_at: 1710000000,
        });

        let my_favorites = ["Chez Panisse", "Shizen", "Tartine"];
        let peer_favorites = ["State Bird", "Shizen", "Nobu"];
        let dietary_blocklist = ["Pork"];

        let match_res = coord
            .coordinate_dining("agent_colleague", &my_favorites, &peer_favorites, &dietary_blocklist)
            .unwrap();

        assert_eq!(match_res, Some("Shizen".to_string()));
    }
}
