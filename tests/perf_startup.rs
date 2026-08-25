//! Startup budget tests (Tier 0): in-process direct-exec paths stay inside
//! the 10 µs median contract. CI-safe hard-fail at 50 µs.

use kineti::ffi::kineti_version;
use kineti::light;

fn median_us<T: FnMut()>(mut f: T, iters: usize) -> f64 {
    // warmup
    for _ in 0..64 {
        f();
    }
    let mut samples = Vec::with_capacity(iters);
    for _ in 0..iters {
        let t0 = std::time::Instant::now();
        f();
        samples.push(t0.elapsed().as_nanos() as f64 / 1000.0);
    }
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[iters / 2]
}

#[test]
fn ffi_version_call_is_within_10us_median() {
    let iters = 10_000;
    let m = median_us(
        || {
            let p = kineti_version();
            assert!(!p.is_null());
            // consume so the call can't be fully optimized into nothing
            std::hint::black_box(unsafe { std::ffi::CStr::from_ptr(p) });
        },
        iters,
    );
    println!("kineti_version median = {m:.2} µs");
    assert!(m <= 10.0, "FFI version median {m}µs exceeds the 10µs direct-exec budget");
}

#[test]
fn light_reply_is_within_10us_median() {
    let argv = vec!["--version".to_string()];
    let m = median_us(
        || {
            if let Some(r) = light::reply(&argv) {
                std::hint::black_box(&r);
            } else {
                panic!("--version must fast-path");
            }
        },
        10_000,
    );
    println!("light::reply(--version) median = {m:.2} µs");
    assert!(m <= 10.0, "fast-path median {m}µs exceeds the 10µs budget");

    // help path too
    let argv_h = vec!["-h".to_string()];
    let mh = median_us(
        || {
            if let Some(r) = light::reply(&argv_h) {
                std::hint::black_box(&r);
            }
        },
        10_000,
    );
    assert!(mh <= 10.0);

    // non-light input must defer (no false positives)
    assert!(light::reply(&["run".to_string()]).is_none());
    assert!(light::reply(&["--version".to_string(), "extra".to_string()]).is_none());
}

/// CI guard rail: even on noisy runners these must never blow past 50 µs.
#[test]
fn ci_hard_ceiling_50us() {
    let argv = vec!["--version".to_string()];
    let mv = median_us(
        || {
            let p = kineti_version();
            assert!(!p.is_null());
            std::hint::black_box(unsafe { std::ffi::CStr::from_ptr(p) });
        },
        2_000,
    );
    let ml = median_us(
        || {
            if let Some(r) = light::reply(&argv) {
                std::hint::black_box(&r);
            }
        },
        2_000,
    );
    assert!(mv < 50.0 && ml < 50.0, "mv={mv}µs ml={ml}µs");
}
