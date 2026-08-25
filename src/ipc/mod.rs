//! Transport abstraction (§R1 — one code path). All domain logic lives in
//! `pool::Pool`, the journal file layer, and callers; `DirectBackend` and
//! `SocketBackend` are two shells around the same brain. The daemon runs the
//! SAME `DirectBackend` types internally.

pub mod dto;
pub mod pool;

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::memory::journal::{Record, GENESIS};

/// Env: skip every socket attempt and run the direct implementation.
pub const ENV_FORCE_DIRECT: &str = "KINETI_FORCE_DIRECT";
/// Env: never spawn a daemon; fall back to direct when no socket answers.
pub const ENV_NO_DAEMON: &str = "KINETI_NO_DAEMON";

pub fn socket_path(root: &Path) -> PathBuf {
    root.join(".kineti/kineti.sock")
}


// ── Ledger lock: direct mode must never run two pools on one project ────────

/// Advisory exclusive lock on `.kineti/spend.lock`. Held for the backend's
/// lifetime so two standalone CLIs can never race the ledger (cap bypass via
/// lost updates). Contention spins up to `timeout`, then FAILS CLOSED —
/// the correct answer for a shared governed ledger is the daemon.
pub struct LedgerLock {
    file: std::fs::File,
}

impl LedgerLock {
    pub const LOCK_PATH: &str = ".kineti/spend.lock";

    fn acquire(root: &Path, timeout: Duration) -> Result<Self, String> {
        let path = root.join(Self::LOCK_PATH);
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&path)
            .map_err(|e| format!("LEDGER LOCK: cannot open {}: {e}", path.display()))?;
        let deadline = Instant::now() + timeout;
        loop {
            match file.try_lock() {
                Ok(()) => return Ok(LedgerLock { file }),
                Err(std::fs::TryLockError::WouldBlock) => {}
                Err(std::fs::TryLockError::Error(e)) => {
                    return Err(format!("LEDGER LOCK: {}: {e}", path.display()))
                }
            }
            if Instant::now() >= deadline {
                return Err(format!(
                    "LEDGER LOCK: another kineti process holds {} and did not release it within {}s. \
                     Run `kineti serve` so concurrent runs share one governed pool, or wait for it to exit.",
                    path.display(),
                    timeout.as_secs()
                ));
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}

impl Drop for LedgerLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

// ── Traits ───────────────────────────────────────────────────────────────────

pub trait SpendService: Send + Sync {
    fn reserve(&self, ctx: &dto::ReserveCtx) -> Result<dto::Reservation, String>;
    /// Returns the post-settle total.
    fn settle(&self, res: &dto::Reservation, actual_micro_usd: u64) -> Result<u64, String>;
    fn snapshot(&self) -> Result<dto::SpendSnapshot, String>;
    /// ETHOS §3.3: consumes .kineti/spend.reset if a human created it.
    fn reset_if_human_requested(&self, root: &Path) -> Result<bool, String>;
}

pub trait JournalWriter: Send {
    /// branch "" = main chain; any other name → journal.<branch>.jsonl.
    fn head(&self, branch: &str) -> Result<String, String>;
    /// Records must already carry correct hashes; the writer enforces that
    /// each `prev_hash` matches the current tail before accepting ANY of them.
    fn append_batch(&self, branch: &str, records: Vec<Record>) -> Result<(), String>;
}

// ── Framing: newline-delimited JSON over the UDS stream ─────────────────────

fn send_recv(stream: &UnixStream, req: &dto::Req) -> Result<dto::Resp, String> {
    let line = serde_json::to_string(req).map_err(|e| e.to_string())?;
    let mut s = stream.try_clone().map_err(|e| e.to_string())?;
    s.set_write_timeout(Some(Duration::from_secs(5))).ok();
    writeln!(s, "{line}").map_err(|e| format!("socket write: {e}"))?;
    stream.set_read_timeout(Some(Duration::from_secs(30))).ok();
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader.read_line(&mut buf).map_err(|e| format!("socket read: {e}"))?;
    if buf.is_empty() {
        return Err("daemon closed connection".into());
    }
    serde_json::from_str(&buf).map_err(|e| format!("bad response: {e}"))
}

pub fn request(sock: &Path, req: dto::Req) -> Result<dto::Resp, String> {
    let stream =
        UnixStream::connect(sock).map_err(|e| format!("connect {}: {e}", sock.display()))?;
    send_recv(&stream, &req)
}

/// One probe round-trip. Local UDS refuses instantly on a dead endpoint.
pub fn ping(sock: &Path) -> bool {
    matches!(request(sock, dto::Req::Ping), Ok(dto::Resp::Pong))
}

// ── DirectBackend: local atomics + files (also the daemon's engine) ─────────

struct Journals {
    tails: std::collections::HashMap<String, String>, // branch → head hash
}

pub struct DirectBackend {
    pub root: PathBuf,
    pub pool: pool::Pool,
    journals: Mutex<Journals>,
    /// Some = this instance owns the direct-mode ledger lock.
    _lock: Option<LedgerLock>,
}

impl DirectBackend {
    /// Spend-capable construction: takes the ledger lock (fail-closed on
    /// contention). The daemon calls this once per project.
    pub fn new(root: &Path, ceilings: pool::Ceilings) -> Result<Self, String> {
        Self::new_with_lock_timeout(root, ceilings, Duration::from_secs(30))
    }

    /// Same as [`new`] with a caller-chosen contention budget.
    pub fn new_with_lock_timeout(
        root: &Path,
        ceilings: pool::Ceilings,
        timeout: Duration,
    ) -> Result<Self, String> {
        let lock = LedgerLock::acquire(root, timeout)?;
        Ok(Self::build(root, ceilings, Some(lock)))
    }

    /// Journal-only construction: no ledger lock, no cap. Used by writers
    /// that must never contend for money state (gate logs, receipts-adjacent).
    pub fn new_journal_only(root: &Path) -> Self {
        Self::build(
            root,
            pool::Ceilings::global_only(u64::MAX),
            None,
        )
    }

    fn build(root: &Path, ceilings: pool::Ceilings, lock: Option<LedgerLock>) -> Self {
        let backend = DirectBackend {
            root: root.to_path_buf(),
            pool: pool::Pool::new(ceilings),
            journals: Mutex::new(Journals { tails: std::collections::HashMap::new() }),
            _lock: lock,
        };
        // continuity: seed from whatever the ledger already recorded
        let legacy = backend.root.join(".kineti/spend.json");
        if let Ok(s) = std::fs::read_to_string(&legacy) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&s) {
                if let Some(micro) = v["total_micro_usd"].as_u64() {
                    backend.pool.seed_total(micro);
                } else if let Some(usd) = v["total_usd"].as_f64() {
                    backend.pool.seed_total((usd * 1_000_000.0).round() as u64);
                }
            }
        }
        backend
    }

