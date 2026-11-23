//! Project-local files, strict records, and interruption-visible locks.
use crate::compat::{canonical, digest};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
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
/// Read raw bytes of a regular file without following leaf or Kineti-owned symlinks.
pub fn read_bytes_nofollow(path: &Path) -> Result<Vec<u8>> {
    safe_destination(path)?;
    #[cfg(unix)]
    let mut f = {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
            .open(path)
            .map_err(|e| {
                if e.raw_os_error() == Some(libc::ELOOP) {
                    Error(3, "symbolic-link state/config destination refused".into())
                } else {
                    Error::from(e)
                }
            })?
    };
    #[cfg(not(unix))]
    let mut f = OpenOptions::new().read(true).open(path)?;
    if !f.metadata()?.is_file() {
        return fail(3, format!("non-regular file refused: {}", path.display()));
    }
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)?;
    Ok(bytes)
}
/// Read UTF-8 text of a regular file without following leaf or Kineti-owned symlinks.
pub fn read_text_nofollow(path: &Path) -> Result<String> {
    let bytes = read_bytes_nofollow(path)?;
    String::from_utf8(bytes).map_err(|e| Error(3, e.to_string()))
}
/// Read a JSON value; missing and corruption remain distinct errors.
pub fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&read_bytes_nofollow(path)?)?)
}
/// Strict JSONL reader; never skip malformed or incomplete lines.
pub fn lines(path: &Path) -> Result<Vec<Value>> {
    safe_destination(path)?;
    if fs::symlink_metadata(path).is_err() {
        return Ok(vec![]);
    }
    let bytes = read_bytes_nofollow(path)?;
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
/// Stream-hash a regular file in 64 KiB chunks without following a trailing symlink.
/// Refuses symlinks, directories, FIFOs, sockets, and device nodes with exit code 3.
pub fn hash_file_nofollow(path: &Path) -> Result<String> {
    #[cfg(unix)]
    let mut f = {
        use std::os::unix::fs::OpenOptionsExt;
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
            .open(path)?
    };
    #[cfg(not(unix))]
    let mut f = OpenOptions::new().read(true).open(path)?;
    if !f.metadata()?.is_file() {
        return fail(3, format!("non-regular file refused: {}", path.display()));
    }
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 65536];
    loop {
        match f.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    let out = hasher.finalize();
    Ok(out.iter().map(|b| format!("{b:02x}")).collect())
}
/// Refuse symbolic links at the destination leaf and within Kineti-owned subtrees,
/// while allowing canonical OS or user-root ancestor symlinks (such as `/tmp` or a
/// symlinked project root or Codex home).
pub fn safe_destination(path: &Path) -> Result<()> {
    fn reject_symlink(p: &Path) -> Result<()> {
        if let Ok(meta) = fs::symlink_metadata(p) {
            if meta.file_type().is_symlink() {
                return fail(3, "symbolic-link state/config destination refused");
            }
        }
        Ok(())
    }
    reject_symlink(path)?;
    let comps: Vec<Component<'_>> = path.components().collect();
    let len = comps.len();
    let mut boundary: Option<usize> = None;
    for i in (0..len.saturating_sub(1)).rev() {
        if let Component::Normal(name) = comps[i] {
            if name == ".kineti-backups" {
                boundary = Some(i);
                break;
            }
            if name == "kineti-gate"
                && i > 0
                && matches!(comps[i - 1], Component::Normal(prev) if prev == "skills")
            {
                boundary = Some(i);
                break;
            }
            if name == ".kineti" {
                let outer_harness = i + 1 < len
                    && matches!(
                        comps[i + 1],
                        Component::Normal(next)
                            if next == "native-gate-tests"
                                || next == "runner-tests"
                                || next == "self-test"
                    )
                    && (len - 1 - i) > 2;
                if !outer_harness {
                    boundary = Some(i);
                }
                break;
            }
        }
    }
    if let Some(start) = boundary {
        let mut prefix = PathBuf::new();
        for (idx, comp) in comps.iter().enumerate() {
            if idx >= start && matches!(comp, Component::ParentDir) {
                return fail(3, "parent escape in state/config destination refused");
            }
            prefix.push(comp.as_os_str());
            if idx >= start {
                reject_symlink(&prefix)?;
            }
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
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)?;
    }
    safe_destination(path)?;
    let tmp = parent.join(format!(".kineti-{}.tmp", nonce()?));
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&tmp)?;
    if let Err(e) = f.write_all(bytes).and_then(|()| f.sync_all()) {
        let _ = fs::remove_file(&tmp);
        return Err(e.into());
    }
    drop(f);
    if let Ok(meta) = fs::symlink_metadata(path) {
        if meta.file_type().is_symlink() {
            let _ = fs::remove_file(&tmp);
            return fail(3, "symbolic-link state/config destination refused");
        }
    }
    fs::rename(&tmp, path)?;
    let sync_dir = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    fs::File::open(sync_dir)?.sync_all()?;
    Ok(())
}
/// Durable append. Caller holds the appropriate shared operation lock.
pub fn append(path: &Path, value: &Value) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    safe_destination(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| Error(2, "missing parent".into()))?;
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)?;
    }
    let mut line = serde_json::to_string(value)?;
    line.push('\n');
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    f.write_all(line.as_bytes())?;
    f.sync_all()?;
    Ok(())
}
/// A crashed owner leaves the lock for explicit inspection, never timeout reclamation.
pub struct Lock(PathBuf);
impl Lock {
    /// Acquire an exclusive operation lock without touching protected payload files.
    pub fn take(path: &Path) -> Result<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        safe_destination(path)?;
        let parent = path
            .parent()
            .ok_or_else(|| Error(2, "missing parent".into()))?;
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match OpenOptions::new()
                .create_new(true)
                .write(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
                .open(path)
            {
                Ok(mut f) => {
                    f.write_all(
                        json!({"pid":std::process::id(),"at":now()})
                            .to_string()
                            .as_bytes(),
                    )?;
                    f.sync_all()?;
                    return Ok(Self(path.into()));
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
                        return fail(3, "symbolic-link state/config destination refused");
                    }
                    if Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(10));
                    } else {
                        return fail(
                            3,
                            "operation locked; inspect interrupted work before explicit recovery",
                        );
                    }
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
    safe_destination(&p)?;
    if fs::symlink_metadata(&p).is_err() {
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
const MAX_INVENTORY_DEPTH: usize = 64;

const TIER2_SKIP_DIRS: &[&str] = &[
    ".git",
    ".kineti",
    ".agents",
    "node_modules",
    ".next",
    "coverage",
    ".cache",
    ".DS_Store",
    ".pytest_cache",
    "__pycache__",
    ".mypy_cache",
    ".ruff_cache",
    ".venv",
    ".turbo",
    ".tsbuildinfo",
];

fn is_rust_build_target(root: &Path, depth: usize, name: &str, rel: &str) -> bool {
    if name != "target" {
        return false;
    }
    (depth == 0
        && (root.join("Cargo.toml").is_file() || root.join("core-native/Cargo.toml").is_file()))
        || (depth == 1
            && rel == "core-native/target"
            && root.join("core-native/Cargo.toml").is_file())
}

fn is_git_worktree_root(root: &Path) -> bool {
    let Ok(inside) = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };
    if !inside.status.success() || String::from_utf8_lossy(&inside.stdout).trim() != "true" {
        return false;
    }
    let Ok(top) = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };
    if !top.status.success() {
        return false;
    }
    let Ok(top_str) = std::str::from_utf8(&top.stdout) else {
        return false;
    };
    let top_path = Path::new(top_str.trim());
    matches!(
        (top_path.canonicalize(), root.canonicalize()),
        (Ok(a), Ok(b)) if a == b
    )
}

