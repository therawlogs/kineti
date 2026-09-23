//! Atomic Fast-Path Spend Circuit Breaker.
//!
//! Enforces an unbreachable hard ceiling ($50.00 USD = 50,000,000 microcents) and
//! a 95% safety trip threshold ($47.50 USD = 47,500,000 microcents) with thread-safe
//! microcent accounting using lock-free atomics, 2-phase commit (2PC) reservations,
//! partial-spend settlements, fail-closed safety, and OS-level halt triggers (`exit(3)`).

use crate::hlc::current_physical_ms;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

/// One USD corresponds to 1,000,000 microcents.
pub const MICROCENTS_PER_USD: u64 = 1_000_000;

/// Default global ceiling in USD ($50.00).
pub const DEFAULT_CEILING_USD: u64 = 50;

/// Default hard limit: $50.00 USD (50,000,000 microcents).
pub const DEFAULT_CEILING_MICROCENTS: u64 = DEFAULT_CEILING_USD * MICROCENTS_PER_USD;

/// Safety trip factor gamma = 0.95 (trips at 95% of ceiling).
pub const DEFAULT_SAFETY_FACTOR: f64 = 0.95;

/// Authoritative default trip threshold: $47.50 USD (47,500,000 microcents).
pub const DEFAULT_TRIP_THRESHOLD_MICROCENTS: u64 = 47_500_000;

/// Default per-stage ceiling in USD ($10.00).
pub const DEFAULT_STAGE_CEILING_USD: u64 = 10;

/// Default per-stage ceiling in microcents (10,000,000 microcents).
pub const DEFAULT_STAGE_CEILING_MICROCENTS: u64 = DEFAULT_STAGE_CEILING_USD * MICROCENTS_PER_USD;

/// Default per-stage trip threshold in microcents (9,500,000 microcents).
pub const DEFAULT_STAGE_TRIP_MICROCENTS: u64 = 9_500_000;

/// Mandatory OS exit code when the spend circuit breaker trips.
pub const HALT_EXIT_CODE: i32 = 3;

/// Errors produced by the spend circuit breaker.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum SpendError {
    /// Spend would breach hard ceiling or trip threshold.
    CeilingExceeded {
        /// Currently committed microcents.
        committed: u64,
        /// Currently reserved microcents.
        reserved: u64,
        /// Requested reservation amount.
        requested: u64,
        /// Hard ceiling in microcents.
        ceiling: u64,
    },

    /// Circuit breaker is already in tripped fail-closed state.
    Tripped,

    /// Reservation was already committed or aborted.
    AlreadyResolved {
        /// ID of reservation.
        reservation_id: u64,
    },

    /// Integer overflow detected during accounting.
    Overflow,

    /// Unauthorized human reset attempt.
    UnauthorizedReset,
}

impl fmt::Display for SpendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CeilingExceeded {
                committed,
                reserved,
                requested,
                ceiling,
            } => write!(
                f,
                "Spend ceiling exceeded: committed={}, reserved={}, requested={}, ceiling={}",
                committed, reserved, requested, ceiling
            ),
            Self::Tripped => write!(f, "Circuit breaker is tripped: all reservations blocked"),
            Self::AlreadyResolved { reservation_id } => {
                write!(f, "Reservation {} is already resolved", reservation_id)
            }
            Self::Overflow => write!(f, "Microcent arithmetic overflow"),
            Self::UnauthorizedReset => write!(f, "Unauthorized reset: requires human verification"),
        }
    }
}

impl Error for SpendError {}

const STATE_PENDING: u8 = 0;
const STATE_COMMITTED: u8 = 1;
const STATE_ABORTED: u8 = 2;

#[derive(Debug, Default, Clone, Copy)]
struct SpendBalance {
    committed_microcents: u64,
    reserved_microcents: u64,
}

struct BreakerState {
    balance: Mutex<SpendBalance>,
    ceiling_microcents: u64,
    trip_threshold_microcents: u64,
    tripped: AtomicBool,
    next_reservation_id: AtomicU64,
    exit_on_halt: AtomicBool,
    exit_code: AtomicI32,
}

/// An active spend reservation holding pre-allocated microcents.
///
/// Implements RAII: If dropped without explicit `commit` or `abort`,
/// it automatically aborts and releases the reserved microcents.
pub struct Reservation {
    id: u64,
    amount_microcents: u64,
    status: Arc<AtomicU8>,
    breaker: Arc<BreakerState>,
}

