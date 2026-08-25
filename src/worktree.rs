//! Worktree isolation (Phase 5): each swarm worker gets a private directory
//! — a real `git worktree` when the project is a repo, a plain scratchpad
//! copy otherwise. Because `.kineti/` is gitignored, every worktree boots
//! with VIRGIN governance state: its own saga stack, journal chain, spend
//! view, and egress log. The existing root-scoped machinery (fence, evidence
//! fingerprints) needs zero changes — workers just point `LoopCtx.root` at
//! their worktree (wired by the Phase 6 orchestrator).
//!
//! Guards: ids are strictly validated (no traversal), destroy() refuses any
//! path that escapes `.kineti/worktrees/` or is a symlink, and copies never
//! follow symlinks.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Where worker trees live, relative to the project root.
pub const WORKTREES_DIR: &str = ".kineti/worktrees";

const SKIP_COPY: [&str; 4] = [".git", "target", "node_modules", ".kineti"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Try git worktree; fall back to scratchpad with a warning.
    Auto,
    /// Require git; error if unavailable.
    Git,
    /// Always copy (no git needed).
    Scratchpad,
}

#[derive(Clone, Debug)]
pub struct Worktree {
    pub id: String,
    pub path: PathBuf,
    /// Mode actually used after Auto resolution.
    pub mode: Mode,
}

pub fn worktrees_root(repo: &Path) -> PathBuf {
    repo.join(WORKTREES_DIR)
}

pub fn worktree_path(repo: &Path, id: &str) -> PathBuf {
    worktrees_root(repo).join(id)
}

fn branch_name(id: &str) -> String {
    format!("kineti/{id}")
}

fn valid_id(id: &str) -> Result<(), String> {
    let ok = !id.is_empty()
        && id.len() <= 32
        && id.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    if ok && id != "." && id != ".." {
        Ok(())
    } else {
        Err(format!(
            "invalid worker id '{id}': use 1-32 chars [A-Za-z0-9] then [A-Za-z0-9_-] \
             (ids become paths and branch names)"
        ))
    }
}

/// Create an isolated tree for worker `id`.
pub fn create(repo: &Path, id: &str, mode: Mode) -> Result<Worktree, String> {
    valid_id(id)?;
    let path = worktree_path(repo, id);
    if path.symlink_metadata().is_ok() {
        return Err(format!("worktree '{id}' already exists at {}", path.display()));
    }
    let _ = std::fs::create_dir_all(worktrees_root(repo));

    let resolved = match mode {
        Mode::Git => {
            git_add(repo, &path, id)?;
            Mode::Git
        }
        Mode::Scratchpad => {
            copy_tree(repo, &path)?;
            Mode::Scratchpad
        }
        Mode::Auto => match git_add(repo, &path, id) {
            Ok(()) => Mode::Git,
            Err(e) => {
                // clean partial state before falling back
                let _ = std::fs::remove_dir_all(&path);
                eprintln!("⚠ git worktree unavailable ({e}) — using scratchpad isolation");
                copy_tree(repo, &path)?;
                Mode::Scratchpad
            }
        },
    };

    // Coordinator artifacts (spec.md etc.) are READ-ONLY INPUTS for the
    // worker; mutable governance state stays virgin per-worktree.
    copy_stage_inputs(repo, &path)?;

    Ok(Worktree { id: id.to_string(), path, mode: resolved })
}

