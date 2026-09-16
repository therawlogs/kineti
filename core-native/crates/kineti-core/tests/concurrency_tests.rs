//! High-concurrency stress test suite for kineti-core.
//!
//! Verifies:
//! 1. Zero torn reads under 80 concurrent writer threads and 40 reader threads.
//! 2. p99 snapshot acquisition latency < 0.1ms (100,000 ns) under 80 contending writers.
//! 3. Concurrent spend reservation under 100 threads preventing any overdraft.

use kineti_core::prelude::*;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChecksumPayload {
    seq: u64,
    words: [u64; 6],
    checksum: u64,
}

impl ChecksumPayload {
    fn new(seq: u64) -> Self {
        Self {
            seq,
            words: [seq; 6],
            checksum: seq.wrapping_mul(7),
        }
    }

    fn verify_untorn(&self) -> bool {
        let expected_checksum = self.seq.wrapping_mul(7);
        self.words.iter().all(|&w| w == self.seq) && self.checksum == expected_checksum
    }
}

#[test]
fn test_80_writer_zero_torn_reads_stress() {
    let engine = Arc::new(SnapshotEngine::new(ChecksumPayload::new(0)));
    let running = Arc::new(AtomicBool::new(true));
    let torn_reads_detected = Arc::new(AtomicU64::new(0));
    let version_inversions_detected = Arc::new(AtomicU64::new(0));
    let total_reads = Arc::new(AtomicU64::new(0));

    let num_writers = 80;
    let num_readers = 40;
    let test_duration = Duration::from_millis(1500);

    let mut handles = Vec::new();

    // Spawn 80 concurrent writer threads
    for thread_idx in 0..num_writers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        handles.push(thread::spawn(move || {
            let mut seq = (thread_idx as u64 + 1) * 1_000_000;
            while running_clone.load(Ordering::Relaxed) {
                seq = seq.wrapping_add(1);
                engine_clone.publish(ChecksumPayload::new(seq));
            }
        }));
    }

    // Spawn 40 concurrent reader threads
    for _ in 0..num_readers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        let torn_clone = Arc::clone(&torn_reads_detected);
        let inversions_clone = Arc::clone(&version_inversions_detected);
        let total_clone = Arc::clone(&total_reads);

        handles.push(thread::spawn(move || {
            let mut last_version = 0;
            while running_clone.load(Ordering::Relaxed) {
                let snap = engine_clone.acquire();
                let ver = snap.version();
                if ver < last_version {
                    inversions_clone.fetch_add(1, Ordering::SeqCst);
                }
                last_version = ver;

                let data = snap.data();
                if !data.verify_untorn() {
                    torn_clone.fetch_add(1, Ordering::SeqCst);
                }
                total_clone.fetch_add(1, Ordering::Relaxed);
            }
        }));
    }

    thread::sleep(test_duration);
    running.store(false, Ordering::SeqCst);

    for handle in handles {
        handle.join().expect("Thread joined successfully");
    }

    let reads = total_reads.load(Ordering::SeqCst);
    let torn = torn_reads_detected.load(Ordering::SeqCst);
    let inversions = version_inversions_detected.load(Ordering::SeqCst);

    assert!(reads > 10_000, "Should execute substantial read operations");
    assert_eq!(torn, 0, "FATAL: Torn reads detected under 80-writer contention!");
    assert_eq!(inversions, 0, "FATAL: Version inversions detected under 80-writer contention!");
}

#[test]
fn test_80_writer_p99_snapshot_latency() {
    let engine = Arc::new(SnapshotEngine::new(vec![0u8; 1024]));
    let running = Arc::new(AtomicBool::new(true));

    let num_writers = 80;
    let mut handles = Vec::new();

    // 80 Concurrent Writers hammering publish
    for _ in 0..num_writers {
        let engine_clone = Arc::clone(&engine);
        let running_clone = Arc::clone(&running);
        handles.push(thread::spawn(move || {
            let mut counter = 0u8;
            while running_clone.load(Ordering::Relaxed) {
                counter = counter.wrapping_add(1);
                engine_clone.publish(vec![counter; 1024]);
            }
        }));
    }

    // Benchmark reader measuring high-precision latency
    let sample_count = 50_000;
    let mut latencies_nanos = Vec::with_capacity(sample_count);

    // Warm up
    for _ in 0..1_000 {
        let _ = engine.acquire();
    }

    for _ in 0..sample_count {
        let start = Instant::now();
        let handle = engine.acquire();
        let elapsed = start.elapsed();
        latencies_nanos.push(elapsed.as_nanos() as u64);
        let _ = handle.version();
    }

    running.store(false, Ordering::SeqCst);
    for handle in handles {
        handle.join().unwrap();
    }

    latencies_nanos.sort_unstable();
    let p99 = latencies_nanos[sample_count * 99 / 100];

    // Verification criteria: p99 latency < 0.1ms = 100,000 ns
    assert!(
        p99 < 100_000,
        "p99 latency {} ns exceeded 0.1ms (100,000 ns) threshold under 80-thread write contention!",
        p99
    );
}

#[test]
fn test_100_thread_spend_breaker_concurrency() {
    let breaker = Arc::new(SpendCircuitBreaker::default());
    let num_threads = 100;
    let reservation_amount = 1_000_000; // $1.00

    let successful = Arc::new(AtomicUsize::new(0));
    let rejected = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let breaker_clone = Arc::clone(&breaker);
        let s_clone = Arc::clone(&successful);
        let r_clone = Arc::clone(&rejected);

        handles.push(thread::spawn(move || {
            match breaker_clone.reserve(reservation_amount) {
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

    let successes = successful.load(Ordering::SeqCst);
    let rejects = rejected.load(Ordering::SeqCst);
    assert_eq!(successes + rejects, num_threads);

    let committed = breaker.committed_microcents();
    assert!(committed <= DEFAULT_TRIP_THRESHOLD_MICROCENTS);
    assert!(committed <= DEFAULT_CEILING_MICROCENTS);
    assert!(breaker.is_tripped());
}
