//! Project-local files, strict records, and interruption-visible locks.
use crate::compat::{canonical, digest};
use serde_json::{json, Value};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    time::{Duration, Instant},
};

/// Gate failure with stable process exit meaning.
#[derive(Debug)]
pub struct Error(pub i32, pub String);
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.1)
    }
}
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self(3, e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self(3, e.to_string())
    }
}
/// Gate result.
pub type Result<T> = std::result::Result<T, Error>;
/// Explicit failure.
pub fn fail<T>(code: i32, text: impl Into<String>) -> Result<T> {
    Err(Error(code, text.into()))
}
/// UTC timestamp independent of the local timezone.
pub fn now() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}
/// Random identifier from the supported operating systems.
pub fn nonce() -> Result<String> {
    let mut b = [0u8; 16];
    fs::File::open("/dev/urandom")?.read_exact(&mut b)?;
    Ok(b.iter().map(|v| format!("{v:02x}")).collect())
}
/// Read a JSON value; missing and corruption remain distinct errors.
pub fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
/// Strict JSONL reader; never skip malformed or incomplete lines.
pub fn lines(path: &Path) -> Result<Vec<Value>> {
    if !path.exists() {
        return Ok(vec![]);
    }
    let bytes = fs::read(path)?;
    if !bytes.is_empty() && !bytes.ends_with(b"\n") {
        return fail(
            3,
            format!(
                "incomplete history: {}; inspect before recovery",
                path.display()
            ),
        );
    }
    let text = std::str::from_utf8(&bytes).map_err(|e| Error(3, e.to_string()))?;
    text.lines()
        .map(|line| serde_json::from_str(line).map_err(Error::from))
        .collect()
}
/// Atomic replacement, synced before rename, with private permissions.
pub fn write(path: &Path, value: &Value) -> Result<()> {
    atomic(
        path,
        &(serde_json::to_string_pretty(value)? + "\n").into_bytes(),
    )
}
/// Refuse symbolic-link destinations and parent escapes.
pub fn safe_destination(path: &Path) -> Result<()> {
    let mut p = path.to_path_buf();
    loop {
        if let Ok(meta) = fs::symlink_metadata(&p) {
            if meta.file_type().is_symlink() {
                return fail(3, "symbolic-link state/config destination refused");
            }
        }
        if !p.pop() {
            break;
        }
    }
    Ok(())
}
/// Atomic bytes for config/backups as well as JSON.
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    safe_destination(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| Error(2, "missing parent".into()))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(".kineti-{}.tmp", nonce()?));
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
    f.write_all(bytes)?;
    f.sync_all()?;
    fs::rename(&tmp, path)?;
    fs::File::open(parent)?.sync_all()?;
    Ok(())
}
/// Durable append. Caller holds the appropriate shared operation lock.
pub fn append(path: &Path, value: &Value) -> Result<()> {
    safe_destination(path)?;
    fs::create_dir_all(path.parent().unwrap())?;
    let mut f = OpenOptions::new().create(true).append(true).open(path)?;
    f.write_all(serde_json::to_string(value)?.as_bytes())?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    Ok(())
}
/// A crashed owner leaves the lock for explicit inspection, never timeout reclamation.
pub struct Lock(PathBuf);
impl Lock {
    /// Acquire an exclusive operation lock without touching protected payload files.
    pub fn take(path: &Path) -> Result<Self> {
        safe_destination(path)?;
        fs::create_dir_all(path.parent().unwrap())?;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match OpenOptions::new().create_new(true).write(true).open(path) {
                Ok(mut f) => {
                    f.write_all(
                        json!({"pid":std::process::id(),"at":now()})
                            .to_string()
                            .as_bytes(),
                    )?;
                    f.sync_all()?;
                    return Ok(Self(path.into()));
                }
                Err(e)
                    if e.kind() == std::io::ErrorKind::AlreadyExists
                        && Instant::now() < deadline =>
                {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    return fail(
                        3,
                        "operation locked; inspect interrupted work before explicit recovery",
                    )
                }
                Err(e) => return Err(e.into()),
            }
        }
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
/// Require initialized state; never invent a locked goal.
pub fn state(root: &Path) -> Result<Value> {
    let p = root.join(".kineti/state.json");
    if !p.exists() {
        return fail(3,"project state missing: preview kineti init --goal <explicit-goal>, then apply it; damaged records require recovery");
    }
    let v = read(&p)?;
    if !v.is_object() || !(v["root_goal"].is_string() || v["root_goal"]["description"].is_string())
    {
        return fail(
            3,
            "damaged project state: inspect and use reviewed recovery; init cannot repair records",
        );
    }
    Ok(v)
}
/// A declared project input must resolve inside the project.
pub fn input(root: &Path, rel: &str) -> Result<PathBuf> {
    let p = Path::new(rel);
    if rel.is_empty()
        || p.is_absolute()
        || p.components().any(|c| matches!(c, Component::ParentDir))
    {
        return fail(2, "input must be a relative project path");
    }
    let full = root.join(p).canonicalize()?;
    if !full.starts_with(root.canonicalize()?) {
        return fail(2, "input resolves outside project");
    }
    Ok(full)
}
/// Native raw-byte inventory, fully hashing files; ignored directories are explicit.
pub fn inventory(root: &Path) -> Result<Value> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<Value>) -> Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name();
            let name = name.to_string_lossy();
            if [
                ".git",
                ".kineti",
                ".agents",
                "node_modules",
                "dist",
                "build",
                ".next",
                "coverage",
                "tmp",
                ".cache",
                "legacy",
                "target",
                ".DS_Store",
            ]
            .contains(&name.as_ref())
            {
                continue;
            }
            let p = e.path();
            let rel = p
                .strip_prefix(root)
                .unwrap()
                .to_str()
                .ok_or_else(|| Error(2, "non-UTF8 input path".into()))?;
            if e.file_type()?.is_symlink() {
                out.push(json!({"path":rel,"sha256":digest(format!("symlink:{}",fs::read_link(&p)?.display()))}));
            } else if e.file_type()?.is_dir() {
                walk(root, &p, out)?;
            } else if e.file_type()?.is_file() {
                out.push(json!({"path":rel,"sha256":digest(fs::read(p)?)}));
            }
        }
        Ok(())
    }
    let mut out = vec![];
    walk(root, root, &mut out)?;
    Ok(json!(out))
}
/// Content digest of an inventory.
pub fn inventory_digest(v: &Value) -> Result<String> {
    Ok(digest(canonical(v).map_err(|e| Error(3, e))?))
}
