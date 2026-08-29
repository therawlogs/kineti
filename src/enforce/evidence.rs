//! Evidence (ETHOS §5, D1): verification results bind to a fingerprint of ANY artifacts.
//! Ship refuses STALE or MISSING proofs. Works for code, docs, datasets, configs — any work product.
//! Fingerprint respects [artifacts] in kineti.toml (include/exclude globs, max bytes).

use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Proof {
    pub v: String, // schema version, e.g. "1"
    pub fingerprint: String,
    pub command: String,
    pub passed: bool,
    pub exit_code: i32,
    pub at: String,
}

/// Fingerprint = sha256 over sorted (rel_path, content) of tracked artifact files.
/// Honors [artifacts] config (include/exclude/max_file_bytes) — generic for any agent.
pub fn fingerprint(root: &Path) -> String {
    let cfg = crate::config::Config::load_from(root).artifacts;
    fingerprint_with_config(root, &cfg)
}

pub fn fingerprint_with_config(root: &Path, cfg: &crate::config::Artifacts) -> String {
    let mut files = vec![];
    crate::tools::walk_all_with_config(root, cfg, &mut files);
    files.sort();
    let mut h = Sha256::new();
    for f in files {
        let rel = f.strip_prefix(root).unwrap_or(&f).to_string_lossy().to_string();
        let content = std::fs::read(&f).unwrap_or_default();
        h.update(rel.as_bytes());
        h.update(&content);
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn proof_path(root: &Path) -> std::path::PathBuf {
    root.join(".kineti/evidence.json")
}

pub fn record(root: &Path, cmd: &str, passed: bool, exit_code: i32) -> Proof {
    let proof = Proof {
        v: "1".into(),
        fingerprint: fingerprint(root),
        command: cmd.to_string(),
        passed,
        exit_code,
        at: crate::memory::journal::now_iso(),
    };
    let _ = std::fs::create_dir_all(root.join(".kineti"));
    let _ = std::fs::write(proof_path(root), serde_json::to_string_pretty(&proof).unwrap());
    proof
}

pub fn load(root: &Path) -> Option<Proof> {
    std::fs::read_to_string(proof_path(root))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Ship gate (ETHOS §10.3 / §5.2): fresh + passing proof on the CURRENT artifacts.
pub fn check_ship(root: &Path) -> Result<Proof, String> {
    let proof = load(root).ok_or_else(|| {
        "SHIP REFUSED — MISSING proof: no proof evidence exists. Run your proof command first (e.g. kineti evidence --cmd \"...\")."
            .to_string()
    })?;
    if !proof.passed {
        return Err(format!(
            "SHIP REFUSED — proof FAILED ({} exited {})",
            proof.command, proof.exit_code
        ));
    }
    let current = fingerprint(root);
    if current != proof.fingerprint {
        return Err(format!(
            "SHIP REFUSED — STALE proof: artifacts changed since {} ran at {}. Re-run proofs.",
            proof.command, proof.at
        ));
    }
    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_proof_detected() {
        let d = std::env::temp_dir().join(format!("kev-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&d);
        std::fs::write(d.join("code.txt"), "v1").unwrap();
        record(&d, "echo ok", true, 0);
        assert!(check_ship(&d).is_ok());

        // code changed → proof goes stale
        std::fs::write(d.join("code.txt"), "v2").unwrap();
        match check_ship(&d) {
            Err(e) => assert!(e.contains("STALE")),
            Ok(_) => panic!("stale proof must block ship"),
        }
    }
}
