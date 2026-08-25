//! The atomic spend pool — the ONE implementation of breaker math (ETHOS §3).
//!
//! Reservation protocol: `reserve(est)` optimistically adds the estimate and
//! denies when ANY projection crosses its ceiling; `settle(actual)` corrects
//! by `actual - est` and TRIPS the breaker if settled reality crosses any
//! ceiling. A trip halts everything immediately (§3.1); only a human-created
//! `.kineti/spend.reset` clears it (§3.3). A crash between reserve and settle
//! can only OVERCOUNT (safe direction).
//!
//! All amounts are integer micro-USD. Ledgers sit behind one mutex — reserve
//! and settle fire once per model call, so contention is a non-issue and the
//! math stays trivially exact.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use super::dto::{Reservation, ReserveCtx};

pub const BREAKER_MSG: &str =
    "SPEND BREAKER: cap reached. A human must create .kineti/spend.reset to resume.";

#[derive(Clone, Debug)]
pub struct Ceilings {
    pub global_micro: u64,
    /// None = unbounded. `limits.per_stage_usd`, 0/negative → None.
    pub stage_micro: Option<u64>,
    /// None = unbounded until swarm mode sets it.
    pub worker_micro: Option<u64>,
}

impl Ceilings {
    pub fn global_only(global_micro: u64) -> Self {
        Ceilings { global_micro, stage_micro: None, worker_micro: None }
    }
}

#[derive(Default)]
struct Ledgers {
    total: u64,
    stages: HashMap<String, u64>,
    workers: HashMap<String, u64>,
    tripped: bool,
}

pub struct Pool {
    inner: Mutex<Ledgers>,
    pub ceilings: Ceilings,
    seq: AtomicU64,
}

fn usd(micro: u64) -> String {
    format!("${:.2}", micro as f64 / 1_000_000.0)
}

fn breaker(scope: &str, projected: u64, cap: u64) -> String {
    format!(
        "SPEND BREAKER ({scope}: projected {projected} > cap {cap}). \
         A human must create .kineti/spend.reset to resume.",
        scope = scope,
        projected = usd(projected),
        cap = usd(cap),
    )
}

impl Pool {
    pub fn new(ceilings: Ceilings) -> Self {
        Pool {
            inner: Mutex::new(Ledgers::default()),
            ceilings,
            seq: AtomicU64::new(1),
        }
    }

    /// Seed from persisted state (daemon restart / direct construction).
    pub fn seed_total(&self, micro: u64) {
        self.inner.lock().unwrap().total = micro;
    }

    pub fn total(&self) -> u64 {
        self.inner.lock().unwrap().total
    }

    pub fn is_tripped(&self) -> bool {
        self.inner.lock().unwrap().tripped
    }

    fn trip_locked(l: &mut Ledgers) {
        l.tripped = true;
    }

    /// Optimistic hold. Denied holds leave every ledger untouched.
    /// Zero estimates pass through untouched (free-tier providers).
    pub fn reserve(&self, ctx: &ReserveCtx) -> Result<Reservation, String> {
        let id = self.seq.fetch_add(1, Ordering::Relaxed);
        if ctx.est_micro_usd == 0 {
            return Ok(Reservation {
                id,
                reserved_micro: 0,
                stage: ctx.stage.clone(),
                worker: ctx.worker.clone(),
            });
        }
        let mut l = self.inner.lock().unwrap();
        if l.tripped {
            return Err(BREAKER_MSG.into());
        }

        let projected_global = l.total as u128 + ctx.est_micro_usd as u128;
        if projected_global > self.ceilings.global_micro as u128 {
            return Err(breaker("global", l.total + ctx.est_micro_usd, self.ceilings.global_micro));
        }
        if let Some(cap) = self.ceilings.stage_micro {
            if !ctx.stage.is_empty() {
                let cur = *l.stages.get(&ctx.stage).unwrap_or(&0);
                let projected = cur.saturating_add(ctx.est_micro_usd);
                if projected > cap {
                    return Err(breaker(&format!("per-stage '{}'", ctx.stage), projected, cap));
                }
            }
        }
        if let Some(cap) = self.ceilings.worker_micro {
            if !ctx.worker.is_empty() {
                let cur = *l.workers.get(&ctx.worker).unwrap_or(&0);
                let projected = cur.saturating_add(ctx.est_micro_usd);
                if projected > cap {
                    return Err(breaker(&format!("per-worker '{}'", ctx.worker), projected, cap));
                }
            }
        }

        // grant: commit to global + both scoped ledgers
        l.total += ctx.est_micro_usd;
        if !ctx.stage.is_empty() {
            *l.stages.entry(ctx.stage.clone()).or_insert(0) += ctx.est_micro_usd;
        }
        if !ctx.worker.is_empty() {
            *l.workers.entry(ctx.worker.clone()).or_insert(0) += ctx.est_micro_usd;
        }
        Ok(Reservation {
            id,
            reserved_micro: ctx.est_micro_usd,
            stage: ctx.stage.clone(),
            worker: ctx.worker.clone(),
        })
    }

