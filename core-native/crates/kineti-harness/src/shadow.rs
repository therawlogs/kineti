//! Git worktree workspace isolation.
//!
//! A worktree separates file edits from the primary checkout. It is not a
//! process, filesystem, credential, or network sandbox.

use std::path::{Path, PathBuf};
use std::process::Command;

/// A real Git worktree used for isolated file changes.
#[derive(Debug, Clone)]
pub struct ShadowWorkspace {
    /// Task identifier used for the worktree and branch.
    pub task_id: String,
    /// Branch created for this worktree.
    pub branch_name: String,
    /// Path to the worktree checkout.
    pub worktree_path: PathBuf,
    repo_root: PathBuf,
}

impl ShadowWorkspace {
    /// Creates a Git worktree under `.kineti/shadow/<task_id>`.
    ///
    /// Returns an error if `base_dir` is not the root of a Git repository, if
    /// the task ID is unsafe, if the target already exists, or if Git cannot
    /// create and verify the worktree. It never reports success for an empty
    /// fallback directory.
    pub fn create(base_dir: &Path, task_id: &str) -> std::io::Result<Self> {
        if !valid_task_id(task_id) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "task_id must contain 1-64 ASCII letters, digits, underscores, or hyphens",
            ));
        }

        let requested_root = base_dir.canonicalize()?;
        let root_output = Command::new("git")
            .args([
                "-C",
                requested_root.to_string_lossy().as_ref(),
                "rev-parse",
                "--show-toplevel",
            ])
            .output()?;
        if !root_output.status.success() {
            return Err(command_error("not a Git repository", &root_output.stderr));
        }
        let repo_root = PathBuf::from(String::from_utf8_lossy(&root_output.stdout).trim());
        let repo_root = repo_root.canonicalize()?;
        if repo_root != requested_root {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "base_dir must be the Git repository root",
            ));
        }

        let shadow_parent = repo_root.join(".kineti").join("shadow");
        std::fs::create_dir_all(&shadow_parent)?;
        let canonical_parent = shadow_parent.canonicalize()?;
        if !canonical_parent.starts_with(&repo_root) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "shadow directory resolves outside the Git repository",
            ));
        }

        let worktree_path = canonical_parent.join(task_id);
        if worktree_path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                format!(
                    "shadow worktree already exists: {}",
                    worktree_path.display()
                ),
            ));
        }
        let branch_name = format!("kineti-shadow/{task_id}");
        let output = Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                &branch_name,
                worktree_path.to_string_lossy().as_ref(),
                "HEAD",
            ])
            .current_dir(&repo_root)
            .output()?;
        if !output.status.success() {
            let _ = std::fs::remove_dir_all(&worktree_path);
            return Err(command_error("git worktree add failed", &output.stderr));
        }

        let verify = Command::new("git")
            .args([
                "-C",
                worktree_path.to_string_lossy().as_ref(),
                "rev-parse",
                "--show-toplevel",
            ])
            .output();
        let verified_root = match verify {
            Ok(result) if result.status.success() => {
                PathBuf::from(String::from_utf8_lossy(&result.stdout).trim()).canonicalize()?
            }
            Ok(result) => {
                cleanup_worktree(&repo_root, &worktree_path, &branch_name);
                return Err(command_error(
                    "created path is not a Git worktree",
                    &result.stderr,
                ));
            }
            Err(error) => {
                cleanup_worktree(&repo_root, &worktree_path, &branch_name);
                return Err(error);
            }
        };
        if verified_root != worktree_path.canonicalize()? {
            cleanup_worktree(&repo_root, &worktree_path, &branch_name);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Git reported a different root for the created worktree",
            ));
        }

        Ok(Self {
            task_id: task_id.to_string(),
            branch_name,
            worktree_path,
            repo_root,
        })
    }

    /// Removes this worktree and its task branch, returning failures to caller.
    pub fn rollback(self) -> std::io::Result<()> {
        let expected_parent = self
            .repo_root
            .join(".kineti")
            .join("shadow")
            .canonicalize()?;
        let actual_parent = self
            .worktree_path
            .parent()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "worktree path has no parent",
                )
            })?
            .canonicalize()?;
        let expected_path = expected_parent.join(&self.task_id);
        if actual_parent != expected_parent
            || !valid_task_id(&self.task_id)
            || self.worktree_path != expected_path
            || self.branch_name != format!("kineti-shadow/{}", self.task_id)
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "refusing to remove a worktree outside this task's shadow directory",
            ));
        }

        let remove = Command::new("git")
            .args([
                "worktree",
                "remove",
                "--force",
                self.worktree_path.to_string_lossy().as_ref(),
            ])
            .current_dir(&self.repo_root)
            .output()?;
        if !remove.status.success() {
            return Err(command_error("git worktree remove failed", &remove.stderr));
        }

        let delete_branch = Command::new("git")
            .args(["branch", "-D", &self.branch_name])
            .current_dir(&self.repo_root)
            .output()?;
        if !delete_branch.status.success() {
            return Err(command_error(
                "git branch delete failed",
                &delete_branch.stderr,
            ));
        }
        Ok(())
    }
}

