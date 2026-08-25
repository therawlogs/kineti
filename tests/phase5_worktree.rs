//! Phase 5 acceptance: workers get isolated trees (git worktree or
//! scratchpad), writes never leak between siblings or into the main tree,
//! destroy() is guarded against traversal/symlinks, and coordinator stage
//! documents are handed over as read-only inputs.

use std::path::{Path, PathBuf};
use std::process::Command;

use kineti::tools;
use kineti::worktree::{self, Mode, Worktree};

// ── fixture: a real git repo with one commit ─────────────────────────────────

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git spawn");
    assert!(out.status.success(), "git {args:?} failed: {}", String::from_utf8_lossy(&out.stderr));
}

fn git_repo(tag: &str) -> PathBuf {
    let d = tmp(tag);
    git(&d, &["init", "-q"]);
    git(
        &d,
        &[
            "-c",
            "user.email=fixture@local",
            "-c",
            "user.name=Fixture",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            "seed",
        ],
    );
    std::fs::write(d.join("shared.rs"), "fn base() {}\n").unwrap();
    git(&d, &["add", "."]);
    git(
        &d,
        &[
            "-c",
            "user.email=fixture@local",
            "-c",
            "user.name=Fixture",
            "commit",
            "-q",
            "-m",
            "code",
        ],
    );
    d
}

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp5-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    // NOT a git repo by default — plain project dir
    d
}

fn branch_exists(repo: &Path, id: &str) -> bool {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["branch", "--list", &format!("kineti/{id}")])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).contains(&format!("kineti/{id}"))
}

// ── id validation ────────────────────────────────────────────────────────────

#[test]
fn invalid_ids_are_refused_before_any_fs_action() {
    let repo = tmp("ids");
    for bad in ["../evil", "a/b", "", ".hidden", "-flag", ".", "..", "has space"] {
        let err = worktree::create(&repo, bad, Mode::Scratchpad).unwrap_err();
        assert!(err.contains("invalid worker id"), "{bad}: {err}");
    }
    // nothing was created for any rejected id
    assert!(worktree::list(&repo).is_empty());
}

// ── git worktree mode ────────────────────────────────────────────────────────

#[test]
fn git_worktree_is_isolated_until_merged() {
    let repo = git_repo("iso");
    std::fs::create_dir_all(repo.join(".kineti/stages")).unwrap();
    std::fs::write(repo.join(".kineti/stages/spec.md"), "# approved spec").unwrap();

    let w = worktree::create(&repo, "alpha", Mode::Git).unwrap();
    assert_eq!(w.mode, Mode::Git);
    assert!(w.path.is_dir());
    assert_eq!(std::fs::read_to_string(w.path.join("shared.rs")).unwrap(), "fn base() {}\n");
    assert!(branch_exists(&repo, "alpha"));

    // coordinator inputs handed over; mutable governance state virgin
    assert_eq!(
        std::fs::read_to_string(w.path.join(".kineti/stages/spec.md")).unwrap(),
        "# approved spec"
    );
    assert!(!w.path.join(".kineti/journal.jsonl").exists());

    // worker edits stay inside its tree
    std::fs::write(w.path.join("shared.rs"), "fn changed_by_alpha() {}\n").unwrap();
    assert_eq!(
        std::fs::read_to_string(repo.join("shared.rs")).unwrap(),
        "fn base() {}\n",
        "main tree untouched by worker write"
    );

    // second worker cannot see the first's changes
    let w2 = worktree::create(&repo, "beta", Mode::Git).unwrap();
    assert_eq!(
        std::fs::read_to_string(w2.path.join("shared.rs")).unwrap(),
        "fn base() {}\n"
    );

    // fence scopes to the WORKER root like any other root
    assert!(tools::resolve_in_root(&w.path, "shared.rs").is_ok());
    assert!(tools::resolve_in_root(&w.path, "../../escape.txt").is_err());

    // list sees both
    let mut ids = worktree::list(&repo);
    assert_eq!(ids, vec!["alpha".to_string(), "beta".to_string()]);

    // teardown removes tree + branch, leaves main and sibling intact
    worktree::destroy(&repo, &w).unwrap();
    assert!(!w.path.exists());
    assert!(!branch_exists(&repo, "alpha"));
    assert!(w2.path.exists());
    assert_eq!(
        std::fs::read_to_string(repo.join("shared.rs")).unwrap(),
        "fn base() {}\n"
    );
    worktree::destroy(&repo, &w2).unwrap();
    ids = worktree::list(&repo);
    assert!(ids.is_empty());
}