fn collect_gitignores(
    root: &Path,
    dir: &Path,
    depth: usize,
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    if depth >= MAX_INVENTORY_DEPTH {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for e in entries {
        let name = e.file_name();
        let name_str = name.to_string_lossy();
        let ft = e.file_type()?;
        if name_str == ".gitignore" {
            let p = e.path();
            let rel = p
                .strip_prefix(root)
                .map_err(|err| Error(3, err.to_string()))?
                .to_str()
                .ok_or_else(|| Error(2, "non-UTF8 input path".into()))?
                .to_string();
            if ft.is_symlink() {
                out.push((
                    rel,
                    digest(format!("symlink:{}", fs::read_link(&p)?.display())),
                ));
            } else if ft.is_file() {
                out.push((rel, hash_file_nofollow(&p)?));
            }
            continue;
        }
        if !ft.is_dir()
            || [
                ".git",
                ".kineti",
                ".agents",
                "node_modules",
                ".venv",
                ".pytest_cache",
                "__pycache__",
                ".mypy_cache",
                ".ruff_cache",
            ]
            .contains(&name_str.as_ref())
        {
            continue;
        }
        let p = e.path();
        if let Some(rel) = p.strip_prefix(root).ok().and_then(Path::to_str) {
            if is_rust_build_target(root, depth, &name_str, rel) {
                continue;
            }
        }
        collect_gitignores(root, &p, depth + 1, out)?;
    }
    Ok(())
}

fn git_exclude_file(root: &Path) -> PathBuf {
    let resolved = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-path", "info/exclude"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    if let Some(trimmed) = resolved {
        let p = Path::new(&trimmed);
        return if p.is_absolute() {
            p.to_path_buf()
        } else {
            root.join(p)
        };
    }
    root.join(".git/info/exclude")
}

