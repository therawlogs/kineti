//! Swarm orchestration (Phase 6): waves of parallel workers in isolated
//! trees, cooperative halt broadcast, panic isolation (§R3), then a strict
//! integration ladder (§R2): clean merges → ONE arbitrator attempt → human
//! escalation. All LLM-facing behavior is injected as closures so the
//! machinery is testable without any provider.

use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::plan::Task;
use crate::worktree::{self, Mode, Worktree};

/// What one worker run reported.
#[derive(Clone, Debug, PartialEq)]
pub enum TaskStatus {
    Complete,
    Failed(String),
    Panicked,
    StoppedBySibling(String),
}

pub struct WaveReport {
    /// id → status for every attempted task
    pub statuses: HashMap<String, TaskStatus>,
    /// ids that completed and still hold a mergeable tree
    pub completed: Vec<String>,
    /// trees destroyed during teardown of failed/panicked workers
    pub torn_down: Vec<String>,
    /// completed workers' trees, kept for integration + teardown by caller
    pub kept_trees: Vec<crate::worktree::Worktree>,
    pub halted: bool,
}

type Body<'a> = dyn Fn(&Worktree, &AtomicBool) -> Result<(), String> + Send + Sync + 'a;

/// Run `tasks` as dependency waves with at most `max_parallel` threads.
///
/// `bodies` maps task-id → the work that worker performs inside its isolated
/// tree. `Err(reason)` marks the worker FAILED; a panic marks it PANICKED
/// (§R3). Any failure sets the shared halt flag — siblings stop BETWEEN
/// iterations (their loop checks the flag), finish their current tool call,
/// and report `StoppedBySibling`. Failed/panicked workers lose their
/// worktree; completed workers keep theirs for integration.
pub fn run_waves(
    repo: &Path,
    tasks: Vec<Vec<Task>>,
    max_parallel: usize,
    bodies: &HashMap<String, &Body<'_>>,
    iso_mode: Mode,
) -> WaveReport {
    let halt = Arc::new(AtomicBool::new(false));
    let mut statuses = HashMap::new();
    let mut completed = Vec::new();
    let mut torn_down = Vec::new();
    let mut kept_trees: Vec<crate::worktree::Worktree> = Vec::new();

    // Scoped threads: workers borrow `halt` and the body closures directly —
    // no 'static gymnastics; every join completes before statuses are read.
    std::thread::scope(|s| {
        'waves: for wave in &tasks {
            for chunk in wave.chunks(max_parallel.max(1)) {
                if halt.load(Ordering::Relaxed) {
                    break 'waves;
                }

                let mut handles = Vec::new();
                for task in chunk {
                    let wt = match worktree::create(repo, &task.id, iso_mode) {
                        Ok(w) => w,
                        Err(e) => {
                            statuses.insert(task.id.clone(), TaskStatus::Failed(e));
                            continue;
                        }
                    };
                    let body = bodies.get(&task.id).copied();
                    let halt = halt.clone();
                    let handle = std::thread::Builder::new()
                        .name(format!("worker-{}", task.id))
                        .spawn_scoped(s, move || {
                            let work = match body {
                                None => Err("no worker body registered".into()),
                                Some(b) => match catch_unwind(AssertUnwindSafe(|| b(&wt, &halt))) {
                                    Ok(r) => r,
                                    Err(_) => Err("PANICKED".into()), // §R3 containment
                                },
                            };
                            (wt, work)
                        });
                    handles.push((task.id.clone(), handle));
                }

                for (id, handle) in handles {
                    let res: Result<(Worktree, Result<(), String>), String> = match handle {
                        Ok(h) => h.join().map_err(|_| "worker thread crashed hard".to_string()),
                        Err(e) => Err(format!("spawn failure: {e}")),
                    };
                    let (wt, work) = match res {
                        Ok(pair) => pair,
                        Err(e) => (placeholder(repo, &id), Err(e)),
                    };
                    match work {
                        Ok(()) => {
                            statuses.insert(id.clone(), TaskStatus::Complete);
                            completed.push(id);
                            kept_trees.push(wt.clone());
                        }
                        Err(reason) => {
                            halt.store(true, Ordering::Relaxed);
                            let panicked = reason == "PANICKED";
                            // §R3: a failed/panicked worker loses ONLY its own tree
                            if wt.path.exists() {
                                if let Err(e) = worktree::destroy(repo, &wt) {
                                    eprintln!("⚠ teardown of '{id}' incomplete: {e}");
                                } else {
                                    torn_down.push(wt.id.clone());
                                }
                            }
                            statuses.insert(
                                id.clone(),
                                if panicked {
                                    TaskStatus::Panicked
                                } else {
                                    TaskStatus::Failed(reason.clone())
                                },
                            );
                            eprintln!(
                                "⛔ worker '{id}' {} ({reason}) — halting wave",
                                if panicked { "panicked" } else { "failed" }
                            );
                        }
                    }
                }
            }
        }
    });

    // mark never-started tasks when a halt cut the waves short
    for wave in &tasks {
        for t in wave {
            statuses
                .entry(t.id.clone())
                .or_insert_with(|| TaskStatus::StoppedBySibling("halted".into()));
        }
    }

    WaveReport {
        statuses,
        completed,
        torn_down,
        kept_trees,
        halted: halt.load(Ordering::Relaxed),
    }
}

