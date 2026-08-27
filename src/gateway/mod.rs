//! Gateway meter — minimal OpenAI-compatible proxy with reserve/settle + receipt.
//! Demo in public tree; hosted version lives in therawlogs/kineti-pro.
//! Never stores raw prompts — hashes + metadata only.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;

use crate::config::Config;
use crate::ipc::pool::Ceilings;
use crate::memory::journal::sha256_hex;
use crate::provider::estimate_cost_micro;

const RECEIPT_FILE: &str = ".kineti/receipt.jsonl";

fn ceilings_from_cfg(cfg: &Config) -> Ceilings {
    let pos = |v: f64| (v > 0.0).then(|| (v * 1_000_000.0).round() as u64);
    Ceilings {
        global_micro: (cfg.limits.global_usd.max(0.0) * 1_000_000.0).round() as u64,
        stage_micro: pos(cfg.limits.per_stage_usd),
        worker_micro: pos(cfg.limits.max_worker_usd),
    }
}

fn append_receipt(root: &Path, entry: serde_json::Value) {
    let path = root.join(RECEIPT_FILE);
    if let Some(dir) = path.parent() { let _ = std::fs::create_dir_all(dir); }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", entry);
    }
}

/// Run a blocking proxy on 127.0.0.1:port that forwards POST /v1/chat/completions
/// to the configured provider with reserve/settle. Ctrl-C to stop.
pub fn serve(port: u16) -> i32 {
    let root = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    let cfg = Config::load();
    let ceilings = ceilings_from_cfg(&cfg);
    let provider = cfg.provider("grok");
    let default_model = provider.default_model.clone();
    let backend = match crate::ipc::DirectBackend::new(&root, ceilings.clone()) {
        Ok(b) => b,
        Err(e) => { eprintln!("gateway: ledger: {e}"); return 1; }
    };
    let addr = format!("127.0.0.1:{port}");
    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => { eprintln!("gateway: bind {addr}: {e}"); return 1; }
    };
    println!("kineti gateway listening on http://{addr}/v1 (ledger cap ${:.2}, model {})", ceilings.global_micro as f64/1_000_000.0, default_model);
    println!("  Set agents to: base_url = http://{addr}/v1");
    for stream in listener.incoming() {
        let Ok(stream) = stream else { continue };
        handle_one(stream, &root, &provider, &default_model, &backend);
    }
    0
}

fn handle_one(mut stream: TcpStream, root: &Path, provider: &crate::config::ProviderCfg, default_model: &str, backend: &crate::ipc::DirectBackend) {
    use crate::ipc::SpendService;
    let mut reader = BufReader::new(stream.try_clone().unwrap());
    let mut req_line = String::new();
    if reader.read_line(&mut req_line).is_err() || req_line.is_empty() { return; }
    let mut headers = std::collections::HashMap::new();
    let mut content_len = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() { return; }
        if line == "\r\n" || line == "\n" || line.trim().is_empty() { break; }
        if let Some((k,v)) = line.split_once(':') {
            let k = k.trim().to_ascii_lowercase();
            let v = v.trim().to_string();
            if k == "content-length" { content_len = v.parse().unwrap_or(0); }
            headers.insert(k,v);
        }
    }
    let mut body = vec![0u8; content_len];
    if content_len > 0 { let _ = reader.read_exact(&mut body); }
    let body_str = String::from_utf8_lossy(&body).to_string();
    let is_chat = req_line.contains("/v1/chat/completions") && req_line.starts_with("POST");
    if !is_chat {
        let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found";
        let _ = stream.write_all(resp.as_bytes());
        return;
    }
    // parse model + messages for estimate (hash only, never log raw)
    let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap_or(serde_json::json!({}));
    let model = parsed["model"].as_str().unwrap_or(default_model).to_string();
    let prompt_chars = body_str.len();
    let est = estimate_cost_micro(provider.price_per_1m_input, provider.price_per_1m_output, prompt_chars);
    let run_id = format!("gw-{}", &sha256_hex(&format!("{}{}", crate::memory::journal::now_iso(), prompt_chars))[..12]);
    let ctx = crate::ipc::dto::ReserveCtx { stage: "gateway".into(), worker: String::new(), est_micro_usd: est };
    if let Err(e) = backend.reserve(&ctx) {
        let body = serde_json::json!({"error": e}).to_string();
        let resp = format!("HTTP/1.1 429 Too Many Requests\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = stream.write_all(resp.as_bytes());
        append_receipt(root, serde_json::json!({"v":"1","at": crate::memory::journal::now_iso(),"run_id": run_id, "model": model, "prompt_hash": sha256_hex(&body_str), "error": e, "est_micro": est}));
        return;
    }
    let reservation = crate::ipc::dto::Reservation { id: 0, reserved_micro: est, stage: "gateway".into(), worker: String::new() };
    // forward to provider
    let url = format!("{}/chat/completions", provider.base_url.trim_end_matches('/'));
    let key = crate::auth::resolve_bearer(&provider.name, provider).unwrap_or_default();
    let fwd = ureq::post(&url)
        .set("Authorization", &format!("Bearer {}", key))
        .set("Content-Type", "application/json")
        .send_string(&body_str);
    let (status, resp_body) = match fwd {
        Ok(r) => (200, r.into_string().unwrap_or_default()),
        Err(ureq::Error::Status(c, r)) => (c, r.into_string().unwrap_or_default()),
        Err(e) => (502, format!("{{\"error\":\"gateway forward: {e}\"}}")),
    };
    // settle on usage if present
    let actual_micro = serde_json::from_str::<serde_json::Value>(&resp_body).ok()
        .map(|v| {
            let pt = v["usage"]["prompt_tokens"].as_u64().unwrap_or(0);
            let ct = v["usage"]["completion_tokens"].as_u64().unwrap_or(0);
            let cost = (pt as f64/1e6)*provider.price_per_1m_input + (ct as f64/1e6)*provider.price_per_1m_output;
            (cost*1_000_000.0).round() as u64
        }).unwrap_or(est);
    let _ = backend.settle(&reservation, actual_micro);
    let prompt_hash = sha256_hex(&body_str);
    let response_hash = sha256_hex(&resp_body);
    append_receipt(root, serde_json::json!({
        "v":"1","at": crate::memory::journal::now_iso(),"run_id": run_id,
        "who": "gateway","model": model,
        "prompt_hash": prompt_hash, "response_hash": response_hash,
        "prompt_chars": prompt_chars, "est_micro": est, "actual_micro": actual_micro,
        "status": status
    }));
    let resp = format!("HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}", status, resp_body.len(), resp_body);
    let _ = stream.write_all(resp.as_bytes());
}
