//! Supported guest storage/resource admission. This grants no task authority.
use crate::store::{self, Result};
use serde_json::Value;
/// Verify the exact guest prerequisites before the owner backend can execute.
/// This operation is deliberately unavailable on a physical host or as a task UID.
pub fn preflight() -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        macos::preflight()
    }
    #[cfg(not(target_os = "macos"))]
    {
        store::fail(
            3,
            "macOS guest admission unavailable on this platform; no fallback",
        )
    }
}
/// Root-worker cold checkpoint primitive. It grants no execution authority.
pub fn cold(phase: &str, request: Value) -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        macos::cold(phase, request)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (phase, request);
        store::fail(
            3,
            "cold macOS guest checkpoint unavailable on this platform",
        )
    }
}
#[cfg(target_os = "macos")]
#[path = "task_guest_worker.rs"]
mod worker;
/// Root-only fixed guest control service, never a task approval service.
pub fn worker_serve() -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        worker::serve()
    }
    #[cfg(not(target_os = "macos"))]
    {
        store::fail(3, "macOS guest worker unavailable")
    }
}
/// Existing owner-role bridge; task callers cannot connect as that role.
pub fn worker_connect() -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        worker::connect()
    }
    #[cfg(not(target_os = "macos"))]
    {
        store::fail(3, "macOS guest worker bridge unavailable")
    }
}
#[cfg(any(target_os = "macos", test))]
fn guest_identity(uid: u32, model: &str) -> Result<()> {
    if uid != 0 || !model.trim().starts_with("VirtualMac") {
        return store::fail(3,"guest control requires the independent guest root role on a VirtualMac; flags do not prove identity");
    }
    Ok(())
}
#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use crate::{compat::digest, store::Error};
    use serde_json::json;
    use std::os::fd::AsRawFd;
    use std::{
        fs::{self, File, OpenOptions},
        io::{Read, Write},
        os::unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        },
        path::Path,
        process::{Command, Stdio},
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };
    const BASE: &str = "/private/var/lib/kineti-owner-guest-v1";
    const CAPACITY: u64 = 256 * 1024 * 1024;
    pub(super) fn command(program: &str, args: &[&str]) -> Result<String> {
        let mut c = Command::new(program)
            .args(args)
            .env_clear()
            .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
            .env("LANG", "C")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let out = c.stdout.take().unwrap();
        let err = c.stderr.take().unwrap();
        let (tx, rx) = mpsc::channel();
        let readers: [(&str, Box<dyn Read + Send>); 2] =
            [("stdout", Box::new(out)), ("stderr", Box::new(err))];
        for (label, mut reader) in readers {
            let tx = tx.clone();
            thread::spawn(move || {
                let mut b = vec![];
                let result = reader.by_ref().take(65537).read_to_end(&mut b);
                let _ = tx.send((label, result, b));
            });
        }
        let end = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(s) = c.try_wait()? {
                break s;
            }
            if Instant::now() >= end {
                let _ = c.kill();
                let _ = c.wait();
                return store::fail(3, format!("guest admission command timed out: {program}"));
            }
            thread::sleep(Duration::from_millis(10));
        };
        let mut stdout = vec![];
        let mut stderr = vec![];
        for _ in 0..2 {
            let (label, result, b) = rx
                .recv_timeout(Duration::from_secs(1))
                .map_err(|_| Error(3, "guest admission output incomplete".into()))?;
            result?;
            if b.len() > 65536 {
                return store::fail(3, "guest admission output exceeds bound");
            }
            if label == "stdout" {
                stdout = b
            } else {
                stderr = b
            }
        }
        if !status.success() {
            return store::fail(
                3,
                format!(
                    "supported guest command failed: {program} {:?}: stderr={} stdout={}",
                    args,
                    String::from_utf8_lossy(&stderr),
                    String::from_utf8_lossy(&stdout)
                ),
            );
        }
        String::from_utf8(stdout)
            .map_err(|_| Error(3, "guest admission output is not UTF-8".into()))
    }
    // Public Darwin API. No entitlement is added or security setting changed.
    // The actual kernel result, including entitlement denial, is admission
    // evidence rather than a reason to substitute a custom snapshot mechanism.
    fn create_snapshot(root: &Path, name: &str) -> Result<()> {
        unsafe extern "C" {
            fn fs_snapshot_create(
                fd: libc::c_int,
                name: *const libc::c_char,
                flags: u32,
            ) -> libc::c_int;
        }
        let fd = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(root)?;
        let name = std::ffi::CString::new(name)
            .map_err(|_| Error(3, "invalid fixed snapshot name".into()))?;
        if unsafe { fs_snapshot_create(fd.as_raw_fd(), name.as_ptr(), 0) } != 0 {
            return store::fail(3,format!("native fs_snapshot_create denied: {}; Apple documents root plus an additional entitlement; no signing or security bypass is enabled",std::io::Error::last_os_error()));
        }
        Ok(())
    }
    pub(super) fn sysctl(name: &str, value: Option<i32>) -> Result<i32> {
        let name = std::ffi::CString::new(name).unwrap();
        let mut old = 0i32;
        let mut size = std::mem::size_of::<i32>();
        let mut new = value.unwrap_or(0);
        let (ptr, len) = if value.is_some() {
            ((&mut new as *mut i32).cast(), size)
        } else {
            (std::ptr::null_mut(), 0)
        };
        if unsafe {
            libc::sysctlbyname(
                name.as_ptr(),
                (&mut old as *mut i32).cast(),
                &mut size,
                ptr,
                len,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error().into());
        }
        if size != std::mem::size_of::<i32>() {
            return store::fail(3, "unsupported guest kernel limit encoding");
        };
        Ok(old)
    }
    pub(super) fn secure(path: &Path) -> Result<()> {
        if path.canonicalize()? != path {
            return store::fail(3, "guest control path is redirected");
        }
        for p in path.ancestors() {
            let m = fs::symlink_metadata(p)?;
            if m.uid() != 0 || m.mode() & 0o022 != 0 || m.file_type().is_symlink() {
                return store::fail(3, "guest control path is not protected from task writes");
            }
        }
        Ok(())
    }
    fn filesystem(path: &Path) -> Result<(String, u64, u64, String)> {
        let p = std::ffi::CString::new(path.as_os_str().as_bytes())
            .map_err(|_| Error(3, "invalid volume path".into()))?;
        let mut s = std::mem::MaybeUninit::<libc::statfs>::uninit();
        if unsafe { libc::statfs(p.as_ptr(), s.as_mut_ptr()) } != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let s = unsafe { s.assume_init() };
        let name = unsafe { std::ffi::CStr::from_ptr(s.f_fstypename.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let device = unsafe { std::ffi::CStr::from_ptr(s.f_mntfromname.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        if !device.starts_with("/dev/disk")
            || !device[9..].bytes().all(|b| b.is_ascii_alphanumeric())
        {
            return store::fail(3, "unexpected guest APFS device identity");
        }
        Ok((
            name,
            s.f_blocks * s.f_bsize as u64,
            s.f_flags as u64,
            device,
        ))
    }
    pub(super) struct Limits {
        processes: i32,
        per_uid: i32,
    }
    impl Limits {
        pub(super) fn admit() -> Result<Self> {
            let restore = Self {
                processes: sysctl("kern.maxproc", None)?,
                per_uid: sysctl("kern.maxprocperuid", None)?,
            };
            sysctl("kern.maxproc", Some(512))?;
            sysctl("kern.maxprocperuid", Some(128))?;
            if sysctl("kern.maxproc", None)? != 512 || sysctl("kern.maxprocperuid", None)? != 128 {
                return store::fail(
                    3,
                    "guest process caps failed readback; no broker/process-group fallback",
                );
            };
            Ok(restore)
        }
        fn restore(&self) -> Result<()> {
            sysctl("kern.maxproc", Some(self.processes))?;
            sysctl("kern.maxprocperuid", Some(self.per_uid))?;
            Ok(())
        }
    }
    impl Drop for Limits {
        fn drop(&mut self) {
            let _ = self.restore();
        }
    }
    pub(super) fn cold(phase: &str, q: Value) -> Result<Value> {
        if unsafe { libc::geteuid() } != 0 {
            return store::fail(
                3,
                "cold checkpoint requires the independent guest root worker",
            );
        }
        guest_identity(
            unsafe { libc::geteuid() },
            &command("/usr/sbin/sysctl", &["-n", "hw.model"])?,
        )?;
        crate::task_grant::keys(
            &q,
            &[
                "schema",
                "id",
                "session",
                "grant_digest",
                "expires_at",
                "checkpoint",
            ],
        )?;
        if q["schema"] != "kineti-guest-cold-checkpoint-v1" {
            return store::fail(3, "unsupported guest checkpoint protocol");
        }
        let text = |k: &str| {
            q[k].as_str()
                .ok_or_else(|| Error(3, format!("checkpoint field absent: {k}")))
        };
        let id = text("id")?;
        let session = text("session")?;
        crate::task_grant::id(id)?;
        crate::task_grant::id(session)?;
        let grant = text("grant_digest")?;
        if grant.len() != 64
            || !grant
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return store::fail(3, "invalid checkpoint grant digest");
        }
        let now = time::OffsetDateTime::now_utc().unix_timestamp();
        let expires = q["expires_at"]
            .as_i64()
            .filter(|v| *v > now && *v <= now + 86400)
            .ok_or_else(|| Error(3, "expired or unsupported checkpoint lifetime".into()))?;
        let boot = command("/usr/sbin/sysctl", &["-n", "kern.bootsessionuuid"])?;
        let boot = boot.trim();
        let base = Path::new(BASE);
        if !base.exists() {
            secure(Path::new("/private/var/lib"))?;
            fs::create_dir(base)?;
            fs::set_permissions(base, fs::Permissions::from_mode(0o700))?
        }
        secure(base)?;
        let case = base.join(format!("cold-{id}"));
        let live = case.join("live");
        let image = case.join("task.dmg");
        let state = case.join("state.json");
        let common = json!({"schema":"kineti-guest-cold-state-v1","id":id,"session":session,"grant_digest":grant,"expires_at":expires,"task_authority":false,"release_authority":false});
        if phase == "prepare" {
            if fs::symlink_metadata(&case).is_ok() {
                return store::fail(
                    3,
                    "inspect-uncertain: cold checkpoint already exists; partial work is preserved",
                );
            }
            if q["checkpoint"].is_null() {
                return store::fail(3, "initial copied workspace is absent");
            }
            fs::create_dir(&case)?;
            fs::set_permissions(&case, fs::Permissions::from_mode(0o700))?;
            fs::create_dir(&live)?;
            let mut meta = common;
            meta["phase"] = json!("preparing");
            meta["prepared_boot"] = json!(boot);
            store::write(&state, &meta)?;
            command(
                "/usr/bin/hdiutil",
                &[
                    "create",
                    "-size",
                    "256m",
                    "-type",
                    "UDIF",
                    "-fs",
                    "APFS",
                    "-volname",
                    "KinetiTask",
                    image.to_str().unwrap(),
                ],
            )?;
            fs::set_permissions(&image, fs::Permissions::from_mode(0o600))?;
            command(
                "/usr/bin/hdiutil",
                &[
                    "attach",
                    "-owners",
                    "on",
                    "-nobrowse",
                    "-mountpoint",
                    live.to_str().unwrap(),
                    image.to_str().unwrap(),
                ],
            )?;
            let (kind, capacity, flags, _) = filesystem(&live)?;
            if kind != "apfs" || capacity > CAPACITY || flags & libc::MNT_RDONLY as u64 != 0 {
                return store::fail(3, "cold task volume identity or capacity differs");
            }
            let copied = crate::task_checkpoint::decode(
                &live.join("work"),
                &q["checkpoint"],
                &[],
                crate::task_checkpoint::MAX_BYTES,
                31001,
                31001,
            )?;
            meta["phase"] = json!("prepared");
            meta["capacity"] = json!(capacity);
            meta["initial_inventory_digest"] = json!(digest(serde_json::to_vec(&copied)?));
            store::write(&state, &meta)?;
            return Ok(
                json!({"schema":"kineti-guest-cold-result-v1","phase":"prepared","workspace":live.join("work"),"boot":boot,"capacity":capacity,"task_authority":false,"release_authority":false}),
            );
        }
        if !q["checkpoint"].is_null() {
            return store::fail(
                3,
                "checkpoint payload is supported only for initial prepare",
            );
        }
        secure(&case)?;
        secure(&state)?;
        secure(&image)?;
        let m = fs::metadata(&image)?;
        if !m.is_file() || m.nlink() != 1 || m.mode() & 0o077 != 0 || m.len() > CAPACITY + 1_048_576
        {
            return store::fail(3, "root-controlled task image metadata differs");
        }
        let mut meta = store::read(&state)?;
        for key in ["id", "session", "grant_digest", "expires_at"] {
            if meta[key] != common[key] {
                return store::fail(3, "cold checkpoint binding differs");
            }
        }
        if phase == "shutdown" || phase == "arm-shutdown" {
            if meta["phase"] != "prepared" || meta["prepared_boot"] != boot {
                return store::fail(
                    3,
                    "inspect-uncertain: shutdown phase or boot binding differs; no replay",
                );
            }
            meta["phase"] = json!("shutdown-intent");
            meta["shutdown_boot"] = json!(boot);
            store::write(&state, &meta)?;
            // This is a fixed OS shutdown operation, never a caller-supplied
            // privileged command. The host must still join its owned driver
            // and verify stopped state before accepting any checkpoint.
            command("/bin/sync", &[])?;
            if phase == "arm-shutdown" {
                return Ok(
                    json!({"schema":"kineti-guest-cold-result-v1","phase":"shutdown-armed","stopped_verified":false,"task_authority":false,"release_authority":false}),
                );
            }
            let child = Command::new("/sbin/shutdown")
                .args(["-h", "now"])
                .env_clear()
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            return Ok(
                json!({"schema":"kineti-guest-cold-result-v1","phase":"shutdown-requested","shutdown_process":child.id(),"stopped_verified":false,"task_authority":false,"release_authority":false}),
            );
        }
        if phase != "export" {
            return store::fail(3, "unsupported cold checkpoint operation");
        }
        if meta["phase"] != "shutdown-intent" || meta["shutdown_boot"] == boot {
            return store::fail(3,"cold export requires a fresh worker-only boot after shutdown; no live-directory fallback");
        }
        command(
            "/usr/bin/hdiutil",
            &[
                "attach",
                "-readonly",
                "-owners",
                "on",
                "-nobrowse",
                "-mountpoint",
                live.to_str().unwrap(),
                image.to_str().unwrap(),
            ],
        )?;
        let (kind, capacity, flags, _) = filesystem(&live)?;
        if kind != "apfs" || capacity > CAPACITY || flags & libc::MNT_RDONLY as u64 == 0 {
            return store::fail(
                3,
                "cold export mount is not a bounded read-only APFS projection",
            );
        }
        let wire = crate::task_checkpoint::encode(
            &live.join("work"),
            &[],
            crate::task_checkpoint::MAX_BYTES,
        )?;
        let seal = digest(serde_json::to_vec(&wire)?);
        command("/usr/bin/hdiutil", &["detach", live.to_str().unwrap()])?;
        meta["phase"] = json!("exported");
        meta["export_boot"] = json!(boot);
        meta["checkpoint_digest"] = json!(seal);
        store::write(&state, &meta)?;
        Ok(
            json!({"schema":"kineti-guest-cold-result-v1","phase":"exported","checkpoint":wire,"checkpoint_digest":seal,"previous_boot":meta["shutdown_boot"],"export_boot":boot,"read_only_verified":true,"task_authority":false,"release_authority":false}),
        )
    }
    pub(super) fn preflight() -> Result<Value> {
        if unsafe { libc::geteuid() } != 0 {
            return store::fail(
                3,
                "guest resource admission requires the existing independent guest root service",
            );
        }
        let model = command("/usr/sbin/sysctl", &["-n", "hw.model"])?;
        guest_identity(unsafe { libc::geteuid() }, &model)?;
        let base = Path::new(BASE);
        secure(Path::new("/private/var"))?;
        if !Path::new("/private/var/lib").exists() {
            fs::create_dir("/private/var/lib")?;
            fs::set_permissions("/private/var/lib", fs::Permissions::from_mode(0o755))?
        }
        secure(Path::new("/private/var/lib"))?;
        if !base.exists() {
            fs::create_dir(base)?;
            fs::set_permissions(base, fs::Permissions::from_mode(0o700))?
        }
        secure(base)?;
        let case = base.join(format!("admission-{}", store::nonce()?));
        fs::create_dir(&case)?;
        fs::set_permissions(&case, fs::Permissions::from_mode(0o700))?;
        let live = case.join("live");
        let sealed = case.join("sealed");
        fs::create_dir(&live)?;
        fs::create_dir(&sealed)?;
        let image = case.join("task.dmg");
        let image_text = image.to_str().unwrap();
        let live_text = live.to_str().unwrap();
        let sealed_text = sealed.to_str().unwrap();
        let mut record = json!({"schema":"kineti-macos-guest-admission-v1","task_authority":false,"release_authority":false,"verified":false,"case":case,"volume_capacity_limit":CAPACITY,"task_uid_process_limit":128,"whole_vm_process_limit":512,"checks":{}});
        let result = (|| -> Result<()> {
            let limits = Limits::admit()?;
            record["checks"]["kernel_process_caps_readback"] = json!(true);
            record["kernel_limits_before"] =
                json!({"processes":limits.processes,"per_uid":limits.per_uid});
            command(
                "/usr/bin/hdiutil",
                &[
                    "create",
                    "-size",
                    "256m",
                    "-type",
                    "UDIF",
                    "-fs",
                    "APFS",
                    "-volname",
                    "KinetiTask",
                    image_text,
                ],
            )?;
            command(
                "/usr/bin/hdiutil",
                &[
                    "attach",
                    "-owners",
                    "on",
                    "-nobrowse",
                    "-mountpoint",
                    live_text,
                    image_text,
                ],
            )?;
            let (kind, capacity, flags, device) = filesystem(&live)?;
            if kind != "apfs"
                || capacity == 0
                || capacity > CAPACITY
                || flags & libc::MNT_RDONLY as u64 != 0
            {
                return store::fail(3, "bounded APFS volume readback differs");
            };
            record["checks"]["bounded_apfs_capacity"] = json!(capacity);
            fs::create_dir(live.join("work"))?;
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(live.join("work/value.txt"))?;
            f.write_all(b"before\n")?;
            f.sync_all()?;
            File::open(live.join("work"))?.sync_all()?;
            let snap = "kineti-sealed-admission";
            create_snapshot(&live, snap)?;
            // Mount the kernel-reported APFS device snapshot, never substitute
            // a scan of the mutable task directory.
            command(
                "/sbin/mount_apfs",
                &[
                    "-s",
                    snap,
                    "-o",
                    "rdonly,nosuid,nodev,noexec",
                    &device,
                    sealed_text,
                ],
            )?;
            let (kind, _, flags, _) = filesystem(&sealed)?;
            if kind != "apfs" || flags & libc::MNT_RDONLY as u64 == 0 {
                return store::fail(3, "snapshot is not a read-only APFS mount");
            };
            fs::write(live.join("work/value.txt"), b"after\n")?;
            if fs::read(sealed.join("work/value.txt"))? != b"before\n" {
                return store::fail(3, "sealed checkpoint changed after live writes");
            }
            match fs::write(sealed.join("work/value.txt"), b"forbidden\n") {
                Err(e) if e.raw_os_error() == Some(libc::EROFS) => {}
                _ => {
                    return store::fail(
                        3,
                        "snapshot write denial was not read-only filesystem enforcement",
                    )
                }
            }
            let fill_path = live.join("quota-control");
            let mut fill = OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .open(&fill_path)?;
            let block = vec![0x5au8; 1_048_576];
            let mut written = 0u64;
            let mut exhausted = false;
            for _ in 0..=256 {
                match fill.write_all(&block) {
                    Ok(()) => written += block.len() as u64,
                    Err(e) if e.raw_os_error() == Some(libc::ENOSPC) => {
                        exhausted = true;
                        break;
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            if !exhausted || written > CAPACITY {
                return store::fail(
                    3,
                    "bounded guest volume did not enforce ENOSPC at the declared capacity",
                );
            }
            drop(fill);
            fs::remove_file(fill_path)?;
            record["checks"]["disk_enospc_control"] =
                json!({"observed":true,"bytes_written_before_denial":written});
            let wire = crate::task_checkpoint::encode(&sealed.join("work"), &[], 1024)?;
            record["checks"]["sealed_checkpoint_sha256"] =
                json!(digest(serde_json::to_vec(&wire)?));
            record["checks"]["snapshot_readonly_and_stable"] = json!(true);
            command("/sbin/umount", &[sealed_text])?;
            command("/usr/bin/hdiutil", &["detach", live_text])?;
            limits.restore()?;
            record["checks"]["kernel_limits_restored"] = json!(true);
            Ok(())
        })();
        // Preserve failed state for inspection. The host owns the whole-VM
        // lifetime and stops the guest before any execution can be admitted.
        match result {
            Ok(()) => {
                record["verified"] = json!(true);
                store::write(&case.join("result.json"), &record)?;
                Ok(record)
            }
            Err(e) => {
                record["failure"] = json!(e.1);
                store::write(&case.join("result.json"), &record)?;
                store::fail(
                    3,
                    format!(
                        "guest capability admission failed; preserved {}: {}",
                        case.display(),
                        e.1
                    ),
                )
            }
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn caller_flags_cannot_admit_guest_control() {
        assert!(super::guest_identity(501, "VirtualMac2,1").is_err());
        assert!(super::guest_identity(0, "MacBookPro").is_err());
        assert!(super::guest_identity(0, "VirtualMac2,1").is_ok());
    }
}
