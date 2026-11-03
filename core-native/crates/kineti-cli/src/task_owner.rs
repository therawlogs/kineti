//! Shared owner service. Platform boundaries are implemented by the admitted backend.
use super::*;
use crate::{
    compat::{canonical, digest},
    task_snapshot as snapshot,
};
use std::{
    collections::BTreeSet,
    ffi::CString,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, IsTerminal, Write},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
            net::{UnixListener, UnixStream},
        },
    },
    path::PathBuf,
    process::{Child, ChildStdin, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

fn now() -> i64 {
    time::OffsetDateTime::now_utc().unix_timestamp()
}
#[path = "task_backend.rs"]
mod backend;
use backend::{Boundary, Peer, Platform};

fn owner() -> Result<()> {
    Platform::admit()
}
fn secure(path: &Path) -> Result<()> {
    let mut p = PathBuf::from("/");
    for c in path.components().skip(1) {
        p.push(c);
        let m = fs::symlink_metadata(&p)?;
        if m.file_type().is_symlink() || m.uid() != 0 || m.mode() & 0o022 != 0 {
            return store::fail(
                3,
                "owner path is redirected or writable outside the owner role",
            );
        }
    }
    Ok(())
}
fn directory(path: &Path, mode: u32) -> Result<()> {
    if !path.exists() {
        fs::create_dir(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    secure(path)?;
    Ok(())
}
fn layout(base: &Path) -> Result<()> {
    owner()?;
    if base != Path::new("/var/lib/kineti-owner-native-v1") {
        return store::fail(3,"needs-scope: this adapter uses only /var/lib/kineti-owner-native-v1 so per-UID leases cannot be multiplied across owner stores");
    }
    secure(Path::new("/var/lib"))?;
    directory(base, 0o711)?;
    for (p, mode) in [
        ("authority", 0o700),
        ("sockets", 0o711),
        ("work", 0o711),
        ("runtime", 0o711),
    ] {
        directory(&base.join(p), mode)?;
    }
    Ok(())
}
fn actor(g: &Grant) -> Result<(u32, u32, PathBuf)> {
    let name =
        CString::new(g.user.as_str()).map_err(|_| Error(2, "invalid task account".into()))?;
    let mut entry = std::mem::MaybeUninit::<libc::passwd>::uninit();
    let mut buffer = vec![0u8; 65536];
    let mut found = std::ptr::null_mut();
    let result = unsafe {
        libc::getpwnam_r(
            name.as_ptr(),
            entry.as_mut_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            &mut found,
        )
    };
    if result != 0 || found.is_null() {
        return store::fail(
            3,
            "task account not installed by owner or account lookup failed",
        );
    }
    let entry = unsafe { entry.assume_init() };
    let uid = entry.pw_uid;
    let gid = entry.pw_gid;
    if uid != g.uid || uid == 0 {
        return store::fail(
            3,
            "task account binding differs; no agent identity flag is accepted",
        );
    }
    let home = PathBuf::from(
        unsafe { std::ffi::CStr::from_ptr(entry.pw_dir) }
            .to_string_lossy()
            .into_owned(),
    );
    Ok((uid, gid, home))
}
fn host(g: &Grant) -> Result<()> {
    secure(&g.host)?;
    if !fs::metadata(&g.host)?.is_file() || fs::metadata(&g.host)?.mode() & 0o111 == 0 {
        return store::fail(3, "pinned host is not an owner-installed executable");
    }
    if g.model_repair.is_some() {
        let helper = g.host.with_file_name("codex-code-mode-host");
        secure(&helper)?;
        if !fs::metadata(&helper)?.is_file() || fs::metadata(&helper)?.mode() & 0o111 == 0 {
            return store::fail(
                3,
                "matching code-mode helper is not an owner-installed executable",
            );
        }
    }
    g.current()?;
    Ok(())
}
fn account(g: &Grant) -> Result<Value> {
    let a = crate::spend::assessment(&g.project)?;
    if a["accounting_allows_zero_incremental_local"] != true {
        return store::fail(3, format!("accounting hard blocker: {}", a["reason"]));
    }
    Ok(a)
}
fn lock(path: &Path) -> Result<File> {
    let f = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return store::fail(
            3,
            "owner/task UID lease already active; concurrent authority is not multiplied",
        );
    }
    Ok(f)
}
struct Audit {
    dir: PathBuf,
    head: String,
    seq: u64,
    state: Value,
}
impl Audit {
    fn load(dir: &Path) -> Result<Self> {
        secure(dir)?;
        let checkpoint = store::read(&dir.join("head.json"))?;
        if fs::metadata(dir.join("events.jsonl"))?.len() > 32_000_000 {
            return store::fail(3, "owner evidence storage limit reached; inspect originals");
        }
        let (head, seq, state) =
            Self::decode(&checkpoint, store::lines(&dir.join("events.jsonl"))?)?;
        Ok(Self {
            dir: dir.into(),
            head,
            seq,
            state,
        })
    }
    fn decode(checkpoint: &Value, rows: Vec<Value>) -> Result<(String, u64, Value)> {
        let mut head = "GENESIS".to_string();
        let mut seq = 0;
        let mut state = Value::Null;
        for row in rows {
            let mut plain = row.clone();
            plain
                .as_object_mut()
                .ok_or_else(|| Error(3, "owner record malformed".into()))?
                .remove("hash");
            if row["previous"] != head
                || row["sequence"] != seq + 1
                || row["hash"] != digest(canonical(&plain).map_err(|s| Error(3, s))?)
            {
                return store::fail(
                    3,
                    "owner evidence integrity failure; no flag bypass or replay",
                );
            }
            seq += 1;
            head = row["hash"].as_str().unwrap().into();
            state = row["state"].clone();
        }
        if seq == 0 || checkpoint["head"] != head || checkpoint["sequence"] != seq {
            return store::fail(3, "inspect-uncertain: interrupted owner evidence commit");
        }
        Ok((head, seq, state))
    }
    fn add(&mut self, event: &str, state: Value) -> Result<()> {
        if self.seq > 0 {
            let disk = Self::load(&self.dir)?;
            if disk.head != self.head || disk.state != self.state {
                return store::fail(3, "owner evidence changed before append");
            }
        }
        let mut row = json!({"schema":"kineti-owner-event-v1","sequence":self.seq+1,"previous":self.head,"event":event,"at":now(),"state":state});
        let h = digest(canonical(&row).map_err(|s| Error(3, s))?);
        row["hash"] = json!(h);
        if fs::metadata(self.dir.join("events.jsonl"))
            .map(|m| m.len())
            .unwrap_or(0)
            + serde_json::to_vec(&row)?.len() as u64
            > 32_000_000
        {
            return store::fail(
                3,
                "inspect-uncertain: owner evidence storage limit reached; preserve and inspect",
            );
        }
        store::append(&self.dir.join("events.jsonl"), &row)?;
        store::write(
            &self.dir.join("head.json"),
            &json!({"head":h,"sequence":self.seq+1}),
        )?;
        self.head = h;
        self.seq += 1;
        self.state = state;
        Ok(())
    }
}
/// Inspect protected records without opening a service or requiring the old
/// executable to approve the reader. Inspection never grants execution authority.
pub(super) fn inspect(base: &Path, id: &str) -> Result<Value> {
    owner()?;
    if base != Path::new("/var/lib/kineti-owner-native-v1") {
        return store::fail(3, "needs-scope: inspect only the protected owner store");
    }
    crate::task_grant::id(id)?;
    secure(base)?;
    let dir = base.join("authority").join(id);
    secure(&dir)?;
    let mut files = serde_json::Map::new();
    for (name, limit) in [
        ("grant.json", 65536),
        ("approval.json", 65536),
        ("head.json", 65536),
        ("events.jsonl", 32_000_000),
    ] {
        let path = dir.join(name);
        secure(&path)?;
        if fs::metadata(&path)?.len() > limit {
            return store::fail(
                3,
                "inspect-uncertain: protected owner record exceeds its read bound",
            );
        }
        files.insert(name.into(), json!(snapshot::artifact_digest(&path)?));
    }
    let grant = store::read(&dir.join("grant.json"))?;
    let approval = store::read(&dir.join("approval.json"))?;
    let digest = digest(canonical(&grant).map_err(|s| Error(3, s))?);
    let audit = Audit::load(&dir);
    let valid = audit.as_ref().is_ok_and(|a| {
        grant["id"] == id
            && approval["owner_uid"] == 0
            && approval["grant_digest"] == digest
            && a.state["grant_digest"] == digest
    });
    let mut out = super::decision(
        if valid { "allow" } else { "inspect-uncertain" },
        "inspect",
        if valid {
            "read-only-protected-owner-state; no execution or recovery authority"
        } else {
            "protected owner integrity is unresolved; preserve records and review an exact proposal"
        },
    );
    out["read_only"] = json!(true);
    out["task_authority"] = json!(false);
    out["grant_digest"] = json!(digest);
    out["record_sha256"] = json!(files);
    out["service_matches_approval"] =
        json!(approval["service_sha256"] == snapshot::artifact_digest(&std::env::current_exe()?)?);
    if let Ok(a) = audit {
        out["state"] = a.state;
    }
    out["active_marker_present"] = json!(dir.join("active").exists());
    out["inspection_digest"] = json!(crate::compat::digest(
        canonical(&out).map_err(|s| Error(3, s))?
    ));
    Ok(out)
}
/// Initial approval is root-owned TTY input, not an agent confirmation flag.
pub(super) fn approve(base: &Path, path: &Path) -> Result<Value> {
    layout(base)?;
    secure(&std::env::current_exe()?)?;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return store::fail(3,"owner approval requires the separate owner console; noninteractive approval flags are unsupported");
    }
    let g = Grant::parse(super::read_request(path)?)?;
    host(&g)?;
    actor(&g)?;
    let a = account(&g)?;
    if g.expires > now() + 86400 {
        return store::fail(2, "initial host grants may expire at most 24 hours ahead");
    }
    let dir = base.join("authority").join(&g.id);
    if dir.exists() {
        return store::fail(
            3,
            "grant ID already exists; agents cannot replace, extend or renew it",
        );
    }
    println!("{}", serde_json::to_string_pretty(&g.summary())?);
    println!("Historical valuation remains {}. New paid spending: 0. Release actions: separate approval.\n1. Approve this exact bounded task\n2. Deny",a["historical_valuation"]["status"]);
    let mut answer = String::new();
    std::io::stdin().read_line(&mut answer)?;
    if answer.trim() != "1" {
        return Ok(super::decision("deny", "approve", "owner-declined"));
    }
    directory(&dir, 0o700)?;
    let hash = g.digest()?;
    store::write(&dir.join("grant.json"), &g.request)?;
    store::write(
        &dir.join("approval.json"),
        &json!({"schema":"kineti-owner-approval-v1","grant_digest":hash,"owner_uid":0,"channel":"independent-owner-tty","at":now(),"human_identity_proven":false,"service_sha256":snapshot::artifact_digest(&std::env::current_exe()?)?}),
    )?;
    let state = json!({"grant_digest":hash,"phase":"approved","approved_at":now(),"started_at":0,"used_repairs":0,"used_checks":0,"used_model_turns":0,"group":null,"last_output":null,"last_evidence":null,"terminal":false,"outcome":null,"service_open":false,"pending_execution":false});
    let mut audit = Audit {
        dir,
        head: "GENESIS".into(),
        seq: 0,
        state: Value::Null,
    };
    audit.add("owner-approved", state)?;
    store::atomic(&audit.dir.join("active"), hash.as_bytes())?;
    let mut v = super::decision("allow", "approve", "owner-role-approved-exact-request");
    v["grant_digest"] = json!(hash);
    v["task_id"] = json!(g.id);
    v["human_identity_proven"] = json!(false);
    Ok(v)
}
fn load(base: &Path, id: &str) -> Result<(Grant, Audit)> {
    secure(&std::env::current_exe()?)?;
    crate::task_grant::id(id)?;
    let dir = base.join("authority").join(id);
    secure(&dir)?;
    for p in ["grant.json", "approval.json", "events.jsonl", "head.json"] {
        secure(&dir.join(p))?;
    }
    let g = Grant::parse(store::read(&dir.join("grant.json"))?)?;
    let approval = store::read(&dir.join("approval.json"))?;
    let a = Audit::load(&dir)?;
    if g.id != id
        || approval["grant_digest"] != g.digest()?
        || a.state["grant_digest"] != g.digest()?
        || approval["owner_uid"] != 0
        || approval["service_sha256"] != snapshot::artifact_digest(&std::env::current_exe()?)?
    {
        return store::fail(
            3,
            "protected grant/approval binding differs; a project-local copy is not authority",
        );
    }
    Ok((g, a))
}
struct Control {
    g: Grant,
    base: PathBuf,
    work: PathBuf,
    audit: Mutex<Audit>,
    group: Mutex<Option<PathBuf>>,
    supervisor: Mutex<Option<Child>>,
    lifecycle: Mutex<()>,
    login_once: AtomicBool,
    busy: AtomicBool,
    halted: AtomicBool,
    notices: Mutex<BTreeSet<String>>,
}
impl Control {
    fn state(&self) -> Result<Value> {
        Ok(self
            .audit
            .lock()
            .map_err(|_| Error(3, "owner state poisoned".into()))?
            .state
            .clone())
    }
    fn event(&self, event: &str, edit: impl FnOnce(&mut Value)) -> Result<()> {
        let mut a = self
            .audit
            .lock()
            .map_err(|_| Error(3, "owner state poisoned".into()))?;
        if self.halted.load(Ordering::SeqCst)
            && ![
                "stop-intent",
                "stop-complete",
                "service-closed",
                "cleanup-uncertain",
                "execution-group-stopped",
            ]
            .contains(&event)
        {
            return store::fail(3, "deny: stopped service cannot record new progress");
        }
        let mut s = a.state.clone();
        edit(&mut s);
        a.add(event, s)
    }
    fn check(&self) -> Result<()> {
        if self.halted.load(Ordering::SeqCst) {
            return store::fail(3, "deny: owner stopped this service execution");
        }
        let a = self
            .audit
            .lock()
            .map_err(|_| Error(3, "owner evidence poisoned".into()))?;
        let disk = Audit::load(&a.dir)?;
        if disk.head != a.head || disk.state != a.state {
            return store::fail(
                3,
                "owner evidence changed during execution; no integrity bypass",
            );
        }
        if store::read(&a.dir.join("grant.json"))? != self.g.request
            || !a.dir.join("active").is_file()
            || fs::read(a.dir.join("active"))? != self.g.digest()?.as_bytes()
        {
            return store::fail(3, "deny: protected grant revoked or changed");
        }
        drop(a);
        let s = self.state()?;
        if s["terminal"] == true {
            return store::fail(3, format!("deny: task authority stopped: {}", s["phase"]));
        }
        let started = s["started_at"].as_i64().unwrap_or(0);
        if now() < s["approved_at"].as_i64().unwrap_or(i64::MAX)
            || now() >= self.g.expires
            || started > 0 && now().saturating_sub(started) >= self.g.wall as i64
        {
            return store::fail(
                3,
                "deny: task expiry/wall limit reached; no automatic extension",
            );
        }
        if crate::task_grant::project_identity(&self.g.project)? != self.g.identity {
            return store::fail(3, "deny: project/locked-goal identity changed");
        }
        account(&self.g)?;
        if self
            .supervisor
            .lock()
            .map_err(|_| Error(3, "supervisor state poisoned".into()))?
            .as_mut()
            .is_some_and(|child| child.try_wait().map_or(true, |v| v.is_some()))
        {
            return store::fail(
                3,
                "inspect-uncertain: crash supervisor exited; stop dependent work",
            );
        }
        Ok(())
    }
    fn start_supervisor(&self, group: &Path) -> Result<()> {
        let mut child = Platform::supervisor_command(group, self.g.uid)?
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;
        let output = child.stdout.take().unwrap();
        let mut poll = libc::pollfd {
            fd: output.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        let mut reader = BufReader::new(output);
        let mut line = String::new();
        let ready = unsafe { libc::poll(&mut poll, 1, 5000) } > 0
            && reader.by_ref().take(4096).read_line(&mut line).is_ok()
            && serde_json::from_str::<Value>(&line)
                .ok()
                .is_some_and(|v| v["ready"] == true);
        if !ready {
            let _ = child.kill();
            let _ = child.wait();
            return store::fail(3, "crash supervisor unavailable; no task code launched");
        }
        child.stdout = Some(reader.into_inner());
        *self
            .supervisor
            .lock()
            .map_err(|_| Error(3, "supervisor state poisoned".into()))? = Some(child);
        self.event("crash-supervisor-ready", |s| {
            s["crash_supervised"] = json!(true)
        })
    }
    fn empty_group(&self) -> Result<()> {
        let mut group = self
            .group
            .lock()
            .map_err(|_| Error(3, "owner group state poisoned".into()))?;
        if let Some(p) = group.as_ref() {
            Platform::stop(p, self.g.uid)?;
            if let Some(mut child) = self
                .supervisor
                .lock()
                .map_err(|_| Error(3, "supervisor state poisoned".into()))?
                .take()
            {
                if let Some(mut input) = child.stdin.take() {
                    let _ = input.write_all(b"D");
                }
                let until = Instant::now() + Duration::from_secs(2);
                while child.try_wait()?.is_none() && Instant::now() < until {
                    thread::sleep(Duration::from_millis(5));
                }
                if child.try_wait()?.is_none() {
                    let _ = child.kill();
                    child.wait()?;
                }
            }
            Platform::remove(p)?;
            *group = None;
        } else {
            return Ok(());
        }
        self.event("execution-group-stopped", |s| s["group"] = Value::Null)
    }
    fn stop(&self, phase: &str, reason: &str) -> Result<()> {
        self.halted.store(true, Ordering::SeqCst);
        let recorded = self.event("stop-intent", |s| {
            s["terminal"] = json!(phase != "paused");
            s["phase"] = json!(phase);
            s["reason"] = json!(reason);
        });
        // An audit failure must never prevent the safety stop itself.
        // Setup, host creation and restoration run as the owner. Signal
        // cancellation first, then wait before draining: setup cannot create
        // a new group after this stop's final drain or acknowledgment.
        let _lifecycle = self.lifecycle.lock().unwrap_or_else(|e| e.into_inner());
        let stopped = self.empty_group();
        if phase != "paused" {
            let marker = self.base.join("authority").join(&self.g.id).join("active");
            if marker.exists() {
                fs::remove_file(marker)?;
                File::open(self.base.join("authority").join(&self.g.id))?.sync_all()?;
            }
        }
        recorded?;
        stopped?;
        self.event("stop-complete", |s| {
            s["phase"] = json!(phase);
            s["terminal"] = json!(phase != "paused");
            s["reason"] = json!(reason);
            s["service_open"] = json!(false);
        })
    }
    fn frozen(&self) -> Result<bool> {
        let group = self
            .group
            .lock()
            .map_err(|_| Error(3, "owner group state poisoned".into()))?;
        let p = group
            .as_ref()
            .ok_or_else(|| Error(3, "execution group missing".into()))?;
        Platform::frozen(p)
    }
    fn freeze(&self, value: bool) -> Result<()> {
        let group = self
            .group
            .lock()
            .map_err(|_| Error(3, "owner group state poisoned".into()))?;
        let p = group
            .as_ref()
            .ok_or_else(|| Error(3, "execution group missing".into()))?;
        Platform::freeze(p, value)
    }
    fn inventory(&self) -> Result<Value> {
        snapshot::inventory(&self.work, &self.g.excludes, self.g.snapshot_bytes)
    }
    fn fingerprint(&self, inputs: &[String]) -> Result<String> {
        let all = self.inventory()?;
        let declared = snapshot::input_digest(&all, inputs)?;
        let work = digest(canonical(&all).map_err(|s| Error(3, s))?);
        let mut runtime = vec![];
        let mut runtime_paths = self.g.runtime.clone();
        for argv in self
            .g
            .checks
            .iter()
            .map(|x| &x.argv)
            .chain(std::iter::once(&self.g.repair))
            .filter(|argv| !argv.is_empty())
        {
            let p = if Path::new(&argv[0]).is_absolute() {
                PathBuf::from(&argv[0])
            } else if argv[0].contains('/') {
                self.work.join(&argv[0])
            } else {
                ["/usr/bin", "/bin"]
                    .iter()
                    .map(|r| Path::new(r).join(&argv[0]))
                    .find(|p| p.is_file())
                    .ok_or_else(|| Error(3, "approved command runtime unavailable".into()))?
            };
            let actual = p.canonicalize()?;
            if actual.starts_with(&self.work) {
                continue;
            }
            if !["/usr", "/lib", "/bin", "/sbin"]
                .iter()
                .any(|r| actual.starts_with(r))
            {
                return store::fail(
                    3,
                    "approved executable redirected outside task/public runtime scope",
                );
            }
            runtime_paths.push(actual.to_string_lossy().to_string());
        }
        runtime_paths.sort();
        runtime_paths.dedup();
        for p in &runtime_paths {
            let resolved = Path::new(p).canonicalize()?;
            if !["/usr", "/lib", "/bin", "/sbin"]
                .iter()
                .any(|s| resolved.starts_with(s))
            {
                return store::fail(3, "runtime input redirected outside public system paths");
            }
            let value = if resolved.is_file() {
                let parent = resolved.parent().unwrap();
                let one = snapshot::inventory(
                    parent,
                    &fs::read_dir(parent)?
                        .filter_map(|e| e.ok())
                        .filter_map(|e| {
                            let p = e.path();
                            if p == resolved {
                                None
                            } else {
                                p.file_name().and_then(|x| x.to_str()).map(String::from)
                            }
                        })
                        .collect::<Vec<_>>(),
                    100_000_000,
                )?;
                snapshot::input_digest(
                    &one,
                    &[resolved.file_name().unwrap().to_str().unwrap().to_owned()],
                )?
            } else {
                digest(
                    canonical(&snapshot::inventory(&resolved, &[], 100_000_000)?)
                        .map_err(|s| Error(3, s))?,
                )
            };
            runtime.push(json!({"path":p,"resolved":resolved,"digest":value}));
        }
        Ok(digest(canonical(&json!({"work":work,"declared":declared,"runtime":runtime,"host":self.g.host_hash,"grant":self.g.digest()?,"kernel":Platform::kernel_identity()?,"environment":{"PATH":"/usr/bin:/bin","TZ":"UTC","LANG":"C.UTF-8"}})).map_err(|s|Error(3,s))?))
    }
    fn denial(&self, method: &str, reason: &str) -> Value {
        let key = digest(format!("{method}:{reason}"));
        let first = self
            .notices
            .lock()
            .map(|mut n| n.len() < 128 && n.insert(key))
            .unwrap_or(false);
        let mut v = super::decision(
            if reason.starts_with("inspect-uncertain") {
                "inspect-uncertain"
            } else if reason.starts_with("needs-scope") {
                "needs-scope"
            } else {
                "deny"
            },
            method,
            reason,
        );
        v["notify"] = json!(first);
        v["smallest_change"]=json!("Use the valid owner-approved task scope; no project-file or confirmation-flag override.");
        v
    }
}
fn peer(s: &UnixStream) -> Result<Peer> {
    Platform::peer(s)
}
// Stock Bubblewrap cannot mount onto a symlink destination. Resolve only
// already-declared public runtimes, matching the evidence fingerprint boundary.
fn profile_runtime(path: &Path) -> Result<PathBuf> {
    let resolved = path.canonicalize()?;
    if !["/usr", "/lib", "/bin", "/sbin"]
        .iter()
        .any(|root| resolved.starts_with(root))
    {
        return store::fail(3, "runtime input redirected outside public system paths");
    }
    Ok(resolved)
}
fn profile(c: &Control) -> Result<String> {
    let mut fs = json!({":minimal":"read","glob_scan_max_depth":32});
    fs[c.work.to_str().unwrap()] = json!("write");
    for p in &c.g.excludes {
        fs[c.work.join(p).to_str().unwrap()] = json!("deny");
    }
    for pattern in [
        "**/.git",
        "**/.kineti",
        "**/.ssh",
        "**/.aws",
        "**/.codex",
        "**/.env*",
        "**/*.pem",
        "**/*.key",
        "**/*.private",
    ] {
        fs[c.work.join(pattern).to_str().unwrap()] = json!("deny");
    }
    fs[c.g.project.to_str().unwrap()] = json!("deny");
    fs[c.base.to_str().unwrap()] = json!("deny");
    fs[c.work.to_str().unwrap()] = json!("write");
    fs[c.base
        .join("sockets")
        .join(format!("{}.sock", c.g.id))
        .to_str()
        .unwrap()] = json!("read");
    fs[c.g.host.to_str().unwrap()] = json!("read");
    if c.g.model_repair.is_some() {
        fs[c.g
            .host
            .with_file_name("codex-code-mode-host")
            .to_str()
            .unwrap()] = json!("read");
    }
    for p in &c.g.runtime {
        fs[profile_runtime(Path::new(p))?
            .to_str()
            .ok_or_else(|| Error(3, "runtime path is not UTF-8".into()))?] = json!("read");
    }
    fn toml(v: &Value) -> String {
        match v {
            Value::Object(m) => format!(
                "{{{}}}",
                m.iter()
                    .map(|(k, v)| format!("{}={}", serde_json::to_string(k).unwrap(), toml(v)))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
            _ => v.to_string(),
        }
    }
    Ok(toml(&json!({"filesystem":fs,"network":{"enabled":false}})))
}
fn toml_value(v: &Value) -> String {
    match v {
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!("{}={}", json!(k), toml_value(v)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(toml_value).collect::<Vec<_>>().join(",")
        ),
        _ => v.to_string(),
    }
}
struct Host {
    child: Child,
    input: ChildStdin,
    rx: mpsc::Receiver<Value>,
    seq: u64,
    c: Arc<Control>,
    captured: Vec<u8>,
    capture_overflow: bool,
    output_id: String,
    thread_id: String,
    completed_turn: Option<Value>,
}
impl Host {
    fn start(c: Arc<Control>) -> Result<Self> {
        let _lifecycle = c
            .lifecycle
            .lock()
            .map_err(|_| Error(3, "inspect-uncertain: lifecycle lock poisoned".into()))?;
        c.check()?;
        host(&c.g)?;
        if c.g.model_repair.is_some() {
            let home = actor(&c.g)?.2;
            match fs::symlink_metadata(home.join(".codex/auth.json")) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                _ => {
                    return store::fail(
                        3,
                        "needs-scope: model task cannot inherit a persistent auth cache",
                    )
                }
            }
            let path = home.join(".codex/config.toml");
            match OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(&path)
            {
                Ok(file) => {
                    if !file.metadata()?.is_file() || file.metadata()?.len() > 65536 {
                        return store::fail(3, "model config exceeds trust-only policy");
                    }
                    let mut text = String::new();
                    file.take(65537).read_to_string(&mut text)?;
                    if text.len() > 65536 {
                        return store::fail(3, "model config exceeds trust-only policy");
                    }
                    crate::task_grant::validate_model_host_config(&text)?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {
                    return store::fail(
                        3,
                        "model config is redirected or unreadable; preserve it and stop",
                    )
                }
            }
        }
        let group = Platform::group_path(&store::nonce()?);
        c.check()?;
        fs::create_dir(&group)?;
        // Register immediately so every subsequent error can drain this group.
        *c.group
            .lock()
            .map_err(|_| Error(3, "owner group state poisoned".into()))? = Some(group.clone());
        Platform::configure(&group, c.g.uid, c.g.memory, c.g.pids)?;
        c.event("host-start-intent", |s| s["group"] = json!(group))?;
        c.start_supervisor(&group)?;
        let runtime = c.base.join("runtime").join(&c.g.id);
        if !runtime.exists() {
            fs::create_dir(&runtime)?;
            snapshot::assign(&runtime, c.g.uid, actor(&c.g)?.1, false)?;
        }
        let mut args = vec![
            c.g.host.to_string_lossy().to_string(),
            "app-server".into(),
            "--stdio".into(),
            "-c".into(),
            "default_permissions=\"kineti-task\"".into(),
            "-c".into(),
            format!("permissions.kineti-task={}", profile(&c)?),
            "-c".into(),
            "analytics.enabled=false".into(),
            "-c".into(),
            "cli_auth_credentials_store=\"ephemeral\"".into(),
            "-c".into(),
            "features.apps=false".into(),
            "-c".into(),
            "features.plugins=false".into(),
            "-c".into(),
            "web_search=\"disabled\"".into(),
            "-c".into(),
            format!("log_dir={}", json!(runtime.join("logs"))),
            "-c".into(),
            format!("sqlite_home={}", json!(runtime.join("db"))),
            "-c".into(),
            "project_doc_max_bytes=0".into(),
            // The minimal Linux adapter does not support the host's optional
            // snapshot replay prologue. Keep normal shell startup and the same
            // filesystem/network sandbox; do not widen reads into host caches.
            "-c".into(),
            "features.shell_snapshot=false".into(),
            "-c".into(),
            "features.shell_snapshot_v2=false".into(),
        ];
        if c.g.model_repair.is_some() {
            args.extend(["-c".into(), "features.code_mode_host=true".into()]);
        }
        // No hidden secondary inference or unrelated account-backed tool surface.
        for feature in [
            "multi_agent",
            "multi_agent_v2",
            "multi_agent_v2_dynamic_tools",
            "image_generation",
            "memories",
            "skill_search",
        ] {
            args.extend(["-c".into(), format!("features.{feature}=false")]);
        }
        let sidecar = json!({"command":std::env::current_exe()?,"args":["mcp","--root",c.work.to_str().unwrap(),"--task-socket",c.base.join("sockets").join(format!("{}.sock",c.g.id)).to_str().unwrap(),"--task-id",&c.g.id,"--task-session",&c.g.session],"required":true});
        // Whole table replacement avoids inheriting a legacy unrestricted execution sidecar.
        args.extend([
            "-c".into(),
            format!("mcp_servers={{kineti_task={}}}", toml_value(&sidecar)),
        ]);
        let mut child = Platform::spawn(&c.g, &c.work, &args, &group)?;
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::sync_channel(8);
        thread::spawn(move || {
            let mut r = BufReader::new(output);
            loop {
                let mut line = String::new();
                match r.by_ref().take(2_000_001).read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        if line.len() > 2_000_000 {
                            break;
                        }
                        if let Ok(v) = serde_json::from_str(&line) {
                            if tx.send(v).is_err() {
                                break;
                            }
                        }
                    }
                }
            }
        });
        let mut h = Self {
            child,
            input,
            rx,
            seq: 0,
            c: c.clone(),
            captured: vec![],
            capture_overflow: false,
            output_id: String::new(),
            thread_id: String::new(),
            completed_turn: None,
        };
        h.call("initialize",json!({"clientInfo":{"name":"kineti-owner-task","version":"1"},"capabilities":{"experimentalApi":true}}),15)?;
        writeln!(h.input, "{}", json!({"method":"initialized"}))?;
        h.input.flush()?;
        // This owned-local mode has no credential grant and makes no model turns.
        // Never inherit an ambient account while bootstrapping the real sidecar.
        let initial_account = h.call("account/read", json!({"refreshToken":false}), 10)?;
        if h.c.g.model_repair.is_none() && initial_account.get("account") != Some(&Value::Null) {
            return store::fail(
                3,
                "needs-scope: owned-local host must not inherit credentials",
            );
        }
        let initially_authenticated = initial_account["account"]["type"] == "chatgpt";
        let mut params = json!({"cwd":h.c.work,"ephemeral":true,"permissions":"kineti-task","approvalPolicy":"on-request","approvalsReviewer":"user","allowProviderModelFallback":false});
        if let Some(m) = &h.c.g.model_repair {
            params["model"] = json!(m.model);
            params["modelProvider"] = json!("openai");
        }
        let thread = h.call("thread/start", params, 20)?;
        if let Some(m) = &h.c.g.model_repair {
            if thread["modelProvider"] != "openai" || thread["model"] != m.model {
                return store::fail(
                    3,
                    "model/provider fallback differs from approved repair grant",
                );
            }
        }
        let thread_id = thread["thread"]["id"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error(3, "host did not bind the ephemeral task thread".into()))?;
        h.thread_id = thread_id.to_owned();
        let status = h.call("mcpServerStatus/list", json!({"threadId":thread_id}), 15)?;
        crate::mcp::validate_task_host_catalog(&status)?;
        h.c.event("host-capabilities-verified", |s| {
            s["host_capabilities"] = json!({"schema":"kineti-task-host-capabilities-v1","ephemeral_credentials":true,"authenticated":initially_authenticated,"model_turns":0,"model_authority":h.c.g.model_repair.is_some(),"native_mcp":"kineti_task","tools":crate::mcp::task_tools(),"thread_id":thread_id});
        })?;
        Ok(h)
    }
    fn call(&mut self, method: &str, params: Value, seconds: u64) -> Result<Value> {
        let wait_turn = if method == "kineti-internal-wait-turn" {
            params["turnId"].as_str().map(str::to_owned)
        } else {
            None
        };
        self.seq += 1;
        let id = self.seq;
        if wait_turn.is_none() {
            writeln!(
                self.input,
                "{}",
                json!({"jsonrpc":"2.0","id":id,"method":method,"params":params})
            )?;
            self.input.flush()?;
        }
        let end = Instant::now() + Duration::from_secs(seconds);
        let mut inspect_at = Instant::now();
        let mut eligibility_at = Instant::now() + Duration::from_secs(10);
        loop {
            if self.c.state()?["terminal"] == true {
                return store::fail(3, "deny: task cancelled/revoked");
            }
            if Instant::now() >= end {
                return store::fail(
                    3,
                    "inspect-uncertain: command/host timeout; completion not assumed",
                );
            }
            if Instant::now() >= inspect_at {
                self.c.check()?;
                inspect_at = Instant::now() + Duration::from_secs(1);
            }
            if let Some(expected) = &wait_turn {
                if self
                    .completed_turn
                    .as_ref()
                    .is_some_and(|v| v["id"].as_str() == Some(expected))
                {
                    let v = self.completed_turn.take().unwrap();
                    if v["status"] != "completed" {
                        return store::fail(3, "inspect-uncertain: model turn failed");
                    }
                    return Ok(v);
                }
                if Instant::now() >= eligibility_at {
                    self.subscription_eligibility()?;
                    eligibility_at = Instant::now() + Duration::from_secs(10);
                }
            }
            match self.rx.recv_timeout(Duration::from_millis(50)) {
                Ok(v)
                    if wait_turn.is_some()
                        && v["method"] == "turn/completed"
                        && v["params"]["turn"]["id"].as_str() == wait_turn.as_deref() =>
                {
                    if v["params"]["turn"]["status"] != "completed" {
                        return store::fail(
                            3,
                            "inspect-uncertain: model turn did not complete successfully",
                        );
                    }
                    return Ok(v["params"]["turn"].clone());
                }
                Ok(v)
                    if v["method"] == "command/exec/outputDelta"
                        && v["params"]["processId"] == self.output_id =>
                {
                    use base64::Engine;
                    let part = base64::engine::general_purpose::STANDARD
                        .decode(v["params"]["deltaBase64"].as_str().unwrap_or(""))
                        .map_err(|_| {
                            Error(3, "inspect-uncertain: malformed host output capture".into())
                        })?;
                    if v["params"]["capReached"] == true
                        || self.captured.len() + part.len() > 1_000_000
                    {
                        self.capture_overflow = true;
                    } else {
                        self.captured.extend(part);
                    }
                }
                Ok(v) if v["method"] == "account/login/completed" => {
                    let success = v["params"]["success"] == true;
                    self.c.event("owner-login-result", |s| {
                        s["login_success"] = json!(success)
                    })?;
                    if !success {
                        return store::fail(3, "owner device login failed; no automatic retry");
                    }
                }
                Ok(v) if v["method"] == "thread/tokenUsage/updated" => {
                    let total = &v["params"]["tokenUsage"]["total"];
                    let mut numbers = serde_json::Map::new();
                    for key in [
                        "totalTokens",
                        "inputTokens",
                        "cachedInputTokens",
                        "outputTokens",
                        "reasoningOutputTokens",
                    ] {
                        if let Some(n) = total[key].as_u64() {
                            numbers.insert(key.into(), json!(n));
                        }
                    }
                    if !numbers.is_empty() {
                        let record = json!({"thread_id":self.thread_id,"host_reported_cumulative":numbers,"monetary_valuation":"unknown"});
                        self.c
                            .event("model-usage-observed", |s| s["last_model_usage"] = record)?;
                    }
                }
                Ok(v) if v["method"] == "turn/completed" => {
                    self.completed_turn = Some(v["params"]["turn"].clone());
                }
                Ok(v)
                    if ["item/started", "item/completed"]
                        .contains(&v["method"].as_str().unwrap_or("")) =>
                {
                    let item = &v["params"]["item"];
                    let record = json!({"method":v["method"],"type":item["type"],"status":item["status"],"payload_sha256":digest(canonical(item).map_err(|s| Error(3,s))?)});
                    self.c
                        .event("model-action-observed", |s| s["last_model_action"] = record)?;
                }
                Ok(v) if v["id"] == id && v.get("method").is_none() => {
                    if v.get("error").is_some() {
                        return store::fail(
                            3,
                            "host rejected task operation; no automatic approval",
                        );
                    }
                    return Ok(v["result"].clone());
                }
                Ok(v) if v.get("id").is_some() && v.get("method").is_some() => {
                    writeln!(
                        self.input,
                        "{}",
                        json!({"jsonrpc":"2.0","id":v["id"],"error":{"code":-32001,"message":"Owner grant does not permit authority expansion"}})
                    )?;
                    self.input.flush()?;
                    return store::fail(3,"needs-scope: host requested an authority expansion; it was denied, not auto-approved");
                }
                Ok(_) => {}
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => {
                    return store::fail(
                        3,
                        "inspect-uncertain: host exited before observed completion",
                    )
                }
            }
        }
    }
    fn subscription_eligibility(&mut self) -> Result<()> {
        let account = self.call("account/read", json!({"refreshToken":false}), 10)?;
        if account["account"]["type"] != "chatgpt" {
            return store::fail(
                3,
                "needs-action-approval: a fresh memory-only owner login is required; none started",
            );
        }
        let limits = self.call("account/rateLimits/read", json!({}), 10)?;
        if limits["ordinaryUsageAllowed"] != true {
            return store::fail(3, "subscription allowance unavailable; no paid fallback");
        }
        let mut records = vec![&limits["rateLimits"]];
        if let Some(m) = limits["rateLimitsByLimitId"].as_object() {
            records.extend(m.values());
        }
        for r in records {
            if r["credits"]["hasCredits"] != false || r["spendControlReached"] == true {
                return store::fail(
                    3,
                    "subscription spending eligibility uncertain; stop without purchasing",
                );
            }
            let mut found = false;
            for k in ["primary", "secondary"] {
                if !r[k].is_null() {
                    found = true;
                    if r[k]["usedPercent"]
                        .as_f64()
                        .is_none_or(|n| !(0.0..95.0).contains(&n))
                    {
                        return store::fail(
                            3,
                            "subscription 95-percent stop or missing quota evidence",
                        );
                    }
                }
            }
            if !found {
                return store::fail(3, "subscription quota evidence absent; no inference");
            }
        }
        self.c.event("subscription-eligible", |_| {})
    }
    fn prepare_model_auth(&mut self) -> Result<()> {
        // Only an independent owner console can arm one login. The normal service never does.
        if self.c.login_once.swap(false, Ordering::SeqCst) {
            self.c.event("owner-device-login-intent", |_| {})?;
            let challenge = self.call(
                "account/login/start",
                json!({"type":"chatgptDeviceCode"}),
                45,
            )?;
            if challenge["verificationUrl"] != "https://auth.openai.com/codex/device" {
                return store::fail(3, "unexpected authentication destination");
            }
            let code = challenge["userCode"]
                .as_str()
                .ok_or_else(|| Error(3, "device challenge missing".into()))?;
            let mut tty = OpenOptions::new().write(true).open("/dev/tty")?;
            writeln!(tty,"Open https://auth.openai.com/codex/device and enter {code}. Credentials stay in this host's memory.")?;
            let until = Instant::now() + Duration::from_secs(600);
            loop {
                let r = self.call("account/read", json!({"refreshToken":false}), 10)?;
                if r["account"]["type"] == "chatgpt" {
                    break;
                }
                if Instant::now() >= until {
                    return store::fail(3, "memory-only login expired; no retry login");
                }
                thread::sleep(Duration::from_millis(500));
            }
        }
        self.subscription_eligibility()
    }
    fn model_repair(&mut self) -> Result<()> {
        let mode = self
            .c
            .g
            .model_repair
            .clone()
            .ok_or_else(|| Error(3, "model authority absent".into()))?;
        self.subscription_eligibility()?;
        let used = self.c.state()?["used_model_turns"]
            .as_u64()
            .ok_or_else(|| Error(3, "model counter missing; no quota reset".into()))?;
        if used >= mode.max_turns {
            return store::fail(3, "aggregate model-turn limit reached; no extension");
        }
        self.c.event("model-turn-intent", |s| {
            s["used_model_turns"] = json!(used + 1);
            s["pending_execution"] = json!(true);
            s["phase"] = json!("running");
        })?;
        self.completed_turn = None;
        let response = self.call("turn/start",json!({"threadId":self.thread_id,"input":[{"type":"text","text":mode.prompt}],"cwd":self.c.work,"permissions":"kineti-task","approvalPolicy":"on-request","approvalsReviewer":"user"}),10)?;
        let id = response["turn"]["id"]
            .as_str()
            .ok_or_else(|| Error(3, "inspect-uncertain: turn identity missing".into()))?;
        self.call(
            "kineti-internal-wait-turn",
            json!({"turnId":id}),
            self.c.g.command_seconds,
        )?;
        self.c.freeze(true)?;
        let after = self.c.inventory()?;
        self.c.event("model-turn-complete", |s| {
            s["phase"] = json!("idle");
            s["pending_execution"] = json!(false);
            s["last_output"] = after;
        })
    }
    fn execute(&mut self, argv: &[String]) -> Result<Value> {
        self.c.check()?;
        let c = self.c.clone();
        c.event("command-intent", |s| {
            s["phase"] = json!("running");
            s["pending_execution"] = json!(true);
        })?;
        self.captured.clear();
        self.capture_overflow = false;
        self.output_id = format!("task-command-{}", self.seq + 1);
        let result=self.call("command/exec",json!({"command":argv,"cwd":c.work,"permissionProfile":"kineti-task","timeoutMs":c.g.command_seconds*1000,"processId":self.output_id,"streamStdoutStderr":true,"outputBytesCap":1_000_000,"env":{"PATH":"/usr/bin:/bin","TZ":"UTC","LANG":"C.UTF-8"}}),c.g.command_seconds+2)?;
        let exit = result["exitCode"]
            .as_i64()
            .ok_or_else(|| Error(3, "inspect-uncertain: command outcome absent".into()))?;
        if exit < 0 || exit == 124 || self.capture_overflow {
            return store::fail(
                3,
                "inspect-uncertain: timeout or incomplete output; no passing evidence assumed",
            );
        }
        let output = String::from_utf8_lossy(&self.captured);
        let summary = crate::receipt::summary(&output);
        c.freeze(true)?;
        let after = c.inventory()?;
        let evidence = json!({"schema":"kineti-task-execution-v1","argv":argv,"exit":exit,"output_sha256":digest(&self.captured),"output_bytes":self.captured.len(),"capture_complete":true,"test_summary":summary,"independent_execution":false,"grant_digest":c.g.digest()?,"host_sha256":c.g.host_hash});
        c.event("command-complete", |s| {
            s["phase"] = json!("idle");
            s["pending_execution"] = json!(false);
            s["last_output"] = after;
            s["last_execution"] = evidence.clone();
        })?;
        Ok(evidence)
    }
}
impl Host {
    fn shutdown(&mut self) -> Result<()> {
        self.c.empty_group()?;
        let end = Instant::now() + Duration::from_secs(1);
        while self.child.try_wait()?.is_none() {
            if Instant::now() >= end {
                return store::fail(
                    3,
                    "inspect-uncertain: task leader not reaped after cgroup stop",
                );
            }
            thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn run(c: Arc<Control>) -> Result<Value> {
    if c.busy.swap(true, Ordering::SeqCst) {
        return Ok(super::decision(
            "retry-within-grant",
            "run",
            "another worker already owns this grant; quotas are shared",
        ));
    }
    struct Busy(Arc<Control>);
    impl Drop for Busy {
        fn drop(&mut self) {
            self.0.busy.store(false, Ordering::SeqCst);
        }
    }
    let _busy = Busy(c.clone());
    let result = (|| {
        c.check()?;
        host(&c.g)?;
        let current = c.state()?;
        if ["running", "preparing", "restoring", "uncertain"]
            .contains(&current["phase"].as_str().unwrap_or("uncertain"))
        {
            return store::fail(
                3,
                "inspect-uncertain: interrupted action cannot be replayed",
            );
        }
        if !c.work.exists() {
            let _lifecycle = c
                .lifecycle
                .lock()
                .map_err(|_| Error(3, "inspect-uncertain: lifecycle lock poisoned".into()))?;
            c.check()?;
            c.event("snapshot-intent", |s| s["phase"] = json!("preparing"))?;
            let dir = c.base.join("authority").join(&c.g.id);
            let baseline = snapshot::prepare_checked(
                &c.g.project,
                &c.work,
                &dir.join("blobs"),
                &c.g.excludes,
                c.g.snapshot_bytes,
                c.g.uid,
                actor(&c.g)?.1,
                || c.check(),
            )?;
            // Backend-specific admission prepares only its declared exclusions.
            for excluded in &c.g.excludes {
                c.check()?;
                snapshot::guard(&c.work, excluded)?;
                Platform::placeholder(&c.work, excluded)?;
            }
            c.check()?;
            store::write(&dir.join("baseline.json"), &baseline)?;
            c.event("snapshot-complete", |s| {
                s["phase"] = json!("idle");
                s["last_output"] = c.inventory().unwrap_or(Value::Null);
            })?;
        } else if current["last_output"].is_null() {
            return store::fail(
                3,
                "inspect-uncertain: partial setup preserved; no overwrite or replay",
            );
        }
        let mut fingerprints = vec![];
        for check in &c.g.checks {
            fingerprints.push(c.fingerprint(&check.inputs)?);
        }
        if current["last_evidence"]["fingerprints"] == json!(fingerprints) {
            let mut v = super::decision(
                "allow",
                "run",
                "fresh-evidence-reused-within-valid-owner-grant",
            );
            v["state"] = current;
            return Ok(v);
        }
        c.event("work-started", |s| {
            if s["started_at"] == 0 {
                s["started_at"] = json!(now());
            }
            s["phase"] = json!("idle");
        })?;
        let mut h = Host::start(c.clone())?;
        loop {
            c.check()?;
            let state = c.state()?;
            if state["used_checks"].as_u64().unwrap_or(u64::MAX) >= c.g.attempts {
                return store::fail(
                    3,
                    "verification-attempt limit reached; no extension or purchase",
                );
            }
            c.event("verification-attempt", |s| {
                s["used_checks"] = json!(s["used_checks"].as_u64().unwrap() + 1)
            })?;
            let mut all_pass = true;
            let mut stable = true;
            let mut evidence = vec![];
            let mut fresh = vec![];
            for check in &c.g.checks {
                let before = c.fingerprint(&check.inputs)?;
                // The host is frozen only while the owner records checkpoints.
                if c.frozen()? {
                    c.freeze(false)?;
                }
                let e = h.execute(&check.argv)?;
                let after = c.fingerprint(&check.inputs)?;
                let step_stable = before == after;
                stable &= step_stable;
                let counts = e["test_summary"].as_array();
                let passed = e["exit"] == 0
                    && (check.kind == "check"
                        || counts.is_some_and(|v| {
                            v.len() == 3 && v[0].as_u64().unwrap_or(0) > 0 && v[1] == 0 && v[2] == 0
                        }));
                all_pass &= passed;
                fresh.push(after);
                evidence.push(json!({"label":check.label,"kind":check.kind,"execution":e,"stable":step_stable}));
            }
            if all_pass && stable {
                h.shutdown()?;
                drop(h);
                c.event("verification-complete",|s|{s["phase"]=json!("complete");s["last_evidence"]=json!({"fingerprints":fresh,"checks":evidence,"grant_digest":c.g.digest().unwrap_or_default(),"test_trust":"observed local results; not independent execution"});})?;
                let mut v = super::decision("allow", "run", "checks-fresh-and-passing");
                v["state"] = c.state()?;
                return Ok(v);
            }
            if all_pass {
                continue;
            }
            if c.state()?["used_repairs"].as_u64().unwrap_or(u64::MAX) >= c.g.repairs {
                return store::fail(3, "repair-attempt limit reached; no extension or purchase");
            }
            if let Some(mode) = &c.g.model_repair {
                if c.state()?["used_model_turns"]
                    .as_u64()
                    .ok_or_else(|| Error(3, "model counter missing; no quota reset".into()))?
                    >= mode.max_turns
                {
                    return store::fail(
                        3,
                        "aggregate model-turn limit reached; no login or extension",
                    );
                }
                c.freeze(false)?;
                h.prepare_model_auth()?;
            }
            c.event("repair-attempt", |s| {
                s["used_repairs"] = json!(s["used_repairs"].as_u64().unwrap() + 1)
            })?;
            c.freeze(false)?;
            if c.g.model_repair.is_some() {
                h.model_repair()?;
                continue;
            }
            let e = h.execute(&c.g.repair)?;
            if e["exit"] != 0 {
                return store::fail(
                    3,
                    "approved repair command failed; preserve useful partial work",
                );
            }
        }
    })();
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            let uncertain =
                e.1.starts_with("inspect-uncertain") || c.state()?["pending_execution"] == true;
            if let Err(cleanup) = c.empty_group() {
                c.halted.store(true, Ordering::SeqCst);
                let reason = format!(
                    "inspect-uncertain: execution drain failed: {}; original action: {}",
                    cleanup.1, e.1
                );
                c.event("cleanup-uncertain", |s| {
                    s["phase"] = json!("uncertain");
                    s["terminal"] = json!(true);
                    s["reason"] = json!(reason);
                })?;
                return Ok(c.denial("run", &reason));
            }
            if e.1.starts_with("needs-action-approval") && !uncertain {
                c.event("awaiting-owner-action", |s| {
                    s["phase"] = json!("idle");
                    s["terminal"] = json!(false);
                    s["reason"] = json!(e.1);
                })?;
                return Ok(super::decision("needs-action-approval", "run", &e.1));
            }
            if c.state()?["terminal"] != true {
                c.event("work-stopped", |s| {
                    s["phase"] = json!(if uncertain { "uncertain" } else { "stopped" });
                    s["terminal"] = json!(true);
                    s["reason"] = json!(e.1);
                })?;
            }
            if !uncertain && c.g.rollback {
                c.busy.store(false, Ordering::SeqCst);
                let _ = recover(&c);
            }
            Ok(c.denial("run", &e.1))
        }
    }
}
fn recover(c: &Control) -> Result<Value> {
    let _lifecycle = c
        .lifecycle
        .lock()
        .map_err(|_| Error(3, "inspect-uncertain: lifecycle lock poisoned".into()))?;
    // Recovery remains bounded by the original grant even after a definite
    // task failure; revocation or expiry never authorizes new writes.
    c.g.current()?;
    account(&c.g)?;
    let state = c.state()?;
    let started = state["started_at"].as_i64().unwrap_or(0);
    let marker = c.base.join("authority").join(&c.g.id).join("active");
    if c.halted.load(Ordering::SeqCst)
        || !marker.is_file()
        || fs::read(&marker)? != c.g.digest()?.as_bytes()
        || now() < state["approved_at"].as_i64().unwrap_or(i64::MAX)
        || (started > 0 && now().saturating_sub(started) >= c.g.wall as i64)
    {
        return Ok(c.denial(
            "recover",
            "deny: original recovery grant is stopped or outside its time limit",
        ));
    }
    if c.busy.swap(true, Ordering::SeqCst) {
        return Ok(c.denial(
            "recover",
            "inspect-uncertain: running task cannot be restored",
        ));
    }
    struct RecoveryBusy<'a>(&'a AtomicBool);
    impl Drop for RecoveryBusy<'_> {
        fn drop(&mut self) {
            self.0.store(false, Ordering::SeqCst);
        }
    }
    let _busy = RecoveryBusy(&c.busy);
    let s = c.state()?;
    if !c.g.rollback
        || s["pending_execution"] == true
        || s["phase"] == "restoring"
        || s["phase"] == "uncertain"
        || s["phase"] == "running"
        || s["last_output"].is_null()
    {
        return Ok(c.denial(
            "recover",
            "inspect-uncertain: recovery is not a known preauthorized local action",
        ));
    }
    let dir = c.base.join("authority").join(&c.g.id);
    let baseline = store::read(&dir.join("baseline.json"))?;
    let q = dir.join(format!("recovery-{}", store::nonce()?));
    c.event("restore-intent", |s| s["phase"] = json!("restoring"))?;
    match snapshot::restore_checked(
        &c.work,
        &dir.join("blobs"),
        &baseline,
        &s["last_output"],
        &c.g.excludes,
        c.g.snapshot_bytes,
        &q,
        c.g.uid,
        actor(&c.g)?.1,
        || {
            if c.halted.load(Ordering::SeqCst)
                || now() >= c.g.expires
                || (started > 0 && now().saturating_sub(started) >= c.g.wall as i64)
                || fs::read(&marker)? != c.g.digest()?.as_bytes()
            {
                return store::fail(3, "inspect-uncertain: recovery authority stopped; preserve partial work and quarantine");
            }
            account(&c.g)?;
            Ok(())
        },
    ) {
        Ok(v) => {
            c.event("restore-complete", |s| {
                s["phase"] = json!("restored");
                s["last_output"] = c.inventory().unwrap_or(Value::Null);
            })?;
            let mut out = super::decision(
                "allow",
                "recover",
                "known-local-task-work-restored-original-project-untouched",
            );
            out["recovery"] = v;
            Ok(out)
        }
        Err(e) => {
            c.event("restore-conflict", |s| {
                s["phase"] = json!("uncertain");
                s["terminal"] = json!(true);
                s["reason"] = json!(e.1);
            })?;
            Ok(c.denial("recover", &e.1))
        }
    }
}
static SHUTDOWN: AtomicBool = AtomicBool::new(false);
extern "C" fn shutdown_signal(_: libc::c_int) {
    SHUTDOWN.store(true, Ordering::SeqCst);
}
fn dispatch(c: Arc<Control>, cred: Peer, v: Value) -> Result<Value> {
    crate::task_grant::keys(&v, &["method", "grant_id", "session", "workspace"])?;
    let method = v["method"].as_str().unwrap_or("");
    if v["grant_id"] != c.g.id || v["session"] != c.g.session {
        return Ok(c.denial(method, "deny: project/session/grant binding differs"));
    }
    if cred.uid != 0 {
        if cred.uid != c.g.uid {
            return Ok(c.denial(method, "deny: peer task identity differs"));
        }
        if v["workspace"] != json!(c.work) {
            return Ok(c.denial(method, "deny: peer project workspace differs"));
        }
        let active = c.state()?["group"].as_str().map(PathBuf::from);
        if let Err(e) = Platform::validate_client(&cred, &c.work, active.as_deref(), method) {
            return Ok(c.denial(method, &e.1));
        }
        if ["revoke", "pause"].contains(&method) {
            return Ok(c.denial(method, "deny: owner operation is not an agent tool"));
        }
    }
    match method {
        "status" => {
            let a = c
                .audit
                .lock()
                .map_err(|_| Error(3, "owner audit poisoned".into()))?;
            let current = Audit::load(&a.dir)?;
            if current.head != a.head {
                return store::fail(3, "inspect-uncertain: owner evidence changed");
            }
            let mut out =
                super::decision("allow", "read-status", "read-only-owner-grant-inspection");
            out["state"] = current.state;
            out["workspace"] = json!(c.work);
            out["busy"] = json!(c.busy.load(Ordering::SeqCst));
            out["accounting"] = crate::spend::assessment(&c.g.project)?;
            out["authority_source"] = json!(
                "protected owner store and kernel IPC peer credentials, never a project file"
            );
            Ok(out)
        }
        "run" | "check" => run(c),
        "recover" => recover(&c),
        "cancel" | "revoke" | "pause" => {
            let phase = if method == "pause" {
                "paused"
            } else {
                "revoked"
            };
            c.stop(phase, "explicit owner/task stop")?;
            let mut out = super::decision("allow", method, "task-stopped-with-work-preserved");
            out["state"] = c.state()?;
            Ok(out)
        }
        _ => Ok(c.denial(
            method,
            "needs-scope: undeclared operation; no self-approval or release authority",
        )),
    }
}
/// Server stays outside all task cgroups; only the task account enters the host.
pub(super) fn serve(base: &Path, id: &str) -> Result<Value> {
    serve_mode(base, id, false)
}
pub(super) fn serve_model(base: &Path, id: &str) -> Result<Value> {
    owner()?;
    let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    writeln!(tty,"Start one separately approved memory-only device login for this model grant?\n1. Start one login\n2. Deny")?;
    let mut choice = String::new();
    BufReader::new(tty).take(32).read_line(&mut choice)?;
    if choice.trim() != "1" {
        return store::fail(3, "fresh login not authorized by owner console");
    }
    serve_mode(base, id, true)
}
fn serve_mode(base: &Path, id: &str, login_once: bool) -> Result<Value> {
    layout(base)?;
    let (g, audit) = load(base, id)?;
    actor(&g)?;
    host(&g)?;
    if login_once && g.model_repair.is_none() {
        return store::fail(3, "model grant required before a login session");
    }
    let _uid_lease = lock(&base.join("authority").join(format!("uid-{}.lease", g.uid)))?;
    let _task_lease = lock(&audit.dir.join("service.lease"))?;
    // A crashed service cannot leave a second grant running under the same UID.
    for e in fs::read_dir(base.join("authority"))? {
        let e = e?;
        if !e.file_type()?.is_dir() || e.file_name() == id {
            continue;
        }
        {
            let other = store::read(&e.path().join("grant.json"))?;
            if other["task_uid"].as_u64().is_none() {
                return store::fail(3,"inspect-uncertain: damaged sibling owner grant; outstanding task identity is unknown");
            }
            if other["task_uid"] == g.uid {
                let previous = Audit::load(&e.path())?;
                if previous.state["service_open"] == true || previous.state["group"].is_string() {
                    return store::fail(3,"inspect-uncertain: task UID has an outstanding owner lease; no concurrent authority multiplication");
                }
            }
        }
    }
    let work = base.join("work").join(&g.id);
    let control = Arc::new(Control {
        g,
        base: base.into(),
        work,
        audit: Mutex::new(audit),
        group: Mutex::new(None),
        supervisor: Mutex::new(None),
        lifecycle: Mutex::new(()),
        login_once: AtomicBool::new(login_once),
        busy: AtomicBool::new(false),
        halted: AtomicBool::new(false),
        notices: Mutex::new(BTreeSet::new()),
    });
    let old = control.state()?;
    if let Some(p) = old["group"].as_str() {
        let path = PathBuf::from(p);
        if path.exists() {
            *control
                .group
                .lock()
                .map_err(|_| Error(3, "owner group poisoned".into()))? = Some(path);
        }
    }
    if old["service_open"] == true
        || ["running", "preparing", "restoring"]
            .contains(&old["phase"].as_str().unwrap_or("uncertain"))
        || old["pending_execution"] == true
    {
        control.stop(
            "uncertain",
            "owner service interrupted; inspect exact work and evidence before resume",
        )?;
        return Ok(control.denial("serve", "inspect-uncertain: interrupted owner service fenced; use task inspect; no task action was replayed"));
    }
    control.event("service-opened", |s| {
        s["service_open"] = json!(true);
        if s["phase"] == "paused" {
            s["phase"] = json!("idle");
        }
    })?;
    let socket = base.join("sockets").join(format!("{}.sock", id));
    if let Ok(m) = fs::symlink_metadata(&socket) {
        if !m.file_type().is_socket() || m.uid() != 0 {
            return store::fail(3, "owner socket destination is not an owned stale socket");
        }
        fs::remove_file(&socket)?;
    }
    let listener = UnixListener::bind(&socket)?;
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o666))?;
    listener.set_nonblocking(true)?;
    SHUTDOWN.store(false, Ordering::SeqCst);
    unsafe {
        libc::signal(libc::SIGTERM, shutdown_signal as *const () as usize);
        libc::signal(libc::SIGINT, shutdown_signal as *const () as usize);
    }
    let connections = Arc::new(AtomicUsize::new(0));
    let mut clock = Instant::now();
    while !SHUTDOWN.load(Ordering::SeqCst) && control.state()?["phase"] != "paused" {
        match listener.accept() {
            Ok((mut stream, _)) => {
                if connections.fetch_add(1, Ordering::SeqCst) >= 8 {
                    connections.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }
                let c = control.clone();
                let active = connections.clone();
                thread::spawn(move || {
                    let result = (|| {
                        let cred = peer(&stream)?;
                        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
                        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
                        let mut text = String::new();
                        BufReader::new(&stream).take(65537).read_line(&mut text)?;
                        if text.len() > 65536 {
                            return store::fail(2, "task IPC request too large");
                        }
                        let request = serde_json::from_str(&text)
                            .map_err(|_| Error(2, "invalid task IPC JSON".into()))?;
                        dispatch(c.clone(), cred, request)
                    })();
                    let value = result.unwrap_or_else(|e| c.denial("request", &e.1));
                    let _ = writeln!(stream, "{value}");
                    active.fetch_sub(1, Ordering::SeqCst);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25))
            }
            Err(e) => return Err(e.into()),
        }
        if clock.elapsed() >= Duration::from_secs(1) {
            let s = control.state()?;
            if s["terminal"] != true {
                if let Err(e) = control.check() {
                    let _ = control.stop("blocked", &e.1);
                }
            }
            clock = Instant::now();
        }
    }
    if control.state()?["phase"] != "paused" {
        control.stop("revoked", "owner service stopped")?;
    }
    control.event("service-closed", |s| s["service_open"] = json!(false))?;
    fs::remove_file(socket)?;
    Ok(super::decision(
        "allow",
        "serve",
        "owner-service-stopped-and-evidence-preserved",
    ))
}
#[cfg(target_os = "linux")]
pub(super) fn request(
    socket: &Path,
    id: &str,
    session: &str,
    method: &str,
    root: &Path,
) -> Result<Value> {
    let mut s = UnixStream::connect(socket)?;
    if peer(&s)?.uid != 0 {
        return store::fail(3,"deny: task IPC peer is not the independent owner; project-local socket cannot mint authority");
    }
    s.set_read_timeout(Some(Duration::from_secs(86410)))?;
    writeln!(
        s,
        "{}",
        json!({"method":method,"grant_id":id,"session":session,"workspace":root.canonicalize()?})
    )?;
    let mut text = String::new();
    BufReader::new(s).take(4_000_001).read_line(&mut text)?;
    if text.len() > 4_000_000 {
        return store::fail(3, "task response exceeds bounded owner protocol");
    }
    serde_json::from_str(&text).map_err(|_| {
        Error(
            3,
            "owner IPC response incomplete; inspect status before retry".into(),
        )
    })
}

