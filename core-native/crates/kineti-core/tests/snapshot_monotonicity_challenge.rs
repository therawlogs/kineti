//! Adversarial Empirical Challenge Suite for Snapshot Monotonicity & Spend Guarantees.
//!
//! Evaluates:
//! 1. Snapshot Engine Version Monotonicity under 100 concurrent writers & 50 readers (0% inversions).
//! 2. Snapshot Engine Zero Torn Reads under high writer churn (0 torn reads).
//! 3. Snapshot Engine Monotonicity under thread yielding / adversarial scheduler preemption.
//! 4. Spend Circuit Breaker Boundary Race under 200 threads (zero overdraft at $47.50).
//! 5. Spend Circuit Breaker Mixed Operations under 500 threads (zero overdraft, 0 leaked reservations).

use kineti_core::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MultiWordPayload {
    seq: u64,
    words: [u64; 8],
    checksum: u64,
}

impl MultiWordPayload {
    fn new(seq: u64) -> Self {
        Self {
            seq,
            words: [seq; 8],
            checksum: seq.wrapping_mul(31),
        }
    }

    fn verify_untorn(&self) -> bool {
        let expected_checksum = self.seq.wrapping_mul(31);
        self.words.iter().all(|&w| w == self.seq) && self.checksum == expected_checksum
    }
}

