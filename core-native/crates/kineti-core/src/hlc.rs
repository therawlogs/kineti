//! Hybrid Logical Clock (HLC) implementation.
//!
//! Tracks physical timestamp (milliseconds since Unix epoch) and a logical counter (l, c),
//! guaranteeing strict causal monotonicity under physical clock skew and drift.
//! Based on Kulkarni et al. (2014) "Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases".

use std::cmp::Ordering;
use std::error::Error;
use std::fmt;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current physical time in milliseconds since the Unix epoch.
pub fn current_physical_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Errors emitted by the Hybrid Logical Clock.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum HlcError {
    /// Physical clock drift exceeds maximum allowed tolerance.
    ClockSkewExceeded {
        /// Local system physical timestamp in ms.
        physical_ms: u64,
        /// Clock physical component in ms.
        hlc_physical_ms: u64,
        /// Maximum allowable skew in ms.
        max_skew_ms: u64,
    },

    /// Logical counter overflow for a single physical millisecond.
    CounterOverflow {
        /// The physical millisecond where counter overflowed.
        physical_ms: u64,
    },
}

impl fmt::Display for HlcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ClockSkewExceeded {
                physical_ms,
                hlc_physical_ms,
                max_skew_ms,
            } => write!(
                f,
                "Physical clock skew exceeded: local={}ms, hlc={}ms, max_skew={}ms",
                physical_ms, hlc_physical_ms, max_skew_ms
            ),
            Self::CounterOverflow { physical_ms } => {
                write!(f, "HLC logical counter overflow at physical time {}ms", physical_ms)
            }
        }
    }
}

impl Error for HlcError {}

/// An immutable Hybrid Logical Clock timestamp (l, c, node).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct HlcTimestamp {
    /// Physical component: maximum observed physical time in milliseconds.
    pub physical_ms: u64,
    /// Logical component: monotonic sequence counter for events within same physical millisecond.
    pub logical: u32,
    /// Unique identifier of the originating node/agent to break ties.
    pub node_id: u32,
}

impl HlcTimestamp {
    /// Creates a new HLC timestamp.
    pub const fn new(physical_ms: u64, logical: u32, node_id: u32) -> Self {
        Self {
            physical_ms,
            logical,
            node_id,
        }
    }

    /// Represents the absolute minimum timestamp.
    pub const fn min_value() -> Self {
        Self {
            physical_ms: 0,
            logical: 0,
            node_id: 0,
        }
    }
}

impl PartialOrd for HlcTimestamp {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HlcTimestamp {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.physical_ms
            .cmp(&other.physical_ms)
            .then_with(|| self.logical.cmp(&other.logical))
            .then_with(|| self.node_id.cmp(&other.node_id))
    }
}

impl fmt::Display for HlcTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}#{}", self.physical_ms, self.logical, self.node_id)
    }
}

impl fmt::Debug for HlcTimestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HlcTimestamp({}.{}#{})", self.physical_ms, self.logical, self.node_id)
    }
}

#[derive(Debug, Clone, Copy)]
struct HlcState {
    latest_physical_ms: u64,
    logical: u32,
}

/// A thread-safe Hybrid Logical Clock generator.
pub struct HybridLogicalClock {
    node_id: u32,
    max_skew_ms: u64,
    state: Mutex<HlcState>,
}

impl HybridLogicalClock {
    /// Default clock skew tolerance: 60,000 milliseconds (1 minute).
    pub const DEFAULT_MAX_SKEW_MS: u64 = 60_000;

    /// Creates a new HLC instance for node `node_id`.
    pub fn new(node_id: u32) -> Self {
        Self::with_max_skew(node_id, Self::DEFAULT_MAX_SKEW_MS)
    }

    /// Creates a new HLC with custom maximum clock skew tolerance.
    pub fn with_max_skew(node_id: u32, max_skew_ms: u64) -> Self {
        let now_ms = current_physical_ms();
        Self {
            node_id,
            max_skew_ms,
            state: Mutex::new(HlcState {
                latest_physical_ms: now_ms,
                logical: 0,
            }),
        }
    }

