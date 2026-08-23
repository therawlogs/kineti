//! Spend circuit breaker (ASI-04): per-stage + global caps. Crossing a cap
//! halts mid-run; ONLY a human-created reset file resumes spending (ETHOS §3.3).

use std::path::Path;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Spend {
    pub total_usd: f64,
}

impl Spend {
    pub fn load(root: &Path) -> Spend {
        let p = root.join(".kineti/spend.json");
        std::fs::read_to_string(&p)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(Spend { total_usd: 0.0 })
    }

    pub fn save(&self, root: &Path) {
        let p = root.join(".kineti/spend.json");
        let _ = std::fs::create_dir_all(p.parent().unwrap());
        let _ = std::fs::write(p, serde_json::to_string(self).unwrap());
    }

    /// Human-only reset: consumes .kineti/spend.reset if present, zeroing spend.
    pub fn human_reset_if_requested(root: &Path) -> bool {
        let flag = root.join(".kineti/spend.reset");
        if flag.exists() {
            let _ = std::fs::remove_file(&flag);
            Spend { total_usd: 0.0 }.save(root);
            true
        } else {
            false
        }
    }

    pub fn pre_check(&self, global_usd: f64) -> Result<(), String> {
        if self.total_usd > global_usd {
            Err(format!(
                "SPEND BREAKER TRIPPED: ${:.4} exceeds cap ${:.2}. \
                 A human must create .kineti/spend.reset to resume.",
                self.total_usd, global_usd
            ))
        } else {
            Ok(())
        }
    }

    pub fn add(&mut self, cost_usd: f64, root: &Path) {
        self.total_usd += cost_usd;
        self.save(root);
    }
}
