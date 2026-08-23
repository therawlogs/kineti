//! Saga undo journal (ETHOS §4, D1): file mutations register their undo FIRST.
//! Rollback runs newest-first; a failed undo is logged and the rest continue.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Step {
    pub at: String,
    pub kind: String, // "file_backup" | "bash_note"
    pub path: Option<String>,
    /// prior content — None means the file did not exist before (undo = delete)
    pub prev_content: Option<String>,
    pub note: Option<String>,
}

pub struct Saga {
    pub path: PathBuf,
    pub steps: Vec<Step>,
}

impl Saga {
    pub fn load(root: &Path) -> Self {
        let path = root.join(".kineti/saga.jsonl");
        let mut steps = Vec::new();
        if let Ok(c) = std::fs::read_to_string(&path) {
            for l in c.lines() {
                if let Ok(s) = serde_json::from_str::<Step>(l) {
                    steps.push(s);
                }
            }
        }
        Saga { path, steps }
    }

    fn persist(&self, s: &Step) {
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&self.path) {
            let _ = writeln!(f, "{}", serde_json::to_string(s).unwrap());
        }
    }

    /// ETHOS §4.1: capture prior content BEFORE mutation.
    pub fn register_file_backup(&mut self, abs_path: &Path) {
        let prev_content = std::fs::read_to_string(abs_path).ok();
        let step = Step {
            at: crate::memory::journal::now_iso(),
            kind: "file_backup".into(),
            path: Some(abs_path.to_string_lossy().to_string()),
            prev_content,
            note: None,
        };
        self.persist(&step);
        self.steps.push(step);
    }

    pub fn register_bash_note(&mut self, cmd: &str) {
        let step = Step {
            at: crate::memory::journal::now_iso(),
            kind: "bash_note".into(),
            path: None,
            prev_content: None,
            note: Some(format!("bash ran: {cmd} — manual review for undo")),
        };
        self.persist(&step);
        self.steps.push(step);
    }

    /// Rollback newest-first. Failed undos are logged, never abort the rest.
    pub fn rollback_all(&self) -> usize {
        let mut undone = 0;
        for step in self.steps.iter().rev() {
            match step.kind.as_str() {
                "file_backup" => {
                    let p = Path::new(step.path.as_deref().unwrap_or(""));
                    let res = match &step.prev_content {
                        Some(prev) => {
                            if let Some(d) = p.parent() {
                                let _ = std::fs::create_dir_all(d);
                            }
                            std::fs::write(p, prev)
                        }
                        None => {
                            if p.exists() {
                                std::fs::remove_file(p)
                            } else {
                                Ok(())
                            }
                        }
                    };
                    match res {
                        Ok(()) => undone += 1,
                        Err(e) => eprintln!("   ⚠ undo failed for {}: {e} — continuing", p.display()),
                    }
                }
                _ => {} // notes have no mechanical undo
            }
        }
        undone
    }

    /// Clear the journal after a successful rollback cycle or explicit commit.
    pub fn clear(&mut self) {
        self.steps.clear();
        let _ = std::fs::remove_file(&self.path);
    }
}
