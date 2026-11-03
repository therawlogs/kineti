//! Owner-only Linux lifecycle controls. This is not a sandbox or a task grant.
//!
//! The owner must put the app-server in its protected cgroup before it executes
//! task code. All descendants must inherit membership, without delegation to the
//! task. Revocation is acknowledged only after the kernel reports no population.
use crate::store::{self, Result};
use serde_json::{json, Value};
use std::path::{Component, Path};

/// Report potential platform support without claiming configured enforcement.
pub fn status() -> Value {
    json!({"schema":"kineti-host-control-v1","platform":std::env::consts::OS,
        "linux_candidate":cfg!(target_os="linux"),"configured":false,"enforced":false,
        "task_authority":false,"mechanism":"linux-cgroup-v2-cgroup.kill",
        "requires":"independent owner UID 0, protected task cgroup, non-root task identity, launch-time membership and no task delegation",
        "limitations":["not task approval","not filesystem, network or credential isolation","not supported on native macOS","not exported as an agent MCP revocation tool"]})
}

fn validate(path: &Path, task_uid: u32) -> Result<()> {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let suffix = name.strip_prefix("kineti-task-").unwrap_or("");
    if task_uid == 0
        || path.parent() != Some(Path::new("/sys/fs/cgroup"))
        || path
            .components()
            .any(|c| !matches!(c, Component::RootDir | Component::Normal(_)))
        || suffix.is_empty()
        || suffix.len() > 64
        || !suffix
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return store::fail(2, "require non-root task UID and exact /sys/fs/cgroup/kineti-task-<id> path; no root, nested, traversal or arbitrary service group");
    }
    Ok(())
}

/// Inspect a concrete kernel group; OS ownership does not prove human identity.
pub fn inspect(path: &Path, task_uid: u32) -> Result<Value> {
    validate(path, task_uid)?;
    #[cfg(target_os = "linux")]
    {
        let group = linux::Group::open(path, task_uid)?;
        group.view(false)
    }
    #[cfg(not(target_os = "linux"))]
    store::fail(
        3,
        "Linux cgroup v2 owner control unsupported on this platform; no process-group fallback",
    )
}

/// Stop the entire protected group and verify emptiness before acknowledging it.
///
/// A timeout means uncertain completion, never permission to resume execution.
/// This irreversible local stop neither rolls back files nor revokes remote work.
pub fn revoke(path: &Path, task_uid: u32, wait_ms: u64) -> Result<Value> {
    validate(path, task_uid)?;
    if !(1..=10_000).contains(&wait_ms) {
        return store::fail(2, "host revocation wait must be 1..10000 milliseconds");
    }
    #[cfg(target_os = "linux")]
    {
        let group = linux::Group::open(path, task_uid)?;
        group.stop(wait_ms)
    }
    #[cfg(not(target_os = "linux"))]
    store::fail(
        3,
        "Linux cgroup v2 owner control unsupported on this platform; no process-group fallback",
    )
}

