//! C-ABI surface (Phase 9): three exports + allocator, safe at every seam.
//!
//! Contract:
//! - Inputs are NUL-terminated UTF-8 C strings (`*const c_char`); NULL is an
//!   error, never a crash. Inputs are NEVER freed by Kineti.
//! - Outputs are `KinetiResult` BY VALUE: `ok` plus a heap `*mut c_char`
//!   holding JSON (success payload or error text). Free payloads with
//!   [`kineti_free_string`]. NULL payload = allocation failure.
//! - Every export is wrapped in catch_unwind — panics never cross FFI.
//! - The caller MUST have the project directory as its working directory
//!   (same contract as every kineti subcommand).

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Owned heap string handed across the boundary. Free with
/// [`kineti_free_string`].
#[repr(C)]
pub struct FfiResult {
    pub ok: bool,
    pub payload: *mut c_char,
}

impl FfiResult {
    fn ok_json(v: serde_json::Value) -> Self {
        FfiResult::from_str(&v.to_string(), true)
    }
    fn err(msg: &str) -> Self {
        FfiResult::from_str(msg, false)
    }
    fn from_str(s: &str, ok: bool) -> Self {
        match CString::new(s.replace('\0', " ")) {
            Ok(c) => FfiResult { ok, payload: c.into_raw() },
            Err(_) => FfiResult { ok: false, payload: std::ptr::null_mut() },
        }
    }
}

fn read_cstr(ptr: *const c_char) -> Result<String, String> {
    if ptr.is_null() {
        return Err("NULL input string".into());
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| "input is not valid UTF-8".to_string())
}

fn guard<T>(f: impl FnOnce() -> T) -> Option<T> {
    catch_unwind(AssertUnwindSafe(f)).ok()
}

/// Library version (static storage, never freed). Derived from Cargo's
/// package version — the single source of truth.
#[no_mangle]
pub extern "C" fn kineti_version() -> *const c_char {
    use std::sync::OnceLock;
    static VERSION: OnceLock<CString> = OnceLock::new();
    VERSION
        .get_or_init(|| CString::new(env!("CARGO_PKG_VERSION")).expect("no NUL in version"))
        .as_ptr()
}

/// Release a payload previously returned in a `KinetiResult`.
///
/// # Safety
/// `ptr` must be NULL or a pointer returned by this library that has not
/// been freed already.
#[no_mangle]
pub unsafe extern "C" fn kineti_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

/// Args JSON: {"goal": str, "provider"?: str, "model"?: str, "cap"?: f64,
///              "mode"?: "single"|"swarm", "auto_approve_spec"?: bool}
/// Runs the governed pipeline against the CURRENT working directory.
/// `auto_approve_spec=true` is the CALLER taking §10.2 responsibility; the
/// audit trail marks it "ffi auto-approval".
#[no_mangle]
pub extern "C" fn kineti_run(args_json: *const c_char) -> FfiResult {
    match guard(|| run_inner(args_json)) {
        Some(r) => r,
        None => FfiResult::err("internal panic during run"),
    }
}

fn run_inner(args_json: *const c_char) -> FfiResult {
    let raw = match read_cstr(args_json) {
        Ok(s) => s,
        Err(e) => return FfiResult::err(&e),
    };
    let args: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(e) => return FfiResult::err(&format!("bad args json: {e}")),
    };
    let goal = args["goal"].as_str().unwrap_or_default().to_string();
    if goal.trim().is_empty() {
        return FfiResult::err("args.goal must be a non-empty string");
    }

    let cfg = crate::config::Config::load();
    let provider_name = args["provider"].as_str().unwrap_or("gemini").to_string();
    let p = cfg.provider(&provider_name);
    let model = args["model"].as_str().unwrap_or(&p.default_model).to_string();
    let cap = args["cap"].as_f64();
    let ceilings = crate::ipc::pool::Ceilings {
        global_micro: (cap.unwrap_or(cfg.limits.global_usd).max(0.0) * 1_000_000.0).round()
            as u64,
        stage_micro: (cfg.limits.per_stage_usd > 0.0)
            .then(|| (cfg.limits.per_stage_usd * 1_000_000.0).round() as u64),
        worker_micro: (cfg.limits.max_worker_usd > 0.0)
            .then(|| (cfg.limits.max_worker_usd * 1_000_000.0).round() as u64),
    };

    if args["auto_approve_spec"].as_bool().unwrap_or(false) {
        crate::stages::AUTO_APPROVE_SPEC.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    let mut exec = cfg.execution.clone();
    if let Some(m) = args["mode"].as_str() {
        exec.mode = m.to_string();
    }

    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let code = crate::stages::drive(&root, &p, &model, &goal, ceilings, &exec);

    // reset for any subsequent call in-process
    crate::stages::AUTO_APPROVE_SPEC.store(false, std::sync::atomic::Ordering::Relaxed);

    let state = crate::stages::State::load(&root);
    FfiResult::ok_json(serde_json::json!({
        "exit": code,
        "stage_reached": state.stage,
        "spec_approved": state.spec_approved.is_some(),
        "shipped_at": state.shipped_at,
    }))
}

/// Verify journal history against the current directory.
/// Args JSON: {"all"?: bool} — all=true runs the full DAG check.
#[no_mangle]
pub extern "C" fn kineti_verify(args_json: *const c_char) -> FfiResult {
    match guard(|| verify_inner(args_json)) {
        Some(r) => r,
        None => FfiResult::err("internal panic during verify"),
    }
}

fn verify_inner(args_json: *const c_char) -> FfiResult {
    let empty = serde_json::json!({});
    let args: serde_json::Value = match read_cstr(args_json) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|_| empty.clone()),
        Err(e) => return FfiResult::err(&e),
    };
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    if args["all"].as_bool().unwrap_or(false) {
        let rep = crate::memory::merge::verify_project(&root);
        let payload = serde_json::json!({
            "ok": rep.is_clean(),
            "main_records": rep.main_records,
            "main_head": rep.main_head,
            "branches": rep.branches.iter().map(|(b, n, h)| serde_json::json!({
                "branch": b, "records": n, "head": h,
            })).collect::<Vec<_>>(),
            "orphans": rep.orphans,
            "errors": rep.errors,
        });
        return FfiResult::ok_json(payload);
    }

    let j = crate::memory::journal::Journal::load(&root.join(".kineti/journal.jsonl"));
    match j.verify() {
        Ok(()) => FfiResult::ok_json(serde_json::json!({
            "ok": true,
            "records": j.records.len(),
            "head": j.records.last().map(|r| r.hash.clone())
                .unwrap_or_else(|| crate::memory::journal::GENESIS.into()),
        })),
        Err(e) => FfiResult::err(&format!("TAMPERED: {e}")),
    }
}

/// Full receipt summary as JSON against the current directory.
/// Args JSON may be NULL or "{}".
#[no_mangle]
pub extern "C" fn kineti_receipt(args_json: *const c_char) -> FfiResult {
    match guard(|| receipt_inner(args_json)) {
        Some(r) => r,
        None => FfiResult::err("internal panic during receipt"),
    }
}

fn receipt_inner(args_json: *const c_char) -> FfiResult {
    // args unused today; validated anyway so callers get early errors
    if let Ok(s) = read_cstr(args_json) {
        if !s.trim().is_empty() && serde_json::from_str::<serde_json::Value>(&s).is_err() {
            return FfiResult::err("bad args json");
        }
    }
    let root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let summary = crate::receipt::build(&root);
    FfiResult::ok_json(crate::receipt::to_json(&summary))
}