#[test]
fn test_snapshot_monotonicity_100_writers_50_readers() {
    let engine = Arc::new(SnapshotEngine::new(MultiWordPayload::new(0)));
    let running = Arc::new(AtomicBool::new(true));
    let total_reads = Arc::new(AtomicU64::new(0));
    let total_writes = Arc::new(AtomicU64::new(0));
    let torn_reads = Arc::new(AtomicU64::new(0));
    let version_inversions = Arc::new(AtomicU64::new(0));

    let num_writers = 100;
    let num_readers = 50;
    let test_duration = Duration::from_millis(2500);

    let mut handles = Vec::with_capacity(num_writers + num_readers);

    // 100 Writers
    for thread_idx in 0..num_writers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        let writes_clone = Arc::clone(&total_writes);
        handles.push(thread::spawn(move || {
            let mut seq = (thread_idx as u64 + 1) * 10_000_000;
            while running_clone.load(Ordering::Relaxed) {
                seq = seq.wrapping_add(1);
                engine_clone.publish(MultiWordPayload::new(seq));
                writes_clone.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    // 50 Readers
    for reader_idx in 0..num_readers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        let reads_clone = Arc::clone(&total_reads);
        let torn_clone = Arc::clone(&torn_reads);
        let inversions_clone = Arc::clone(&version_inversions);

        handles.push(thread::spawn(move || {
            let mut last_version = 0;
            while running_clone.load(Ordering::Relaxed) {
                let snap = engine_clone.acquire();
                let ver = snap.version();

                if ver < last_version {
                    inversions_clone.fetch_add(1, Ordering::SeqCst);
                    eprintln!(
                        "FATAL INVERSION on Reader {}: ver {} < last_version {} (diff: -{})",
                        reader_idx, ver, last_version, last_version - ver
                    );
                }
                last_version = ver;

                let data = snap.data();
                if !data.verify_untorn() {
                    torn_clone.fetch_add(1, Ordering::SeqCst);
                }

                reads_clone.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    thread::sleep(test_duration);
    running.store(false, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Worker thread joined successfully");
    }

    let writes = total_writes.load(Ordering::SeqCst);
    let reads = total_reads.load(Ordering::SeqCst);
    let torn = torn_reads.load(Ordering::SeqCst);
    let inversions = version_inversions.load(Ordering::SeqCst);

    println!(
        "Snapshot 100W/50R Results: Writes={}, Reads={}, Torn={}, Inversions={}",
        writes, reads, torn, inversions
    );

    assert!(writes > 500, "Must perform write operations");
    assert!(reads > 10_000, "Must perform substantial read operations");
    assert_eq!(torn, 0, "EMPERICAL DEFECT: Torn reads detected under 100-writer contention!");
    assert_eq!(
        inversions, 0,
        "EMPIRICAL DEFECT: Version inversions detected under 100-writer contention!"
    );
}

#[test]
fn test_snapshot_monotonicity_with_adversarial_thread_yielding() {
    let engine = Arc::new(SnapshotEngine::new(MultiWordPayload::new(0)));
    let running = Arc::new(AtomicBool::new(true));
    let total_reads = Arc::new(AtomicU64::new(0));
    let torn_reads = Arc::new(AtomicU64::new(0));
    let version_inversions = Arc::new(AtomicU64::new(0));

    let num_writers = 80;
    let num_readers = 40;
    let test_duration = Duration::from_millis(2000);

    let mut handles = Vec::with_capacity(num_writers + num_readers);

    // 80 Writers with periodic yield
    for thread_idx in 0..num_writers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        handles.push(thread::spawn(move || {
            let mut seq = (thread_idx as u64 + 1) * 1_000_000;
            let mut iter = 0usize;
            while running_clone.load(Ordering::Relaxed) {
                seq = seq.wrapping_add(1);
                engine_clone.publish(MultiWordPayload::new(seq));
                iter += 1;
                if iter % 7 == 0 {
                    thread::yield_now();
                }
            }
        }));
    }

    // 40 Readers with periodic yield
    for reader_idx in 0..num_readers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        let reads_clone = Arc::clone(&total_reads);
        let torn_clone = Arc::clone(&torn_reads);
        let inversions_clone = Arc::clone(&version_inversions);

        handles.push(thread::spawn(move || {
            let mut last_version = 0;
            let mut iter = 0usize;
            while running_clone.load(Ordering::Relaxed) {
                let snap = engine_clone.acquire();
                let ver = snap.version();

                if ver < last_version {
                    inversions_clone.fetch_add(1, Ordering::SeqCst);
                    eprintln!(
                        "FATAL INVERSION with yielding Reader {}: ver {} < last_version {}",
                        reader_idx, ver, last_version
                    );
                }
                last_version = ver;

                let data = snap.data();
                if !data.verify_untorn() {
                    torn_clone.fetch_add(1, Ordering::SeqCst);
                }

                reads_clone.fetch_add(1, Ordering::Relaxed);
                iter += 1;
                if iter % 11 == 0 {
                    thread::yield_now();
                }
            }
        }));
    }

    thread::sleep(test_duration);
    running.store(false, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Worker thread joined successfully");
    }

    let reads = total_reads.load(Ordering::SeqCst);
    let torn = torn_reads.load(Ordering::SeqCst);
    let inversions = version_inversions.load(Ordering::SeqCst);

    println!(
        "Snapshot Adversarial Yield Results: Reads={}, Torn={}, Inversions={}",
        reads, torn, inversions
    );

    assert!(reads > 10_000, "Substantial reads expected");
    assert_eq!(torn, 0, "Torn reads found during adversarial yielding");
    assert_eq!(inversions, 0, "Version inversions found during adversarial yielding");
}

#[test]
fn test_spend_circuit_breaker_boundary_race_200_threads() {
    let ceiling = 50_000_000;
    let trip_threshold = 47_500_000;
    let breaker = Arc::new(SpendCircuitBreaker::with_trip_threshold(ceiling, trip_threshold));

    // Fill up to exactly 1 microcent below trip threshold
    let prefill = trip_threshold - 1;
    let res = breaker.reserve(prefill).expect("Prefill reservation succeeds");
    breaker.commit(res).expect("Prefill commit succeeds");
    assert_eq!(breaker.committed_microcents(), prefill);
    assert!(!breaker.is_tripped());

    let num_threads = 200;
    let successful = Arc::new(AtomicUsize::new(0));
    let rejected = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let breaker_clone = Arc::clone(&breaker);
        let s_clone = Arc::clone(&successful);
        let r_clone = Arc::clone(&rejected);

        handles.push(thread::spawn(move || {
            match breaker_clone.reserve(1) {
                Ok(res) => {
                    let _ = breaker_clone.commit(res);
                    s_clone.fetch_add(1, Ordering::SeqCst);
                }
                Err(_) => {
                    r_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let s = successful.load(Ordering::SeqCst);
    let r = rejected.load(Ordering::SeqCst);

    println!("200-Thread Boundary Race: Successful={}, Rejected={}", s, r);
    assert_eq!(s, 1, "Exactly 1 thread must capture the boundary microcent");
    assert_eq!(r, num_threads - 1, "All remaining threads must be rejected");
    assert_eq!(breaker.committed_microcents(), trip_threshold);
    assert_eq!(breaker.reserved_microcents(), 0);
    assert!(breaker.is_tripped());
}

#[test]
fn test_spend_circuit_breaker_mixed_500_threads() {
    let ceiling = 50_000_000;
    let trip_threshold = 47_500_000;
    let breaker = Arc::new(SpendCircuitBreaker::with_trip_threshold(ceiling, trip_threshold));

    let num_threads = 500;
    let completed = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for thread_idx in 0..num_threads {
        let breaker_clone = Arc::clone(&breaker);
        let c_clone = Arc::clone(&completed);

        handles.push(thread::spawn(move || {
            let amount = 100_000; // $0.10
            match thread_idx % 5 {
                0 => {
                    // Full reserve + commit
                    if let Ok(res) = breaker_clone.reserve(amount) {
                        let _ = breaker_clone.commit(res);
                    }
                }
                1 => {
                    // Reserve + abort
                    if let Ok(res) = breaker_clone.reserve(amount) {
                        let _ = breaker_clone.abort(res);
                    }
                }
                2 => {
                    // Reserve + RAII drop compensation
                    if let Ok(res) = breaker_clone.reserve(amount) {
                        drop(res);
                    }
                }
                3 => {
                    // Reserve + partial spend settlement
                    if let Ok(res) = breaker_clone.reserve(amount) {
                        let _ = breaker_clone.commit_with_actual(res, 40_000);
                    }
                }
                _ => {
                    // Fast path record spend
                    let _ = breaker_clone.record_spend(25_000);
                }
            }
            c_clone.fetch_add(1, Ordering::SeqCst);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(completed.load(Ordering::SeqCst), num_threads);

    let committed = breaker.committed_microcents();
    let reserved = breaker.reserved_microcents();

    println!(
        "500-Thread Mixed Ops: Committed={} ($ {:.4}), Reserved={}",
        committed,
        committed as f64 / 1_000_000.0,
        reserved
    );

    assert_eq!(reserved, 0, "All reservations must resolve to 0 reserved microcents");
    assert!(
        committed <= trip_threshold,
        "FATAL: committed microcents {} breached trip threshold {}",
        committed,
        trip_threshold
    );
    assert!(
        committed <= ceiling,
        "FATAL: committed microcents {} breached hard ceiling {}",
        committed,
        ceiling
    );
}