fn valid_task_id(task_id: &str) -> bool {
    !task_id.is_empty()
        && task_id.len() <= 64
        && task_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn command_error(message: &str, stderr: &[u8]) -> std::io::Error {
    std::io::Error::other(format!(
        "{message}: {}",
        String::from_utf8_lossy(stderr).trim()
    ))
}

fn cleanup_worktree(repo_root: &Path, worktree_path: &Path, branch_name: &str) {
    let _ = Command::new("git")
        .args([
            "worktree",
            "remove",
            "--force",
            worktree_path.to_string_lossy().as_ref(),
        ])
        .current_dir(repo_root)
        .output();
    let _ = Command::new("git")
        .args(["branch", "-D", branch_name])
        .current_dir(repo_root)
        .output();
    let _ = std::fs::remove_dir_all(worktree_path);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn unique_scratch_dir(prefix: &str) -> PathBuf {
        let parent = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.kineti/test-shadow");
        std::fs::create_dir_all(&parent).expect("create project-local test scratch");
        loop {
            let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("{prefix}-{}-{id}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return path,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("create test directory: {error}"),
            }
        }
    }

    fn scratch_repo() -> PathBuf {
        let root = unique_scratch_dir("repo");
        for args in [
            vec!["init", "-q"],
            vec!["config", "user.name", "Kineti Test"],
            vec!["config", "user.email", "kineti-test@example.invalid"],
        ] {
            let output = Command::new("git")
                .args(args)
                .current_dir(&root)
                .output()
                .expect("run git");
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        std::fs::write(root.join("tracked.txt"), "initial\n").expect("write initial file");
        let output = Command::new("git")
            .args(["add", "tracked.txt"])
            .current_dir(&root)
            .output()
            .expect("git add");
        assert!(output.status.success());
        let output = Command::new("git")
            .args(["commit", "-m", "initial"])
            .current_dir(&root)
            .output()
            .expect("git commit");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        root
    }

    #[test]
    fn creates_a_real_worktree_and_removes_it() {
        let root = scratch_repo();
        let workspace = ShadowWorkspace::create(&root, "task_abc").expect("create shadow worktree");
        let verify = Command::new("git")
            .args([
                "-C",
                workspace.worktree_path.to_string_lossy().as_ref(),
                "rev-parse",
                "--is-inside-work-tree",
            ])
            .output()
            .expect("verify worktree");
        assert!(verify.status.success());
        assert_eq!(String::from_utf8_lossy(&verify.stdout).trim(), "true");
        assert!(workspace.worktree_path.exists());

        workspace.rollback().expect("remove shadow worktree");
        assert!(!root.join(".kineti/shadow/task_abc").exists());
        let status = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&root)
            .output()
            .expect("git status");
        assert!(status.status.success());
        assert!(
            status.stdout.is_empty(),
            "main checkout changed during worktree lifecycle"
        );
        std::fs::remove_dir_all(root).expect("remove scratch repo");
    }

    #[test]
    fn rejects_non_repository_without_creating_a_fallback_directory() {
        let dir = unique_scratch_dir("not-a-repo");
        let err = ShadowWorkspace::create(&dir, "task_abc").unwrap_err();
        assert!(err.to_string().contains("repository"));
        assert!(!dir.join(".kineti/shadow/task_abc").exists());
        std::fs::remove_dir_all(dir).expect("remove non-repo scratch");
    }

    #[test]
    fn rejects_unsafe_task_ids() {
        let root = scratch_repo();
        assert!(ShadowWorkspace::create(&root, "../escape").is_err());
        assert!(!root.join(".kineti/shadow/../escape").exists());
        std::fs::remove_dir_all(root).expect("remove scratch repo");
    }
}
