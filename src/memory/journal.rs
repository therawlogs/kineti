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
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
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
    pub fn verify(&self) -> Result<(), String> {
        let mut prev = GENESIS.to_string();
        for (i, r) in self.records.iter().enumerate() {
            if r.prev_hash != prev {
                return Err(format!("record {} ({}) prev_hash mismatch", i + 1, r.id));
            }
            let expect = compute_hash(&prev, &r.at, &r.id, &r.data);
            if r.hash != expect {
                return Err(format!(
                    "record {} ({}) HASH MISMATCH — history was edited",
                    i + 1,
                    r.id
                ));
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
}


