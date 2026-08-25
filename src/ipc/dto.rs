//! Wire DTOs (§R1): pure serde types shared by both transports. The daemon
//! protocol is serialization over a UDS stream — never a second brain.

use serde::{Deserialize, Serialize};

/// Who wants money and how much they estimate. All amounts are micro-USD
/// (1_000_000 = $1.00) so the ledger stays integer-exact end to end.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ReserveCtx {
    pub stage: String,
    pub worker: String,
    pub est_micro_usd: u64,
}

/// A granted budget hold. `id` is informational (monotonic per owner);
/// settlement corrects global AND scoped ledgers by the reserved amount.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Reservation {
    pub id: u64,
    pub reserved_micro: u64,
    #[serde(default)]
    pub stage: String,
    #[serde(default)]
    pub worker: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpendSnapshot {
    pub total_micro_usd: u64,
    pub cap_micro_usd: u64,
    pub tripped: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Req {
    Ping,
    SpendReserve { ctx: ReserveCtx },
    SpendSettle { res: Reservation, actual_micro_usd: u64 },
    SpendSnapshot,
    /// Human-only breaker reset (ETHOS §3.3): consumes .kineti/spend.reset.
    SpendResetIfRequested,
    JournalHead { branch: String },
    /// Fully-hashed records; daemon validates each prev_hash against its tail
    /// before accepting — the daemon is the serialization point of the chain.
    AppendBatch { branch: String, records: Vec<String> },
    HaltStatus,
    Shutdown,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Resp {
    Pong,
    Reserved(Reservation),
    Settled { total_micro_usd: u64 },
    Snapshot(SpendSnapshot),
    Reset(bool),
    Head(String),
    Appended,
    Halted { tripped: bool },
    Err(String),
}