fn git_add(repo: &Path, path: &Path, id: &str) -> Result<(), String> {
    if !repo.join(".git").exists() {
        return Err("not a git repository".into());
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["worktree", "add", "-b", &branch_name(id)])
        .arg(path)
        .output()
        .map_err(|e| format!("git spawn: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git worktree add failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Copy coordinator stage documents into the fresh worktree.
fn copy_stage_inputs(repo: &Path, wt_path: &Path) -> Result<(), String> {
    let src = repo.join(".kineti/stages");
    if !src.is_dir() {
        return Ok(()); // nothing to hand over yet
    }
    let dst = wt_path.join(".kineti/stages");
    std::fs::create_dir_all(&dst).map_err(|e| format!("stages dir: {e}"))?;
    copy_recursive(&src, &dst, &[]).map_err(|e| format!("stage inputs: {e}"))
}

/// Recursive copy skipping build/VCS/governance dirs and NEVER following
/// symlinks (a symlinked file could point anywhere).
fn copy_tree(src: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| format!("mkdir {}: {e}", dst.display()))?;
    copy_recursive(src, dst, &SKIP_COPY)
}

fn copy_recursive(src: &Path, dst: &Path, skip: &[&str]) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| format!("readdir {}: {e}", src.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry.file_name().to_string_lossy().to_string();
        if skip.contains(&name.as_str()) {
            continue;
        }
        let from = entry.path();
        let to = dst.join(&name);
        let ft = entry.file_type().map_err(|e| e.to_string())?;
        if ft.is_symlink() {
            continue; // safety: never follow
        } else if ft.is_dir() {
            std::fs::create_dir_all(&to).map_err(|e| format!("mkdir {}: {e}", to.display()))?;
            copy_recursive(&from, &to, skip)?;
        } else {
            std::fs::copy(&from, &to)
                .map_err(|e| format!("copy {}: {e}", from.display()))?;
        }
    }
    Ok(())
}

/// List existing worker ids (directories under .kineti/worktrees).
pub fn list(repo: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(worktrees_root(repo)) {
        for e in entries.flatten() {
            if e.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                out.push(e.file_name().to_string_lossy().to_string());
            }
        }
    }
    out.sort();
    out
}

/// Remove a worktree and its kineti/<id> branch. Guards run BEFORE any
/// deletion: registered location must match, real directory (no symlink),
/// canonically inside `<repo>/.kineti/worktrees/`. Tolerant of already-gone
/// trees so failure-path teardown can be called unconditionally.
pub fn destroy(repo: &Path, wt: &Worktree) -> Result<(), String> {
    let expected = worktree_path(repo, &wt.id);
    if wt.path != expected {
        return Err(format!(
            "refusing: worktree '{}' registered at {} but expected {}",
            wt.id,
            wt.path.display(),
            expected.display()
        ));
    }

    let gone = !wt.path.symlink_metadata().is_ok();
    if !gone {
        let meta =
            std::fs::symlink_metadata(&wt.path).map_err(|e| format!("stat: {e}"))?;
        if meta.file_type().is_symlink() {
            return Err(format!(
                "refusing: {} is a symlink — not a managed worktree",
                wt.path.display()
            ));
        }
        let base = worktrees_root(repo);
        let base_canon = base.canonicalize().map_err(|e| format!("canonicalize base: {e}"))?;
        let path_canon =
            wt.path.canonicalize().map_err(|e| format!("canonicalize {}: {e}", wt.path.display()))?;
        if !path_canon.starts_with(&base_canon) {
            return Err(format!(
                "refusing: {} escapes {}",
                wt.path.display(),
                base.display()
            ));
        }
    }

    let result = match wt.mode {
        Mode::Git => {
            if gone {
                // path vanished (crashed teardown?) — prune stale admin data
                git(repo, &["worktree", "prune"])
                    .map_err(|e| format!("git worktree prune: {e}"))?;
                Ok(())
            } else {
                let path_str = wt.path.to_string_lossy().to_string();
                git(repo, &["worktree", "remove", "--force", &path_str])
            }
        }
        Mode::Scratchpad => {
            if gone {
                Ok(())
            } else {
                std::fs::remove_dir_all(&wt.path)
                    .map_err(|e| format!("remove {}: {e}", wt.path.display()))
            }
        }
        // Auto should have been resolved at create(); treat like Scratchpad defensively
        Mode::Auto => {
            if !gone {
                std::fs::remove_dir_all(&wt.path)
                    .map_err(|e| format!("remove {}: {e}", wt.path.display()))?;
            }
            Ok(())
        }
    };

    // branch cleanup best-effort AFTER the tree itself (worker branches may be
    // deliberately unmerged on the failure path → -D is intentional here)
    if wt.mode != Mode::Scratchpad {
        let _ = git(repo, &["branch", "-D", &branch_name(&wt.id)]);
    }

    // drop the worktrees parent dir when it's now empty (keep .kineti itself)
    let parent = worktrees_root(repo);
    if std::fs::read_dir(&parent)
        .map(|mut d| d.next().is_none())
        .unwrap_or(false)
    {
        let _ = std::fs::remove_dir(&parent);
    }

    result
}

fn git(repo: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| format!("git spawn: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}
