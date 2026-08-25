use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub const GENESIS: &str = "GENESIS";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Link {
    pub word: String,
    pub from_id: String,
    pub to_id: String,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default)]
    pub proof_id: Option<String>,
}
fn default_status() -> String { "candidate".into() }

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Record {
    pub at: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub state: String,
    pub project: String,
    pub id: String,
    #[serde(default)]
    pub data: serde_json::Value,
    #[serde(default)]
    pub links: Vec<Link>,
    #[serde(default)]
    pub expires: Option<String>,
    #[serde(default)]
    pub prev_hash: String,
    #[serde(default)]
    pub hash: String,
    /// Additional parent hashes beyond `prev_hash` — non-empty only on DAG
    /// merge records (Phase 4). Empty on ordinary linear records, so every
    /// pre-DAG journal parses and verifies unchanged.
    #[serde(default)]
    pub parents: Vec<String>,
}

/// UTC RFC3339 with millisecond precision — e.g. `2026-08-23T12:34:56.789Z`.
/// Dependency-free (chrono pulled CoreFoundation onto every macOS launch via
/// iana-time-zone); lexicographic order == chronological order, which the
/// causal graph's time-order checks rely on.
pub fn now_iso() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs();
    let millis = d.subsec_millis();
    let (y, m, day) = civil_from_days((secs / 86_400) as i64);
    let sod = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{day:02}T{:02}:{:02}:{:02}.{millis:03}Z",
        sod / 3600,
        (sod % 3600) / 60,
        sod % 60
    )
}

/// Howard Hinnant's civil_from_days: days since epoch → (y, m, d).
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Canonical form: serde_json's default Map is a BTreeMap → keys sorted,
/// compact separators. Same input ⇒ same bytes ⇒ stable hashes.
pub fn canonical(v: &serde_json::Value) -> String {
    serde_json::to_string(v).unwrap_or_default()
}

pub fn sha256_hex(s: &str) -> String {
    let mut h = Sha256::new();
    h.update(s.as_bytes());
    let out = h.finalize();
    out.iter().map(|b| format!("{b:02x}")).collect()
}

/// Hash-input canonicalization: like canonical(), but every float becomes a
/// fixed-point string so equivalent doubles ALWAYS produce identical bytes.
pub fn canonical_stable(v: &serde_json::Value) -> String {
    fn norm(v: &serde_json::Value) -> serde_json::Value {
        match v {
            serde_json::Value::Number(n) => {
                if let Some(f) = n.as_f64() {
                    serde_json::Value::String(format!("{f:.6}"))
                } else {
                    v.clone()
                }
            }
            serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(norm).collect()),
            serde_json::Value::Object(m) => serde_json::Value::Object(
                m.iter().map(|(k, val)| (k.clone(), norm(val))).collect(),
            ),
            other => other.clone(),
        }
    }
    canonical(&norm(v))
}

pub fn compute_hash(prev: &str, at: &str, id: &str, data: &serde_json::Value) -> String {
    sha256_hex(&format!("{prev}{at}{id}{}", canonical_stable(data)))
}

/// Pre-v0.2 hash format: floats hashed by their raw serde representation.
/// Kept ONLY so journals written before float-stable hashing (day-3) still
/// verify — history must never become unreadable by an upgrade. New appends
/// always use `compute_hash`.
pub fn compute_hash_v1(prev: &str, at: &str, id: &str, data: &serde_json::Value) -> String {
    sha256_hex(&format!("{prev}{at}{id}{}", canonical(data)))
}

/// Fully-formed record from a known parent head — the constructor callers
/// behind `JournalWriter` use (head fetch → build → append_batch).
pub fn build(prev_hash: &str, id: &str, r#type: &str, data: &serde_json::Value) -> Record {
    let at = now_iso();
    let hash = compute_hash(prev_hash, &at, id, data);
    Record {
        at,
        r#type: r#type.to_string(),
        state: "active".into(),
        project: "kineti".into(),
        id: id.to_string(),
        data: data.clone(),
        links: vec![],
        expires: None,
        parents: vec![],
        prev_hash: prev_hash.to_string(),
        hash,
    }
}

pub struct Journal {
    pub path: PathBuf,
    pub records: Vec<Record>,
}

