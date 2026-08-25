//! Phase 6 acceptance: wave orchestration with injected bodies (no provider),
//! panic isolation §R3, integration ladder §R2 including the ONE arbitrator
//! attempt and human escalation, plus progress persistence.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::AtomicBool;

use kineti::plan::{self, Plan, Task};
use kineti::swarm::{
    abort_merge, arbitrate_once, integrate, load_progress, run_waves, save_progress,
    clear_progress, Integration, Progress, TaskStatus,
};
use kineti::worktree::{self, Mode, Worktree};

// ── fixtures ─────────────────────────────────────────────────────────────────

fn tmp(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp6-{tag}-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("git spawn");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_repo(tag: &str) -> PathBuf {
    let d = tmp(tag);
    git(&d, &["init", "-q", "-b", "main"]);
    std::fs::write(d.join("shared.txt"), "line: base\n").unwrap();
    git(&d, &["add", "."]);
    git(
        &d,
        &[
            "-c", "user.email=f@local", "-c", "user.name=F",
            "commit", "-q", "-m", "seed",
        ],
    );
    d
}

fn head(repo: &Path) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// Simulate a COMPLETED worker by committing its scope onto kineti/<id>.
fn fake_completed_worker(repo: &Path, id: &str, file: &str, content: &str) {
    let start = head(repo);
    git(repo, &["checkout", "-q", "-B", &format!("kineti/{id}"), &start]);
    if let Some(parent) = Path::new(file).parent() {
        std::fs::create_dir_all(repo.join(parent)).unwrap();
    }
    std::fs::write(repo.join(file), content).unwrap();
    git(repo, &["add", "."]);
    git(
        repo,
        &[
            "-c", "user.email=w@local", "-c", "user.name=W",
            "commit", "-q", "-m", &format!("work {id}"),
        ],
    );
    git(repo, &["checkout", "-q", "main"]);
}

type Spec<'a> = (&'a str, &'a [&'a str], &'a [&'a str]);

fn tasks(specs: &[Spec]) -> Vec<Task> {
    specs
        .iter()
        .map(|(id, scopes, deps)| Task {
            id: id.to_string(),
            title: format!("t-{id}"),
            scopes: scopes.iter().map(|s| s.to_string()).collect(),
            deps: deps.iter().map(|s| s.to_string()).collect(),
        })
        .collect()
}

// ── orchestration mechanics ──────────────────────────────────────────────────

#[test]
fn parallel_workers_complete_and_keep_their_trees() {
    let repo = git_repo("waves-ok");
    let waves = vec![tasks(&[
        ("w1", &["src/a/**"], &[]),
        ("w2", &["src/b/**"], &[]),
    ])];

    type Body = dyn Fn(&Worktree, &AtomicBool) -> Result<(), String> + Send + Sync;
    let bodies: HashMap<String, &Body> = [
        (
            "w1".to_string(),
            &(|wt: &Worktree, _: &AtomicBool| {
                std::fs::write(wt.path.join("a_out.txt"), "A").unwrap();
                Ok(())
            }) as &Body,
        ),
        (
            "w2".to_string(),
            &(|wt: &Worktree, _: &AtomicBool| {
                std::fs::write(wt.path.join("b_out.txt"), "B").unwrap();
                Ok(())
            }) as &Body,
        ),
    ]
    .into();

    // NOTE: run_waves creates REAL worktrees under the repo — needs git mode
    let report = run_waves(&repo, waves.clone(), 2, &bodies, Mode::Auto);
    assert!(!report.halted);
    assert_eq!(report.statuses["w1"], TaskStatus::Complete);
    assert_eq!(report.statuses["w2"], TaskStatus::Complete);
    assert!(report.torn_down.is_empty());
    // trees survive for integration
    assert!(worktree::worktree_path(&repo, "w1").exists());
    assert!(worktree::worktree_path(&repo, "w2").exists());

    // cleanup for later assertions on this repo instance
    let _ = std::fs::remove_dir_all(worktree::worktree_path(&repo, "w1"));
    let _ = std::fs::remove_dir_all(worktree::worktree_path(&repo, "w2"));
}

