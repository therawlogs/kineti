//! Bounded copied-workspace transfer. This codec creates no task authority.
use crate::{
    compat::digest,
    store::{self, Error, Result},
    task_snapshot as snapshot,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path},
};
/// A transfer is deliberately smaller than a model/API payload could make it.
pub const MAX_BYTES: u64 = 16_000_000;
fn mode(row: &Value) -> Result<u32> {
    row["mode"].as_u64().filter(|n|*n<=0o777).map(|n|n as u32).ok_or_else(||Error(3,"needs-scope: special or invalid file modes are unsupported by the VM checkpoint codec".into()))
}
fn link(path: &str, target: &str) -> Result<()> {
    if target.is_empty()
        || target.len() > 4096
        || target.contains('\0')
        || Path::new(target).is_absolute()
    {
        return store::fail(
            3,
            "needs-scope: VM transfer requires relative links within the copied workspace",
        );
    }
    let mut depth = Path::new(path).components().count() - 1;
    for c in Path::new(target).components() {
        match c {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir if depth > 0 => depth -= 1,
            _ => return store::fail(3, "needs-scope: link escapes the copied workspace"),
        }
    }
    Ok(())
}
/// Encode a bounded private source or sealed read-only guest projection.
pub fn encode(root: &Path, ex: &[String], limit: u64) -> Result<Value> {
    if limit == 0 || limit > MAX_BYTES {
        return store::fail(
            3,
            "needs-scope: VM copied-workspace limit exceeds the 16 MB transfer class",
        );
    }
    let before = snapshot::inventory(root, ex, limit)?;
    let mut blobs = serde_json::Map::new();
    for (path, row) in before.as_object().unwrap() {
        mode(row)?;
        match row["kind"].as_str() {
            Some("link") => link(
                path,
                row["target"]
                    .as_str()
                    .ok_or_else(|| Error(3, "invalid checkpoint link".into()))?,
            )?,
            Some("file") => {
                snapshot::guard(root, path)?;
                let mut file = OpenOptions::new()
                    .read(true)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                    .open(root.join(path))?;
                let m = file.metadata()?;
                if !m.is_file() || m.nlink() != 1 {
                    return store::fail(
                        3,
                        "needs-scope: hard-linked or special checkpoint file unsupported",
                    );
                }
                let mut bytes = Vec::new();
                Read::by_ref(&mut file)
                    .take(limit + 1)
                    .read_to_end(&mut bytes)?;
                let hash = row["sha256"]
                    .as_str()
                    .ok_or_else(|| Error(3, "checkpoint digest absent".into()))?;
                if bytes.len() as u64 != row["bytes"].as_u64().unwrap_or(u64::MAX)
                    || digest(&bytes) != hash
                {
                    return store::fail(3, "inspect-uncertain: checkpoint changed during transfer");
                }
                blobs.insert(hash.into(), json!(STANDARD.encode(bytes)));
            }
            Some("directory") => {}
            _ => return store::fail(3, "checkpoint kind unsupported"),
        }
    }
    if snapshot::inventory(root, ex, limit)? != before {
        return store::fail(3, "inspect-uncertain: checkpoint changed during transfer");
    }
    Ok(json!({"schema":"kineti-vm-checkpoint-v1","entries":before,"blobs":blobs}))
}
/// Decode only into a new private destination; never replace an existing tree.
/// Identity is supplied by the independent owner, not accepted from the wire.
pub fn decode(
    root: &Path,
    wire: &Value,
    ex: &[String],
    limit: u64,
    uid: u32,
    gid: u32,
) -> Result<Value> {
    crate::task_grant::keys(wire, &["schema", "entries", "blobs"])?;
    if wire["schema"] != "kineti-vm-checkpoint-v1" || limit == 0 || limit > MAX_BYTES {
        return store::fail(3, "unsupported VM checkpoint schema or resource class");
    }
    if unsafe { libc::geteuid() } != 0
        && (uid != unsafe { libc::geteuid() } || gid != unsafe { libc::getegid() })
    {
        return store::fail(3,"checkpoint identity requires the independent owner role; caller fields cannot change OS identity");
    }
    if fs::symlink_metadata(root).is_ok() {
        return store::fail(
            3,
            "inspect-uncertain: checkpoint destination already exists; preserve it",
        );
    }
    let parent = root
        .parent()
        .ok_or_else(|| Error(3, "checkpoint parent absent".into()))?;
    if parent.canonicalize()? != parent || !parent.is_dir() {
        return store::fail(3, "checkpoint parent is redirected");
    }
    let entries = wire["entries"]
        .as_object()
        .filter(|m| m.len() <= 10000)
        .ok_or_else(|| Error(3, "checkpoint entry bound exceeded".into()))?;
    let blobs = wire["blobs"]
        .as_object()
        .filter(|m| m.len() <= 10000)
        .ok_or_else(|| Error(3, "checkpoint blob bound exceeded".into()))?;
    let mut used = BTreeSet::new();
    let mut bytes = 0u64;
    // Validate the entire namespace, data and declared limits before creating files.
    for (path, row) in entries {
        crate::task_grant::relative(path, false)?;
        if Path::new(path).components().count() > 32 || snapshot::excluded(path, ex) {
            return store::fail(3, "checkpoint path is excluded or exceeds depth bound");
        }
        let fields: &[&str] = match row["kind"].as_str() {
            Some("file") => &["kind", "mode", "uid", "gid", "sha256", "bytes"],
            Some("link") => &["kind", "mode", "uid", "gid", "sha256", "target"],
            Some("directory") => &["kind", "mode", "uid", "gid"],
            _ => return store::fail(3, "unsupported checkpoint object"),
        };
        crate::task_grant::keys(row, fields)?;
        mode(row)?;
        if row["uid"].as_u64().is_none_or(|v| v > u32::MAX as u64)
            || row["gid"].as_u64().is_none_or(|v| v > u32::MAX as u64)
        {
            return store::fail(3, "invalid checkpoint metadata identity");
        }
        let mut p = Path::new(path).parent();
        while let Some(parent) = p {
            if parent.as_os_str().is_empty() {
                break;
            }
            if entries
                .get(parent.to_str().unwrap())
                .is_none_or(|v| v["kind"] != "directory")
            {
                return store::fail(3, "checkpoint parent is missing or redirected");
            }
            p = parent.parent();
        }
        match row["kind"].as_str() {
            Some("file") => {
                let h = row["sha256"]
                    .as_str()
                    .filter(|s| {
                        s.len() == 64
                            && s.bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    })
                    .ok_or_else(|| Error(3, "invalid checkpoint digest".into()))?;
                let n = row["bytes"]
                    .as_u64()
                    .ok_or_else(|| Error(3, "checkpoint size absent".into()))?;
                bytes = bytes
                    .checked_add(n)
                    .ok_or_else(|| Error(3, "checkpoint size overflow".into()))?;
                if bytes > limit {
                    return store::fail(3, "checkpoint resource limit exceeded");
                }
                let encoded = blobs
                    .get(h)
                    .and_then(Value::as_str)
                    .filter(|s| s.len() as u64 <= limit.div_ceil(3) * 4)
                    .ok_or_else(|| Error(3, "checkpoint blob absent or too large".into()))?;
                let data = STANDARD
                    .decode(encoded)
                    .map_err(|_| Error(3, "invalid checkpoint encoding".into()))?;
                if data.len() as u64 != n || digest(&data) != h {
                    return store::fail(3, "checkpoint blob or digest differs");
                }
                used.insert(h);
            }
            Some("link") => {
                let target = row["target"]
                    .as_str()
                    .ok_or_else(|| Error(3, "checkpoint link absent".into()))?;
                link(path, target)?;
                if row["sha256"] != digest(target) {
                    return store::fail(3, "checkpoint link metadata differs");
                }
                #[cfg(not(target_os = "macos"))]
                if mode(row)? != 0o777 {
                    return store::fail(
                        3,
                        "needs-scope: this platform cannot preserve the link mode",
                    );
                }
            }
            Some("directory") => {}
            _ => return store::fail(3, "unsupported checkpoint object"),
        }
    }
    if used.len() != blobs.len() {
        return store::fail(3, "checkpoint contains undeclared data");
    }
    fs::create_dir(root)?;
    fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
    let mut ordered = entries.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(p, _)| Path::new(p).components().count());
    for (path, row) in &ordered {
        snapshot::guard(root, path)?;
        let p = root.join(path);
        match row["kind"].as_str().unwrap() {
            "directory" => fs::create_dir(&p)?,
            "link" => {
                std::os::unix::fs::symlink(row["target"].as_str().unwrap(), &p)?;
                #[cfg(target_os = "macos")]
                {
                    use std::os::unix::ffi::OsStrExt;
                    let name = std::ffi::CString::new(p.as_os_str().as_bytes())
                        .map_err(|_| Error(3, "invalid link path".into()))?;
                    if unsafe { libc::lchmod(name.as_ptr(), mode(row)? as libc::mode_t) } != 0 {
                        return Err(std::io::Error::last_os_error().into());
                    }
                }
            }
            "file" => {
                let data = STANDARD
                    .decode(blobs[row["sha256"].as_str().unwrap()].as_str().unwrap())
                    .map_err(|_| Error(3, "checkpoint encoding changed".into()))?;
                let mut file = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                    .open(&p)?;
                file.write_all(&data)?;
                file.sync_all()?;
            }
            _ => unreachable!(),
        }
        snapshot::assign(&p, uid, gid, row["kind"] == "link")?;
        let metadata = fs::symlink_metadata(&p)?;
        if metadata.uid() != uid || metadata.gid() != gid {
            return store::fail(
                3,
                "inspect-uncertain: checkpoint ownership differs; partial copy preserved",
            );
        }
    }
    for (path, row) in ordered.into_iter().rev() {
        if row["kind"] != "link" {
            fs::set_permissions(root.join(path), fs::Permissions::from_mode(mode(row)?))?;
        }
    }
    snapshot::assign(root, uid, gid, false)?;
    snapshot::inventory(root, ex, limit)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_preserves_bytes_modes_and_relative_links() {
        let parent =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.kineti/checkpoint-codec-tests");
        fs::create_dir_all(&parent).unwrap();
        let parent = parent.canonicalize().unwrap();
        let source = parent.join(crate::store::nonce().unwrap());
        let target = parent.join(crate::store::nonce().unwrap());
        fs::create_dir(&source).unwrap();
        fs::create_dir(source.join("dir")).unwrap();
        fs::write(source.join("dir/data"), [0, 255, 10, 13]).unwrap();
        fs::set_permissions(source.join("dir/data"), fs::Permissions::from_mode(0o640)).unwrap();
        fs::write(source.join(".env.new"), b"excluded").unwrap();
        std::os::unix::fs::symlink("dir/data", source.join("alias")).unwrap();
        let wire = encode(&source, &[], 1024).unwrap();
        assert!(wire["entries"].get(".env.new").is_none());
        let mut bad = wire.clone();
        bad["entries"]["alias"]["sha256"] = json!("0".repeat(64));
        assert!(decode(
            &target,
            &bad,
            &[],
            1024,
            unsafe { libc::geteuid() },
            unsafe { libc::getegid() }
        )
        .is_err());
        assert!(!target.exists());
        let decoded = decode(
            &target,
            &wire,
            &[],
            1024,
            unsafe { libc::geteuid() },
            unsafe { libc::getegid() },
        )
        .unwrap();
        assert_eq!(decoded, wire["entries"]);
        assert_eq!(fs::read(target.join("alias")).unwrap(), [0, 255, 10, 13]);
        assert_eq!(
            fs::metadata(target.join("dir/data")).unwrap().mode() & 0o777,
            0o640
        );
        fs::remove_dir_all(source).unwrap();
        fs::remove_dir_all(target).unwrap();
    }
    #[test]
    fn invalid_wire_never_creates_destination() {
        let parent =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.kineti/checkpoint-codec-tests");
        fs::create_dir_all(&parent).unwrap();
        let parent = parent.canonicalize().unwrap();
        let root = parent.join(crate::store::nonce().unwrap());
        let h = digest(b"safe");
        let valid = json!({"schema":"kineti-vm-checkpoint-v1","entries":{"file":{"kind":"file","mode":420,"uid":0,"gid":0,"bytes":4,"sha256":h}},"blobs":{h.clone():STANDARD.encode(b"safe")}});
        for bad in [
            json!({"kind":"file","mode":420,"uid":0,"gid":0,"bytes":5,"sha256":h}),
            json!({"kind":"link","mode":511,"uid":0,"gid":0,"target":"../../escape","sha256":digest("../../escape")}),
            json!({"kind":"file","mode":0o4755,"uid":0,"gid":0,"bytes":4,"sha256":h}),
        ] {
            let mut wire = valid.clone();
            wire["entries"]["file"] = bad;
            assert!(decode(
                &root,
                &wire,
                &[],
                1024,
                unsafe { libc::geteuid() },
                unsafe { libc::getegid() }
            )
            .is_err());
            assert!(!root.exists());
        }
        for path in ["../escape", ".env.new", "link/child"] {
            let mut wire = valid.clone();
            let row = wire["entries"]["file"].take();
            wire["entries"] = json!({path:row});
            assert!(decode(
                &root,
                &wire,
                &[],
                1024,
                unsafe { libc::geteuid() },
                unsafe { libc::getegid() }
            )
            .is_err());
            assert!(!root.exists());
        }
        decode(
            &root,
            &valid,
            &[],
            1024,
            unsafe { libc::geteuid() },
            unsafe { libc::getegid() },
        )
        .unwrap();
        assert_eq!(fs::read(root.join("file")).unwrap(), b"safe");
        assert!(decode(
            &root,
            &valid,
            &[],
            1024,
            unsafe { libc::geteuid() },
            unsafe { libc::getegid() }
        )
        .is_err());
        fs::remove_file(root.join("file")).unwrap();
        fs::remove_dir(root).unwrap();
    }
}
