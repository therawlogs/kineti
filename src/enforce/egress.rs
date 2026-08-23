//! Egress firewall (ASI-05, D1): every outbound send is recorded BEFORE it
//! happens, into a hash-chained log. Payloads are never logged verbatim —
//! only redacted purpose text + payload hash.

use crate::memory::journal::{compute_hash, now_iso, GENESIS};
use regex::Regex;
use std::sync::OnceLock;

fn key_patterns() -> &'static Vec<Regex> {
    static P: OnceLock<Vec<Regex>> = OnceLock::new();
    P.get_or_init(|| {
        [
            r"xai-[A-Za-z0-9]{20,}",
            r"AIza[0-9A-Za-z_\-]{30,}",
            r"AQ\.[A-Za-z0-9_\-]{20,}",
            r"sk-[A-Za-z0-9_\-]{20,}",
            r"gh[pousr]_[A-Za-z0-9_]{20,}",
            r"Bearer\s+[A-Za-z0-9_\-\.]{20,}",
            r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}",
        ]
        .iter()
        .filter_map(|p| Regex::new(p).ok())
        .collect()
    })
}

pub fn redact(s: &str) -> String {
    let mut out = s.to_string();
    for re in key_patterns() {
        out = re.replace_all(&out, "[REDACTED]").to_string();
    }
    out
}

/// Append a hash-chained egress record under <cwd>/.kineti/egress.jsonl.
pub fn record(dest: &str, purpose: &str, payload_sha: &str) {
    let dir = std::path::Path::new(".kineti");
    let _ = std::fs::create_dir_all(dir);
    let path = dir.join("egress.jsonl");

    let mut prev = GENESIS.to_string();
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Some(last) = content.lines().filter(|l| !l.trim().is_empty()).last() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(last) {
                prev = v["hash"].as_str().unwrap_or(GENESIS).to_string();
            }
        }
    }

    let base = serde_json::json!({
        "at": now_iso(),
        "dest": dest,
        "purpose": redact(purpose),
        "payload_sha": payload_sha,
    });
    let hash = compute_hash(&prev, &base["at"].as_str().unwrap_or(""), "egress", &base);
    let mut rec = base.clone();
    rec["prev_hash"] = prev.into();
    rec["hash"] = hash.into();

    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{rec}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn redaction_catches_key_shapes() {
        assert!(redact("key is xai-AbCdEf123456789012345678 done").contains("[REDACTED]"));
        assert!(redact("Bearer abcdef123456789012345678").contains("[REDACTED]"));
        assert!(redact("mail me a@b.com now").contains("[REDACTED]"));
        assert_eq!(redact("clean text"), "clean text");
    }

    #[test]
    fn egress_chain_appends_and_verifies() {
        // runs in test cwd; isolated by unique temp not needed — append-only
        let before = std::fs::read_to_string(".kineti/egress.jsonl").unwrap_or_default();
        record("unit.test", "test send", "abc");
        let after = std::fs::read_to_string(".kineti/egress.jsonl").unwrap_or_default();
        assert!(after.len() > before.len());
        let lines: Vec<&str> = after.lines().collect();
        let last: serde_json::Value = serde_json::from_str(lines.last().unwrap()).unwrap();
        assert!(last["hash"].as_str().is_some());
    }
}