#[test]
fn panicking_worker_is_contained_and_torn_down() {
    let repo = git_repo("waves-panic");
    let waves = vec![tasks(&[("boom", &["src/x/**"], &[])])];

    type Body = dyn Fn(&Worktree, &AtomicBool) -> Result<(), String> + Send + Sync;
    let bodies: HashMap<String, &Body> = [(
        "boom".to_string(),
        &(|_: &Worktree, _: &AtomicBool| -> Result<(), String> { panic!("worker exploded") }) as &Body,
    )]
    .into();

    let before = head(&repo);
    let report = run_waves(&repo, waves, 1, &bodies, Mode::Auto); // must NOT crash
    assert_eq!(report.statuses["boom"], TaskStatus::Panicked);
    assert!(report.halted);
    assert!(!worktree::worktree_path(&repo, "boom").exists(), "tree torn down");
    assert_eq!(head(&repo), before, "main untouched");
}

#[test]
fn failing_worker_halts_later_tasks() {
    let repo = git_repo("waves-fail");
    // T_bad first, dependent T_after second wave → never starts
    let waves = vec![
        tasks(&[("bad", &["x/**"], &[])]),
        tasks(&[("after", &["y/**"], &["bad"])]),
    ];
    type Body = dyn Fn(&Worktree, &AtomicBool) -> Result<(), String> + Send + Sync;
    let bodies: HashMap<String, &Body> = [(
        "bad".to_string(),
        &(|_: &Worktree, _: &AtomicBool| Err("qa failed".into())) as &Body,
    )]
    .into();

    let report = run_waves(&repo, waves, 2, &bodies, Mode::Auto);
    match &report.statuses["bad"] {
        TaskStatus::Failed(r) => assert!(r.contains("qa failed"), "{r}"),
        other => panic!("expected Failed, got {other:?}"),
    }
    assert!(matches!(
        report.statuses["after"],
        TaskStatus::StoppedBySibling(_)
    ));
    assert!(report.halted);
}

// ── integration ladder §R2 ───────────────────────────────────────────────────

#[test]
fn clean_sequential_merge_of_two_workers() {
    let repo = git_repo("integ-ok");
    fake_completed_worker(&repo, "w1", "feat_a.txt", "A\n");
    fake_completed_worker(&repo, "w2", "feat_b.txt", "B\n");

    match integrate(&repo, &["w1".into(), "w2".into()]).unwrap() {
        Integration::Merged(m) => assert_eq!(m, vec!["w1", "w2"]),
        other => panic!("unexpected {other:?}"),
    }
    assert!(repo.join("feat_a.txt").exists());
    assert!(repo.join("feat_b.txt").exists());

    // idempotent resume: merging again is a no-op success
    match integrate(&repo, &["w1".into(), "w2".into()]).unwrap() {
        Integration::Merged(m) => assert_eq!(m, vec!["w1", "w2"]),
        other => panic!("resume must skip merged branches, got {other:?}"),
    }
}

#[test]
fn conflict_detected_then_arbitrator_fixes_once() {
    let repo = git_repo("integ-conflict");
    // both workers edit the SAME line of shared.txt
    fake_completed_worker(&repo, "wa", "shared.txt", "line: alpha\n");
    fake_completed_worker(&repo, "wb", "shared.txt", "line: beta\n");
    // wa's branch also carries a distinct feature so the merge is non-trivial
    let _ = std::fs::remove_file(repo.join("only_a.txt"));

    match integrate(&repo, &["wa".into()]).unwrap() {
        Integration::Merged(_) => {}
        other => panic!("first merge should be clean, got {other:?}"),
    }
    // wb conflicts on shared.txt
    match integrate(&repo, &["wb".into()]).unwrap() {
        Integration::Conflict { worker, files } => {
            assert_eq!(worker, "wb");
            assert!(files.iter().any(|f| f.contains("shared.txt")), "{files:?}");
        }
        other => panic!("expected conflict, got {other:?}"),
    }

    // ONE arbitrator attempt: resolve + stage + verify passes
    let resolver = |r: &Path, _files: &[String]| -> Result<(), String> {
        std::fs::write(r.join("shared.txt"), "line: alpha-beta (merged)\n").unwrap();
        git(r, &["add", "."]);
        Ok(())
    };
    let verifier = |_r: &Path| Ok(());
    arbitrate_once(&repo, "wb", &resolver, &verifier).expect("arbitration succeeds");

    // commit the resolution so the ladder can continue
    git(&repo, &["commit", "-q", "--no-edit"]);

    // a fresh integrate call sees wb fully merged now
    match integrate(&repo, &["wb".into()]).unwrap() {
        Integration::Merged(m) => assert_eq!(m, vec!["wb"]),
        other => panic!("{other:?}"),
    }
}