fn placeholder(repo: &Path, id: &str) -> Worktree {
    Worktree {
        id: id.to_string(),
        path: worktree::worktree_path(repo, id),
        mode: Mode::Scratchpad,
    }
}

// ── integration ladder (§R2) ─────────────────────────────────────────────────

#[derive(Debug)]
pub enum Integration {
    Merged(Vec<String>),
    Conflict { worker: String, files: Vec<String> },
}

/// Sequentially `git merge --no-ff kineti/<id>` in dependency order.
/// Stops at the FIRST conflict, reporting its file list. Already-merged
/// branches are skipped via merge-base ancestry (idempotent resume).
pub fn integrate(repo: &Path, worker_ids: &[String]) -> Result<Integration, String> {
    let mut merged = Vec::new();
    for id in worker_ids {
        let branch = format!("kineti/{id}");
        if already_ancestor(repo, &branch)? {
            merged.push(id.clone());
            continue;
        }
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["merge", "--no-ff", "-m", &format!("merge worker {id}")])
            .arg(&branch)
            .output()
            .map_err(|e| format!("git spawn: {e}"))?;
        if out.status.success() {
            merged.push(id.clone());
            continue;
        }
        return Ok(Integration::Conflict { worker: id.clone(), files: unmerged_files(repo) });
    }
    Ok(Integration::Merged(merged))
}

fn already_ancestor(repo: &Path, branch: &str) -> Result<bool, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge-base", "--is-ancestor", branch, "HEAD"])
        .output()
        .map_err(|e| format!("git spawn: {e}"))?;
    Ok(out.status.success())
}

pub fn unmerged_files(repo: &Path) -> Vec<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["diff", "--name-only", "--diff-filter=U"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    out.lines().filter(|l| !l.trim().is_empty()).map(String::from).collect()
}

pub fn conflict_markers_present(repo: &Path) -> bool {
    unmerged_files(repo).is_empty() && scan_markers(repo)
}

fn scan_markers(repo: &Path) -> bool {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["grep", "-l", "^<<<<<<< ", "--", "."])
        .output()
        .ok()
        .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
        .unwrap_or(false);
    out
}

/// Fixer/verifier closures injected into [`arbitrate_once`].
pub type Resolver<'a> = &'a (dyn Fn(&Path, &[String]) -> Result<(), String> + Send + Sync);
pub type Verifier<'a> = &'a (dyn Fn(&Path) -> Result<(), String> + Send + Sync);

/// The ONE arbitrator attempt slot (§R2). `resolve` is the LLM-backed fixer;
/// tests inject fakes. After resolution, `verify` must pass or we escalate.
pub fn arbitrate_once(
    repo: &Path,
    worker: &str,
    resolve: Resolver<'_>,
    verify: Verifier<'_>,
) -> Result<(), String> {
    let files = unmerged_files(repo);
    resolve(repo, &files).map_err(|e| format!("arbitrator failed on {worker}: {e}"))?;
    if !unmerged_files(repo).is_empty() || scan_markers(repo) {
        return Err(format!(
            "arbitrator left conflicts behind after {worker} — escalating to human"
        ));
    }
    verify(repo).map_err(|e| format!("post-arbitration verification failed: {e}"))
}

/// Abort an in-progress conflicted merge back to pre-merge state.
pub fn abort_merge(repo: &Path) {
    let _ = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["merge", "--abort"])
        .output();
}

// ── progress persistence (resume-friendly) ───────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Default, Debug)]
pub struct Progress {
    pub merged: Vec<String>,
    pub pending: Vec<String>,
}

pub fn progress_path(root: &Path) -> std::path::PathBuf {
    root.join(".kineti/swarm_progress.json")
}

pub fn save_progress(root: &Path, p: &Progress) -> Result<(), String> {
    let path = progress_path(root);
    if let Some(d) = path.parent() {
        let _ = std::fs::create_dir_all(d);
    }
    std::fs::write(&path, serde_json::to_string_pretty(p).unwrap())
        .map_err(|e| format!("progress write: {e}"))
}

pub fn load_progress(root: &Path) -> Option<Progress> {
    std::fs::read_to_string(progress_path(root))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

pub fn clear_progress(root: &Path) {
    let _ = std::fs::remove_file(progress_path(root));
}
