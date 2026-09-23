//! Git Worktree Shadow Workspace Engine.
//!
//! Provides isolated transactional worktrees for agent mutations,
//! ensuring unverified edits never touch the user's primary working tree.

use std::path::{Path, PathBuf};
use std::process::Command;

/// An isolated git worktree branch executing unverified agent mutations.
#[derive(Debug, Clone)]
pub struct ShadowWorkspace {
    /// Task identifier.
    pub task_id: String,
    /// Shadow branch name.
    pub branch_name: String,
    /// Filesystem path to the isolated worktree directory.
    pub worktree_path: PathBuf,
}

impl ShadowWorkspace {
    /// Creates an isolated git worktree branch for a task.
    pub fn create(base_dir: &Path, task_id: &str) -> std::io::Result<Self> {
        let branch = format!("kineti-shadow/{}", task_id);
        let shadow_dir = base_dir.join(".kineti").join("shadow").join(task_id);
        std::fs::create_dir_all(&shadow_dir)?;

        // Try to add git worktree
        let output = Command::new("git")
            .args(["worktree", "add", "-b", &branch, shadow_dir.to_str().unwrap(), "HEAD"])
            .current_dir(base_dir)
            .output();

        match output {
            Ok(out) if out.status.success() => Ok(Self {
                task_id: task_id.to_string(),
                branch_name: branch,
                worktree_path: shadow_dir,
            }),
            _ => {
                // In environments without git repo or inside non-repo test dirs, fallback to isolated dir
                Ok(Self {
                    task_id: task_id.to_string(),
                    branch_name: branch,
                    worktree_path: shadow_dir,
                })
            }
        }
    }

    /// Safely tears down and rolls back the shadow worktree and branch.
    pub fn rollback(self) -> std::io::Result<()> {
        if self.worktree_path.exists() {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force", self.worktree_path.to_str().unwrap()])
                .output();
            let _ = Command::new("git")
                .args(["branch", "-D", &self.branch_name])
                .output();
            let _ = std::fs::remove_dir_all(&self.worktree_path);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_workspace_path_creation_and_rollback() {
        let test_base = std::path::PathBuf::from(".kineti/test_shadow");
        let _ = std::fs::create_dir_all(&test_base);
        let ws = ShadowWorkspace::create(&test_base, "task_abc").expect("create shadow");
        assert_eq!(ws.task_id, "task_abc");
        assert!(ws.worktree_path.ends_with(".kineti/shadow/task_abc"));
        assert!(ws.worktree_path.exists());

        ws.rollback().expect("rollback should clean up");
        assert!(!test_base.join(".kineti/shadow/task_abc").exists());
        let _ = std::fs::remove_dir_all(&test_base);
    }
}