#[test]
fn arbitrator_leaving_markers_escalates_exactly_once() {
    let repo = git_repo("integ-escalate");
    fake_completed_worker(&repo, "wa", "shared.txt", "line: alpha\n");
    fake_completed_worker(&repo, "wb", "shared.txt", "line: beta\n");
    let _ = integrate(&repo, &["wa".into()]);
    let conflict = integrate(&repo, &["wb".into()]).unwrap();
    assert!(matches!(conflict, Integration::Conflict { .. }));

    let calls = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let calls2 = calls.clone();
    let bad_resolver = move |_: &Path, _: &[String]| -> Result<(), String> {
        calls2.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        // "fixes" nothing — markers remain staged-resolved? leave unmerged
        Ok(())
    };
    let verifier = |_r: &Path| Ok(());
    let err = arbitrate_once(&repo, "wb", &bad_resolver, &verifier).unwrap_err();
    assert!(err.contains("escalating"), "{err}");
    assert_eq!(
        calls.load(std::sync::atomic::Ordering::SeqCst),
        1,
        "§R2: exactly ONE attempt"
    );

    abort_merge(&repo); // human-style recovery
    // abort rewinds only wb's conflicted merge — wa's earlier merge remains
    assert_eq!(
        std::fs::read_to_string(repo.join("shared.txt")).unwrap(),
        "line: alpha\n"
    );
}

#[test]
fn post_arbitration_verify_failure_escalates() {
    let repo = git_repo("integ-verifyfail");
    fake_completed_worker(&repo, "wa", "shared.txt", "line: alpha\n");
    fake_completed_worker(&repo, "wb", "shared.txt", "line: beta\n");
    let _ = integrate(&repo, &["wa".into()]);
    let _conflict = integrate(&repo, &["wb".into()]);

    let resolver = |r: &Path, _: &[String]| -> Result<(), String> {
        std::fs::write(r.join("shared.txt"), "resolved but broken\n").unwrap();
        git(r, &["add", "."]);
        Ok(())
    };
    let verifier = |_r: &Path| -> Result<(), String> { Err("tests red".into()) };
    let err = arbitrate_once(&repo, "wb", &resolver, &verifier).unwrap_err();
    assert!(err.contains("verification failed"), "{err}");
    abort_merge(&repo);
}

// ── progress persistence ─────────────────────────────────────────────────────

#[test]
fn progress_roundtrip_and_clear() {
    let d = tmp("progress");
    let p = Progress { merged: vec!["w1".into()], pending: vec!["w2".into()] };
    save_progress(&d, &p).unwrap();
    let loaded = load_progress(&d).unwrap();
    assert_eq!(loaded.merged, vec!["w1"]);
    clear_progress(&d);
    assert!(load_progress(&d).is_none());
}

// ── plan wiring smoke ────────────────────────────────────────────────────────

#[test]
fn partition_plan_survives_json_and_topo_sorting() {
    let doc = "## Task Partition\n\
               - T1: db | scope: src/db/** | deps: -\n\
               - T2: api | scope: src/api/** | deps: T1\n\
               - T3: ui | scope: src/ui/** | deps: T1\n";
    let plan: Plan = plan::parse_partition(doc).unwrap();
    assert_eq!(plan.tasks.len(), 3);
    let waves = plan::topo_waves(&plan).unwrap();
    assert_eq!(waves.len(), 2, "T1 first, then T2+T3 together");
    assert_eq!(waves[1].len(), 2);
}