    fn persist_spend(&self) {
        let p = self.root.join(".kineti/spend.json");
        if let Some(dir) = p.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let micro = self.pool.total();
        let doc = serde_json::json!({
            "total_usd": micro as f64 / 1_000_000.0,
            "total_micro_usd": micro,
        });
        // tmp+rename so a crash never leaves a torn ledger
        let tmp = p.with_extension("json.tmp");
        if std::fs::write(&tmp, doc.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, &p);
        }
    }

    fn journal_rel(branch: &str) -> PathBuf {
        if branch.is_empty() {
            PathBuf::from(".kineti/journal.jsonl")
        } else {
            PathBuf::from(format!(".kineti/journal.{branch}.jsonl"))
        }
    }

    fn load_tail(abs: &Path) -> String {
        let Ok(content) = std::fs::read_to_string(abs) else {
            return GENESIS.into();
        };
        for line in content.lines().rev() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                return v["hash"].as_str().unwrap_or(GENESIS).to_string();
            }
        }
        GENESIS.into()
    }
}

impl SpendService for DirectBackend {
    fn reserve(&self, ctx: &dto::ReserveCtx) -> Result<dto::Reservation, String> {
        self.pool.reserve(ctx)
    }

    fn settle(&self, res: &dto::Reservation, actual_micro_usd: u64) -> Result<u64, String> {
        let total = self.pool.settle(res, actual_micro_usd);
        self.persist_spend();
        Ok(total)
    }

    fn snapshot(&self) -> Result<dto::SpendSnapshot, String> {
        Ok(dto::SpendSnapshot {
            total_micro_usd: self.pool.total(),
            cap_micro_usd: self.pool.ceilings.global_micro,
            tripped: self.pool.is_tripped(),
        })
    }

