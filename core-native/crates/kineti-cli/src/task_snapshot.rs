//! Bounded, no-follow task checkpoints. Original projects are never restored over.
use crate::{
    compat::{canonical, digest},
    store::{self, Error, Result},
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path},
};

/// Snapshot and host exclusions share this definition; no credential-value scanner.
pub fn excluded(path: &str, exclusions: &[String]) -> bool {
    let p = Path::new(path);
    exclusions.iter().any(|x| p.starts_with(x))
        || p.components().any(|x| {
            let s = x.as_os_str().to_string_lossy();
            [".git", ".kineti", ".ssh", ".aws", ".codex"].contains(&s.as_ref())
                || s.starts_with(".env")
                || [".pem", ".key", ".private"].iter().any(|e| s.ends_with(e))
        })
}
/// Reject redirected parent directories before any task-owned file operation.
pub fn guard(root: &Path, rel: &str) -> Result<()> {
    crate::task_grant::relative(rel, false)?;
    if root.canonicalize()? != root || fs::symlink_metadata(root)?.file_type().is_symlink() {
        return store::fail(3, "inspect-uncertain: task root redirected");
    }
    let mut p = root.to_path_buf();
    let parts = Path::new(rel).components().collect::<Vec<_>>();
    for x in &parts[..parts.len() - 1] {
        if let Component::Normal(x) = x {
            p.push(x);
            if fs::symlink_metadata(&p)?.file_type().is_symlink() {
                return store::fail(
                    3,
                    "inspect-uncertain: parent symlink; no restoration performed",
                );
            }
        }
    }
    Ok(())
}
fn regular(root: &Path, rel: &str) -> Result<File> {
    guard(root, rel)?;
    // Pin each directory descriptor, so replacing a parent with a symlink cannot
    // redirect a later read outside the checkpoint root.
    use std::{
        ffi::CString,
        os::{
            fd::{AsRawFd, FromRawFd},
            unix::ffi::OsStrExt,
        },
    };
    let mut fd = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root)?;
    let parts = Path::new(rel).components().collect::<Vec<_>>();
    for (i, c) in parts.iter().enumerate() {
        let c = CString::new(c.as_os_str().as_bytes())
            .map_err(|_| Error(2, "invalid checkpoint path".into()))?;
        let flags = libc::O_RDONLY
            | libc::O_NOFOLLOW
            | libc::O_CLOEXEC
            | if i + 1 < parts.len() {
                libc::O_DIRECTORY
            } else {
                libc::O_NONBLOCK
            };
        let n = unsafe { libc::openat(fd.as_raw_fd(), c.as_ptr(), flags) };
        if n < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        fd = unsafe { File::from_raw_fd(n) };
    }
    if !fd.metadata()?.is_file() {
        return store::fail(3, "checkpoint input is not a regular file");
    }
    Ok(fd)
}
fn hash_file(root: &Path, rel: &str, limit: u64) -> Result<(String, u64)> {
    let mut f = regular(root, rel)?;
    let before = f.metadata()?;
    let mut h = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        let n = f.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count = count
            .checked_add(n as u64)
            .ok_or_else(|| Error(3, "checkpoint size overflow".into()))?;
        if count > limit {
            return store::fail(3, "snapshot storage limit reached");
        }
        h.update(&buffer[..n]);
    }
    let after = f.metadata()?;
    if before.ino() != after.ino()
        || before.len() != after.len()
        || before.mtime() != after.mtime()
        || before.mtime_nsec() != after.mtime_nsec()
        || count != after.len()
    {
        return store::fail(3, "inspect-uncertain: file changed during checkpoint");
    }
    Ok((format!("{:x}", h.finalize()), count))
}
/// Full byte/mode/type/ownership inventory with explicit exclusions and bounds.
pub fn inventory(root: &Path, exclusions: &[String], limit: u64) -> Result<Value> {
    fn walk(
        root: &Path,
        dir: &Path,
        ex: &[String],
        limit: u64,
        total: &mut u64,
        out: &mut Map<String, Value>,
    ) -> Result<()> {
        let mut entries = fs::read_dir(dir)?
            .take(10001)
            .collect::<std::io::Result<Vec<_>>>()?;
        if entries.len() > 10000 || out.len() > 10000 {
            return store::fail(3, "snapshot file-count limit reached");
        }
        entries.sort_by_key(|x| x.file_name());
        for e in entries {
            let p = e.path();
            let rel = p
                .strip_prefix(root)
                .map_err(|_| Error(3, "checkpoint root differs".into()))?
                .to_str()
                .ok_or_else(|| Error(3, "non-UTF8 task paths unsupported".into()))?
                .to_string();
            if excluded(&rel, ex) {
                continue;
            }
            if Path::new(&rel).components().count() > 32 {
                return store::fail(
                    3,
                    "needs-scope: snapshot exceeds supported 32-level filesystem boundary",
                );
            }
            guard(root, &rel)?;
            let m = fs::symlink_metadata(&p)?;
            let mut row = json!({"mode":m.mode()&0o7777,"uid":m.uid(),"gid":m.gid()});
            if m.file_type().is_symlink() {
                let t = fs::read_link(&p)?;
                let t = t
                    .to_str()
                    .ok_or_else(|| Error(3, "non-UTF8 link unsupported".into()))?;
                row["kind"] = json!("link");
                row["target"] = json!(t);
                row["sha256"] = json!(digest(t));
            } else if m.is_dir() {
                row["kind"] = json!("directory");
            } else if m.is_file() {
                let (h, n) = hash_file(root, &rel, limit.saturating_sub(*total))?;
                *total = total
                    .checked_add(n)
                    .ok_or_else(|| Error(3, "snapshot overflow".into()))?;
                row["kind"] = json!("file");
                row["sha256"] = json!(h);
                row["bytes"] = json!(n);
            } else {
                return store::fail(
                    3,
                    "special task files are unsupported; preserve and inspect",
                );
            }
            out.insert(rel, row);
            if out.len() > 10000 {
                return store::fail(3, "snapshot file-count limit reached");
            }
            if m.is_dir() {
                walk(root, &p, ex, limit, total, out)?;
            }
        }
        Ok(())
    }
    let mut out = Map::new();
    walk(root, root, exclusions, limit, &mut 0, &mut out)?;
    Ok(Value::Object(out))
}
/// Digest selected declared inputs; dot conservatively selects the full inventory.
pub fn input_digest(all: &Value, inputs: &[String]) -> Result<String> {
    let o = all
        .as_object()
        .ok_or_else(|| Error(3, "checkpoint inventory malformed".into()))?;
    let mut selected = Map::new();
    for input in inputs {
        let mut count = 0;
        for (p, row) in o {
            if input == "." || Path::new(p).starts_with(input) {
                selected.insert(p.clone(), row.clone());
                count += 1;
            }
        }
        if count == 0 {
            return store::fail(3, format!("declared verification input missing: {input}"));
        }
    }
    Ok(digest(
        canonical(&Value::Object(selected)).map_err(|s| Error(3, s))?,
    ))
}
/// Copy a verified baseline to private blobs and a separate task-owned workspace.
pub fn prepare(
    source: &Path,
    work: &Path,
    blobs: &Path,
    ex: &[String],
    limit: u64,
    uid: u32,
    gid: u32,
) -> Result<Value> {
    prepare_checked(source, work, blobs, ex, limit, uid, gid, || Ok(()))
}
/// Cancellable setup. Partial checkpoints stay intact and are never reused as
/// complete baselines; callers synchronize owner stop with this operation.
#[allow(clippy::too_many_arguments)]
pub fn prepare_checked(
    source: &Path,
    work: &Path,
    blobs: &Path,
    ex: &[String],
    limit: u64,
    uid: u32,
    gid: u32,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Value> {
    check()?;
    let baseline = inventory(source, ex, limit)?;
    check()?;
    fs::create_dir(work)?;
    check()?;
    fs::create_dir(blobs)?;
    for (p, row) in baseline.as_object().unwrap() {
        check()?;
        let target = work.join(p);
        let kind = row["kind"].as_str().unwrap();
        if kind == "directory" {
            fs::create_dir(&target)?;
        } else if kind == "link" {
            std::os::unix::fs::symlink(row["target"].as_str().unwrap(), &target)?;
        } else {
            let h = row["sha256"].as_str().unwrap();
            let blob = blobs.join(h);
            if !blob.exists() {
                let mut input = regular(source, p)?;
                let mut output = OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&blob)?;
                setup_copy(
                    &mut input,
                    &mut output,
                    row["bytes"].as_u64().unwrap(),
                    &mut check,
                )?;
                output.sync_all()?;
            }
            if hash_file(blobs, h, limit)?.0 != h {
                return store::fail(3, "checkpoint blob differs");
            }
            check()?;
            let mut input = regular(blobs, h)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&target)?;
            setup_copy(
                &mut input,
                &mut output,
                row["bytes"].as_u64().unwrap(),
                &mut check,
            )?;
            output.sync_all()?;
        }
        check()?;
        if kind != "link" {
            fs::set_permissions(
                &target,
                fs::Permissions::from_mode(row["mode"].as_u64().unwrap() as u32),
            )?;
        }
        assign(&target, uid, gid, kind == "link")?;
    }
    check()?;
    assign(work, uid, gid, false)?;
    if inventory(source, ex, limit)? != baseline {
        return store::fail(
            3,
            "inspect-uncertain: original work changed during setup; preserve partial checkpoint",
        );
    }
    check()?;
    Ok(baseline)
}
fn setup_copy(
    input: &mut impl Read,
    output: &mut impl Write,
    expected: u64,
    check: &mut impl FnMut() -> Result<()>,
) -> Result<()> {
    let mut bytes = 0u64;
    let mut buffer = [0u8; 65536];
    loop {
        check()?;
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes = bytes
            .checked_add(n as u64)
            .ok_or_else(|| Error(3, "setup byte overflow".into()))?;
        if bytes > expected {
            return store::fail(
                3,
                "inspect-uncertain: source grew during setup; preserve partial checkpoint",
            );
        }
        output.write_all(&buffer[..n])?;
    }
    if bytes != expected {
        return store::fail(3, "inspect-uncertain: source length changed during setup");
    }
    Ok(())
}
/// Assign only a newly created task projection, never the original project.
pub fn assign(path: &Path, uid: u32, gid: u32, link: bool) -> Result<()> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let p = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| Error(2, "invalid checkpoint path".into()))?;
    if unsafe { libc::geteuid() } == 0 {
        let r = unsafe {
            if link {
                libc::lchown(p.as_ptr(), uid, gid)
            } else {
                libc::chown(p.as_ptr(), uid, gid)
            }
        };
        if r != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }
    Ok(())
}
/// Restore only a known, quiescent task projection. Conflicts preserve all bytes.
/// A quarantine keeps displaced files through partial failures; no recursive clean.
#[allow(clippy::too_many_arguments)] // Explicit baseline, output and ownership are distinct recovery boundaries.
pub fn restore(
    work: &Path,
    blobs: &Path,
    baseline: &Value,
    expected: &Value,
    ex: &[String],
    limit: u64,
    quarantine: &Path,
    uid: u32,
    gid: u32,
) -> Result<Value> {
    restore_checked(
        work,
        blobs,
        baseline,
        expected,
        ex,
        limit,
        quarantine,
        uid,
        gid,
        || Ok(()),
    )
}
/// Cancellable owner restoration. Callers must synchronize stop acknowledgments
/// with this operation; cancellation preserves displaced and partial files.
#[allow(clippy::too_many_arguments)]
pub fn restore_checked(
    work: &Path,
    blobs: &Path,
    baseline: &Value,
    expected: &Value,
    ex: &[String],
    limit: u64,
    quarantine: &Path,
    uid: u32,
    gid: u32,
    mut check: impl FnMut() -> Result<()>,
) -> Result<Value> {
    check()?;
    if inventory(work, ex, limit)? != *expected {
        return store::fail(
            3,
            "inspect-uncertain: concurrent work or ownership differs; restoration refused",
        );
    }
    let old = baseline
        .as_object()
        .ok_or_else(|| Error(3, "baseline malformed".into()))?;
    let current = expected
        .as_object()
        .ok_or_else(|| Error(3, "output inventory malformed".into()))?;
    for p in old.keys().chain(current.keys()) {
        guard(work, p)?;
    }
    check()?;
    fs::create_dir(quarantine)?;
    let mut moved = vec![];
    let mut paths = current.keys().cloned().collect::<Vec<_>>();
    paths.sort_by_key(|p| std::cmp::Reverse(p.matches('/').count()));
    for p in paths {
        check()?;
        let row = &current[&p];
        if row["kind"] == "directory" {
            continue;
        }
        let q = quarantine.join(format!("{}-{}", moved.len(), digest(&p)));
        fs::rename(work.join(&p), &q)?;
        let m = fs::symlink_metadata(&q)?;
        let same = m.mode() & 0o7777 == row["mode"].as_u64().unwrap() as u32
            && m.uid() == row["uid"].as_u64().unwrap() as u32
            && m.gid() == row["gid"].as_u64().unwrap() as u32
            && if m.file_type().is_symlink() {
                row["kind"] == "link" && fs::read_link(&q)?.to_str() == row["target"].as_str()
            } else {
                row["kind"] == "file"
                    && hash_file(quarantine, q.file_name().unwrap().to_str().unwrap(), limit)?.0
                        == row["sha256"].as_str().unwrap()
            };
        if !same {
            if m.file_type().is_symlink() {
                let _ = std::os::unix::fs::symlink(fs::read_link(&q)?, work.join(&p));
            } else {
                let _ = fs::hard_link(&q, work.join(&p));
            }
            return store::fail(
                3,
                format!(
                    "inspect-uncertain: concurrent change preserved in {}",
                    q.display()
                ),
            );
        }
        moved.push(json!({"path":p,"quarantine":q}));
    }
    let mut dirs = current
        .keys()
        .filter(|p| current[*p]["kind"] == "directory")
        .cloned()
        .collect::<Vec<_>>();
    dirs.sort_by_key(|p| std::cmp::Reverse(p.matches('/').count()));
    for p in dirs {
        check()?;
        fs::remove_dir(work.join(p))?;
    }
    for (p, row) in old {
        check()?;
        let target = work.join(p);
        let kind = row["kind"].as_str().unwrap();
        if kind == "directory" {
            fs::create_dir(&target)?;
        } else if kind == "link" {
            std::os::unix::fs::symlink(row["target"].as_str().unwrap(), &target)?;
        } else {
            let h = row["sha256"].as_str().unwrap();
            if hash_file(blobs, h, limit)?.0 != h {
                return store::fail(3, "baseline blob changed; preserve quarantine and inspect");
            }
            let mut input = regular(blobs, h)?;
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&target)?;
            let mut buffer = [0u8; 65536];
            loop {
                check()?;
                let count = input.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                f.write_all(&buffer[..count])?;
            }
            f.sync_all()?;
        }
        if kind != "link" {
            fs::set_permissions(
                &target,
                fs::Permissions::from_mode(row["mode"].as_u64().unwrap() as u32),
            )?;
        }
        assign(&target, uid, gid, kind == "link")?;
    }
    check()?;
    Ok(json!({"restored":true,"original_project_modified":false,"preserved_displaced":moved}))
}

/// Stream a pinned public artifact without allocating its full contents.
pub fn artifact_digest(path: &Path) -> Result<String> {
    let parent = path
        .parent()
        .ok_or_else(|| Error(2, "artifact parent missing".into()))?;
    Ok(hash_file(
        parent,
        path.file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error(2, "artifact name unsupported".into()))?,
        536_870_912,
    )?
    .0)
}
