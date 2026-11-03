//! Guest control transport. Host approval remains outside this VM; no agent or
//! privileged arbitrary-command API is exposed by this endpoint.
use super::{
    guest_identity,
    macos::{command, secure, sysctl, Limits},
};
use crate::{
    compat::digest,
    store::{self, Error, Result},
};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{
            fs::{FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
            net::{UnixListener, UnixStream},
        },
    },
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
const BASE: &str = "/private/var/lib/kineti-owner-guest-v1";
const SOCKET: &str = "/private/var/lib/kineti-owner-guest-v1/control.sock";
const BOUND: u64 = 32_000_000;
fn peer(stream: &UnixStream) -> Result<u32> {
    let mut uid = 0;
    let mut gid = 0;
    if unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    };
    Ok(uid)
}
fn config() -> Result<Value> {
    let p = Path::new(BASE).join("worker.json");
    secure(&p)?;
    let m = fs::metadata(&p)?;
    if !m.is_file() || m.len() > 16384 || m.mode() & 0o077 != 0 {
        return store::fail(
            3,
            "worker config must be private, bounded and root-controlled",
        );
    }
    let v = store::read(&p)?;
    crate::task_grant::keys(
        &v,
        &[
            "schema",
            "owner_uid",
            "owner_gid",
            "task_uid",
            "task_gid",
            "worker_sha256",
            "sdk",
            "sdk_sha256",
        ],
    )?;
    if v["schema"] != "kineti-macos-guest-worker-v1"
        || v["owner_uid"] != 501
        || v["owner_gid"] != 20
        || v["task_uid"] != 31001
        || v["task_gid"] != 31001
    {
        return store::fail(3, "unsupported exact guest worker identity class");
    }
    let executable = std::env::current_exe()?.canonicalize()?;
    secure(&executable)?;
    if v["worker_sha256"] != digest(fs::read(&executable)?) {
        return store::fail(
            3,
            "guest worker executable differs from its protected installation",
        );
    }
    let sdk = Path::new(
        v["sdk"]
            .as_str()
            .ok_or_else(|| Error(3, "worker SDK path absent".into()))?,
    );
    secure(sdk)?;
    let m = fs::metadata(sdk)?;
    if !m.is_file()
        || m.len() > 256 * 1024 * 1024
        || m.mode() & 0o111 == 0
        || v["sdk_sha256"] != digest(fs::read(sdk)?)
    {
        return store::fail(3, "guest SDK installation differs");
    };
    Ok(v)
}
fn dispatch(q: Value, cfg: &Value) -> Result<Value> {
    crate::task_grant::keys(&q, &["schema", "method", "payload"])?;
    if q["schema"] != "kineti-guest-control-v1" {
        return store::fail(3, "unsupported guest control envelope");
    }
    match q["method"].as_str(){
  Some("probe")=>{
   if !q["payload"].is_null(){return store::fail(3,"probe takes no task scope or command arguments")}
   Ok(json!({"schema":"kineti-guest-worker-capabilities-v1","worker_sha256":cfg["worker_sha256"],"sdk_sha256":cfg["sdk_sha256"],"sdk_version":command(cfg["sdk"].as_str().unwrap(),&["--version"])?.trim(),"kernel":command("/usr/sbin/sysctl",&["-n","kern.osrelease"])?.trim(),"boot":command("/usr/sbin/sysctl",&["-n","kern.bootsessionuuid"])?.trim(),"memory_bytes":command("/usr/sbin/sysctl",&["-n","hw.memsize"])?.trim(),"processes":sysctl("kern.maxproc",None)?,"processes_per_uid":sysctl("kern.maxprocperuid",None)?,"methods":["probe","cold-prepare","cold-shutdown","cold-export"],"arbitrary_root_execution":false,"task_execution_admitted":false,"task_authority":false,"release_authority":false}))
  },
  Some("cold-prepare")=>super::cold("prepare",q["payload"].clone()),
  Some("cold-shutdown")=>super::cold("arm-shutdown",q["payload"].clone()),
  Some("cold-export")=>super::cold("export",q["payload"].clone()),
  _=>store::fail(3,"guest control operation is unsupported; no command/argv, grant-approval or release fallback")
 }
}
pub(super) fn serve() -> Result<Value> {
    if unsafe { libc::geteuid() } != 0 {
        return store::fail(3, "guest worker requires existing root service authority");
    };
    guest_identity(
        unsafe { libc::geteuid() },
        &command("/usr/sbin/sysctl", &["-n", "hw.model"])?,
    )?;
    let base = Path::new(BASE);
    secure(base)?;
    let cfg = config()?;
    let _limits = Limits::admit()?;
    let lease = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(base.join("worker.lock"))?;
    if unsafe { libc::flock(lease.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return store::fail(3, "guest worker already has an active lease");
    }
    fs::set_permissions(base, fs::Permissions::from_mode(0o711))?;
    if let Ok(m) = fs::symlink_metadata(SOCKET) {
        if !m.file_type().is_socket() || m.uid() != 0 {
            return store::fail(3, "guest control socket is not an owned stale endpoint");
        };
        fs::remove_file(SOCKET)?
    }
    let listener = UnixListener::bind(SOCKET)?;
    let name = std::ffi::CString::new(SOCKET).unwrap();
    if unsafe { libc::chown(name.as_ptr(), 0, 20) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    };
    fs::set_permissions(SOCKET, fs::Permissions::from_mode(0o660))?;
    listener.set_nonblocking(true)?;
    store::write(
        &base.join("ready.json"),
        &json!({"schema":"kineti-guest-worker-ready-v1","worker_sha256":cfg["worker_sha256"],"pid":std::process::id(),"owner_role_uid":501,"task_role_uid":31001,"task_authority":false,"task_execution_admitted":false}),
    )?;
    let count = Arc::new(AtomicUsize::new(0));
    let mutation = Arc::new(Mutex::new(()));
    let cfg = Arc::new(cfg);
    loop {
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Authenticate before parsing or buffering attacker-controlled input.
                if peer(&stream)? != 501 {
                    continue;
                }
                if count.fetch_add(1, Ordering::SeqCst) >= 8 {
                    count.fetch_sub(1, Ordering::SeqCst);
                    continue;
                }
                let count = count.clone();
                let cfg = cfg.clone();
                let mutation = mutation.clone();
                thread::spawn(move || {
                    let answer = (|| -> Result<Value> {
                        stream.set_read_timeout(Some(Duration::from_secs(15)))?;
                        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
                        let mut line = String::new();
                        BufReader::new(&stream)
                            .take(BOUND + 1)
                            .read_line(&mut line)?;
                        if line.len() as u64 > BOUND {
                            return store::fail(3, "guest control request exceeds bound");
                        };
                        let q: Value = serde_json::from_str(&line)
                            .map_err(|_| Error(3, "invalid guest control JSON".into()))?;
                        let _lease = if q["method"] != "probe" {
                            Some(mutation.try_lock().map_err(|_|Error(3,"guest checkpoint operation already in progress; no concurrent mutation".into()))?)
                        } else {
                            None
                        };
                        dispatch(q, &cfg)
                    })();
                    let out = match answer {
                        Ok(value) => {
                            json!({"schema":"kineti-guest-control-response-v1","ok":true,"result":value})
                        }
                        Err(e) => {
                            json!({"schema":"kineti-guest-control-response-v1","ok":false,"code":e.0,"error":e.1})
                        }
                    };
                    // Acknowledge durable intent before launchd tears down this
                    // job and its connections. A lost response never proves stop.
                    let halt = out["ok"] == true && out["result"]["phase"] == "shutdown-armed";
                    let sent = writeln!(stream, "{out}")
                        .and_then(|()| stream.flush())
                        .is_ok();
                    if halt && sent {
                        // Fixed supported launchd operation, not a command API.
                        // Host authority still joins and verifies its VM driver.
                        let _ = command("/bin/launchctl", &["reboot", "halt"]);
                    }
                    count.fetch_sub(1, Ordering::SeqCst);
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(20))
            }
            Err(e) => return Err(e.into()),
        }
    }
}
/// Existing guest-owner SSH role bridges one bounded request to a root peer.
/// The task UID is rejected by both the bridge and kernel-authenticated server.
pub(super) fn connect() -> Result<Value> {
    if unsafe { libc::geteuid() } != 501 {
        return store::fail(
            3,
            "guest control bridge requires the existing owner role; task flags cannot assume it",
        );
    }
    let mut stream = UnixStream::connect(SOCKET)?;
    if peer(&stream)? != 0 {
        return store::fail(3, "guest worker peer is not kernel-authenticated root");
    };
    stream.set_read_timeout(Some(Duration::from_secs(90)))?;
    stream.set_write_timeout(Some(Duration::from_secs(10)))?;
    let mut bytes = vec![];
    std::io::stdin().take(BOUND + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > BOUND {
        return store::fail(3, "guest control input exceeds bound");
    };
    let q: Value =
        serde_json::from_slice(&bytes).map_err(|_| Error(3, "invalid control request".into()))?;
    writeln!(stream, "{q}")?;
    let mut line = String::new();
    BufReader::new(stream)
        .take(BOUND + 1)
        .read_line(&mut line)?;
    if line.len() as u64 > BOUND {
        return store::fail(3, "guest control response exceeds bound");
    };
    let response: Value = serde_json::from_str(&line).map_err(|_| {
        Error(
            3,
            "guest worker response incomplete; inspect before retry".into(),
        )
    })?;
    if response["schema"] != "kineti-guest-control-response-v1" {
        return store::fail(3, "unexpected guest control response schema");
    }
    if response["ok"] != true {
        return store::fail(
            3,
            response["error"]
                .as_str()
                .unwrap_or("guest worker denied the operation"),
        );
    }
    Ok(response["result"].clone())
}