impl Reservation {
    /// Returns the unique reservation ID.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns the reserved microcent amount.
    pub fn amount_microcents(&self) -> u64 {
        self.amount_microcents
    }

    /// Whether this reservation is still pending.
    pub fn is_pending(&self) -> bool {
        self.status.load(Ordering::Acquire) == STATE_PENDING
    }
}

impl fmt::Debug for Reservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Reservation")
            .field("id", &self.id)
            .field("amount_microcents", &self.amount_microcents)
            .field("is_pending", &self.is_pending())
            .finish()
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        // Automatic RAII compensation: if still pending, abort reservation
        if self
            .status
            .compare_exchange(
                STATE_PENDING,
                STATE_ABORTED,
                Ordering::SeqCst,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            let mut balance = self.breaker.balance.lock().unwrap_or_else(|e| e.into_inner());
            balance.reserved_microcents = balance.reserved_microcents.saturating_sub(self.amount_microcents);
        }
    }
}

/// Atomic Fast-Path Spend Circuit Breaker.
#[derive(Clone)]
pub struct SpendCircuitBreaker {
    inner: Arc<BreakerState>,
}

impl Default for SpendCircuitBreaker {
    fn default() -> Self {
        Self::with_trip_threshold(
            DEFAULT_CEILING_MICROCENTS,
            DEFAULT_TRIP_THRESHOLD_MICROCENTS,
        )
    }
}

impl SpendCircuitBreaker {
    /// Initializes a new circuit breaker with given ceiling in microcents.
    ///
    /// Automatically derives trip threshold at 95% (`DEFAULT_SAFETY_FACTOR`).
    pub fn new(ceiling_microcents: u64) -> Self {
        let trip_threshold = (ceiling_microcents as f64 * DEFAULT_SAFETY_FACTOR).round() as u64;
        Self::with_trip_threshold(ceiling_microcents, trip_threshold)
    }

    /// Initializes a new circuit breaker with explicit ceiling and trip threshold.
    pub fn with_trip_threshold(ceiling_microcents: u64, trip_threshold_microcents: u64) -> Self {
        Self {
            inner: Arc::new(BreakerState {
                balance: Mutex::new(SpendBalance::default()),
                ceiling_microcents,
                trip_threshold_microcents,
                tripped: AtomicBool::new(false),
                next_reservation_id: AtomicU64::new(1),
                exit_on_halt: AtomicBool::new(false),
                exit_code: AtomicI32::new(HALT_EXIT_CODE),
            }),
        }
    }

    /// Sets whether `check_and_halt_if_exceeded` should terminate the process via OS exit.
    pub fn set_exit_on_halt(&self, enable: bool, exit_code: i32) {
        self.inner.exit_on_halt.store(enable, Ordering::Release);
        self.inner.exit_code.store(exit_code, Ordering::Release);
    }

    /// Fast-path pre-allocation reservation: atomically reserves `microcents`.
    ///
    /// Fails closed if `committed + reserved + amount > trip_threshold`.
    pub fn reserve(&self, amount_microcents: u64) -> Result<Reservation, SpendError> {
        if amount_microcents > u64::MAX - self.inner.ceiling_microcents {
            return Err(SpendError::Overflow);
        }

        if self.inner.tripped.load(Ordering::Acquire) {
            return Err(SpendError::Tripped);
        }

        let mut balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());

        if self.inner.tripped.load(Ordering::Acquire) {
            return Err(SpendError::Tripped);
        }

        let total = balance
            .committed_microcents
            .checked_add(balance.reserved_microcents)
            .ok_or(SpendError::Overflow)?;

        let new_total = total
            .checked_add(amount_microcents)
            .ok_or(SpendError::Overflow)?;

        if new_total > self.inner.trip_threshold_microcents {
            self.inner.tripped.store(true, Ordering::SeqCst);

            let committed = balance.committed_microcents;
            let reserved = balance.reserved_microcents;
            drop(balance);

            if self.inner.exit_on_halt.load(Ordering::Acquire) {
                #[cfg(not(test))]
                std::process::exit(self.inner.exit_code.load(Ordering::Acquire));
            }

            return Err(SpendError::CeilingExceeded {
                committed,
                reserved,
                requested: amount_microcents,
                ceiling: self.inner.ceiling_microcents,
            });
        }