    fn reset_if_human_requested(&self, root: &Path) -> Result<bool, String> {
        let flag = root.join(".kineti/spend.reset");
        if flag.exists() {
            let _ = std::fs::remove_file(&flag);
            self.pool.reset();
            self.persist_spend();
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl JournalWriter for DirectBackend {
    fn head(&self, branch: &str) -> Result<String, String> {
        let abs = self.root.join(Self::journal_rel(branch));
        let mut j = self.journals.lock().map_err(|_| "journal lock poisoned")?;
        let tail = j
            .tails
            .entry(branch.to_string())
            .or_insert_with(|| Self::load_tail(&abs));
        Ok(tail.clone())
    }

    fn append_batch(&self, branch: &str, records: Vec<Record>) -> Result<(), String> {
        let abs = self.root.join(Self::journal_rel(branch));
        let mut j = self.journals.lock().map_err(|_| "journal lock poisoned")?;
        let tail = j
            .tails
            .entry(branch.to_string())
            .or_insert_with(|| Self::load_tail(&abs));

        // validate everything FIRST, then persist, then advance the cached
        // tail — a failed write must never desynchronize cache from disk.
        let mut prospective = tail.clone();
        let mut lines = Vec::with_capacity(records.len());
        for r in records {
            if r.prev_hash != prospective {
                return Err(format!(
                    "chain mismatch on '{branch}': record {} prev_hash != current tail",
                    r.id
                ));
            }
            lines.push(serde_json::to_string(&r).unwrap_or_default());
            prospective = r.hash.clone();
        }

        if let Some(dir) = abs.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        use std::fs::OpenOptions;
        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&abs)
            .map_err(|e| format!("journal open: {e}"))?;
        use std::io::Write;
        for l in &lines {
            writeln!(f, "{l}").map_err(|e| format!("journal write: {e}"))?;
        }
        f.flush().ok();
        *tail = prospective;
        Ok(())
    }
}

// ── SocketBackend: thin client over the same DTOs ───────────────────────────

#[derive(Clone)]
pub struct SocketBackend {
    pub sock: PathBuf,
}

impl std::fmt::Debug for SocketBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SocketBackend({})", self.sock.display())
    }
}

impl SocketBackend {
    pub fn connect(sock: &Path) -> Result<Self, String> {
        let b = SocketBackend { sock: sock.to_path_buf() };
        match request(&b.sock, dto::Req::Ping) {
            Ok(dto::Resp::Pong) => Ok(b),
            Ok(_) => Err("daemon answered with unexpected frame".into()),
            Err(e) => Err(e),
        }
    }

    fn expect<T>(
        got: Result<dto::Resp, String>,
        f: impl Fn(dto::Resp) -> Option<T>,
    ) -> Result<T, String> {
        match got? {
            dto::Resp::Err(e) => Err(e),
            other => f(other).ok_or_else(|| "unexpected response variant".into()),
        }
    }
}

impl SpendService for SocketBackend {
    fn reserve(&self, ctx: &dto::ReserveCtx) -> Result<dto::Reservation, String> {
        let resp = request(&self.sock, dto::Req::SpendReserve { ctx: ctx.clone() });
        Self::expect(resp, |r| match r {
            dto::Resp::Reserved(x) => Some(x),
            _ => None,
        })
    }
    fn settle(&self, res: &dto::Reservation, actual_micro_usd: u64) -> Result<u64, String> {
        let resp = request(
            &self.sock,
            dto::Req::SpendSettle { res: res.clone(), actual_micro_usd },
        );
        Self::expect(resp, |r| match r {
            dto::Resp::Settled { total_micro_usd } => Some(total_micro_usd),
            _ => None,
        })
    }
    fn snapshot(&self) -> Result<dto::SpendSnapshot, String> {
        let resp = request(&self.sock, dto::Req::SpendSnapshot);
        Self::expect(resp, |r| match r {
            dto::Resp::Snapshot(s) => Some(s),
            _ => None,
        })
    }
    fn reset_if_human_requested(&self, _root: &Path) -> Result<bool, String> {
        let resp = request(&self.sock, dto::Req::SpendResetIfRequested);
        Self::expect(resp, |r| match r {
            dto::Resp::Reset(b) => Some(b),
            _ => None,
        })
    }
}