    /// Correct the hold with actual cost. Trips the WHOLE pool when settled
    /// reality crosses any ceiling ("halts everything immediately", §3.1).
    /// Returns the post-settle global total.
    pub fn settle(&self, res: &Reservation, actual_micro: u64) -> u64 {
        let mut l = self.inner.lock().unwrap();
        let delta = actual_micro as i128 - res.reserved_micro as i128;

        let apply = |v: &mut u64, d: i128| {
            if d >= 0 {
                *v = v.saturating_add(d as u64);
            } else {
                *v = v.saturating_sub((-d) as u64);
            }
        };
        apply(&mut l.total, delta);
        if !res.stage.is_empty() {
            let e = l.stages.entry(res.stage.clone()).or_insert(0);
            apply(e, delta);
        }
        if !res.worker.is_empty() {
            let e = l.workers.entry(res.worker.clone()).or_insert(0);
            apply(e, delta);
        }

        if l.total > self.ceilings.global_micro {
            Self::trip_locked(&mut l);
        }
        if let Some(cap) = self.ceilings.stage_micro {
            if l.stages.values().any(|v| *v > cap) {
                Self::trip_locked(&mut l);
            }
        }
        if let Some(cap) = self.ceilings.worker_micro {
            if l.workers.values().any(|v| *v > cap) {
                Self::trip_locked(&mut l);
            }
        }
        l.total
    }

    pub fn trip(&self) {
        Self::trip_locked(&mut self.inner.lock().unwrap());
    }

    /// Human-only reset: zero every ledger and clear the trip.
    pub fn reset(&self) {
        let mut l = self.inner.lock().unwrap();
        *l = Ledgers::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(stage: &str, worker: &str, est: u64) -> ReserveCtx {
        ReserveCtx { stage: stage.into(), worker: worker.into(), est_micro_usd: est }
    }

    #[test]
    fn reservation_never_overshoots_cap_under_contention() {
        let pool = std::sync::Arc::new(Pool::new(Ceilings::global_only(1000)));
        let mut handles = vec![];
        for _ in 0..8 {
            let p = pool.clone();
            handles.push(std::thread::spawn(move || {
                let mut ok = 0u64;
                for _ in 0..200 {
                    if p.reserve(&ctx("s", "", 10)).is_ok() {
                        ok += 1;
                    }
                }
                ok
            }));
        }
        let granted: u64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert!(pool.total() <= 1000);
        assert_eq!(granted * 10, pool.total());
    }

    #[test]
    fn settle_corrects_and_trips_on_reality() {
        let pool = Pool::new(Ceilings::global_only(100));
        let r = pool.reserve(&ctx("", "", 40)).unwrap();
        assert_eq!(pool.total(), 40);
        assert_eq!(pool.settle(&r, 90), 90); // +50 correction
        assert!(!pool.is_tripped(), "90 ≤ 100 must not trip");

        // projected 110 > cap → denied, ledger untouched
        assert!(pool.reserve(&ctx("", "", 20)).is_err());
        assert_eq!(pool.total(), 90);

        // 95 fits; reality of 210 crosses the cap → breaker trips
        let r3 = pool.reserve(&ctx("", "", 5)).unwrap();
        assert_eq!(pool.settle(&r3, 120), 210);
        assert!(pool.is_tripped());
        assert!(pool.reserve(&ctx("", "", 1)).is_err());

        pool.reset();
        assert_eq!(pool.total(), 0);
        assert!(!pool.is_tripped());
        assert!(pool.reserve(&ctx("", "", 10)).is_ok());
    }

    #[test]
    fn per_stage_ceiling_denies_and_isolates_scopes() {
        let pool = Pool::new(Ceilings {
            global_micro: 100_000_000,
            stage_micro: Some(10_000_000),
            worker_micro: None,
        });
        assert!(pool.reserve(&ctx("build", "", 9_000_000)).is_ok());

        // same stage over its own cap → denied naming the scope
        let err = pool.reserve(&ctx("build", "", 2_000_000)).unwrap_err();
        assert!(err.contains("SPEND BREAKER"), "{err}");
        assert!(err.contains("per-stage 'build'"), "{err}");

        // other stages unaffected by build's ledger
        assert!(pool.reserve(&ctx("qa", "", 9_000_000)).is_ok());

        // empty-stage callers bypass stage ceilings entirely
        assert!(pool.reserve(&ctx("", "", 9_000_000)).is_ok());
    }

    #[test]
    fn per_worker_ceiling_denies_names_worker() {
        let pool = Pool::new(Ceilings {
            global_micro: 100_000_000,
            stage_micro: None,
            worker_micro: Some(25_000_000),
        });
        assert!(pool.reserve(&ctx("", "w1", 24_000_000)).is_ok());
        let err = pool.reserve(&ctx("", "w1", 2_000_000)).unwrap_err();
        assert!(err.contains("per-worker 'w1'"), "{err}");
        assert!(pool.reserve(&ctx("", "w2", 24_000_000)).is_ok());
    }

    #[test]
    fn settled_crossing_of_stage_cap_trips_everything() {
        let pool = Pool::new(Ceilings {
            global_micro: 100_000_000,
            stage_micro: Some(10_000_000),
            worker_micro: None,
        });
        let r = pool.reserve(&ctx("build", "", 8_000_000)).unwrap();
        pool.settle(&r, 11_000_000); // reality crosses the stage cap
        assert!(pool.is_tripped(), "§3.1: crossing halts EVERYTHING");
        // even an unrelated stage is now denied until human reset
        assert!(pool.reserve(&ctx("qa", "", 1)).is_err());

        pool.reset(); // human reset clears scoped ledgers too
        assert!(pool.reserve(&ctx("build", "", 1)).is_ok());
    }

    #[test]
    fn zero_estimate_passes_through_untouched() {
        let pool = Pool::new(Ceilings {
            global_micro: 100,
            stage_micro: Some(50),
            worker_micro: None,
        });
        let r = pool.reserve(&ctx("any", "any", 0)).unwrap();
        assert_eq!(r.reserved_micro, 0);
        assert_eq!(pool.total(), 0);
        // settle of a free call also moves nothing
        assert_eq!(pool.settle(&r, 0), 0);
        assert!(!pool.is_tripped());
    }
}