/// Root-only watchdog: private stdin EOF drains the pinned task group.
/// A single `D` byte disarms after the owner has independently drained it.
pub fn supervise(path: &Path, task_uid: u32) -> Result<Value> {
    validate(path, task_uid)?;
    #[cfg(target_os = "linux")]
    {
        use std::io::{Read, Write};
        let group = linux::Group::open(path, task_uid)?;
        println!(
            "{}",
            json!({"schema":"kineti-crash-supervisor-v1","ready":true})
        );
        std::io::stdout().flush()?;
        let mut byte = [0u8];
        match std::io::stdin().read(&mut byte) {
            Ok(1) if byte[0] == b'D' => Ok(json!({"disarmed":true})),
            _ => group.stop(1000),
        }
    }
    #[cfg(not(target_os = "linux"))]
    store::fail(
        3,
        "Linux crash supervision unavailable; no process-group fallback",
    )
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use crate::store::Error;
    use std::{
        ffi::CString,
        fs::File,
        io::{Read, Write},
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::fs::MetadataExt,
        },
        time::{Duration, Instant},
    };

    pub(super) struct Group {
        directory: File,
        task_uid: u32,
        name: String,
    }

    fn open_dir(path: &Path) -> Result<File> {
        use std::os::unix::ffi::OsStrExt;
        let name = CString::new(path.as_os_str().as_bytes())
            .map_err(|_| Error(2, "invalid cgroup path".into()))?;
        let fd = unsafe {
            libc::open(
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let file = unsafe { File::from_raw_fd(fd) };
        let metadata = file.metadata()?;
        if metadata.uid() != 0 || metadata.mode() & 0o022 != 0 {
            return store::fail(
                3,
                "cgroup control directory is not protected by independent root ownership",
            );
        }
        let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
        if unsafe { libc::fstatfs(file.as_raw_fd(), stat.as_mut_ptr()) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        // Linux UAPI CGROUP2_SUPER_MAGIC: reject ordinary files and cgroup v1.
        if unsafe { stat.assume_init() }.f_type != 0x6367_7270 {
            return store::fail(
                3,
                "host control requires the actual Linux cgroup v2 filesystem",
            );
        }
        Ok(file)
    }

    impl Group {
        pub(super) fn open(path: &Path, task_uid: u32) -> Result<Self> {
            if unsafe { libc::geteuid() } != 0 {
                return store::fail(3, "host revocation belongs to independent owner UID 0; task flags cannot grant this role");
            }
            // Check every fixed ancestor, then pin the directory with O_NOFOLLOW.
            for p in [Path::new("/sys"), Path::new("/sys/fs")] {
                let m = std::fs::symlink_metadata(p)?;
                if !m.is_dir() || m.uid() != 0 || m.mode() & 0o022 != 0 {
                    return store::fail(3, "unprotected or redirected cgroup ancestor");
                }
            }
            let _parent = open_dir(Path::new("/sys/fs/cgroup"))?;
            let name = path.file_name().unwrap().to_str().unwrap().to_owned();
            let target = format!("/{name}");
            let membership = std::fs::read_to_string("/proc/self/cgroup")?;
            let own = membership
                .lines()
                .find_map(|l| l.strip_prefix("0::"))
                .ok_or_else(|| Error(3, "owner cgroup v2 membership unavailable".into()))?;
            if own == target || own.starts_with(&format!("{target}/")) {
                return store::fail(3, "owner controller must remain outside the task cgroup");
            }
            let group = Self {
                directory: open_dir(path)?,
                task_uid,
                name,
            };
            // Both controls must be real files under the pinned protected group.
            group.control("cgroup.kill", true)?;
            group.populated()?;
            Ok(group)
        }

        fn control(&self, name: &str, write: bool) -> Result<File> {
            let name = CString::new(name).unwrap();
            let flags = if write {
                libc::O_WRONLY
            } else {
                libc::O_RDONLY
            };
            let fd = unsafe {
                libc::openat(
                    self.directory.as_raw_fd(),
                    name.as_ptr(),
                    flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            };
            if fd < 0 {
                return Err(std::io::Error::last_os_error().into());
            }
            let file = unsafe { File::from_raw_fd(fd) };
            let m = file.metadata()?;
            if m.uid() != 0 || m.mode() & 0o022 != 0 {
                return store::fail(3, "task-delegated cgroup control is not supported");
            }
            Ok(file)
        }

        fn populated(&self) -> Result<bool> {
            let mut events = String::new();
            self.control("cgroup.events", false)?
                .take(4096)
                .read_to_string(&mut events)?;
            match events.lines().find_map(|l| l.strip_prefix("populated ")) {
                Some("0") => Ok(false),
                Some("1") => Ok(true),
                _ => store::fail(3, "inspect-uncertain: malformed cgroup population state"),
            }
        }

        pub(super) fn view(&self, stopped: bool) -> Result<Value> {
            let populated = self.populated()?;
            if stopped && populated {
                return store::fail(3, "inspect-uncertain: owner repopulated revoked task group; keep task authority revoked");
            }
            Ok(
                json!({"schema":"kineti-host-control-v1","mechanism":"linux-cgroup-v2-cgroup.kill",
                "group":self.name,"declared_task_uid":self.task_uid,"owner_uid":0,
                "populated":populated,"revoked_running_processes":stopped,
                "task_authority":false,"grant_identity_validated":false,
                "launch_membership_validated":false,
                "scope":"owner lifecycle primitive only; not a sandbox or complete task grant",
                "remote_effects_reversed":false,"files_rolled_back":false}),
            )
        }

        pub(super) fn stop(&self, wait_ms: u64) -> Result<Value> {
            self.control("cgroup.kill", true)?.write_all(b"1")?;
            let deadline = Instant::now() + Duration::from_millis(wait_ms);
            while self.populated()? {
                if Instant::now() >= deadline {
                    return store::fail(3, "inspect-uncertain: task cgroup not empty after revocation deadline; keep authority revoked and inspect in-flight effects");
                }
                std::thread::sleep(Duration::from_millis(5));
            }
            self.view(true)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cannot_target_root_arbitrary_services_or_task_owned_authority() {
        for p in [
            "/sys/fs/cgroup",
            "/sys/fs/cgroup/openrc.sshd",
            "/tmp/kineti-task-x",
            "/sys/fs/cgroup/kineti-task-x/../kineti-task-y",
            "/sys/fs/cgroup/kineti-task-x/child",
            "/sys/fs/cgroup/kineti-task-",
            "/sys/fs/cgroup/kineti-task-X",
        ] {
            assert_eq!(revoke(Path::new(p), 2501, 100).unwrap_err().0, 2);
        }
        assert_eq!(
            inspect(Path::new("/sys/fs/cgroup/kineti-task-valid"), 0)
                .unwrap_err()
                .0,
            2
        );
    }
    #[test]
    fn never_advertises_an_unconfigured_host_as_task_authority() {
        let s = status();
        assert_eq!(s["enforced"], false);
        assert_eq!(s["task_authority"], false);
        assert_eq!(
            revoke(Path::new("/sys/fs/cgroup/kineti-task-valid"), 2501, 0)
                .unwrap_err()
                .0,
            2
        );
        assert_eq!(
            revoke(Path::new("/sys/fs/cgroup/kineti-task-valid"), 2501, 10_001)
                .unwrap_err()
                .0,
            2
        );
    }
}
