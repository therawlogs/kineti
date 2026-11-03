//! Host-only macOS VM lifetime control, not a task grant or a sandbox.
//! The task must execute inside the bound guest with no host mounts. Bindings
//! and this control channel stay on the host; same-account host attacks are not
//! covered. A caller-provided flag never proves a human's identity.
#[cfg(target_os = "macos")]
use crate::compat::digest;
#[cfg(any(target_os = "macos", test))]
use crate::store::Error;
use crate::store::{self, Result};
#[cfg(any(target_os = "macos", test))]
use serde_json::json;
use serde_json::Value;
#[cfg(target_os = "macos")]
use std::fs;
use std::path::Path;
#[cfg(any(target_os = "macos", test))]
use std::path::PathBuf;

#[cfg(any(target_os = "macos", test))]
#[derive(Clone)]
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
struct Binding {
    raw: Value,
    owner: u32,
    home: PathBuf,
    executable: PathBuf,
    instance: String,
    directory: PathBuf,
}
#[cfg(any(target_os = "macos", test))]
fn text(v: &Value, k: &str) -> Result<String> {
    v[k].as_str()
        .filter(|s| !s.is_empty() && s.len() <= 4096 && !s.contains('\0'))
        .map(str::to_owned)
        .ok_or_else(|| Error(2, format!("invalid VM binding field: {k}")))
}
#[cfg(any(target_os = "macos", test))]
fn hash(v: &Value, k: &str) -> Result<String> {
    let h = text(v, k)?;
    if h.len() != 64
        || !h
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return store::fail(2, "VM binding hashes must be lowercase SHA-256");
    }
    Ok(h)
}
#[cfg(any(target_os = "macos", test))]
impl Binding {
    fn parse(raw: Value) -> Result<Self> {
        crate::task_grant::keys(
            &raw,
            &[
                "schema",
                "owner_uid",
                "lima_home",
                "lima",
                "lima_sha256",
                "instance",
                "instance_dir",
                "config_sha256",
                "identifier_sha256",
                "hwmodel_sha256",
            ],
        )?;
        if raw["schema"] != "kineti-macos-vm-binding-v1" {
            return store::fail(2, "unsupported VM binding schema");
        }
        let owner = raw["owner_uid"]
            .as_u64()
            .filter(|n| *n <= u32::MAX as u64)
            .ok_or_else(|| Error(2, "invalid VM host owner UID".into()))?
            as u32;
        let instance = text(&raw, "instance")?;
        if instance.len() > 32
            || instance.starts_with('-')
            || !instance
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
        {
            return store::fail(2, "invalid exact VM instance name");
        }
        let home = PathBuf::from(text(&raw, "lima_home")?);
        let executable = PathBuf::from(text(&raw, "lima")?);
        let directory = PathBuf::from(text(&raw, "instance_dir")?);
        if !home.is_absolute()
            || !executable.is_absolute()
            || directory != home.join(&instance)
            || [&home, &executable, &directory].iter().any(|p| {
                p.components().any(|c| {
                    matches!(
                        c,
                        std::path::Component::ParentDir | std::path::Component::CurDir
                    )
                })
            })
        {
            return store::fail(2, "VM binding requires exact absolute host paths");
        }
        for k in [
            "lima_sha256",
            "config_sha256",
            "identifier_sha256",
            "hwmodel_sha256",
        ] {
            hash(&raw, k)?;
        }
        Ok(Self {
            raw,
            owner,
            home,
            executable,
            instance,
            directory,
        })
    }
    #[cfg(target_os = "macos")]
    fn view(&self, row: &Value) -> Value {
        json!({"schema":"kineti-vm-control-v1","instance":self.instance,"state":row["status"],"stopped_verified":stopped(row),"task_authority":false,"release_authority":false,"mechanism":"macos-vz-via-pinned-lima","owner_uid":self.owner})
    }
}
#[cfg(any(target_os = "macos", test))]
fn stopped(row: &Value) -> bool {
    row["status"] == "Stopped"
        && ["hostAgentPID", "driverPID"]
            .iter()
            .all(|k| row.get(*k).is_none() || row[*k].as_u64() == Some(0))
}
#[cfg(any(target_os = "macos", test))]
fn check_row(b: &Binding, row: &Value) -> Result<()> {
    if row["name"] != b.instance
        || row["dir"] != b.directory.to_string_lossy().as_ref()
        || row["vmType"] != "vz"
        || row["arch"] != "aarch64"
        || row["config"]["os"] != "Darwin"
        || row["config"]["plain"] != true
        || !matches!(row["config"].get("mounts"), None | Some(Value::Null))
            && row["config"]["mounts"] != json!([])
        || !matches!(row["status"].as_str(), Some("Running" | "Stopped"))
    {
        return store::fail(
            3,
            "inspect-uncertain: VM identity, isolation configuration or state differs",
        );
    }
    if row
        .get("errors")
        .is_some_and(|e| e != &Value::Null && e != &json!([]))
    {
        return store::fail(3, "inspect-uncertain: VM driver reports errors");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::{
        io::{Read, Write},
        os::unix::fs::{MetadataExt, OpenOptionsExt},
        process::{Command, Stdio},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    fn secure(p: &Path, owner: u32) -> Result<()> {
        if p.canonicalize()? != p {
            return store::fail(3, "VM control path is redirected or noncanonical");
        }
        for path in p.ancestors() {
            let m = fs::symlink_metadata(path)?;
            if m.file_type().is_symlink() || ![0, owner].contains(&m.uid()) || m.mode() & 0o022 != 0
            {
                return store::fail(
                    3,
                    "VM control path is not protected outside guest authority",
                );
            }
        }
        Ok(())
    }
    fn file_hash(p: &Path, owner: u32) -> Result<String> {
        secure(p, owner)?;
        let mut f = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(p)?;
        let m = f.metadata()?;
        if !m.is_file() || m.len() > 256 * 1024 * 1024 {
            return store::fail(3, "invalid bounded VM control artifact");
        }
        let mut bytes = vec![];
        f.read_to_end(&mut bytes)?;
        let z = f.metadata()?;
        if m.ino() != z.ino()
            || m.len() != z.len()
            || m.mtime() != z.mtime()
            || m.mtime_nsec() != z.mtime_nsec()
        {
            return store::fail(3, "VM control artifact changed while reading");
        }
        Ok(digest(bytes))
    }
    fn validate(b: &Binding) -> Result<()> {
        let uid = unsafe { libc::geteuid() };
        if uid != b.owner && uid != 0 {
            return store::fail(3, "VM lifecycle requires its host owner role");
        }
        secure(&b.home, b.owner)?;
        secure(&b.directory, b.owner)?;
        for (p, k) in [
            (b.executable.clone(), "lima_sha256"),
            (b.directory.join("lima.yaml"), "config_sha256"),
            (b.directory.join("vz-identifier"), "identifier_sha256"),
            (b.directory.join("vz-hwmodel"), "hwmodel_sha256"),
        ] {
            if file_hash(&p, b.owner)? != hash(&b.raw, k)? {
                return store::fail(3, "inspect-uncertain: pinned VM control artifact changed");
            }
        }
        Ok(())
    }
    fn output(b: &Binding, args: &[&str]) -> Result<Vec<u8>> {
        validate(b)?;
        let mut c = Command::new(&b.executable);
        c.args(args)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("LIMA_HOME", &b.home)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(home) = std::env::var_os("HOME") {
            c.env("HOME", home);
        }
        let mut child = c.spawn()?;
        let out = child.stdout.take().unwrap();
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut bytes = vec![];
            let r = out.take(1_048_577).read_to_end(&mut bytes);
            let _ = tx.send((r, bytes));
        });
        let start = Instant::now();
        loop {
            if let Some(exit) = child.try_wait()? {
                let (r, bytes) = rx.recv_timeout(Duration::from_secs(1)).map_err(|_| {
                    Error(3, "inspect-uncertain: incomplete VM control output".into())
                })?;
                r?;
                if !exit.success() || bytes.len() > 1_048_576 {
                    return store::fail(3, "inspect-uncertain: bounded VM control command failed");
                }
                return Ok(bytes);
            }
            if start.elapsed() > Duration::from_secs(5) {
                let _ = child.kill();
                let _ = child.wait();
                return store::fail(3, "inspect-uncertain: VM control command timed out");
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
    fn row(b: &Binding) -> Result<Value> {
        let bytes = output(b, &["list", "--json", &b.instance])?;
        let rows: Vec<Value> = std::str::from_utf8(&bytes)
            .map_err(|_| Error(3, "invalid VM inventory encoding".into()))?
            .lines()
            .filter(|s| !s.trim().is_empty())
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        if rows.len() != 1 {
            return store::fail(3, "inspect-uncertain: require exactly one bound VM");
        }
        check_row(b, &rows[0])?;
        Ok(rows[0].clone())
    }
    pub(super) fn load(path: &Path) -> Result<Binding> {
        let meta = fs::symlink_metadata(path)?;
        if !meta.is_file() || meta.len() > 16_384 || meta.mode() & 0o077 != 0 {
            return store::fail(3, "VM binding must be a private bounded host-owned file");
        }
        let b = Binding::parse(store::read(path)?)?;
        if meta.uid() != b.owner {
            return store::fail(3, "VM binding owner UID differs");
        }
        secure(path, b.owner)?;
        validate(&b)?;
        Ok(b)
    }
    pub(super) fn inspect(b: &Binding) -> Result<Value> {
        Ok(b.view(&row(b)?))
    }
    pub(super) fn revoke(b: &Binding) -> Result<Value> {
        let r = row(b)?;
        if !stopped(&r) {
            output(b, &["stop", "--force", &b.instance])?;
        }
        let start = Instant::now();
        loop {
            let r = row(b)?;
            if stopped(&r) {
                return Ok(b.view(&r));
            }
            if start.elapsed() > Duration::from_secs(5) {
                return store::fail(3, "inspect-uncertain: VM stop not acknowledged");
            }
            thread::sleep(Duration::from_millis(25));
        }
    }
    // A separate stop may not acknowledge a transient Stopped row while an
    // owned foreground launcher is still able to activate the VM. The active
    // controller must close its private lifetime channel and join vm-run;
    // vm-run kills/reaps its child before calling the internal revoke above.
    // Holding this same lease also fences a new vm-run during the stop check.
    pub(super) fn revoke_idle(b: &Binding) -> Result<Value> {
        let _lease = lease(b)?;
        revoke(b)
    }
    // An unreaped direct child is owned from before VM activation. Lima's
    // documented Unix foreground mode execs hostagent in that same process.
    struct OwnedDriver(std::process::Child);
    impl Drop for OwnedDriver {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn lease(b: &Binding) -> Result<fs::File> {
        use std::os::fd::AsRawFd;
        validate(b)?;
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(b.directory.join("kineti-owner.lock"))?;
        let m = file.metadata()?;
        if !m.is_file() || m.uid() != b.owner || m.mode() & 0o077 != 0 {
            return store::fail(3, "VM owner lease is not private and host-owned");
        }
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return store::fail(
                3,
                "VM already has an owner supervisor or lease is uncertain",
            );
        }
        // Keep the file, never unlink it: replacing a locked inode could admit
        // another supervisor. CLOEXEC prevents inheritance by the VM child.
        Ok(file)
    }
    pub(super) fn run(b: &Binding) -> Result<Value> {
        let _lease = lease(b)?;
        if !stopped(&row(b)?) {
            return store::fail(3, "VM run requires verified stopped state");
        }
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut byte = [0];
            let _ = std::io::stdin().read(&mut byte);
            let _ = tx.send(());
        });
        validate(b)?;
        if rx.try_recv().is_ok() {
            return revoke(b);
        }
        let mut cmd = Command::new(&b.executable);
        cmd.args(["start", "--foreground", "--tty=false", &b.instance])
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("LIMA_HOME", &b.home)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(home) = std::env::var_os("HOME") {
            cmd.env("HOME", home);
        }
        let mut driver = OwnedDriver(cmd.spawn()?);
        let pid = driver.0.id() as u64;
        println!(
            "{}",
            json!({"schema":"kineti-vm-supervisor-v1","starting":true,"owned_driver_pid":pid,"task_authority":false})
        );
        std::io::stdout().flush()?;
        let started = Instant::now();
        let mut admitted = false;
        loop {
            let cancelled = !matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty));
            if cancelled
                || driver.0.try_wait()?.is_some()
                || (!admitted && started.elapsed() > Duration::from_secs(120))
            {
                // Kill and reap before inspecting stop: no in-flight launcher
                // remains able to activate this VM after acknowledgment.
                drop(driver);
                return revoke(b);
            }
            if !admitted {
                if let Ok(r) = row(b) {
                    if r["status"] == "Running" {
                        if r["hostAgentPID"].as_u64() != Some(pid)
                            || r["driverPID"].as_u64() != Some(pid)
                        {
                            drop(driver);
                            let _ = revoke(b);
                            return store::fail(3, "VM foreground driver identity differs");
                        }
                        admitted = true;
                        println!(
                            "{}",
                            json!({"schema":"kineti-vm-supervisor-v1","ready":true,"owned_driver_pid":pid,"task_authority":false})
                        );
                        std::io::stdout().flush()?;
                    }
                }
            }
            thread::sleep(Duration::from_millis(25));
        }
    }
    pub(super) fn supervise(b: &Binding) -> Result<Value> {
        let _lease = lease(b)?;
        row(b)?;
        println!(
            "{}",
            json!({"schema":"kineti-vm-supervisor-v1","ready":true})
        );
        std::io::stdout().flush()?;
        let mut byte = [0];
        let disarm = matches!(std::io::stdin().read(&mut byte), Ok(1)) && byte[0] == b'D';
        if disarm && stopped(&row(b)?) {
            return Ok(json!({"disarmed":true,"stopped_verified":true}));
        }
        revoke(b)
    }
}
/// Inspect the exact independently owned VM binding, without granting task authority.
pub fn inspect(path: &Path) -> Result<Value> {
    dispatch(path, "inspect")
}
/// Revoke an unleased VM. For an active owner, close its private lifetime
/// channel and join it first. Never acknowledge a stop across an active launcher.
pub fn revoke(path: &Path) -> Result<Value> {
    dispatch(path, "revoke")
}
/// Own foreground startup and VM lifetime through a private owner pipe.
pub fn run(path: &Path) -> Result<Value> {
    dispatch(path, "run")
}
/// Private owner-pipe EOF revokes; disarm is accepted only after verified stop.
pub fn supervise(path: &Path) -> Result<Value> {
    dispatch(path, "supervise")
}
fn dispatch(path: &Path, op: &str) -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let b = macos::load(path)?;
        match op {
            "inspect" => macos::inspect(&b),
            "revoke" => macos::revoke_idle(&b),
            "run" => macos::run(&b),
            _ => macos::supervise(&b),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (path, op);
        store::fail(
            3,
            "macOS VM lifecycle unavailable; no process-group or sandbox fallback",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binding() -> Binding {
        Binding::parse(json!({"schema":"kineti-macos-vm-binding-v1","owner_uid":501,"lima_home":"/owner/vms","lima":"/owner/bin/lima","instance":"task1","instance_dir":"/owner/vms/task1","lima_sha256":"a".repeat(64),"config_sha256":"b".repeat(64),"identifier_sha256":"c".repeat(64),"hwmodel_sha256":"d".repeat(64)})).unwrap()
    }
    fn row() -> Value {
        json!({"name":"task1","dir":"/owner/vms/task1","vmType":"vz","arch":"aarch64","status":"Stopped","hostAgentPID":0,"driverPID":0,"config":{"os":"Darwin","plain":true,"mounts":[]}})
    }
    #[test]
    fn refuse_shared_mounts_or_replaced_vm_identity() {
        let b = binding();
        let mut r = row();
        check_row(&b, &r).unwrap();
        r["config"]["mounts"] = json!([{"location":"/private"}]);
        assert!(check_row(&b, &r).is_err());
        r = row();
        r["dir"] = json!("/owner/vms/other");
        assert!(check_row(&b, &r).is_err());
        r = row();
        r["status"] = json!("Broken");
        assert!(check_row(&b, &r).is_err());
    }
    #[test]
    fn refuse_false_stop_ack_and_unknown_authority_fields() {
        let mut r = row();
        assert!(stopped(&r));
        r["driverPID"] = json!(5);
        assert!(!stopped(&r));
        r["driverPID"] = json!("0");
        assert!(!stopped(&r));
        let mut v = binding().raw;
        v["grant_authority"] = json!(true);
        assert!(Binding::parse(v).is_err());
    }
}