#[test]
fn failed_worker_teardown_does_not_touch_sibling() {
    let repo = git_repo("failteardown");
    let a = worktree::create(&repo, "a-good", Mode::Git).unwrap();
    let b = worktree::create(&repo, "b-bad", Mode::Git).unwrap();

    // both workers mutate their copies; b also creates untracked files
    std::fs::write(a.path.join("a_feature.rs"), "// A's work").unwrap();
    std::fs::write(b.path.join("b_feature.rs"), "// B's broken work").unwrap();
    std::fs::create_dir_all(b.path.join("src/deep")).unwrap();
    std::fs::write(b.path.join("src/deep/x.rs"), "// more of B").unwrap();

    // B fails QA → its saga would roll back inside ITS root (Phase 6 wires
    // that); here we prove the destructive half: destroy(B) is surgical.
    worktree::destroy(&repo, &b).unwrap();
    assert!(!b.path.exists());
    assert!(!branch_exists(&repo, "b-bad"));

    assert!(a.path.exists());
    assert_eq!(
        std::fs::read_to_string(a.path.join("a_feature.rs")).unwrap(),
        "// A's work",
        "sibling work preserved byte-for-byte"
    );
    worktree::destroy(&repo, &a).unwrap();
}

// ── scratchpad mode + auto fallback ─────────────────────────────────────────

#[test]
fn scratchpad_mode_works_without_git_and_copies_cleanly() {
    let proj = tmp("scratch"); // deliberately NOT a git repo
    std::fs::write(proj.join("lib.py"), "print('base')\n").unwrap();
    std::fs::create_dir_all(proj.join("pkg")).unwrap();
    std::fs::write(proj.join("pkg/mod.py"), "x=1\n").unwrap();
    // junk that must NOT be copied
    std::fs::create_dir_all(proj.join("target/release")).unwrap();
    std::fs::write(proj.join("target/release/junk.bin"), "binary-ish").unwrap();

    let w = worktree::create(&proj, "py-worker", Mode::Scratchpad).unwrap();
    assert_eq!(w.mode, Mode::Scratchpad);
    assert_eq!(
        std::fs::read_to_string(w.path.join("pkg/mod.py")).unwrap(),
        "x=1\n"
    );
    assert!(!w.path.join("target").exists(), "build dirs excluded from copies");

    // symlink in source must not be followed into the copy
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("/etc/hosts", proj.join("dangerous_link")).unwrap();
        let _ = worktree::destroy(&proj, &w); // ignore; recreate fresh below
        let w2 = worktree::create(&proj, "py-worker2", Mode::Scratchpad).unwrap();
        assert!(!w2.path.join("dangerous_link").exists(), "symlinks never copied");
        worktree::destroy(&proj, &w2).unwrap();
    }

    worktree::destroy(&proj, &w).unwrap();
    assert!(!w.path.exists());
    // source project fully intact
    assert!(proj.join("lib.py").exists());
    assert!(proj.join("target/release/junk.bin").exists());
}

#[test]
fn auto_mode_falls_back_to_scratchpad_outside_git() {
    let proj = tmp("autofallback");
    std::fs::write(proj.join("f.txt"), "data").unwrap();
    let w = worktree::create(&proj, "auto-w", Mode::Auto).unwrap();
    assert_eq!(w.mode, Mode::Scratchpad, "no repo → scratchpad fallback");
    assert!(w.path.join("f.txt").exists());
    worktree::destroy(&proj, &w).unwrap();
}

#[test]
fn auto_mode_picks_git_inside_a_repo() {
    let repo = git_repo("autopick");
    let w = worktree::create(&repo, "auto-git", Mode::Auto).unwrap();
    assert_eq!(w.mode, Mode::Git);
    worktree::destroy(&repo, &w).unwrap();
    assert!(!branch_exists(&repo, "auto-git"));
}

// ── destroy() guards ─────────────────────────────────────────────────────────

#[test]
fn destroy_refuses_paths_outside_the_managed_dir() {
    let repo = git_repo("guard-path");
    let forged = Worktree {
        id: "innocent".into(),
        path: repo.join("somewhere-else/innocent"),
        mode: Mode::Git,
    };
    std::fs::create_dir_all(&forged.path).unwrap();
    let err = worktree::destroy(&repo, &forged).unwrap_err();
    assert!(err.contains("refusing"), "{err}");
    assert!(forged.path.exists(), "guarded target must survive");
    let _ = std::fs::remove_dir_all(forged.path);
}

#[test]
fn destroy_refuses_symlink_targets() {
    let repo = git_repo("guard-symlink");
    #[cfg(unix)]
    {
        let wt_dir = repo.join(".kineti/worktrees");
        std::fs::create_dir_all(&wt_dir).unwrap();
        let victim = tmp("victim-dir");
        let link = wt_dir.join("evil");
        std::os::unix::fs::symlink(&victim, &link).unwrap();

        let forged = Worktree { id: "evil".into(), path: link.clone(), mode: Mode::Scratchpad };
        let err = worktree::destroy(&repo, &forged).unwrap_err();
        assert!(err.contains("symlink"), "{err}");
        assert!(victim.exists(), "symlink must NOT be followed to its target");
        let _ = std::fs::remove_file(&link);
        let _ = std::fs::remove_dir_all(victim);
    }
}

#[test]
fn destroy_of_vanished_tree_still_prunes_git_state() {
    let repo = git_repo("gone");
    let w = worktree::create(&repo, "ghost", Mode::Git).unwrap();
    // simulate crashed teardown: raw rm without git knowing
    std::fs::remove_dir_all(&w.path).unwrap();
    worktree::destroy(&repo, &w).expect("tolerant of already-gone tree");
    assert!(!branch_exists(&repo, "ghost"), "stale branch pruned anyway");
}