impl JournalWriter for SocketBackend {
    fn head(&self, branch: &str) -> Result<String, String> {
        let resp = request(&self.sock, dto::Req::JournalHead { branch: branch.into() });
        Self::expect(resp, |r| match r {
            dto::Resp::Head(h) => Some(h),
            _ => None,
        })
    }
    fn append_batch(&self, branch: &str, records: Vec<Record>) -> Result<(), String> {
        let lines = records
            .iter()
            .map(|r| serde_json::to_string(r).unwrap_or_default())
            .collect();
        let resp =
            request(&self.sock, dto::Req::AppendBatch { branch: branch.into(), records: lines });
        Self::expect(resp, |r| match r {
            dto::Resp::Appended => Some(()),
            _ => None,
        })
    }
}

// ── Selection (evaluated once per CLI invocation) ───────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BackendKind {
    Direct,
    Daemon,
}

fn selected_kind(root: &Path) -> BackendKind {
    if std::env::var(ENV_FORCE_DIRECT).map(|v| v == "1").unwrap_or(false) {
        return BackendKind::Direct;
    }
    let sock = socket_path(root);
    if ping(&sock) {
        return BackendKind::Daemon;
    }
    if std::env::var(ENV_NO_DAEMON).map(|v| v == "1").unwrap_or(false) {
        return BackendKind::Direct;
    }
    if spawn_daemon(root) && wait_for_socket(&sock, 40) {
        return BackendKind::Daemon;
    }
    BackendKind::Direct
}

/// True when selection would choose the direct implementation right now.
pub fn selects_direct(root: &Path) -> bool {
    selected_kind(root) == BackendKind::Direct
}

/// The pair of services handed out by [`select_backends`].
pub type BackendPair = (Box<dyn SpendService>, Box<dyn JournalWriter>);

/// One decision point for both services (§R1): the socket is probed exactly
/// once so spend and journal can never disagree about which mode they're in.
/// Direct-mode spend construction can fail CLOSED on ledger-lock contention.
pub fn select_backends(
    root: &Path,
    ceilings: pool::Ceilings,
) -> Result<BackendPair, String> {
    let sock = socket_path(root);
    if selected_kind(root) == BackendKind::Daemon {
        if let Ok(b) = SocketBackend::connect(&sock) {
            let j = b.clone();
            return Ok((Box::new(b), Box::new(j)));
        }
    }
    let spend = DirectBackend::new(root, ceilings)?;
    let journal = DirectBackend::new_journal_only(root);
    Ok((Box::new(spend), Box::new(journal)))
}

/// Journal writer WITHOUT daemon auto-spawn — used by one-shot paths
/// (`log_gate`, receipts-adjacent writes). Attaches to a live daemon when
/// present; otherwise opens the file directly. Never spawns.
pub fn journal_writer_no_spawn(root: &Path) -> Box<dyn JournalWriter> {
    if std::env::var(ENV_FORCE_DIRECT).map(|v| v == "1").unwrap_or(false) {
        return Box::new(DirectBackend::new_journal_only(root));
    }
    if let Ok(b) = SocketBackend::connect(&socket_path(root)) {
        return Box::new(b);
    }
    Box::new(DirectBackend::new_journal_only(root))
}

/// Spawn `kineti serve` detached from the current executable.
fn spawn_daemon(root: &Path) -> bool {
    let exe = match std::env::current_exe() {
        Ok(e) => e,
        Err(_) => return false,
    };
    let _ = std::fs::create_dir_all(root.join(".kineti"));
    std::process::Command::new(exe)
        .arg("serve")
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map(|_| true)
        .unwrap_or(false)
}

fn wait_for_socket(sock: &Path, tries: usize) -> bool {
    for i in 0..tries {
        if ping(sock) {
            return true;
        }
        // capped exponential backoff; total < ~2s
        std::thread::sleep(Duration::from_millis(1 << i.min(7)));
    }
    false
}

/// Remove a dead socket file after observing connection refusal.
pub fn clean_stale_socket(root: &Path) -> bool {
    let sock = socket_path(root);
    if !sock.exists() || ping(&sock) {
        return false; // absent or alive — do not touch
    }
    std::fs::remove_file(&sock).is_ok()
}

/// Enforce 0600 on the socket so only the same user can talk governance.
pub fn restrict_socket_perms(sock: &Path) {
    let _ = std::fs::set_permissions(sock, std::fs::Permissions::from_mode(0o600));
}

/// Shared halt flag type for swarm phases.
pub type HaltFlag = AtomicBool;