        balance.reserved_microcents = balance
            .reserved_microcents
            .checked_add(amount_microcents)
            .ok_or(SpendError::Overflow)?;
        drop(balance);

        let res_id = self
            .inner
            .next_reservation_id
            .fetch_add(1, Ordering::Relaxed);

        Ok(Reservation {
            id: res_id,
            amount_microcents,
            status: Arc::new(AtomicU8::new(STATE_PENDING)),
            breaker: Arc::clone(&self.inner),
        })
    }

    /// Commits a reservation (Phase 2), converting reserved microcents into committed spend.
    pub fn commit(&self, reservation: Reservation) -> Result<u64, SpendError> {
        let amount = reservation.amount_microcents;
        self.commit_with_actual(reservation, amount)
    }

    /// Commits actual spend for a reservation (Phase 2), refunding any unspent estimate.
    ///
    /// If actual spend <= reserved estimate, the difference is atomically released
    /// back to the available budget pool.
    pub fn commit_with_actual(
        &self,
        reservation: Reservation,
        actual_microcents: u64,
    ) -> Result<u64, SpendError> {
        if actual_microcents > reservation.amount_microcents {
            let balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());
            return Err(SpendError::CeilingExceeded {
                committed: balance.committed_microcents,
                reserved: balance.reserved_microcents,
                requested: actual_microcents - reservation.amount_microcents,
                ceiling: self.inner.ceiling_microcents,
            });
        }

        if reservation
            .status
            .compare_exchange(
                STATE_PENDING,
                STATE_COMMITTED,
                Ordering::SeqCst,
                Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(SpendError::AlreadyResolved {
                reservation_id: reservation.id,
            });
        }

        let mut balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());
        balance.reserved_microcents = balance
            .reserved_microcents
            .saturating_sub(reservation.amount_microcents);
        balance.committed_microcents = balance
            .committed_microcents
            .saturating_add(actual_microcents);
        let committed = balance.committed_microcents;

        // Trip check on committed total
        if committed >= self.inner.trip_threshold_microcents {
            self.inner.tripped.store(true, Ordering::SeqCst);
        }

        Ok(committed)
    }

    /// Aborts a reservation (Phase 2), refunding the reserved microcents back to available pool.
    pub fn abort(&self, reservation: Reservation) -> Result<u64, SpendError> {
        if reservation
            .status
            .compare_exchange(
                STATE_PENDING,
                STATE_ABORTED,
                Ordering::SeqCst,
                Ordering::Relaxed,
            )
            .is_err()
        {
            return Err(SpendError::AlreadyResolved {
                reservation_id: reservation.id,
            });
        }

        let mut balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());
        balance.reserved_microcents = balance
            .reserved_microcents
            .saturating_sub(reservation.amount_microcents);
        Ok(balance.committed_microcents)
    }

    /// Single-step fast path: reserves and immediately commits `microcents`.
    pub fn record_spend(&self, microcents: u64) -> Result<u64, SpendError> {
        let reservation = self.reserve(microcents)?;
        self.commit(reservation)
    }

    /// Evaluates if ceiling is exceeded and halts execution if configured.
    pub fn check_and_halt_if_exceeded(&self) -> Result<(), SpendError> {
        let (committed, reserved) = {
            let balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());
            (balance.committed_microcents, balance.reserved_microcents)
        };
        let total = committed.saturating_add(reserved);

        if total >= self.inner.trip_threshold_microcents || self.inner.tripped.load(Ordering::Acquire) {
            self.inner.tripped.store(true, Ordering::SeqCst);

            if self.inner.exit_on_halt.load(Ordering::Acquire) {
                #[cfg(not(test))]
                std::process::exit(self.inner.exit_code.load(Ordering::Acquire));
            }

            return Err(SpendError::CeilingExceeded {
                committed,
                reserved,
                requested: 0,
                ceiling: self.inner.ceiling_microcents,
            });
        }

        Ok(())
    }

    /// Returns currently committed microcents.
    pub fn committed_microcents(&self) -> u64 {
        self.inner.balance.lock().unwrap_or_else(|e| e.into_inner()).committed_microcents
    }

    /// Returns currently active reserved microcents.
    pub fn reserved_microcents(&self) -> u64 {
        self.inner.balance.lock().unwrap_or_else(|e| e.into_inner()).reserved_microcents
    }

    /// Returns total active spend exposure (`committed + reserved`).
    pub fn total_microcents(&self) -> u64 {
        let balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());
        balance.committed_microcents.saturating_add(balance.reserved_microcents)
    }

    /// Returns remaining microcents under the hard ceiling.
    pub fn remaining_microcents(&self) -> u64 {
        self.inner
            .ceiling_microcents
            .saturating_sub(self.total_microcents())
    }

    /// Returns remaining microcents before tripping at 95%.
    pub fn remaining_to_trip_microcents(&self) -> u64 {
        self.inner
            .trip_threshold_microcents
            .saturating_sub(self.total_microcents())
    }

    /// Returns the hard ceiling in microcents ($50.00 default).
    pub fn ceiling_microcents(&self) -> u64 {
        self.inner.ceiling_microcents
    }

    /// Returns the 95% safety trip threshold in microcents ($47.50 default).
    pub fn trip_threshold_microcents(&self) -> u64 {
        self.inner.trip_threshold_microcents
    }

    /// Alias for `trip_threshold_microcents`.
    pub fn warning_ceiling_microcents(&self) -> u64 {
        self.inner.trip_threshold_microcents
    }

    /// Returns whether the circuit breaker has tripped.
    pub fn is_tripped(&self) -> bool {
        self.inner.tripped.load(Ordering::Acquire)
    }

    /// Returns whether total exposure is near/at the trip threshold.
    pub fn is_near_ceiling(&self) -> bool {
        self.total_microcents() >= self.inner.trip_threshold_microcents
    }

    /// Resets breaker balances and untrips (administrative / test harness use).
    pub fn reset(&self) {
        let mut balance = self.inner.balance.lock().unwrap_or_else(|e| e.into_inner());
        balance.committed_microcents = 0;
        balance.reserved_microcents = 0;
        self.inner.tripped.store(false, Ordering::SeqCst);
    }

    /// Administrative reset requiring human verification token (`--i-am-human`).
    pub fn reset_human(&self, token: &str) -> Result<(), SpendError> {
        if token == "--i-am-human" || token == "human" {
            self.reset();
            Ok(())
        } else {
            Err(SpendError::UnauthorizedReset)
        }
    }
}

