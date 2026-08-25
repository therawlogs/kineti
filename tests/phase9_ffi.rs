//! Phase 9 acceptance: the C-ABI boundary is safe (nulls, invalid UTF-8,
//! bad JSON, panics) and correct (verify/receipt payloads match the library
//! builders); the dylib actually exports the promised symbols.

use std::ffi::{CStr, CString};
use std::path::PathBuf;

use kineti::ffi::{
    kineti_free_string, kineti_receipt, kineti_run, kineti_verify, kineti_version, FfiResult,
};

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp9-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".kineti")).unwrap();
    d
}

/// cwd is process-global — serialize every test that changes it.
static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// RAII guard for a returned payload.
struct Owned(FfiResult);
impl Drop for Owned {
    fn drop(&mut self) {
        if !self.0.payload.is_null() {
            unsafe { kineti_free_string(self.0.payload) };
        }
    }
}
impl Owned {
    fn text(&self) -> String {
        assert!(!self.0.payload.is_null(), "payload must never be null on ok results");
        unsafe { CStr::from_ptr(self.0.payload) }.to_string_lossy().into_owned()
    }
    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.text()).expect("payload must be valid JSON")
    }
}

#[test]
fn version_returns_static_string() {
    let v = kineti_version();
    assert!(!v.is_null());
    let s = unsafe { CStr::from_ptr(v) }.to_str().unwrap();
    assert_eq!(s, "0.1.0");
    // deliberately NOT freed — static storage
}

#[test]
fn null_and_bad_inputs_fail_cleanly() {
    for r in [kineti_verify(std::ptr::null()), kineti_receipt(std::ptr::null())] {
        let o = Owned(r);
        // receipt accepts NULL; verify must error on it
        let _ = o;
    }
    let v_err = Owned(kineti_verify(std::ptr::null()));
    assert!(!v_err.0.ok);
    assert!(v_err.text().contains("NULL"), "{}", v_err.text());

    let bad = CString::new("not json at all {{{").unwrap();
    let r_err = Owned(kineti_run(bad.as_ptr()));
    assert!(!r_err.0.ok);
    assert!(r_err.text().contains("bad args json"), "{}", r_err.text());

    // non-UTF8 input rejected without crashing
    let invalid = unsafe { CString::from_vec_unchecked(vec![0xFF, 0xFE, b'a', 0]) };
    let u_err = Owned(kineti_verify(invalid.as_ptr()));
    assert!(!u_err.0.ok);
    assert!(u_err.text().contains("UTF-8"), "{}", u_err.text());

    // missing goal
    let no_goal = CString::new("{}").unwrap();
    let g_err = Owned(kineti_run(no_goal.as_ptr()));
    assert!(!g_err.0.ok);
    assert!(g_err.text().contains("goal"), "{}", g_err.text());
}

#[test]
fn verify_ffi_matches_library_on_clean_project() {
    // run in a temp cwd: FFI contract is "project = current working dir"
    let _g = CWD_LOCK.lock().unwrap();
    let d = tmp("verify");
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&d).unwrap();

    // seed one record through the library so the chain exists
    let w = kineti::ipc::journal_writer_no_spawn(&d);
    let r = kineti::memory::journal::build(
        kineti::memory::journal::GENESIS,
        "a-1",
        "action",
        &serde_json::json!({"x": 1}),
    );
    w.append_batch("", vec![r]).unwrap();

    let args = CString::new("{}").unwrap();
    let out = Owned(kineti_verify(args.as_ptr()));
    assert!(out.0.ok, "{}", out.text());
    let j = out.json();
    assert_eq!(j["ok"], true);
    assert_eq!(j["records"], 1);

    // --all variant: clean DAG, no branches
    let all = CString::new(r#"{"all":true}"#).unwrap();
    let out_all = Owned(kineti_verify(all.as_ptr()));
    assert!(out_all.0.ok);
    let j2 = out_all.json();
    assert_eq!(j2["ok"], true);
    assert_eq!(j2["branches"].as_array().unwrap().len(), 0);

    std::env::set_current_dir(prev).unwrap();
}

#[test]
fn receipt_ffi_exposes_spend_and_history() {
    let _g = CWD_LOCK.lock().unwrap();
    let d = tmp("receipt");
    let prev = std::env::current_dir().unwrap();
    std::env::set_current_dir(&d).unwrap();

    std::fs::write(d.join(".kineti/root_goal"), "demo goal").unwrap();
    let w = kineti::ipc::journal_writer_no_spawn(&d);
    let h = w.head("").unwrap();
    w.append_batch(
        "",
        vec![kineti::memory::journal::build(
            &h,
            "rr-0001",
            "run-record",
            &serde_json::json!({"outcome": "complete", "cost_usd": 0.25, "iterations": 4}),
        )],
    )
    .unwrap();

    // NULL args are accepted here by design
    let out = Owned(kineti_receipt(std::ptr::null()));
    assert!(out.0.ok, "{}", out.text());
    let j = out.json();
    assert_eq!(j["goal"], "demo goal");
    assert!((j["spend"]["coordinator_usd"].as_f64().unwrap() - 0.25).abs() < 1e-9);
    assert_eq!(j["history_clean"], true);

    std::env::set_current_dir(prev).unwrap();
}

#[test]
fn free_string_is_null_safe() {
    unsafe { kineti_free_string(std::ptr::null_mut()) }; // must not crash
}

// ── dylib symbol presence (the cdylib target really exports them) ───────────

#[cfg(target_os = "macos")]
#[test]
fn dylib_exports_c_symbols() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let dylib = format!("{manifest}/target/debug/libkineti.dylib");
    let out = nm_globals(&dylib);
    assert!(
        out.contains("_kineti_run") && out.contains("_kineti_verify")
            && out.contains("_kineti_receipt") && out.contains("_kineti_free_string")
            && out.contains("_kineti_version"),
        "missing symbols in {dylib}:\n{out}"
    );
}

#[cfg(not(target_os = "macos"))]
#[test]
fn dylib_exports_c_symbols() {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let dylib = format!("{manifest}/target/debug/libkineti.so");
    let out = std::fs::read_to_string(format!("/dev/null")).unwrap_or_default();
    let nm = std::process::Command::new("nm")
        .arg("-D")
        .arg(&dylib)
        .output()
        .expect("nm spawn");
    let syms = String::from_utf8_lossy(&nm.stdout).to_string();
    assert!(syms.contains("kineti_run") && syms.contains("kineti_receipt"), "{syms}");
    drop(out);
}

#[cfg(target_os = "macos")]
fn nm_globals(path: &str) -> String {
    let out = std::process::Command::new("nm").arg("-gU").arg(path).output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => String::new(),
    }
}
