//! OS boundary for the common grant, audit, repair and recovery engine.
use crate::{
    store::{self, Result},
    task_grant::Grant,
};
use std::{
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Command},
};
/// Kernel identity; caller flags never populate it.
pub(super) struct Peer {
    pub uid: u32,
    #[cfg(target_os = "linux")]
    pub pid: libc::pid_t,
}
pub(super) struct Platform;
pub(super) trait Boundary {
    fn admit() -> Result<()>;
    fn group_path(nonce: &str) -> PathBuf;
    fn configure(group: &Path, uid: u32, memory: u64, pids: u64) -> Result<()>;
    fn supervisor_command(group: &Path, uid: u32) -> Result<Command>;
    fn stop(group: &Path, uid: u32) -> Result<()>;
    fn remove(group: &Path) -> Result<()>;
    fn frozen(group: &Path) -> Result<bool>;
    fn freeze(group: &Path, value: bool) -> Result<()>;
    fn kernel_identity() -> Result<String>;
    fn peer(stream: &UnixStream) -> Result<Peer>;
    fn validate_client(peer: &Peer, work: &Path, group: Option<&Path>, method: &str) -> Result<()>;
    fn spawn(g: &Grant, work: &Path, args: &[String], group: &Path) -> Result<Child>;
    fn placeholder(work: &Path, excluded: &str) -> Result<()>;
}
#[cfg(target_os = "linux")]
impl Boundary for Platform {
    fn admit() -> Result<()> {
        if unsafe { libc::geteuid() } != 0 {
            return store::fail(3,"owner operation requires the independent Linux UID 0 control plane; flags cannot prove identity");
        }
        Ok(())
    }
    fn group_path(nonce: &str) -> PathBuf {
        Path::new("/sys/fs/cgroup").join(format!("kineti-task-{nonce}"))
    }
    fn configure(group: &Path, uid: u32, memory: u64, pids: u64) -> Result<()> {
        crate::host_control::inspect(group, uid)?;
        for (name, value) in [("memory.max", memory), ("pids.max", pids)] {
            if !group.join(name).exists() {
                return store::fail(3,"needs-scope: owner must provide cgroup v2 memory/pids controllers; no custom sandbox fallback");
            }
            std::fs::write(group.join(name), value.to_string())?;
        }
        Ok(())
    }
    fn supervisor_command(group: &Path, uid: u32) -> Result<Command> {
        let mut command = Command::new(std::env::current_exe()?);
        command.args([
            "host",
            "supervise",
            "--cgroup",
            group.to_str().unwrap(),
            "--task-uid",
            &uid.to_string(),
        ]);
        Ok(command)
    }
    fn stop(group: &Path, uid: u32) -> Result<()> {
        crate::host_control::revoke(group, uid, 1000)?;
        Ok(())
    }
    fn remove(group: &Path) -> Result<()> {
        std::fs::remove_dir(group)?;
        Ok(())
    }
    fn frozen(group: &Path) -> Result<bool> {
        Ok(std::fs::read_to_string(group.join("cgroup.events"))?
            .lines()
            .any(|l| l == "frozen 1"))
    }
    fn freeze(group: &Path, value: bool) -> Result<()> {
        use std::{
            thread,
            time::{Duration, Instant},
        };
        std::fs::write(group.join("cgroup.freeze"), if value { b"1" } else { b"0" })?;
        if value {
            let end = Instant::now() + Duration::from_secs(2);
            while !Self::frozen(group)? {
                if Instant::now() >= end {
                    return store::fail(3, "inspect-uncertain: could not quiesce task checkpoint");
                }
                thread::sleep(Duration::from_millis(5));
            }
        }
        Ok(())
    }
    fn kernel_identity() -> Result<String> {
        Ok(std::fs::read_to_string("/proc/sys/kernel/osrelease")?)
    }
    fn peer(stream: &UnixStream) -> Result<Peer> {
        use std::os::fd::AsRawFd;
        let mut cred = std::mem::MaybeUninit::<libc::ucred>::uninit();
        let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
        if unsafe {
            libc::getsockopt(
                stream.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_PEERCRED,
                cred.as_mut_ptr().cast(),
                &mut len,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        let cred = unsafe { cred.assume_init() };
        Ok(Peer {
            uid: cred.uid,
            pid: cred.pid,
        })
    }
    fn validate_client(peer: &Peer, work: &Path, group: Option<&Path>, method: &str) -> Result<()> {
        if std::fs::read_link(format!("/proc/{}/cwd", peer.pid))? != work {
            return store::fail(3, "deny: peer project workspace differs");
        }
        if method != "status" {
            let membership = std::fs::read_to_string(format!("/proc/{}/cgroup", peer.pid))?;
            if !group.is_some_and(|g| {
                membership
                    .lines()
                    .any(|l| l == format!("0::/{}", g.file_name().unwrap().to_string_lossy()))
            }) {
                return store::fail(
                    3,
                    "deny: task client is outside its owner-launched execution group",
                );
            }
        }
        Ok(())
    }
    fn spawn(g: &Grant, work: &Path, args: &[String], group: &Path) -> Result<Child> {
        use std::{os::unix::process::CommandExt, process::Stdio};
        let script = format!(
            "exec {}",
            args.iter()
                .map(|s| shell_words::quote(s))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let mut command = Command::new("/bin/su");
        command
            .args(["-s", "/bin/sh", &g.user, "-c", &script])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("LANG", "C.UTF-8")
            .env("TZ", "UTC")
            .current_dir(work)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let group = group.to_path_buf();
        unsafe {
            command.pre_exec(move || {
                std::fs::write(group.join("cgroup.procs"), std::process::id().to_string())?;
                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        Ok(command.spawn()?)
    }
    fn placeholder(work: &Path, excluded: &str) -> Result<()> {
        use std::{
            fs::{self, OpenOptions},
            os::unix::fs::{OpenOptionsExt, PermissionsExt},
        };
        let p = work.join(excluded);
        if excluded == ".env" {
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o000)
                .open(&p)?
                .sync_all()?;
        } else {
            fs::create_dir(&p)?;
            fs::set_permissions(&p, fs::Permissions::from_mode(0o555))?;
        }
        Ok(())
    }
}
#[cfg(target_os = "macos")]
impl Boundary for Platform {
    fn admit() -> Result<()> {
        store::fail(3,"macOS owner backend not admitted: a host-controlled VM lease, protected guest service and sealed checkpoint/resource controls are required; no fallback")
    }
    fn group_path(_nonce: &str) -> PathBuf {
        PathBuf::new()
    }
    fn configure(_: &Path, _: u32, _: u64, _: u64) -> Result<()> {
        Self::admit()
    }
    fn supervisor_command(_: &Path, _: u32) -> Result<Command> {
        Self::admit()?;
        unreachable!()
    }
    fn stop(_: &Path, _: u32) -> Result<()> {
        Self::admit()
    }
    fn remove(_: &Path) -> Result<()> {
        Self::admit()
    }
    fn frozen(_: &Path) -> Result<bool> {
        Self::admit()?;
        unreachable!()
    }
    fn freeze(_: &Path, _: bool) -> Result<()> {
        Self::admit()
    }
    fn kernel_identity() -> Result<String> {
        Self::admit()?;
        unreachable!()
    }
    fn peer(_: &UnixStream) -> Result<Peer> {
        Self::admit()?;
        unreachable!()
    }
    fn validate_client(_: &Peer, _: &Path, _: Option<&Path>, _: &str) -> Result<()> {
        Self::admit()
    }
    fn spawn(_: &Grant, _: &Path, _: &[String], _: &Path) -> Result<Child> {
        Self::admit()?;
        unreachable!()
    }
    fn placeholder(_: &Path, _: &str) -> Result<()> {
        Self::admit()
    }
}

#[cfg(all(test, target_os = "macos"))]
mod admission_tests {
    use super::*;
    #[test]
    fn missing_host_lifecycle_and_checkpoint_admission_never_launches_a_task() {
        let e = Platform::admit().unwrap_err();
        assert_eq!(e.0, 3);
    }
}

#[cfg(all(test, target_os = "linux"))]
mod kernel_identity_tests {
    use super::*;
    #[test]
    fn connected_peer_identity_comes_from_the_kernel() {
        let (left, _right) = UnixStream::pair().unwrap();
        let cred = Platform::peer(&left).unwrap();
        assert_eq!(cred.uid, unsafe { libc::geteuid() });
        assert_eq!(cred.pid, unsafe { libc::getpid() });
    }
}
