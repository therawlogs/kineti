use std::path::Path;

use crate::memory::journal::sha256_hex;

/// ETHOS §2.1 — the run's goal is written once and can never be edited.
/// Returns the goal's sha256 (pinned into every envelope + record).
pub fn ensure_goal(root: &Path, goal: &str) -> Result<String, String> {
    let dir = root.join(".kineti");
    std::fs::create_dir_all(&dir).map_err(|e| format!("{e}"))?;
    let file = dir.join("root_goal");
    match std::fs::read_to_string(&file) {
        Ok(existing) => {
            if existing.trim() != goal.trim() {
                return Err(format!(
                    "ROOT GOAL IS IMMUTABLE: refusing to replace\n  locked: {existing}\n  new   : {goal}"
                ));
            }
        }
        Err(_) => {
            std::fs::write(&file, goal).map_err(|e| format!("{e}"))?;
        }
    }
    Ok(sha256_hex(goal.trim()))
}