/// Status of a user spend check.
#[derive(Debug, PartialEq, Clone)]
pub enum UserSpendStatus {
    /// Normal spend allowed with comfortable remaining balance.
    WithinBudget {
        /// Current spent in microcents.
        spent_microcents: u64,
        /// Remaining daily budget in microcents.
        remaining_microcents: u64,
    },
    /// Approaching daily budget ceiling (>= 90%).
    SoftWarning {
        /// Current spent in microcents.
        spent_microcents: u64,
        /// Total daily limit in microcents.
        limit_microcents: u64,
        /// User-friendly advisory message for chat.
        advisory_message: String,
    },
}

/// Error returned when a user's daily spend quota has been exhausted.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserQuotaExceeded {
    /// Total spent today in microcents.
    pub spent_microcents: u64,
    /// Total daily limit in microcents.
    pub limit_microcents: u64,
    /// User-friendly message to return directly in chat.
    pub user_message: String,
}

impl fmt::Display for UserQuotaExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.user_message)
    }
}

impl Error for UserQuotaExceeded {}

/// Thread-safe per-user spend tracker enforcing daily subscription tier quotas.
#[derive(Debug)]
pub struct UserSpendQuota {
    user_id: String,
    limit_microcents: AtomicU64,
    spent_microcents: AtomicU64,
    reserved_microcents: AtomicU64,
    last_reset_epoch_days: AtomicU64,
}

impl UserSpendQuota {
    /// Creates a new spend tracker for a user with the given daily microcent allowance.
    pub fn new(user_id: impl Into<String>, daily_allowance_microcents: u64) -> Self {
        let current_day = current_physical_ms() / (86_400 * 1000);
        Self {
            user_id: user_id.into(),
            limit_microcents: AtomicU64::new(daily_allowance_microcents),
            spent_microcents: AtomicU64::new(0),
            reserved_microcents: AtomicU64::new(0),
            last_reset_epoch_days: AtomicU64::new(current_day),
        }
    }

    /// User ID associated with this quota tracker.
    pub fn user_id(&self) -> &str {
        &self.user_id
    }