fn inventory_git(root: &Path) -> Result<Vec<(String, String)>> {
    let out = Command::new("git")
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "core.excludesFile=/dev/null",
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;
    if !out.status.success() {
        return fail(
            3,
            format!(
                "git inventory failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        );
    }
    let mut entries = Vec::new();
    for raw in out.stdout.split(|b| *b == 0) {
        if raw.is_empty() {
            continue;
        }
        let rel = std::str::from_utf8(raw).map_err(|_| Error(2, "non-UTF8 input path".into()))?;
        if rel.split('/').any(|seg| {
            matches!(
                seg,
                ".kineti"
                    | ".agents"
                    | ".git"
                    | ".pytest_cache"
                    | "__pycache__"
                    | ".mypy_cache"
                    | ".ruff_cache"
            )
        }) {
            continue;
        }
        let full = root.join(rel);
        let meta = match fs::symlink_metadata(&full) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        if meta.file_type().is_symlink() {
            entries.push((
                rel.to_string(),
                digest(format!("symlink:{}", fs::read_link(&full)?.display())),
            ));
        } else if meta.file_type().is_file() {
            entries.push((rel.to_string(), hash_file_nofollow(&full)?));
        } else if meta.file_type().is_dir() {
            // Submodule/gitlink directory entry.
            continue;
        } else {
            return fail(3, format!("non-regular file in inventory refused: {rel}"));
        }
    }
    collect_gitignores(root, root, 0, &mut entries)?;
    let exclude_path = git_exclude_file(root);
    match fs::symlink_metadata(&exclude_path) {
        Ok(meta) if meta.file_type().is_file() => {
            entries.push((
                ".git/info/exclude".to_string(),
                hash_file_nofollow(&exclude_path)?,
            ));
        }
        Ok(meta) if meta.file_type().is_symlink() => {
            entries.push((
                ".git/info/exclude".to_string(),
                digest(format!(
                    "symlink:{}",
                    fs::read_link(&exclude_path)?.display()
                )),
            ));
        }
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e.into()),
    }
    Ok(entries)
}

fn configured_excludes(root: &Path) -> Vec<String> {
    let path = root.join(".kineti/gate.json");
    let Ok(bytes) = fs::read(path) else {
        return vec![];
    };
    let Ok(val) = serde_json::from_slice::<Value>(&bytes) else {
        return vec![];
    };
    val.get("exclude")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(|s| s.trim_start_matches("./").trim_matches('/').to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn excluded_by_config(name: &str, rel: &str, excludes: &[String]) -> bool {
    excludes.iter().any(|excl| {
        if excl.contains('/') {
            rel == excl || rel.starts_with(&format!("{excl}/"))
        } else {
            name == excl || rel == excl || rel.starts_with(&format!("{excl}/"))
        }
    })
}

fn inventory_walk(
    root: &Path,
    dir: &Path,
    depth: usize,
    excludes: &[String],
    out: &mut Vec<(String, String)>,
) -> Result<()> {
    if depth >= MAX_INVENTORY_DEPTH {
        return fail(3, "inventory directory depth limit exceeded");
    }
    let mut entries = fs::read_dir(dir)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for e in entries {
        let name = e.file_name();
        let name_str = name.to_string_lossy();
        if TIER2_SKIP_DIRS.contains(&name_str.as_ref()) {
            continue;
        }
        let p = e.path();
        let rel = p
            .strip_prefix(root)
            .map_err(|err| Error(3, err.to_string()))?
            .to_str()
            .ok_or_else(|| Error(2, "non-UTF8 input path".into()))?;
        if is_rust_build_target(root, depth, &name_str, rel)
            || excluded_by_config(&name_str, rel, excludes)
        {
            continue;
        }
        let ft = e.file_type()?;
        if ft.is_symlink() {
            out.push((
                rel.to_string(),
                digest(format!("symlink:{}", fs::read_link(&p)?.display())),
            ));
        } else if ft.is_dir() {
            inventory_walk(root, &p, depth + 1, excludes, out)?;
        } else if ft.is_file() {
            out.push((rel.to_string(), hash_file_nofollow(&p)?));
        } else {
            return fail(3, format!("non-regular file in inventory refused: {rel}"));
        }
    }
    Ok(())
}

/// Native two-tier streaming inventory: Git-aware when `root` is a Git worktree root with
/// `git` available on `PATH`, falling back to a bounded pure-Rust `O_NOFOLLOW` walker.
pub fn inventory(root: &Path) -> Result<Value> {
    let mut entries = if is_git_worktree_root(root) {
        inventory_git(root)?
    } else {
        let excludes = configured_excludes(root);
        let mut out = Vec::new();
        inventory_walk(root, root, 0, &excludes, &mut out)?;
        out
    };
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.dedup_by(|a, b| a.0 == b.0);
    Ok(Value::Array(
        entries
            .into_iter()
            .map(|(path, sha256)| json!({"path": path, "sha256": sha256}))
            .collect(),
    ))
}
/// Content digest of an inventory.
pub fn inventory_digest(v: &Value) -> Result<String> {
    Ok(digest(canonical(v).map_err(|e| Error(3, e))?))
}

#[cfg(test)]
mod permission_tests {
    use super::*;
    use std::{
        os::unix::{fs::PermissionsExt, process::CommandExt},
        process::{Child, Command, Stdio},
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.kineti/native-gate-tests")
                .join(nonce().unwrap());
            fs::create_dir_all(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    struct ChildGuard(Child);
    impl Drop for ChildGuard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    fn mode(path: &Path) -> u32 {
        fs::metadata(path).unwrap().permissions().mode() & 0o777
    }

    #[test]
    fn owner_only_creation_under_child_umask() {
        const CHILD_ROOT: &str = "KINETI_PRIVATE_PERMISSION_FIXTURE";
        if let Some(root) = std::env::var_os(CHILD_ROOT) {
            let root = PathBuf::from(root);
            let state = root.join(".kineti/state.json");
            write(&state, &json!({"root_goal":"Disposable permission test"})).unwrap();
            let append_path = root.join(".kineti/new-append.jsonl");
            append(&append_path, &json!({"private":true})).unwrap();
            let _held = Lock::take(&root.join(".kineti/held.lock")).unwrap();
            atomic(&root.join(".kineti/atomic.bin"), b"atomic fixture").unwrap();
            fs::write(root.join("test.txt"), b"declared test input").unwrap();
            crate::receipt::run(
                &root,
                crate::receipt::Run {
                    label: "permission-fixture".into(),
                    argv: vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        "printf '1 pass\\n0 fail\\n'".into(),
                    ],
                    tests: vec!["test.txt".into()],
                    timeout: 5,
                    kind: "test".into(),
                    ..Default::default()
                },
            )
            .unwrap();
            // Publish readiness only after all writes; keep the real lock alive
            // until the observing parent releases this subprocess over stdin.
            atomic(&root.join(".kineti/ready"), b"ready").unwrap();
            let mut release = [0];
            std::io::stdin().read_exact(&mut release).unwrap();
            assert_eq!(release, [1]);
            return;
        }

        let fixture = Fixture::new();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "store::permission_tests::owner_only_creation_under_child_umask",
                "--nocapture",
            ])
            .env(CHILD_ROOT, &fixture.0)
            .env("HOME", fixture.0.join("home"))
            .env("CODEX_HOME", fixture.0.join("codex-home"))
            .stdin(Stdio::piped());
        // umask is process-global: change it only after fork in this child,
        // never in the concurrent Cargo test harness.
        unsafe {
            command.pre_exec(|| {
                libc::umask(0);
                Ok(())
            });
        }
        let mut child = ChildGuard(command.spawn().unwrap());
        let watchdog = Instant::now() + Duration::from_secs(20);
        while !fixture.0.join(".kineti/ready").exists() {
            assert!(
                Instant::now() < watchdog,
                "permission fixture readiness timed out"
            );
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "permission fixture exited before readiness"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        for relative in [
            ".kineti/state.json",
            ".kineti/new-append.jsonl",
            ".kineti/held.lock",
            ".kineti/atomic.bin",
            ".kineti/evidence-issuer.pem.private",
            ".kineti/evidence-issuer.pem",
            ".kineti/evidence.jsonl",
        ] {
            assert_eq!(mode(&fixture.0.join(relative)), 0o600, "{relative}");
        }
        assert_eq!(
            read(&fixture.0.join(".kineti/held.lock")).unwrap()["pid"],
            child.0.id()
        );
        assert_eq!(
            lines(&fixture.0.join(".kineti/new-append.jsonl")).unwrap(),
            vec![json!({"private":true})]
        );
        child.0.stdin.take().unwrap().write_all(&[1]).unwrap();
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            assert!(
                Instant::now() < watchdog,
                "permission fixture release timed out"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!(!fixture.0.join(".kineti/held.lock").exists());
    }

    #[test]
    fn historical_modes_bytes_and_interrupted_lock_are_preserved() {
        let fixture = Fixture::new();
        let history = fixture.0.join("history.jsonl");
        let before = b"{\"historical\":\"original bytes\"}\n";
        fs::write(&history, before).unwrap();
        fs::set_permissions(&history, fs::Permissions::from_mode(0o644)).unwrap();
        append(&history, &json!({"new":"normal requested append"})).unwrap();
        let after = fs::read(&history).unwrap();
        assert!(after.starts_with(before));
        assert_eq!(lines(&history).unwrap().len(), 2);
        assert_eq!(mode(&history), 0o644);

        let interrupted = fixture.0.join("interrupted.lock");
        let lock_bytes = b"{\"pid\":424242,\"at\":\"historical interrupted fixture\"}";
        fs::write(&interrupted, lock_bytes).unwrap();
        fs::set_permissions(&interrupted, fs::Permissions::from_mode(0o644)).unwrap();
        let error = Lock::take(&interrupted)
            .err()
            .expect("existing lock must deny");
        assert_eq!(error.0, 3);
        assert!(error.1.contains("inspect interrupted work"));
        assert_eq!(fs::read(&interrupted).unwrap(), lock_bytes);
        assert_eq!(mode(&interrupted), 0o644);
        assert_eq!(fs::read(&history).unwrap(), after);
        assert_eq!(mode(&history), 0o644);

        // New-file creation modes cannot weaken existing symlink denial or
        // replace the target's historical data and permissions.
        let link = fixture.0.join("linked-state");
        std::os::unix::fs::symlink(&history, &link).unwrap();
        assert_eq!(append(&link, &json!({"denied":true})).unwrap_err().0, 3);
        assert_eq!(Lock::take(&link).err().unwrap().0, 3);
        assert_eq!(atomic(&link, b"denied").unwrap_err().0, 3);
        assert!(fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(fs::read(&history).unwrap(), after);
        assert_eq!(mode(&history), 0o644);
    }
}

#[cfg(test)]
mod freshness_tests {
    use super::*;
    use std::os::unix::{ffi::OsStrExt, fs::symlink};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.kineti/native-gate-tests")
                .join(nonce().unwrap());
            fs::create_dir_all(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn hash_file_nofollow_streams_regular_and_refuses_specials() {
        let f = Fixture::new();
        let big = f.0.join("big.bin");
        let payload: Vec<u8> = (0..131_089usize).map(|i| (i % 251) as u8).collect();
        fs::write(&big, &payload).unwrap();
        assert_eq!(hash_file_nofollow(&big).unwrap(), digest(&payload));

        let link = f.0.join("link.bin");
        symlink(&big, &link).unwrap();
        assert_eq!(hash_file_nofollow(&link).unwrap_err().0, 3);

        let dir = f.0.join("subdir");
        fs::create_dir(&dir).unwrap();
        assert_eq!(hash_file_nofollow(&dir).unwrap_err().0, 3);

        let fifo = f.0.join("pipe");
        let cstr = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(cstr.as_ptr(), 0o600) }, 0);
        assert_eq!(hash_file_nofollow(&fifo).unwrap_err().0, 3);
    }

    #[test]
    fn safe_destination_allows_ancestor_symlinks_and_refuses_owned_links() {
        let f = Fixture::new();
        let real_root = f.0.join("real-root");
        let link_root = f.0.join("link-root");
        fs::create_dir_all(&real_root).unwrap();
        symlink(&real_root, &link_root).unwrap();
        write(
            &link_root.join(".kineti/state.json"),
            &json!({"root_goal": "Ancestor symlink allowed"}),
        )
        .unwrap();
        assert!(real_root.join(".kineti/state.json").is_file());

        let bad_proj = f.0.join("bad-proj");
        let external = f.0.join("external");
        fs::create_dir_all(&bad_proj).unwrap();
        fs::create_dir_all(&external).unwrap();
        symlink(&external, bad_proj.join(".kineti")).unwrap();
        assert_eq!(
            write(
                &bad_proj.join(".kineti/state.json"),
                &json!({"root_goal": "denied"})
            )
            .unwrap_err()
            .0,
            3
        );

        let real_codex = f.0.join("real-codex");
        let link_codex = f.0.join("link-codex");
        fs::create_dir_all(&real_codex).unwrap();
        symlink(&real_codex, &link_codex).unwrap();
        atomic(&link_codex.join("config.toml"), b"model = \"x\"\n").unwrap();
        atomic(&link_codex.join(".kineti-backups/b1"), b"backup").unwrap();
        atomic(&link_codex.join("skills/kineti-gate/SKILL.md"), b"skill").unwrap();

        let bad_codex = f.0.join("bad-codex");
        fs::create_dir_all(bad_codex.join("skills")).unwrap();
        symlink(&external, bad_codex.join(".kineti-backups")).unwrap();
        symlink(&external, bad_codex.join("skills/kineti-gate")).unwrap();
        assert_eq!(
            atomic(&bad_codex.join(".kineti-backups/b1"), b"denied")
                .unwrap_err()
                .0,
            3
        );
        assert_eq!(
            atomic(&bad_codex.join("skills/kineti-gate/SKILL.md"), b"denied")
                .unwrap_err()
                .0,
            3
        );
    }

    #[test]
    fn tier2_inventory_covers_legacy_build_target_and_skips_caches() {
        let f = Fixture::new();
        for rel in [
            "src/legacy/auth.py",
            "src/build/gen.py",
            "src/target/backdoor.rs",
            "dist/out.js",
            "tmp/scratch.txt",
            ".pytest_cache/v/cache/nodeids",
            "__pycache__/mod.cpython-312.pyc",
            "custom_skip/ignored.txt",
        ] {
            let p = f.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, rel).unwrap();
        }
        write(
            &f.0.join(".kineti/gate.json"),
            &json!({"exclude": ["custom_skip"]}),
        )
        .unwrap();
        let inv = inventory(&f.0).unwrap();
        let paths: Vec<&str> = inv
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v["path"].as_str())
            .collect();
        for expected in [
            "src/legacy/auth.py",
            "src/build/gen.py",
            "src/target/backdoor.rs",
            "dist/out.js",
            "tmp/scratch.txt",
        ] {
            assert!(paths.contains(&expected), "missing {expected} in {paths:?}");
        }
        for skipped in [
            ".pytest_cache/v/cache/nodeids",
            "__pycache__/mod.cpython-312.pyc",
            "custom_skip/ignored.txt",
        ] {
            assert!(
                !paths.contains(&skipped),
                "unexpected {skipped} in {paths:?}"
            );
        }

        // Root Cargo `target/` is skipped only when Cargo.toml exists; nested `src/target/` stays tracked.
        fs::write(f.0.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        fs::create_dir_all(f.0.join("target/debug")).unwrap();
        fs::write(f.0.join("target/debug/bin"), "artifact").unwrap();
        let inv2 = inventory(&f.0).unwrap();
        let paths2: Vec<&str> = inv2
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v["path"].as_str())
            .collect();
        assert!(!paths2.contains(&"target/debug/bin"));
        assert!(paths2.contains(&"src/target/backdoor.rs"));
    }

    #[test]
    fn tier1_git_inventory_hashes_nested_gitignore_and_info_exclude() {
        let f = Fixture::new();
        let init = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .arg("-C")
            .arg(&f.0)
            .arg("init")
            .output();
        let Ok(init) = init else {
            return;
        };
        assert!(init.status.success());

        fs::create_dir_all(f.0.join("src")).unwrap();
        fs::write(f.0.join("src/app.rs"), "fn main() {}\n").unwrap();
        fs::write(f.0.join("tracked.txt"), "will be deleted\n").unwrap();
        fs::write(f.0.join("src/.gitignore"), "*.tmp\n").unwrap();
        fs::write(f.0.join(".git/info/exclude"), "# initial\n").unwrap();

        let add = Command::new("git")
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .arg("-C")
            .arg(&f.0)
            .args(["add", "tracked.txt"])
            .output()
            .unwrap();
        assert!(add.status.success());
        fs::remove_file(f.0.join("tracked.txt")).unwrap();

        let before = inventory(&f.0).unwrap();
        let paths: Vec<&str> = before
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v["path"].as_str())
            .collect();
        assert!(paths.contains(&"src/app.rs"));
        assert!(paths.contains(&"src/.gitignore"));
        assert!(paths.contains(&".git/info/exclude"));
        assert!(!paths.contains(&"tracked.txt"));

        fs::write(f.0.join("src/.gitignore"), "*.tmp\napp.rs\n").unwrap();
        let after_gitignore = inventory(&f.0).unwrap();
        assert_ne!(
            inventory_digest(&before).unwrap(),
            inventory_digest(&after_gitignore).unwrap()
        );

        fs::write(f.0.join(".git/info/exclude"), "# tampered\n").unwrap();
        let after_exclude = inventory(&f.0).unwrap();
        assert_ne!(
            inventory_digest(&after_gitignore).unwrap(),
            inventory_digest(&after_exclude).unwrap()
        );
    }
}