#[cfg(test)]
mod runtime_profile_tests {
    use super::*;

    #[test]
    fn runtime_mounts_resolve_system_aliases_and_reject_private_targets() {
        let shell = profile_runtime(Path::new("/bin/sh")).unwrap();
        assert_eq!(shell, Path::new("/bin/sh").canonicalize().unwrap());
        assert!(!fs::symlink_metadata(shell)
            .unwrap()
            .file_type()
            .is_symlink());
        assert!(profile_runtime(Path::new("/proc/self/exe")).is_err());
        assert!(profile_runtime(Path::new("/usr/kineti-missing-runtime")).is_err());
    }
}

#[cfg(test)]
mod audit_codec_tests {
    use super::*;
    fn record(sequence: u64, previous: &str, state: Value) -> Value {
        let mut v = json!({"schema":"kineti-owner-event-v1","sequence":sequence,"previous":previous,"event":"fixture","at":1,"state":state});
        v["hash"] = json!(digest(canonical(&v).unwrap()));
        v
    }
    #[test]
    fn shared_audit_rejects_tampering_truncation_and_interrupted_commit() {
        let first = record(1, "GENESIS", json!({"used_checks":1}));
        let second = record(2, first["hash"].as_str().unwrap(), json!({"used_checks":2}));
        let head = json!({"head":second["hash"],"sequence":2});
        assert_eq!(
            Audit::decode(&head, vec![first.clone(), second.clone()])
                .unwrap()
                .2["used_checks"],
            2
        );
        let mut changed = second.clone();
        changed["state"]["used_checks"] = json!(0);
        assert_eq!(
            Audit::decode(&head, vec![first.clone(), changed])
                .unwrap_err()
                .0,
            3
        );
        assert_eq!(Audit::decode(&head, vec![first.clone()]).unwrap_err().0, 3);
        assert_eq!(
            Audit::decode(
                &json!({"head":first["hash"],"sequence":1}),
                vec![first, second]
            )
            .unwrap_err()
            .0,
            3
        );
        assert_eq!(Audit::decode(&head, vec![]).unwrap_err().0, 3);
    }
}