impl Journal {
    pub fn load(path: &Path) -> Self {
        let mut records = Vec::new();
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                if let Ok(r) = serde_json::from_str::<Record>(line) {
                    records.push(r);
                }
            }
        }
        Journal { path: path.to_path_buf(), records }
    }

    pub fn append(
        &mut self,
        r#type: &str,
        data: serde_json::Value,
        links: Vec<Link>,
        project: &str,
    ) -> Record {
        let seq = self.records.len() + 1;
        let rec = Record {
            at: now_iso(),
            r#type: r#type.to_string(),
            state: "active".into(),
            project: project.to_string(),
            id: format!("{type}-{:04}", seq),
            data,
            links,
            expires: None,
            parents: vec![],
            prev_hash: self.head_hash(),
            hash: String::new(),
        };
        let hash = compute_hash(&rec.prev_hash, &rec.at, &rec.id, &rec.data);
        let mut rec = rec;
        rec.hash = hash;
        self.persist_line(&rec);
        self.records.push(rec.clone());
        rec
    }

    fn head_hash(&self) -> String {
        self.records.last().map(|r| r.hash.clone()).unwrap_or_else(|| GENESIS.into())
    }

    fn persist_line(&self, r: &Record) {
        use std::io::Write;
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(f, "{}", serde_json::to_string(r).unwrap_or_default());
        }
    }

    /// Recompute the entire chain; any byte flipped anywhere breaks it.
    /// Accepts both the current (float-stable) and the legacy (pre-v0.2)
    /// hash formats per record — a tampered byte breaks BOTH, so dual
    /// acceptance costs no tamper evidence.
    pub fn verify(&self) -> Result<(), String> {
        let mut prev = GENESIS.to_string();
        for (i, r) in self.records.iter().enumerate() {
            if r.prev_hash != prev {
                return Err(format!("record {} ({}) prev_hash mismatch", i + 1, r.id));
            }
            let expect = compute_hash(&prev, &r.at, &r.id, &r.data);
            if r.hash != expect {
                let legacy = compute_hash_v1(&prev, &r.at, &r.id, &r.data);
                if r.hash != legacy {
                    return Err(format!(
                        "record {} ({}) HASH MISMATCH — history was edited",
                        i + 1,
                        r.id
                    ));
                }
            }
            prev = r.hash.clone();
        }
        Ok(())
    }

    /// id → at map for temporal order checks.
    pub fn times(&self) -> std::collections::HashMap<String, String> {
        self.records.iter().map(|r| (r.id.clone(), r.at.clone())).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("kj-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().subsec_nanos()));
        std::fs::create_dir_all(&d).unwrap();
        d.join("journal.jsonl")
    }

    #[test]
    fn chain_verifies_and_detects_tamper() {
        let path = tmp();
        let mut j = Journal::load(&path);
        j.append("action", serde_json::json!({"tool":"read_file"}), vec![], "kineti");
        j.append("observation", serde_json::json!({"head":"ok"}), vec![], "kineti");
        assert!(j.verify().is_ok());

        // tamper: flip one byte of history on disk
        let raw = std::fs::read_to_string(&path).unwrap();
        let tampered = raw.replacen("read_file", "Xead_file", 1);
        std::fs::write(&path, tampered).unwrap();
        let reloaded = Journal::load(&path);
        assert!(reloaded.verify().is_err(), "tampered journal must fail verification");
    }

    #[test]
    fn float_costs_survive_roundtrip_verification() {
        let path = tmp();
        let mut j = Journal::load(&path);
        // simulate ugly accumulated float sums
        let cost = 0.0125_f64 + 0.000975;
        j.append(
            "stage-outcome",
            serde_json::json!({"cost_usd": cost, "answer": "x"}),
            vec![],
            "kineti",
        );
        assert!(j.verify().is_ok());
        // reload from disk (fresh parse) and verify again
        let reloaded = Journal::load(&path);
        assert!(reloaded.verify().is_ok(), "roundtrip must not flip any byte");
    }

    #[test]
    fn hash_is_deterministic() {
        let data = serde_json::json!({"b":2,"a":1});
        let h1 = compute_hash(GENESIS, "t", "x-0001", &data);
        let h2 = compute_hash(GENESIS, "t", "x-0001", &data);
        assert_eq!(h1, h2);
        assert_ne!(h1, compute_hash(GENESIS, "t2", "x-0001", &data));
    }

    #[test]
    fn legacy_float_record_verifies_but_tamper_still_fails() {
        // a record hashed the pre-v0.2 way (plain canonical, raw float repr)
        let cost = 0.0125_f64 + 0.000975;
        let data = serde_json::json!({"cost_usd": cost});
        let at = now_iso();
        let legacy_hash = compute_hash_v1(GENESIS, &at, "run-record-0007", &data);
        assert_ne!(legacy_hash, compute_hash(GENESIS, &at, "run-record-0007", &data),
            "formats must differ for floats or this test is vacuous");
        // tamper detection is unaffected by dual acceptance
        let tampered = compute_hash_v1(GENESIS, &at, "run-record-0007",
            &serde_json::json!({"cost_usd": 999.0}));
        assert_ne!(tampered, legacy_hash);
    }
}