    /// Advances the clock for a local event and returns the new monotonic timestamp.
    pub fn now(&self) -> Result<HlcTimestamp, HlcError> {
        let pt = current_physical_ms();
        let mut state = self.state.lock().expect("HLC mutex poisoned");

        if pt > state.latest_physical_ms {
            state.latest_physical_ms = pt;
            state.logical = 0;
        } else {
            // Physical time hasn't advanced or system clock drifted backwards
            let skew = state.latest_physical_ms.saturating_sub(pt);
            if skew > self.max_skew_ms {
                return Err(HlcError::ClockSkewExceeded {
                    physical_ms: pt,
                    hlc_physical_ms: state.latest_physical_ms,
                    max_skew_ms: self.max_skew_ms,
                });
            }

            state.logical = state.logical.checked_add(1).ok_or(HlcError::CounterOverflow {
                physical_ms: state.latest_physical_ms,
            })?;
        }

        Ok(HlcTimestamp {
            physical_ms: state.latest_physical_ms,
            logical: state.logical,
            node_id: self.node_id,
        })
    }

    /// Updates local clock based on a received remote timestamp.
    pub fn update(&self, remote: &HlcTimestamp) -> Result<HlcTimestamp, HlcError> {
        let pt = current_physical_ms();
        let mut state = self.state.lock().expect("HLC mutex poisoned");

        let max_phys = pt.max(state.latest_physical_ms).max(remote.physical_ms);

        let skew = max_phys.saturating_sub(pt);
        if skew > self.max_skew_ms {
            return Err(HlcError::ClockSkewExceeded {
                physical_ms: pt,
                hlc_physical_ms: max_phys,
                max_skew_ms: self.max_skew_ms,
            });
        }

        if max_phys == state.latest_physical_ms && max_phys == remote.physical_ms {
            let max_logical = state.logical.max(remote.logical);
            state.logical = max_logical.checked_add(1).ok_or(HlcError::CounterOverflow {
                physical_ms: max_phys,
            })?;
        } else if max_phys == state.latest_physical_ms {
            state.logical = state.logical.checked_add(1).ok_or(HlcError::CounterOverflow {
                physical_ms: max_phys,
            })?;
        } else if max_phys == remote.physical_ms {
            state.logical = remote.logical.checked_add(1).ok_or(HlcError::CounterOverflow {
                physical_ms: max_phys,
            })?;
        } else {
            state.logical = 0;
        }

        state.latest_physical_ms = max_phys;

        Ok(HlcTimestamp {
            physical_ms: max_phys,
            logical: state.logical,
            node_id: self.node_id,
        })
    }

    /// Peeks at the current HLC timestamp without advancing state.
    pub fn peek(&self) -> HlcTimestamp {
        let state = self.state.lock().expect("HLC mutex poisoned");
        HlcTimestamp {
            physical_ms: state.latest_physical_ms,
            logical: state.logical,
            node_id: self.node_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hlc_monotonicity() {
        let hlc = HybridLogicalClock::new(1);
        let mut prev = hlc.now().unwrap();
        for _ in 0..1000 {
            let next = hlc.now().unwrap();
            assert!(next > prev, "HLC must be strictly monotonic: prev={}, next={}", prev, next);
            prev = next;
        }
    }

    #[test]
    fn test_hlc_logical_advancement_same_ms() {
        let hlc = HybridLogicalClock::new(1);
        let ts1 = hlc.now().unwrap();
        let ts2 = hlc.now().unwrap();
        if ts1.physical_ms == ts2.physical_ms {
            assert_eq!(ts2.logical, ts1.logical + 1);
        } else {
            assert!(ts2.physical_ms > ts1.physical_ms);
        }
    }

    #[test]
    fn test_hlc_update_advances_past_remote() {
        let hlc_local = HybridLogicalClock::new(1);
        let remote_ts = HlcTimestamp::new(current_physical_ms() + 500, 10, 2);

        let updated = hlc_local.update(&remote_ts).unwrap();
        assert!(updated > remote_ts);
        assert_eq!(updated.physical_ms, remote_ts.physical_ms);
        assert_eq!(updated.logical, 11);
        assert_eq!(updated.node_id, 1);
    }

    #[test]
    fn test_hlc_clock_skew_exceeded() {
        let hlc = HybridLogicalClock::with_max_skew(1, 1000);
        let far_future = HlcTimestamp::new(current_physical_ms() + 5000, 0, 2);

        let err = hlc.update(&far_future).unwrap_err();
        assert!(matches!(err, HlcError::ClockSkewExceeded { .. }));
    }
}