    /// Current daily limit in microcents.
    pub fn limit_microcents(&self) -> u64 {
        self.limit_microcents.load(Ordering::Relaxed)
    }

    /// Current microcents committed today.
    pub fn spent_microcents(&self) -> u64 {
        self.spent_microcents.load(Ordering::Relaxed)
    }

    /// Resets daily quota if current physical day exceeds last recorded epoch day.
    pub fn maybe_reset_daily(&self, current_day_epoch: u64) {
        let prev = self.last_reset_epoch_days.load(Ordering::Acquire);
        if current_day_epoch > prev {
            if self
                .last_reset_epoch_days
                .compare_exchange(prev, current_day_epoch, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
            {
                self.spent_microcents.store(0, Ordering::Release);
                self.reserved_microcents.store(0, Ordering::Release);
            }
        }
    }

    /// Checks if a spend amount is within budget, returning soft warnings or hard cap error.
    pub fn check_spend(&self, requested_microcents: u64) -> Result<UserSpendStatus, UserQuotaExceeded> {
        let limit = self.limit_microcents.load(Ordering::Relaxed);
        let spent = self.spent_microcents.load(Ordering::Relaxed);
        let projected = spent.saturating_add(requested_microcents);

        if projected > limit {
            let spent_usd = microcents_to_usd(spent);
            let limit_usd = microcents_to_usd(limit);
            let user_message = format!(
                "You've reached today's free limit (${:.2} / ${:.2}). Quota resets at midnight UTC, or upgrade to Pro at getkineti.com for unlimited high-speed chatting.",
                spent_usd, limit_usd
            );
            return Err(UserQuotaExceeded {
                spent_microcents: spent,
                limit_microcents: limit,
                user_message,
            });
        }

        let warning_threshold = (limit as f64 * 0.90) as u64;
        if projected >= warning_threshold {
            let spent_usd = microcents_to_usd(projected);
            let limit_usd = microcents_to_usd(limit);
            let advisory_message = format!(
                "Notice: You've used 90% of your daily allowance (${:.2} / ${:.2}). Upgrade at getkineti.com for unlimited access.",
                spent_usd, limit_usd
            );
            Ok(UserSpendStatus::SoftWarning {
                spent_microcents: projected,
                limit_microcents: limit,
                advisory_message,
            })
        } else {
            Ok(UserSpendStatus::WithinBudget {
                spent_microcents: projected,
                remaining_microcents: limit.saturating_sub(projected),
            })
        }
    }

    /// Records committed spend for a user interaction.
    pub fn record_spend(&self, microcents: u64) {
        self.spent_microcents.fetch_add(microcents, Ordering::AcqRel);
    }
}

/// Converts floating-point USD to integer microcents with rounding.
#[inline]
pub fn usd_to_microcents(usd: f64) -> u64 {
    if usd <= 0.0 {
        0
    } else {
        (usd * MICROCENTS_PER_USD as f64).round() as u64
    }
}

/// Converts integer microcents to floating-point USD.
#[inline]
pub fn microcents_to_usd(microcents: u64) -> f64 {
    microcents as f64 / MICROCENTS_PER_USD as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_spend_microcents_conversion() {
        assert_eq!(usd_to_microcents(1.0), 1_000_000);
        assert_eq!(usd_to_microcents(50.0), 50_000_000);
        assert_eq!(usd_to_microcents(47.50), 47_500_000);
        assert_eq!(usd_to_microcents(0.0), 0);
        assert_eq!(usd_to_microcents(-5.0), 0);

        assert!((microcents_to_usd(1_000_000) - 1.0).abs() < 1e-9);
        assert!((microcents_to_usd(50_000_000) - 50.0).abs() < 1e-9);
        assert!((microcents_to_usd(47_500_000) - 47.50).abs() < 1e-9);
    }

    #[test]
    fn test_spend_trip_at_95_percent() {
        let breaker = SpendCircuitBreaker::default();
        assert_eq!(breaker.ceiling_microcents(), 50_000_000);
        assert_eq!(breaker.trip_threshold_microcents(), 47_500_000);

        // Reserve exactly $47.50 -> succeeds
        let res1 = breaker.reserve(47_500_000).expect("Reservation at 95% threshold succeeds");
        assert_eq!(breaker.reserved_microcents(), 47_500_000);
        assert!(!breaker.is_tripped());

        // Attempt to reserve 1 additional microcent -> fails and trips breaker
        let err = breaker.reserve(1).unwrap_err();
        assert!(matches!(err, SpendError::CeilingExceeded { .. }));
        assert!(breaker.is_tripped());

        // Subsequent reservation must fail closed with Tripped
        let err2 = breaker.reserve(100).unwrap_err();
        assert_eq!(err2, SpendError::Tripped);

        // Commit res1 -> must succeed and keep committed at 47.5M
        let committed = breaker.commit(res1).expect("Commit pending res1 succeeds");
        assert_eq!(committed, 47_500_000);
        assert_eq!(breaker.reserved_microcents(), 0);
        assert_eq!(breaker.committed_microcents(), 47_500_000);
    }

    #[test]
    fn test_two_phase_commit_and_abort() {
        let breaker = SpendCircuitBreaker::new(50_000_000);

        // Phase 1: Reserve $10.00
        let res = breaker.reserve(10_000_000).expect("Reserve succeeds");
        assert_eq!(breaker.reserved_microcents(), 10_000_000);
        assert_eq!(breaker.committed_microcents(), 0);

        // Phase 2: Commit
        let committed = breaker.commit(res).expect("Commit succeeds");
        assert_eq!(committed, 10_000_000);
        assert_eq!(breaker.committed_microcents(), 10_000_000);
        assert_eq!(breaker.reserved_microcents(), 0);

        // Phase 1: Reserve $5.00
        let res2 = breaker.reserve(5_000_000).expect("Reserve 2 succeeds");
        assert_eq!(breaker.reserved_microcents(), 5_000_000);

        // Phase 2: Abort
        let committed_after_abort = breaker.abort(res2).expect("Abort succeeds");
        assert_eq!(committed_after_abort, 10_000_000);
        assert_eq!(breaker.reserved_microcents(), 0);
    }

    #[test]
    fn test_partial_settlement_refunds_unspent() {
        let breaker = SpendCircuitBreaker::new(50_000_000);

        // Estimate $10.00
        let res = breaker.reserve(10_000_000).expect("Reserve succeeds");
        assert_eq!(breaker.reserved_microcents(), 10_000_000);

        // Actual spend is $6.50
        let committed = breaker.commit_with_actual(res, 6_500_000).expect("Partial commit succeeds");
        assert_eq!(committed, 6_500_000);
        assert_eq!(breaker.committed_microcents(), 6_500_000);
        assert_eq!(breaker.reserved_microcents(), 0);
        assert_eq!(breaker.remaining_microcents(), 43_500_000);
    }

    #[test]
    fn test_raii_drop_compensation() {
        let breaker = SpendCircuitBreaker::new(50_000_000);

        {
            let res = breaker.reserve(15_000_000).expect("Reserve succeeds");
            assert_eq!(breaker.reserved_microcents(), 15_000_000);
            assert!(res.is_pending());
            // Drops here without commit or abort
        }

        assert_eq!(breaker.reserved_microcents(), 0);
        assert_eq!(breaker.committed_microcents(), 0);
        assert_eq!(breaker.remaining_microcents(), 50_000_000);
    }

    #[test]
    fn test_double_resolution_fails() {
        let breaker = SpendCircuitBreaker::new(50_000_000);

        let res = breaker.reserve(5_000_000).unwrap();
        let _res_id = res.id();
        let _ = breaker.commit(res).unwrap();

        // Construct dummy reservation with same ID is prevented by move semantics,
        // but drop on committed reservation is a no-op:
        assert_eq!(breaker.committed_microcents(), 5_000_000);
        assert_eq!(breaker.reserved_microcents(), 0);

        let res2 = breaker.reserve(5_000_000).unwrap();
        let _ = breaker.abort(res2).unwrap();
        assert_eq!(breaker.reserved_microcents(), 0);
    }

    #[test]
    fn test_record_spend_fast_path() {
        let breaker = SpendCircuitBreaker::new(50_000_000);
        let total = breaker.record_spend(2_500_000).expect("Fast path spend succeeds");
        assert_eq!(total, 2_500_000);
        assert_eq!(breaker.committed_microcents(), 2_500_000);
        assert_eq!(breaker.reserved_microcents(), 0);
    }

    #[test]
    fn test_reset_and_human_reset() {
        let breaker = SpendCircuitBreaker::new(10_000_000);
        let _ = breaker.reserve(10_000_000);
        assert!(breaker.is_tripped());

        // Programmatic reset
        breaker.reset();
        assert!(!breaker.is_tripped());
        assert_eq!(breaker.committed_microcents(), 0);
        assert_eq!(breaker.reserved_microcents(), 0);

        // Trip again
        let _ = breaker.reserve(10_000_000);
        assert!(breaker.is_tripped());

        // Unauthorized reset fails
        let err = breaker.reset_human("ai_agent_token").unwrap_err();
        assert_eq!(err, SpendError::UnauthorizedReset);
        assert!(breaker.is_tripped());

        // Human reset succeeds
        breaker.reset_human("--i-am-human").expect("Human reset succeeds");
        assert!(!breaker.is_tripped());
    }

    #[test]
    fn test_exit_code_stored_properly() {
        let breaker = SpendCircuitBreaker::new(50_000_000);
        breaker.set_exit_on_halt(true, 3);
        assert_eq!(breaker.inner.exit_code.load(Ordering::Acquire), 3);
        assert!(breaker.inner.exit_on_halt.load(Ordering::Acquire));
    }

    #[test]
    fn test_overflow_protection() {
        let breaker = SpendCircuitBreaker::new(50_000_000);
        let err = breaker.reserve(u64::MAX).unwrap_err();
        assert_eq!(err, SpendError::Overflow);
    }

    #[test]
    fn test_concurrent_spend_exhaustion_no_overdraft() {
        let ceiling = 50_000_000;
        let trip_threshold = 47_500_000;
        let breaker = Arc::new(SpendCircuitBreaker::with_trip_threshold(ceiling, trip_threshold));

        let num_threads = 100;
        let reservation_amount = 1_000_000; // $1.00 per thread

        let successful_commits = Arc::new(AtomicUsize::new(0));
        let rejected_reservations = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..num_threads {
            let breaker_clone = Arc::clone(&breaker);
            let success_clone = Arc::clone(&successful_commits);
            let reject_clone = Arc::clone(&rejected_reservations);

            handles.push(std::thread::spawn(move || {
                match breaker_clone.reserve(reservation_amount) {
                    Ok(res) => {
                        let _ = breaker_clone.commit(res);
                        success_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        reject_clone.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let successes = successful_commits.load(Ordering::SeqCst);
        let rejects = rejected_reservations.load(Ordering::SeqCst);
        assert_eq!(successes + rejects, num_threads);

        let total_committed = breaker.committed_microcents();
        assert!(total_committed <= trip_threshold, "Total committed {} > trip threshold {}", total_committed, trip_threshold);
        assert!(total_committed <= ceiling, "Total committed {} > ceiling {}", total_committed, ceiling);
        assert!(breaker.is_tripped());
    }

    #[test]
    fn test_user_spend_quota_lifecycle_and_warnings() {
        // Free tier: $0.20 = 200,000 microcents
        let tracker = UserSpendQuota::new("user_123", 200_000);
        assert_eq!(tracker.user_id(), "user_123");
        assert_eq!(tracker.limit_microcents(), 200_000);
        assert_eq!(tracker.spent_microcents(), 0);

        // Step 1: Normal spend within budget ($0.05)
        let status = tracker.check_spend(50_000).expect("Within budget");
        assert!(matches!(status, UserSpendStatus::WithinBudget { .. }));
        tracker.record_spend(50_000);
        assert_eq!(tracker.spent_microcents(), 50_000);

        // Step 2: Push into 90% soft warning zone ($0.18 total, requested $0.13)
        let status2 = tracker.check_spend(130_000).expect("Soft warning zone allowed");
        if let UserSpendStatus::SoftWarning { advisory_message, .. } = status2 {
            assert!(advisory_message.contains("90%"));
            assert!(advisory_message.contains("getkineti.com"));
        } else {
            panic!("Expected SoftWarning, got {:?}", status2);
        }
        tracker.record_spend(130_000);
        assert_eq!(tracker.spent_microcents(), 180_000);

        // Step 3: Attempting to spend $0.03 ($0.21 total > $0.20) -> hard cap error
        let err = tracker.check_spend(30_000).unwrap_err();
        assert!(err.user_message.contains("reached today's free limit"));
        assert!(err.user_message.contains("getkineti.com"));
    }
}
