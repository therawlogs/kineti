//! Versioned native receipts. Output parsing never proves honest test execution.
use crate::{
    compat::{canonical, digest, public_key, verify_legacy},
    store::{self, Error, Lock, Result},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey};
use serde_json::{json, Value};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

/// Declared run inputs, parsed before any writes.
#[derive(Default)]
pub struct Run {
    /// Receipt label.
    pub label: String,
    /// Exact argument vector, never reconstructed from a joined string.
    pub argv: Vec<String>,
    /// Declared test files/directories.
    pub tests: Vec<String>,
    /// Environment names whose presence and values are hashed.
    pub env: Vec<String>,
    /// Additional declared files.
    pub artifacts: Vec<String>,
    /// Timeout in seconds.
    pub timeout: u64,
    /// test/check/package.
    pub kind: String,
    /// Test output parser mode: auto, cargo, bun, or exit-code.
    pub parser: String,
    /// Allow ignored/skipped tests for cargo test.
    pub allow_ignored: bool,
    /// Optional minimum passed test count floor.
    pub min_passed: Option<u64>,
}

fn pem(label: &str, der: &[u8]) -> String {
    format!(
        "-----BEGIN {label}-----\n{}\n-----END {label}-----\n",
        STANDARD.encode(der)
    )
}
fn keys(root: &Path) -> Result<(SigningKey, String)> {
    let pubpath = root.join(".kineti/evidence-issuer.pem");
    let private = root.join(".kineti/evidence-issuer.pem.private");
    let _lock = Lock::take(&root.join(".kineti/evidence-issuer.pem.init.lock"))?;
    if !pubpath.exists() && !private.exists() {
        let mut seed = [0; 32];
        fs::File::open("/dev/urandom")?.read_exact(&mut seed)?;
        let key = SigningKey::from_bytes(&seed);
        let mut der = vec![
            0x30, 0x2e, 2, 1, 0, 0x30, 5, 6, 3, 0x2b, 0x65, 0x70, 4, 0x22, 4, 0x20,
        ];
        der.extend(seed);
        store::atomic(&private, pem("PRIVATE KEY", &der).as_bytes())?;
        let mut der = vec![0x30, 0x2a, 0x30, 5, 6, 3, 0x2b, 0x65, 0x70, 3, 0x21, 0];
        der.extend(key.verifying_key().to_bytes());
        store::atomic(&pubpath, pem("PUBLIC KEY", &der).as_bytes())?;
    }
    if fs::symlink_metadata(&pubpath).is_err() || fs::symlink_metadata(&private).is_err() {
        return store::fail(
            3,
            "incomplete issuer identity; preserve originals and use reviewed recovery",
        );
    }
    let raw = store::read_text_nofollow(&private)?;
    let body = raw
        .trim()
        .strip_prefix("-----BEGIN PRIVATE KEY-----")
        .and_then(|s| s.strip_suffix("-----END PRIVATE KEY-----"))
        .ok_or_else(|| Error(3, "invalid private key".into()))?;
    let der = STANDARD
        .decode(body.split_whitespace().collect::<String>())
        .map_err(|e| Error(3, e.to_string()))?;
    if der.len() != 48
        || der[..16]
            != [
                0x30, 0x2e, 2, 1, 0, 0x30, 5, 6, 3, 0x2b, 0x65, 0x70, 4, 0x22, 4, 0x20,
            ]
    {
        return store::fail(3, "unsupported issuer private-key format");
    }
    let key = SigningKey::from_bytes(der[16..].try_into().unwrap());
    let public = store::read_text_nofollow(&pubpath)?;
    if public_key(&public).map_err(|e| Error(3, e))? != key.verifying_key() {
        return store::fail(3, "issuer keys do not match");
    }
    Ok((key, public))
}
fn unsigned(v: &Value) -> Result<String> {
    let mut v = v.clone();
    v.as_object_mut()
        .ok_or_else(|| Error(7, "receipt object required".into()))?
        .shift_remove("signature");
    canonical(&v).map_err(|e| Error(7, e))
}
/// Sign a canonical-v2 JSON object with the project's Ed25519 issuer key.
pub(crate) fn sign_object(root: &Path, value: &mut Value) -> Result<String> {
    if !value.is_object() {
        return store::fail(7, "receipt object required");
    }
    let (key, public) = keys(root)?;
    value["encoding"] = json!("kineti-canonical-v2");
    value["issuer_key"] = json!(public);
    let payload = unsigned(value)?;
    let sig = STANDARD.encode(key.sign(payload.as_bytes()).to_bytes());
    value["signature"] = json!(sig);
    Ok(sig)
}
/// Verify a canonical-v2 Ed25519 signature on a JSON object against a trusted public key.
pub fn verify_object_signature(value: &Value, trusted: &str) -> bool {
    let check = || -> Option<()> {
        if value.get("encoding")?.as_str()? != "kineti-canonical-v2"
            || value.get("issuer_key")?.as_str()? != trusted
        {
            return None;
        }
        let text = value.get("signature")?.as_str()?;
        let bytes = STANDARD.decode(text).ok()?;
        if bytes.len() != 64 || STANDARD.encode(&bytes) != text {
            return None;
        }
        public_key(trusted)
            .ok()?
            .verify_strict(
                unsigned(value).ok()?.as_bytes(),
                &Signature::from_slice(&bytes).ok()?,
            )
            .ok()
    };
    check().is_some()
}
/// Verify signature without treating parsed output as independent observation.
pub fn signature(v: &Value, trusted: &str) -> bool {
    if v.get("encoding").is_none() {
        return verify_legacy(v, trusted);
    }
    if v["receipt_version"] != 3 || v["encoding"] != "kineti-canonical-v2" {
        return false;
    }
    verify_object_signature(v, trusted)
}
fn resolve(root: &Path, command: &str) -> Result<PathBuf> {
    if command.contains('/') {
        let candidate = if Path::new(command).is_absolute() {
            PathBuf::from(command)
        } else {
            root.join(command)
        };
        return Ok(candidate.canonicalize()?);
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let p = dir.join(command);
        if p.is_file() {
            return Ok(p.canonicalize()?);
        }
    }
    store::fail(2, format!("executable missing: {command}"))
}
fn runtime(root: &Path, argv: &[String]) -> Result<Value> {
    let p = resolve(root, &argv[0])?;
    let exe = std::env::current_exe()?.canonicalize()?;
    Ok(json!({
        "kineti_version": env!("CARGO_PKG_VERSION"),
        "gate_sha256": store::hash_file_nofollow(&exe)?,
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "executable": p,
        "executable_sha256": store::hash_file_nofollow(&p)?,
    }))
}
fn env_digest(name: &str) -> Result<String> {
    let value = std::env::var(name).ok();
    Ok(digest(
        canonical(&json!({"set":value.is_some(),"value":value})).map_err(|e| Error(2, e))?,
    ))
}
fn files(root: &Path, declared: &[String]) -> Result<Value> {
    let mut out = vec![];
    fn walk(root: &Path, p: &Path, out: &mut Vec<Value>) -> Result<()> {
        if fs::symlink_metadata(p)?.file_type().is_symlink() {
            return store::fail(2, "declare a resolved project file instead of a symlink");
        }
        if p.is_dir() {
            let mut es = fs::read_dir(p)?.collect::<std::io::Result<Vec<_>>>()?;
            es.sort_by_key(fs::DirEntry::file_name);
            for e in es {
                walk(root, &e.path(), out)?;
            }
        } else {
            if !p.is_file() {
                return store::fail(2, "declared input must be a regular file or directory");
            }
            let resolved = p.canonicalize()?;
            if !resolved.starts_with(root.canonicalize()?) {
                return store::fail(2, "declared file escapes project");
            }
            out.push(json!({
                "path": p.strip_prefix(root).map_err(|e| Error(2, e.to_string()))?,
                "sha256": store::hash_file_nofollow(&resolved)?,
            }));
        }
        Ok(())
    }
    for rel in declared {
        store::input(root, rel)?;
        walk(root, &root.join(rel), &mut out)?;
    }
    out.sort_by_key(|v| v["path"].as_str().unwrap_or("").to_owned());
    out.dedup_by(|a, b| a["path"] == b["path"]);
    Ok(json!(out))
}

const MAX_SAFE_INT: u64 = 9_007_199_254_740_991;

fn summary_cargo(output: &str) -> Option<(u64, u64, u64)> {
    let rust = regex::Regex::new(
        r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",
    )
    .unwrap();
    let mut matched = false;
    let mut sum = (0u64, 0u64, 0u64);
    for c in rust.captures_iter(output) {
        matched = true;
        sum.0 = sum.0.checked_add(c[1].parse::<u64>().ok()?)?;
        sum.1 = sum.1.checked_add(c[2].parse::<u64>().ok()?)?;
        sum.2 = sum.2.checked_add(c[3].parse::<u64>().ok()?)?;
        if sum.0 > MAX_SAFE_INT || sum.1 > MAX_SAFE_INT || sum.2 > MAX_SAFE_INT {
            return None;
        }
    }
    matched.then_some(sum)
}
fn summary_bun(output: &str) -> Option<(u64, u64, u64)> {
    let pass = regex::Regex::new(r"(?m)^\s*(\d+) pass\s*$").unwrap();
    let fail = regex::Regex::new(r"(?m)^\s*(\d+) fail\s*$").unwrap();
    let skip = regex::Regex::new(r"(?m)^\s*(\d+) skip\s*$").unwrap();
    let mut pass_matches = 0usize;
    let mut fail_matches = 0usize;
    let mut passed = 0u64;
    let mut failed = 0u64;
    let mut skipped = 0u64;
    for c in pass.captures_iter(output) {
        pass_matches += 1;
        passed = passed.checked_add(c[1].parse::<u64>().ok()?)?;
    }
    for c in fail.captures_iter(output) {
        fail_matches += 1;
        failed = failed.checked_add(c[1].parse::<u64>().ok()?)?;
    }
    for c in skip.captures_iter(output) {
        skipped = skipped.checked_add(c[1].parse::<u64>().ok()?)?;
    }
    if pass_matches == 0 && fail_matches == 0 {
        return None;
    }
    if passed > MAX_SAFE_INT || failed > MAX_SAFE_INT || skipped > MAX_SAFE_INT {
        return None;
    }
    Some((passed, failed, skipped))
}
fn xml_attr_u64(attrs: &str, name: &str) -> Option<u64> {
    let pattern = format!(r#"(?i)(?:^|\s){name}\s*=\s*["'](\d+)["']"#);
    let re = regex::Regex::new(&pattern).ok()?;
    re.captures(attrs)?.get(1)?.as_str().parse::<u64>().ok()
}
fn parse_junit_attrs(attrs: &str) -> Option<(u64, u64, u64)> {
    let tests = xml_attr_u64(attrs, "tests")?;
    let failures = xml_attr_u64(attrs, "failures").unwrap_or(0);
    let errors = xml_attr_u64(attrs, "errors").unwrap_or(0);
    let skipped = xml_attr_u64(attrs, "skipped")
        .unwrap_or(0)
        .checked_add(xml_attr_u64(attrs, "disabled").unwrap_or(0))?;
    let failed = failures.checked_add(errors)?;
    let non_passed = failed.checked_add(skipped)?;
    let passed = tests.checked_sub(non_passed)?;
    Some((passed, failed, skipped))
}
fn summary_junit(output: &str) -> Option<(u64, u64, u64)> {
    let suite_re = regex::Regex::new(r"(?is)<testsuite\b([^>]*)>").unwrap();
    let suites_re = regex::Regex::new(r"(?is)<testsuites\b([^>]*)>").unwrap();
    let mut matched = false;
    let mut sum = (0u64, 0u64, 0u64);
    for c in suite_re.captures_iter(output) {
        if xml_attr_u64(&c[1], "tests").is_some() {
            let (p, f, s) = parse_junit_attrs(&c[1])?;
            sum.0 = sum.0.checked_add(p)?;
            sum.1 = sum.1.checked_add(f)?;
            sum.2 = sum.2.checked_add(s)?;
            if sum.0 > MAX_SAFE_INT || sum.1 > MAX_SAFE_INT || sum.2 > MAX_SAFE_INT {
                return None;
            }
            matched = true;
        }
    }
    if !matched {
        for c in suites_re.captures_iter(output) {
            if xml_attr_u64(&c[1], "tests").is_some() {
                let (p, f, s) = parse_junit_attrs(&c[1])?;
                sum.0 = sum.0.checked_add(p)?;
                sum.1 = sum.1.checked_add(f)?;
                sum.2 = sum.2.checked_add(s)?;
                if sum.0 > MAX_SAFE_INT || sum.1 > MAX_SAFE_INT || sum.2 > MAX_SAFE_INT {
                    return None;
                }
                matched = true;
            }
        }
    }
    matched.then_some(sum)
}
fn summary(output: &str, parser: &str) -> Option<(u64, u64, u64)> {
    match parser {
        "cargo" => summary_cargo(output),
        "bun" => summary_bun(output),
        "junit" => summary_junit(output),
        "auto" => summary_cargo(output)
            .or_else(|| summary_bun(output))
            .or_else(|| summary_junit(output)),
        _ => None,
    }
}
fn configured_min_passed(root: &Path, label: &str) -> Result<Option<u64>> {
    let gate = root.join(".kineti/gate.json");
    if fs::symlink_metadata(&gate).is_err() {
        return Ok(None);
    }
    let config = store::read(&gate)?;
    crate::gate::parse_min_passed_value(&config, label)
}
fn configured_parser(root: &Path, label: &str) -> Result<Option<String>> {
    let gate = root.join(".kineti/gate.json");
    if fs::symlink_metadata(&gate).is_err() {
        return Ok(None);
    }
    let config = store::read(&gate)?;
    parse_policy_parser(&config, label)
}
// Keep the direct child waitable until capture is complete. Reaping earlier would
// release its numeric identity while descendants can still hold the output pipes.
struct Execution {
    child: std::process::Child,
    out: Option<std::process::ChildStdout>,
    err: Option<std::process::ChildStderr>,
    owned: bool,
    deadline: Instant,
}
impl Execution {
    fn status_ready(&mut self) -> std::io::Result<bool> {
        let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
        let options = libc::WEXITED | libc::WNOHANG | libc::WNOWAIT;
        #[cfg(test)]
        let options = fault_proof::wait_options(options);
        let rc = unsafe {
            libc::waitid(
                libc::P_PID,
                self.child.id() as libc::id_t,
                &mut info,
                options,
            )
        };
        if rc == 0 {
            return Ok(unsafe { info.si_pid() } != 0);
        }
        let error = std::io::Error::last_os_error();
        #[cfg(test)]
        fault_proof::observe_error("waitid", &error);
        if error.raw_os_error() == Some(libc::ECHILD) {
            // Another waiter or automatic reaping lost our identity pin. Never
            // signal this number again, even if a later lookup finds a process.
            self.owned = false;
        }
        Err(error)
    }
    fn signal_group(&mut self) -> std::io::Result<()> {
        if !self.owned {
            return Err(std::io::Error::other("child ownership was lost"));
        }
        // Revalidate wait ownership immediately before using the numeric PGID.
        self.status_ready()?;
        if unsafe { libc::kill(-(self.child.id() as i32), libc::SIGKILL) } == 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        #[cfg(test)]
        fault_proof::observe_error("killpg", &error);
        if error.raw_os_error() == Some(libc::ESRCH) {
            Ok(())
        } else {
            Err(error)
        }
    }
    fn cleanup(&mut self, require_group_signal: bool) -> Result<std::process::ExitStatus> {
        #[cfg(test)]
        fault_proof::cleanup_started(self.deadline);
        let until =
            (Instant::now() + Duration::from_secs(1)).min(self.deadline + Duration::from_secs(1));
        let mut diagnostic = String::new();
        loop {
            if !self.owned {
                break;
            }
            match self.signal_group() {
                Ok(()) => break,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted && Instant::now() < until => {
                }
                Err(e) => {
                    if require_group_signal {
                        diagnostic = format!("group signal/control failed: {e}");
                    }
                    // After definite status and both EOFs, preserve the old
                    // best-effort descendant signal behavior. An EPERM for a
                    // zombie-only macOS group cannot change the observed exit.
                    // Incomplete work instead reports unconfirmed group control.
                    break;
                }
            }
        }
        self.out.take();
        self.err.take();
        while self.owned {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.owned = false;
                    if diagnostic.is_empty() {
                        return Ok(status);
                    }
                    return Err(Error(3, format!("cleanup unconfirmed for execution group {}; direct child reaped; {diagnostic}", self.child.id())));
                }
                Ok(None) => {
                    #[cfg(test)]
                    fault_proof::cleanup_pending();
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
                Err(e) => {
                    diagnostic = format!("child reap failed: {e}; {diagnostic}");
                    break;
                }
            }
            let now = Instant::now();
            if now >= until {
                break;
            }
            std::thread::sleep((until - now).min(Duration::from_millis(5)));
        }
        // No later destructor may signal a number whose ownership is uncertain.
        self.owned = false;
        Err(Error(3, format!(
            "cleanup unconfirmed for direct child {} / execution group {} within one-second bound; {diagnostic}",
            self.child.id(), self.child.id()
        )))
    }
}
impl Drop for Execution {
    fn drop(&mut self) {
        if self.owned {
            let _ = self.cleanup(true);
        }
    }
}
#[derive(Default)]
struct Capture {
    bytes: Vec<u8>,
    truncated: bool,
    eof: bool,
}
fn drain(stream: &mut impl Read, capture: &mut Capture) -> std::io::Result<()> {
    // Both descriptors get bounded service before status/deadline checks. Keep
    // draining after retention is full so a noisy child cannot block on a pipe.
    let mut chunk = [0u8; 8192];
    for _ in 0..8 {
        match stream.read(&mut chunk) {
            Ok(0) => {
                capture.eof = true;
                break;
            }
            Ok(n) => {
                let keep = n.min(8_000_000usize.saturating_sub(capture.bytes.len()));
                capture.bytes.extend_from_slice(&chunk[..keep]);
                capture.truncated |= keep < n;
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                #[cfg(test)]
                fault_proof::observe_error("read", &e);
                return Err(e);
            }
        }
    }
    Ok(())
}
fn nonblocking(fd: std::os::fd::RawFd, deadline: Instant) -> std::io::Result<()> {
    #[cfg(test)]
    let fd = fault_proof::setup_fd(fd);
    loop {
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
        if flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } >= 0 {
            return Ok(());
        }
        let error = std::io::Error::last_os_error();
        #[cfg(test)]
        fault_proof::observe_error("fcntl", &error);
        if error.kind() != std::io::ErrorKind::Interrupted || Instant::now() >= deadline {
            return Err(error);
        }
    }
}
fn poll_outputs(
    out: i32,
    err: i32,
    stdout: &Capture,
    stderr: &Capture,
    millis: i32,
) -> std::io::Result<()> {
    let mut fds = [
        libc::pollfd {
            fd: if stdout.eof { -1 } else { out },
            events: libc::POLLIN,
            revents: 0,
        },
        libc::pollfd {
            fd: if stderr.eof { -1 } else { err },
            events: libc::POLLIN,
            revents: 0,
        },
    ];
    #[cfg(test)]
    fault_proof::poll_fds(&mut fds);
    let pointer = fds.as_mut_ptr();
    #[cfg(test)]
    let pointer = fault_proof::poll_pointer(pointer);
    #[cfg(test)]
    fault_proof::enter_poll();
    #[cfg(test)]
    if fault_proof::poll_again() {
        return Err(std::io::Error::from_raw_os_error(libc::EAGAIN));
    }
    if unsafe { libc::poll(pointer, 2, millis) } < 0 {
        let error = std::io::Error::last_os_error();
        #[cfg(test)]
        fault_proof::observe_error("poll", &error);
        return Err(error);
    }
    if fds.iter().any(|fd| fd.revents & libc::POLLNVAL != 0) {
        #[cfg(test)]
        fault_proof::observe_poll_invalid();
        return Err(std::io::Error::other(
            "invalid output descriptor during poll",
        ));
    }
    if fds.iter().any(|fd| fd.revents & libc::POLLERR != 0) {
        return Err(std::io::Error::other("output descriptor poll failure"));
    }
    Ok(())
}
fn wait_control_supported() -> Result<()> {
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    return store::fail(
        3,
        "execution lifecycle control is unsupported on this operating system",
    );
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let mut action = unsafe { std::mem::zeroed::<libc::sigaction>() };
        if unsafe { libc::sigaction(libc::SIGCHLD, std::ptr::null(), &mut action) } < 0 {
            return Err(Error(
                3,
                format!(
                    "cannot inspect SIGCHLD wait ownership: {}",
                    std::io::Error::last_os_error()
                ),
            ));
        }
        if action.sa_sigaction != libc::SIG_DFL || action.sa_flags & libc::SA_NOCLDWAIT != 0 {
            return store::fail(3, "execution requires default SIGCHLD disposition without SA_NOCLDWAIT; external child reapers are unsupported");
        }
        Ok(())
    }
}
/// Execute exact argv with one deadline for child status and both output EOFs.
/// Cleanup has a separate one-second bound; escaped descendants are not contained.
pub fn execute(root: &Path, argv: &[String], timeout: u64) -> Result<(Option<i32>, String, bool)> {
    use std::os::{fd::AsRawFd, unix::process::CommandExt};
    wait_control_supported()?;
    let program = if argv[0].contains('/') && !Path::new(&argv[0]).is_absolute() {
        let base = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()?.join(root)
        };
        base.join(&argv[0])
    } else {
        PathBuf::from(&argv[0])
    };
    let child = Command::new(&program)
        .arg0(&argv[0])
        .args(&argv[1..])
        .current_dir(root)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(error) => return Ok((Some(127), format!("command did not start: {error}"), false)),
    };
    let mut execution = Execution {
        out: child.stdout.take(),
        err: child.stderr.take(),
        child,
        owned: true,
        deadline: Instant::now() + Duration::from_secs(timeout),
    };
    let mut stdout = Capture::default();
    let mut stderr = Capture::default();
    let supervision = (|| -> std::io::Result<bool> {
        #[cfg(test)]
        fault_proof::after_spawn(&execution)?;
        let out = execution
            .out
            .as_ref()
            .ok_or_else(|| std::io::Error::other("stdout pipe missing"))?
            .as_raw_fd();
        let err = execution
            .err
            .as_ref()
            .ok_or_else(|| std::io::Error::other("stderr pipe missing"))?
            .as_raw_fd();
        nonblocking(out, execution.deadline)?;
        nonblocking(err, execution.deadline)?;
        #[cfg(test)]
        fault_proof::before_read(&execution)?;
        loop {
            if Instant::now() >= execution.deadline {
                return Ok(false);
            }
            if !stdout.eof {
                drain(execution.out.as_mut().unwrap(), &mut stdout)?;
            }
            if !stderr.eof {
                drain(execution.err.as_mut().unwrap(), &mut stderr)?;
            }
            let ready = match execution.status_ready() {
                Ok(ready) => ready,
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => false,
                Err(e) => return Err(e),
            };
            let now = Instant::now();
            if now >= execution.deadline {
                return Ok(false);
            }
            if ready && stdout.eof && stderr.eof {
                return Ok(true);
            }
            let remaining = execution.deadline - now;
            let millis = remaining.as_millis().min(10) as i32;
            if millis == 0 {
                std::thread::sleep(remaining);
                continue;
            }
            if let Err(error) = poll_outputs(out, err, &stdout, &stderr, millis) {
                match error.kind() {
                    std::io::ErrorKind::Interrupted => {}
                    std::io::ErrorKind::WouldBlock => {
                        // POSIX poll may temporarily lack internal resources.
                        // Retry without resetting the deadline or spinning.
                        let now = Instant::now();
                        if now < execution.deadline {
                            std::thread::sleep(
                                (execution.deadline - now).min(Duration::from_millis(1)),
                            );
                        }
                    }
                    _ => return Err(error),
                }
            }
        }
    })();
    // Every post-spawn outcome, including setup/read/status/poll errors, goes
    // through bounded cleanup. The waitable leader pins identity until signaling.
    let status = execution.cleanup(!matches!(supervision, Ok(true)))?;
    let mut output = format!(
        "{}\n{}",
        String::from_utf8_lossy(&stdout.bytes),
        String::from_utf8_lossy(&stderr.bytes)
    );
    let complete = matches!(supervision, Ok(true));
    if !complete {
        output.push_str(&match supervision {
            Err(e) => format!("\nexecution/capture failed: {e}"),
            _ => "\nexecution/capture deadline expired".into(),
        });
    }
    Ok((
        if complete { status.code() } else { None },
        output,
        stdout.truncated || stderr.truncated,
    ))
}
/// Record an observed command and bind it to exact declared/native inputs.
pub fn run(root: &Path, opts: Run) -> Result<Value> {
    let parser_mode = if opts.parser.is_empty() {
        "auto"
    } else {
        opts.parser.as_str()
    };
    if !["auto", "cargo", "bun", "junit", "exit-code"].contains(&parser_mode) {
        return store::fail(2, format!("invalid parser: {parser_mode}"));
    }
    if opts.min_passed.is_some_and(|n| n == 0 || n > MAX_SAFE_INT) {
        return store::fail(2, "min_passed must be 1..=9007199254740991");
    }
    if opts.label.is_empty()
        || opts.argv.is_empty()
        || opts.argv[0].is_empty()
        || opts.argv.iter().any(|s| s.contains('\0'))
        || opts.timeout == 0
        || opts.timeout > 86400
        || !["test", "check", "package"].contains(&opts.kind.as_str())
    {
        return store::fail(2, "invalid run options");
    }
    if opts.kind == "test" && opts.tests.is_empty() {
        return store::fail(2, "test receipt requires --required-test");
    }
    let configured_min = if opts.kind == "test" {
        configured_min_passed(root, &opts.label)?
    } else {
        None
    };
    let effective_min_opt = match (opts.min_passed, configured_min) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    };
    if opts.kind == "test" && parser_mode == "exit-code" && effective_min_opt.is_some() {
        return store::fail(2, "min_passed is unsupported with exit-code parser");
    }
    let tests = files(root, &opts.tests)?;
    if opts.kind == "test" && tests.as_array().unwrap().is_empty() {
        return store::fail(2, "test inventory is empty");
    }
    let artifacts = files(root, &opts.artifacts)?;
    let before = store::inventory(root)?;
    let before_digest = store::inventory_digest(&before)?;
    let before_count = before.as_array().map_or(0, Vec::len);
    let runtime_before = runtime(root, &opts.argv)?;
    let mut environment = json!({});
    let env_pattern = regex::Regex::new(r"^[A-Za-z_][A-Za-z0-9_]*$").unwrap();
    for name in opts.env.iter().chain(["PATH".to_string()].iter()) {
        if !env_pattern.is_match(name)
            || ["TOKEN", "SECRET", "PASSWORD", "KEY", "CREDENTIAL"]
                .iter()
                .any(|s| name.to_uppercase().contains(s))
        {
            return store::fail(2, "invalid or secret-like environment input");
        }
        environment[name] = json!(env_digest(name)?);
    }
    store::state(root)?;
    let (key, public) = keys(root)?;
    let (exit, output, truncated) = execute(root, &opts.argv, opts.timeout)?;
    let stable = store::inventory(root)? == before
        && runtime(root, &opts.argv)? == runtime_before
        && environment
            .as_object()
            .unwrap()
            .iter()
            .all(|(k, v)| env_digest(k).ok().as_deref() == v.as_str());
    let (observed, verification) = if opts.kind == "test" && parser_mode == "exit-code" {
        (
            true,
            json!({
                "parser": "kineti-exit-code-v1",
                "count_observed": false,
                "status": "valid",
                "passed": Value::Null,
                "failed": Value::Null,
                "skipped": Value::Null,
                "inventory": tests,
                "output_sha256": digest(output.as_bytes()),
                "truncated": truncated,
            }),
        )
    } else {
        let parsed = summary(&output, parser_mode);
        let min_passed = effective_min_opt.unwrap_or(1);
        let observed = opts.kind != "test"
            || parsed.is_some_and(|(p, f, s)| {
                p >= min_passed && f == 0 && (s == 0 || opts.allow_ignored)
            });
        (
            observed,
            json!({
                "parser": "kineti-native-summary-v1",
                "count_observed": opts.kind == "test" && parsed.is_some(),
                "allow_ignored": opts.allow_ignored,
                "min_passed": effective_min_opt,
                "status": if observed { "valid" } else { "unsupported" },
                "passed": parsed.map(|v| v.0),
                "failed": parsed.map(|v| v.1),
                "skipped": parsed.map(|v| v.2),
                "inventory": tests,
                "output_sha256": digest(output.as_bytes()),
                "truncated": truncated,
            }),
        )
    };
    let passed = exit == Some(0) && stable && observed && !truncated;
    let dependencies = files(
        root,
        &[
            "Cargo.lock",
            "core-native/Cargo.lock",
            "package-lock.json",
            "bun.lock",
        ]
        .iter()
        .filter(|p| root.join(p).is_file())
        .map(ToString::to_string)
        .collect::<Vec<_>>(),
    )?;
    let mut receipt = json!({
        "receipt_version": 3,
        "encoding": "kineti-canonical-v2",
        "run_id": store::nonce()?,
        "at": store::now(),
        "label": opts.label,
        "cmd": opts.argv.join(" "),
        "exit_code": exit,
        "result_kind": opts.kind,
        "outcome": if passed { "passed" } else { "failed" },
        "fingerprint": before_digest.as_str(),
        "manifest": {
            "command": opts.argv,
            "inventory_sha256": before_digest.as_str(),
            "inventory_count": before_count,
            "dependencies": dependencies,
            "environment": environment,
            "runtime": runtime_before,
            "artifacts": artifacts,
            "session_ref": std::env::var("KINETI_SESSION_ID").ok(),
        },
        "verification": verification,
        "mutated_during_run": !stable,
        "issuer_key": public,
    });
    receipt["signature"] =
        json!(STANDARD.encode(key.sign(unsigned(&receipt)?.as_bytes()).to_bytes()));
    let _lock = Lock::take(&root.join(".kineti/evidence.lock"))?;
    store::lines(&root.join(".kineti/evidence.jsonl"))?;
    store::append(&root.join(".kineti/evidence.jsonl"), &receipt)?;
    if !passed {
        return Err(Error(
            1,
            format!(
                "command evidence recorded but failed: exit={exit:?}, stable={stable}, output_supported={observed}"
            ),
        ));
    }
    Ok(receipt)
}
/// Signature and freshness are separate. Legacy evidence remains readable without upgraded authority.
pub fn check(root: &Path, label: &str, max_age: u64, expected: Option<&Value>) -> Result<Value> {
    check_kind(root, label, max_age, expected, false)
}
/// Check a test receipt strictly against an explicit base-branch policy JSON object
/// instead of reading `min_passed`/`parsers` from working-tree `.kineti/gate.json`.
pub(crate) fn check_with_policy(
    root: &Path,
    label: &str,
    max_age: u64,
    expected: Option<&Value>,
    policy: &Value,
) -> Result<Value> {
    check_internal(root, label, max_age, expected, false, Some(policy))
}
/// Evidence may describe successful operational commands without test authority.
pub fn check_kind(
    root: &Path,
    label: &str,
    max_age: u64,
    expected: Option<&Value>,
    operational: bool,
) -> Result<Value> {
    check_internal(root, label, max_age, expected, operational, None)
}
fn parse_policy_parser(policy: &Value, label: &str) -> Result<Option<String>> {
    let Some(raw) = policy.get("parsers") else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let map = raw
        .as_object()
        .ok_or_else(|| Error(2, "gate.json parsers must be an object".into()))?;
    let Some(v) = map.get(label) else {
        return Ok(None);
    };
    let mode = v
        .as_str()
        .ok_or_else(|| Error(2, "gate.json parser entry must be a string".into()))?;
    if !["auto", "cargo", "bun", "junit", "exit-code"].contains(&mode) {
        return store::fail(2, format!("invalid configured parser: {mode}"));
    }
    Ok(Some(mode.to_owned()))
}
fn check_internal(
    root: &Path,
    label: &str,
    max_age: u64,
    expected: Option<&Value>,
    operational: bool,
    policy_override: Option<&Value>,
) -> Result<Value> {
    if max_age == 0 || max_age > 5_256_000 {
        return store::fail(2, "receipt age must be 1..5256000 minutes");
    }
    store::state(root)?;
    let records = {
        let _lock = Lock::take(&root.join(".kineti/evidence.lock"))?;
        store::lines(&root.join(".kineti/evidence.jsonl"))?
    };
    let r = records
        .iter()
        .rev()
        .find(|v| v["label"] == label)
        .ok_or_else(|| Error(5, format!("missing evidence: {label}")))?;
    let trusted_path = root.join(".kineti/evidence-issuer.pem");
    store::safe_destination(&trusted_path)?;
    let trusted = store::read_text_nofollow(&trusted_path).map_err(|e| {
        if e.0 == 3 {
            e
        } else {
            Error(7, "trusted issuer missing".into())
        }
    })?;
    if !signature(r, &trusted) {
        return store::fail(7, "altered or untrusted receipt");
    }
    if r["receipt_version"] != 3 || r["encoding"] != "kineti-canonical-v2" {
        return store::fail(6,"legacy signature is valid; exact legacy runtime/parser freshness is unsupported by this native installation, so it cannot authorize CI");
    }
    let at = time::OffsetDateTime::parse(
        r["at"]
            .as_str()
            .ok_or_else(|| Error(7, "missing timestamp".into()))?,
        &time::format_description::well_known::Rfc3339,
    )
    .map_err(|_| Error(7, "invalid timestamp".into()))?;
    let age = (time::OffsetDateTime::now_utc() - at).whole_seconds();
    if age < -60 || age > max_age as i64 * 60 {
        return store::fail(4, "receipt expired or future-dated");
    }
    let is_test = r["result_kind"] == "test";
    let base_ok = (is_test
        || operational && ["check", "package"].iter().any(|k| r["result_kind"] == *k))
        && r["outcome"] == "passed"
        && r["exit_code"] == 0
        && r["mutated_during_run"] == false
        && r["verification"]["status"] == "valid"
        && r["verification"]["truncated"] == false;
    if !base_ok {
        return store::fail(7, "receipt does not record supported passing test evidence");
    }
    if is_test {
        let has_inventory = r["verification"]["inventory"]
            .as_array()
            .is_some_and(|a| !a.is_empty());
        if !has_inventory {
            return store::fail(7, "receipt does not record supported passing test evidence");
        }
        let parser_name = r["verification"]["parser"].as_str().unwrap_or("");
        let (gate_min, gate_parser) = if let Some(policy) = policy_override {
            (
                crate::gate::parse_min_passed_value(policy, label)?,
                parse_policy_parser(policy, label)?,
            )
        } else {
            (
                configured_min_passed(root, label)?,
                configured_parser(root, label)?,
            )
        };
        match parser_name {
            "kineti-native-summary-v1" => {
                let allow_ignored = r["verification"]
                    .get("allow_ignored")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let receipt_min = r["verification"]
                    .get("min_passed")
                    .and_then(Value::as_u64)
                    .unwrap_or(1)
                    .max(1);
                let min_passed = receipt_min.max(gate_min.unwrap_or(1));
                let passed_ok = r["verification"]["passed"]
                    .as_u64()
                    .is_some_and(|p| p >= min_passed);
                let failed_ok = r["verification"]["failed"].as_u64() == Some(0);
                let skipped_ok = r["verification"]["skipped"]
                    .as_u64()
                    .is_some_and(|s| s == 0 || allow_ignored);
                if !(passed_ok && failed_ok && skipped_ok) {
                    return store::fail(
                        7,
                        "receipt does not record supported passing test evidence",
                    );
                }
            }
            "kineti-exit-code-v1" => {
                if r["verification"]["count_observed"] != false
                    || gate_min.is_some()
                    || gate_parser.as_deref().is_some_and(|p| p != "exit-code")
                {
                    return store::fail(
                        7,
                        "receipt does not record supported passing test evidence",
                    );
                }
            }
            _ => {
                return store::fail(7, "receipt does not record supported passing test evidence");
            }
        }
    }
    let m = &r["manifest"];
    let argv = m["command"]
        .as_array()
        .ok_or_else(|| Error(7, "command array missing".into()))?
        .iter()
        .map(|v| {
            v.as_str()
                .map(String::from)
                .ok_or_else(|| Error(7, "command argument invalid".into()))
        })
        .collect::<Result<Vec<_>>>()?;
    let configured = if expected.is_none() {
        crate::gate::configured(root, label, is_test)?
    } else {
        None
    };
    let expected = expected.or(configured.as_ref());
    if argv.is_empty()
        || argv[0].is_empty()
        || argv.iter().any(|s| s.contains('\0'))
        || expected.is_some_and(|e| e != &m["command"])
    {
        return store::fail(7, "command does not match required exact argv");
    }
    let current_inventory = store::inventory(root)?;
    let inventory_matches = if let Some(expected_digest) = m.get("inventory_sha256") {
        let expected_digest = expected_digest
            .as_str()
            .ok_or_else(|| Error(7, "inventory_sha256 invalid".into()))?;
        if r["fingerprint"].as_str() != Some(expected_digest) {
            return store::fail(7, "receipt fingerprint mismatch");
        }
        let current_digest = store::inventory_digest(&current_inventory)?;
        let count_matches = match m.get("inventory_count") {
            Some(c) => c.as_u64() == Some(current_inventory.as_array().map_or(0, Vec::len) as u64),
            None => true,
        };
        current_digest == expected_digest && count_matches
    } else if let Some(expected_inputs) = m.get("inputs") {
        &current_inventory == expected_inputs
    } else {
        return store::fail(7, "manifest inventory missing");
    };
    if !inventory_matches || runtime(root, &argv)? != m["runtime"] {
        return store::fail(4, "inputs or runtime changed");
    }
    for (k, v) in m["environment"]
        .as_object()
        .ok_or_else(|| Error(7, "environment missing".into()))?
    {
        if env_digest(k)? != v.as_str().unwrap_or("") {
            return store::fail(4, format!("environment changed: {k}"));
        }
    }
    for list in [
        &m["dependencies"],
        &m["artifacts"],
        &r["verification"]["inventory"],
    ] {
        for item in list
            .as_array()
            .ok_or_else(|| Error(7, "file inventory missing".into()))?
        {
            let rel = item["path"]
                .as_str()
                .ok_or_else(|| Error(7, "file path missing".into()))?;
            match fs::symlink_metadata(root.join(rel)) {
                Ok(meta) if !meta.file_type().is_symlink() => {}
                _ => return store::fail(4, "declared file missing or replaced with symlink"),
            }
            let p =
                store::input(root, rel).map_err(|_| Error(4, "declared file missing".into()))?;
            let actual = store::hash_file_nofollow(&p)
                .map_err(|_| Error(4, "declared file missing".into()))?;
            if actual != item["sha256"].as_str().unwrap_or("") {
                return store::fail(4, "declared file changed");
            }
        }
    }
    Ok(
        json!({"status":if is_test {"fresh"} else {"operational"},"signature_valid":true,"independent_test_execution":false,"gate_authorized":is_test,"receipt":r}),
    )
}

// Test-only, thread-local syscall argument seams. None of this module or its
// call sites exist in the installed native executable. Tests label these as
// controlled altered-argument faults with real kernel returns, not natural faults.
#[cfg(test)]
mod fault_proof {
    use super::*;
    use std::{cell::RefCell, os::fd::AsRawFd, sync::mpsc::SyncSender};
    pub(super) struct Plan {
        pub mode: String,
        pub ready: fs::File,
        pub pid: u32,
        pub descriptors: [i32; 2],
        pub injected: bool,
        pub observations: Vec<Value>,
        pub poll_calls: usize,
        pub interrupted_polls: usize,
        pub poll_enter: Option<SyncSender<()>>,
        pub cleanup_start: Option<Instant>,
        pub cleanup_pending_count: usize,
        pub cleanup_available: Option<Duration>,
    }
    thread_local! { static PLAN: RefCell<Option<Plan>> = const { RefCell::new(None) }; }
    pub(super) fn install(plan: Plan) {
        PLAN.with(|p| {
            assert!(p.borrow().is_none());
            *p.borrow_mut() = Some(plan);
        });
    }
    pub(super) fn take() -> Plan {
        PLAN.with(|p| p.borrow_mut().take().unwrap())
    }
    fn change(mode: &str) -> bool {
        PLAN.with(|p| {
            let mut p = p.borrow_mut();
            if let Some(plan) = p.as_mut() {
                if plan.mode == mode && !plan.injected {
                    plan.injected = true;
                    return true;
                }
            }
            false
        })
    }
    pub(super) fn after_spawn(execution: &Execution) -> std::io::Result<()> {
        PLAN.with(|p| {
            let mut p = p.borrow_mut();
            let Some(plan) = p.as_mut() else {
                return Ok(());
            };
            plan.pid = execution.child.id();
            plan.descriptors = [
                execution.out.as_ref().unwrap().as_raw_fd(),
                execution.err.as_ref().unwrap().as_raw_fd(),
            ];
            let mut event = libc::pollfd {
                fd: plan.ready.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            if unsafe { libc::poll(&mut event, 1, 2000) } != 1 {
                return Err(std::io::Error::other("live fixture readiness watchdog"));
            }
            let mut byte = [0u8];
            plan.ready.read_exact(&mut byte)?;
            if byte != *b"R" {
                return Err(std::io::Error::other("invalid live fixture readiness"));
            }
            let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
            let rc = unsafe {
                libc::waitid(
                    libc::P_PID,
                    plan.pid as libc::id_t,
                    &mut info,
                    libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
                )
            };
            if rc != 0 || unsafe { info.si_pid() } != 0 {
                return Err(std::io::Error::other(
                    "fault fixture was not live after readiness",
                ));
            }
            plan.observations
                .push(json!({"event":"live-child-ready","pid":plan.pid,"observed_group":unsafe { libc::getpgid(plan.pid as i32) },"supervisor_group":unsafe { libc::getpgrp() }}));
            Ok(())
        })
    }
    pub(super) fn setup_fd(fd: i32) -> i32 {
        if change("setup") {
            -1
        } else {
            fd
        }
    }
    pub(super) fn before_read(execution: &Execution) -> std::io::Result<()> {
        if change("read") {
            let directory = fs::File::open(std::env::current_dir()?)?;
            if unsafe {
                libc::dup2(
                    directory.as_raw_fd(),
                    execution.out.as_ref().unwrap().as_raw_fd(),
                )
            } < 0
            {
                return Err(std::io::Error::last_os_error());
            }
        }
        Ok(())
    }
    pub(super) fn wait_options(options: i32) -> i32 {
        if change("status") {
            -1
        } else {
            options
        }
    }
    pub(super) fn poll_fds(fds: &mut [libc::pollfd; 2]) {
        if change("poll-invalid") {
            fds[0].fd = i32::MAX;
        }
    }
    pub(super) fn poll_pointer(pointer: *mut libc::pollfd) -> *mut libc::pollfd {
        if change("poll-syscall") {
            std::ptr::null_mut()
        } else {
            pointer
        }
    }
    pub(super) fn enter_poll() {
        PLAN.with(|p| {
            if let Some(plan) = p.borrow_mut().as_mut() {
                plan.poll_calls += 1;
                if let Some(sender) = &plan.poll_enter {
                    let _ = sender.try_send(());
                }
            }
        });
    }
    pub(super) fn cleanup_started(deadline: Instant) {
        PLAN.with(|p| {
            if let Some(plan) = p.borrow_mut().as_mut() {
                let now = Instant::now();
                plan.cleanup_start = Some(now);
                plan.cleanup_available = Some(
                    Duration::from_secs(1)
                        .min((deadline + Duration::from_secs(1)).saturating_duration_since(now)),
                );
            }
        });
    }
    pub(super) fn cleanup_pending() {
        PLAN.with(|p| {
            if let Some(plan) = p.borrow_mut().as_mut() {
                plan.cleanup_pending_count += 1;
            }
        });
    }
    pub(super) fn poll_again() -> bool {
        let repeated = PLAN.with(|p| {
            p.borrow()
                .as_ref()
                .is_some_and(|plan| plan.mode == "poll-again-deadline")
        });
        if repeated || change("poll-again") {
            PLAN.with(|p| {
                let mut p = p.borrow_mut(); let plan = p.as_mut().unwrap(); plan.injected = true;
                if plan.observations.len() < 32 { plan.observations.push(json!({"syscall":"poll","errno":libc::EAGAIN,"origin":"injected return, not kernel allocation pressure"})); }
            });
            true
        } else {
            false
        }
    }
    pub(super) fn observe_poll_invalid() {
        PLAN.with(|p| {
            if let Some(plan) = p.borrow_mut().as_mut() {
                plan.observations
                    .push(json!({"syscall":"poll","revents":"POLLNVAL"}));
            }
        });
    }
    pub(super) fn observe_error(syscall: &str, error: &std::io::Error) {
        PLAN.with(|p| { if let Some(plan) = p.borrow_mut().as_mut() {
            if plan.observations.len() < 32 { plan.observations.push(json!({"syscall":syscall,"errno":error.raw_os_error(),"diagnostic":error.to_string()})); }
            if syscall == "poll" && error.kind() == std::io::ErrorKind::Interrupted { plan.interrupted_polls += 1; }
        } });
    }
}

#[cfg(test)]
mod supervision_tests {
    use super::*;
    use std::{io::Write, os::fd::AsRawFd, sync::mpsc};

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.kineti/runner-tests")
                .join(&store::nonce().unwrap()[..12]);
            fs::create_dir_all(root.join(".kineti")).unwrap();
            Self(root.canonicalize().unwrap())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn shell(root: &Path, text: &str, timeout: u64) -> (Option<i32>, String, bool) {
        execute(root, &["/bin/sh".into(), "-c".into(), text.into()], timeout).unwrap()
    }
    // This is a test executable fixture, never a production CLI flag. Two
    // project-local FIFOs supply readiness/release and an independent watchdog.
    #[test]
    fn controlled_holder_fixture() {
        let root = std::env::current_dir().unwrap();
        let Ok(mode) = fs::read_to_string(root.join(".kineti/holder-mode")) else {
            return;
        };
        let ready = fs::OpenOptions::new()
            .write(true)
            .open(root.join(".kineti/ready"))
            .unwrap();
        let release = fs::OpenOptions::new()
            .read(true)
            .open(root.join(".kineti/release"))
            .unwrap();
        if mode == "eof-first" {
            unsafe {
                libc::close(1);
                libc::close(2);
                libc::write(ready.as_raw_fd(), b"R".as_ptr().cast(), 1);
                let mut poll = libc::pollfd {
                    fd: release.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                libc::poll(&mut poll, 1, 4000);
                libc::_exit(0);
            }
        }
        let mut handshake = [0; 2];
        assert_eq!(unsafe { libc::pipe(handshake.as_mut_ptr()) }, 0);
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0);
        if pid == 0 {
            // Only async-signal-safe libc operations after fork in this threaded
            // test executable. No allocator, std destructor, or test harness exit.
            unsafe {
                libc::close(handshake[0]);
                if mode.starts_with("escaped") && libc::setsid() < 0 {
                    libc::_exit(91);
                }
                if mode == "escaped-out" {
                    libc::close(2);
                }
                libc::write(handshake[1], b"R".as_ptr().cast(), 1);
                libc::close(handshake[1]);
                libc::write(ready.as_raw_fd(), b"R".as_ptr().cast(), 1);
                let mut poll = libc::pollfd {
                    fd: release.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                libc::poll(&mut poll, 1, 4000);
                libc::_exit(0);
            }
        }
        let mut byte = [0u8];
        unsafe {
            libc::close(handshake[1]);
            assert_eq!(libc::read(handshake[0], byte.as_mut_ptr().cast(), 1), 1);
            libc::close(handshake[0]);
        }
        assert_eq!(&byte, b"R");
        std::process::exit(0);
    }
    fn controlled(
        mode: &str,
        release_after: Option<Duration>,
    ) -> (Result<(Option<i32>, String, bool)>, Duration) {
        use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};
        let p = Fixture::new();
        fs::write(p.0.join(".kineti/holder-mode"), mode).unwrap();
        for name in ["ready", "release"] {
            let name =
                std::ffi::CString::new(p.0.join(".kineti").join(name).as_os_str().as_bytes())
                    .unwrap();
            assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        }
        let mut ready = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(p.0.join(".kineti/ready"))
            .unwrap();
        let mut release_fifo = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(p.0.join(".kineti/release"))
            .unwrap();
        let (release, wait_release) = mpsc::channel();
        let controller = std::thread::spawn(move || {
            let mut poll = libc::pollfd {
                fd: ready.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            assert_eq!(
                unsafe { libc::poll(&mut poll, 1, 5000) },
                1,
                "fixture readiness watchdog"
            );
            let mut byte = [0];
            ready.read_exact(&mut byte).unwrap();
            assert_eq!(&byte, b"R");
            let _ = wait_release.recv_timeout(release_after.unwrap_or(Duration::from_secs(3)));
            release_fifo.write_all(b"Q").unwrap();
        });
        let start = Instant::now();
        let result = execute(
            &p.0,
            &[
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                "--exact".into(),
                "receipt::supervision_tests::controlled_holder_fixture".into(),
                "--nocapture".into(),
            ],
            1,
        );
        let elapsed = start.elapsed();
        let _ = release.send(());
        controller.join().unwrap();
        (result, elapsed)
    }
    #[test]
    fn leader_exit_and_eof_are_both_required_before_deadline() {
        for mode in ["escaped-both", "escaped-out", "same-group", "eof-first"] {
            let (result, elapsed) = controlled(mode, None);
            match result {
                Ok((exit, output, _)) => {
                    assert_eq!(exit, None, "{mode}: {output}");
                    assert!(output.contains("deadline expired"), "{mode}: {output}");
                }
                Err(error) => {
                    assert_eq!(error.0, 3, "{mode}: {error}");
                    assert!(error.1.contains("cleanup unconfirmed"), "{mode}: {error}");
                }
            }
            assert!(elapsed >= Duration::from_secs(1), "{mode}: {elapsed:?}");
            assert!(elapsed < Duration::from_millis(2300), "{mode}: {elapsed:?}");
        }
        let (result, elapsed) = controlled("escaped-both", Some(Duration::from_millis(80)));
        let (exit, _, truncated) = result.unwrap();
        assert_eq!(exit, Some(0));
        assert!(!truncated);
        assert!(elapsed < Duration::from_secs(1));
    }
    #[test]
    fn completed_output_preserves_exit_utf8_assembly_and_caps() {
        let p = Fixture::new();
        assert_eq!(shell(&p.0, "exit 17", 2), (Some(17), "\n".into(), false));
        assert_eq!(shell(&p.0, "kill -TERM $$", 2).0, None);
        assert_eq!(
            shell(&p.0, "printf '\\377'; printf err >&2", 2),
            (Some(0), "\u{fffd}\nerr".into(), false)
        );
        assert_eq!(
            execute(&p.0, &["./definitely-missing".into()], 1)
                .unwrap()
                .0,
            Some(127)
        );
        for bytes in [8_000_000usize, 8_000_001] {
            let command = format!("dd if=/dev/zero bs={bytes} count=1 2>/dev/null & dd if=/dev/zero bs={bytes} count=1 1>&2 2>/dev/null; wait");
            let (exit, output, truncated) = shell(&p.0, &command, 10);
            assert_eq!(exit, Some(0));
            assert_eq!(output.len(), 16_000_001);
            assert_eq!(truncated, bytes > 8_000_000);
            assert!(output.as_bytes()[..8_000_000].iter().all(|b| *b == 0));
            assert_eq!(output.as_bytes()[8_000_000], b'\n');
            assert!(output.as_bytes()[8_000_001..].iter().all(|b| *b == 0));
        }
    }
    #[test]
    fn noisy_streams_cannot_starve_deadline() {
        let p = Fixture::new();
        let start = Instant::now();
        let (exit, output, _) = shell(&p.0, "while :; do printf out; printf err >&2; done", 1);
        assert_eq!(exit, None);
        assert!(output.contains("out"));
        assert!(output.contains("err"));
        assert!(start.elapsed() < Duration::from_millis(2300));
    }
    #[test]
    fn cleanup_reaps_direct_child_and_lost_ownership_never_signals() {
        use std::os::unix::process::CommandExt;
        let p = Fixture::new();
        let mut child = Command::new("/bin/sh")
            .args(["-c", "exec sleep 5"])
            .current_dir(&p.0)
            .process_group(0)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut execution = Execution {
            out: child.stdout.take(),
            err: child.stderr.take(),
            child,
            owned: true,
            deadline: Instant::now() + Duration::from_secs(2),
        };
        let pid = execution.child.id() as i32;
        execution.cleanup(true).unwrap();
        assert!(!execution.owned);
        assert_eq!(
            unsafe { libc::waitpid(pid, std::ptr::null_mut(), libc::WNOHANG) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ECHILD)
        );
        assert!(execution.signal_group().is_err());
        let mut child = Command::new("/bin/sh")
            .args(["-c", "exit 0"])
            .process_group(0)
            .spawn()
            .unwrap();
        child.wait().unwrap();
        let mut lost = Execution {
            child,
            out: None,
            err: None,
            owned: true,
            deadline: Instant::now() + Duration::from_secs(2),
        };
        assert_eq!(
            lost.status_ready().unwrap_err().raw_os_error(),
            Some(libc::ECHILD)
        );
        assert!(!lost.owned);
        assert!(lost.signal_group().is_err());
        let error = lost.cleanup(true).unwrap_err();
        assert_eq!(error.0, 3);
        assert!(error.1.contains("cleanup unconfirmed"));
    }
    #[test]
    fn pending_interrupted_and_failed_reads_are_not_eof() {
        struct Reads(Vec<std::io::Result<usize>>);
        impl Read for Reads {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                self.0.remove(0)
            }
        }
        for error in [libc::EAGAIN, libc::EINTR] {
            let mut capture = Capture::default();
            let mut reads = Reads(vec![
                Err(std::io::Error::from_raw_os_error(error)),
                Err(std::io::Error::from_raw_os_error(libc::EAGAIN)),
            ]);
            drain(&mut reads, &mut capture).unwrap();
            assert!(!capture.eof);
        }
        let mut capture = Capture::default();
        assert!(drain(
            &mut Reads(vec![Err(std::io::Error::from_raw_os_error(libc::EBADF))]),
            &mut capture
        )
        .is_err());
        assert!(!capture.eof);
        assert!(nonblocking(-1, Instant::now() + Duration::from_secs(1)).is_err());
        let (mut reader, _writer) = std::os::unix::net::UnixStream::pair().unwrap();
        nonblocking(reader.as_raw_fd(), Instant::now() + Duration::from_secs(1)).unwrap();
        drain(&mut reader, &mut capture).unwrap();
        assert!(!capture.eof);
    }
    #[test]
    fn invalid_poll_descriptor_is_a_capture_error() {
        let mut limit = unsafe { std::mem::zeroed::<libc::rlimit>() };
        assert_eq!(
            unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) },
            0
        );
        let fd = i32::MAX;
        assert!(
            limit.rlim_cur < fd as libc::rlim_t,
            "test requires a provably out-of-range descriptor"
        );
        let error = poll_outputs(fd, -1, &Capture::default(), &Capture::default(), 0).unwrap_err();
        assert!(error.to_string().contains("invalid output descriptor"));
    }
    extern "C" fn interrupt_handler(_: i32) {}
    #[test]
    fn live_fault_supervision_fixture() {
        use std::{
            os::unix::{ffi::OsStrExt, fs::OpenOptionsExt},
            sync::{
                atomic::{AtomicBool, Ordering},
                Arc,
            },
        };
        let root = std::env::current_dir().unwrap();
        let Ok(mode) = fs::read_to_string(root.join(".kineti/live-fault-mode")) else {
            return;
        };
        let fifo =
            std::ffi::CString::new(root.join(".kineti/live-ready").as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
        let ready = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(root.join(".kineti/live-ready"))
            .unwrap();
        let (sender, receiver) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        let signaler = if mode == "eintr" {
            let mut action = unsafe { std::mem::zeroed::<libc::sigaction>() };
            action.sa_sigaction = interrupt_handler as *const () as usize;
            unsafe {
                libc::sigemptyset(&mut action.sa_mask);
                assert_eq!(
                    libc::sigaction(libc::SIGUSR1, &action, std::ptr::null_mut()),
                    0
                );
            }
            // POSIX thread IDs can target another live thread. Carry the opaque
            // ID as an address-sized value across this test sender; musl uses a
            // raw pointer typedef. The target stays alive until the sender joins.
            let target = unsafe { libc::pthread_self() } as usize;
            let stopped = stop.clone();
            Some(std::thread::spawn(move || {
                receiver.recv_timeout(Duration::from_secs(3)).unwrap();
                let until = Instant::now() + Duration::from_secs(3);
                let mut count = 0;
                while !stopped.load(Ordering::Relaxed) && Instant::now() < until {
                    assert_eq!(
                        unsafe { libc::pthread_kill(target as libc::pthread_t, libc::SIGUSR1) },
                        0
                    );
                    count += 1;
                    std::thread::sleep(Duration::from_micros(300));
                }
                count
            }))
        } else {
            drop(receiver);
            None
        };
        fault_proof::install(fault_proof::Plan {
            mode: mode.clone(),
            ready,
            pid: 0,
            descriptors: [-1; 2],
            injected: false,
            observations: vec![],
            poll_calls: 0,
            interrupted_polls: 0,
            poll_enter: if mode == "eintr" { Some(sender) } else { None },
            cleanup_start: None,
            cleanup_pending_count: 0,
            cleanup_available: None,
        });
        let script = if mode == "poll-again" {
            "printf R > .kineti/live-ready; /bin/sleep 0.1"
        } else {
            "printf R > .kineti/live-ready; exec /bin/sleep 20"
        };
        let start = Instant::now();
        let result = execute(&root, &["/bin/sh".into(), "-c".into(), script.into()], 1);
        let elapsed = start.elapsed();
        stop.store(true, Ordering::Relaxed);
        let sent = signaler.map(|thread| thread.join().unwrap()).unwrap_or(0);
        let plan = fault_proof::take();
        let mut status = 0;
        let reaped = unsafe { libc::waitpid(plan.pid as i32, &mut status, libc::WNOHANG) } == -1
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ECHILD);
        let closed = plan.descriptors.iter().all(|fd| unsafe { libc::fcntl(*fd, libc::F_GETFD) } == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EBADF));
        let result_json = match &result {
            Ok((code, output, truncated)) => {
                json!({"exit":code,"output":output,"truncated":truncated})
            }
            Err(error) => json!({"error_code":error.0,"diagnostic":error.1}),
        };
        let observation = json!({"mode":mode,"result":result_json,"elapsed_seconds":elapsed.as_secs_f64(),"pid":plan.pid,"direct_child_reaped":reaped,"owned_descriptors_closed":closed,"poll_calls":plan.poll_calls,"kernel_interrupted_polls":plan.interrupted_polls,"directed_signals_sent":sent,"observations":plan.observations,"scope":"real live child; deliberate test-only syscall arguments except genuine directed-signal poll EINTR"});
        fs::write(
            root.join(".kineti/fault-observation.json"),
            observation.to_string(),
        )
        .unwrap();
        println!("live-fault-observation: {observation}");
        assert!(reaped && closed, "{observation}");
        assert!(elapsed < Duration::from_millis(2300), "{observation}");
        if mode == "poll-again" {
            assert_eq!(
                result.unwrap().0,
                Some(0),
                "pending poll EAGAIN must allow subsequent genuine completion"
            );
        } else {
            let (exit, output, _) = result.unwrap();
            assert_eq!(exit, None, "{observation}");
            if mode == "eintr" {
                assert!(
                    plan.interrupted_polls > 0,
                    "no kernel EINTR observed: {observation}"
                );
                assert!(elapsed >= Duration::from_secs(1));
                assert!(output.contains("deadline expired"));
            } else if mode == "poll-again-deadline" {
                assert!(
                    plan.injected && plan.poll_calls > 1 && plan.poll_calls < 2000,
                    "{observation}"
                );
                assert!(elapsed >= Duration::from_secs(1) && output.contains("deadline expired"));
            } else {
                assert!(plan.injected);
                assert!(output.contains("capture failed"));
                let (syscall, errno) = match mode.as_str() {
                    "setup" => ("fcntl", libc::EBADF),
                    "read" => ("read", libc::EISDIR),
                    "status" => ("waitid", libc::EINVAL),
                    "poll-syscall" => ("poll", libc::EFAULT),
                    "poll-invalid" => ("poll", 0),
                    _ => panic!("unknown fault mode"),
                };
                assert!(
                    plan.observations.iter().any(|v| v["syscall"] == syscall
                        && if mode == "poll-invalid" {
                            v["revents"] == "POLLNVAL"
                        } else {
                            v["errno"] == errno
                        }),
                    "{observation}"
                );
            }
        }
    }
    fn isolated_proof(mode: &str) -> Value {
        use std::os::unix::process::CommandExt;
        let p = Fixture::new();
        fs::write(p.0.join(".kineti/live-fault-mode"), mode).unwrap();
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                if mode == "cleanup-unconfirmed" {
                    "receipt::supervision_tests::real_group_loss_cleanup_fixture"
                } else {
                    "receipt::supervision_tests::live_fault_supervision_fixture"
                },
                "--nocapture",
            ])
            .current_dir(&p.0)
            .process_group(0)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(8);
        loop {
            if child.try_wait().unwrap().is_some() {
                break;
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("independent live-fault watchdog: {mode}");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let output = child.wait_with_output().unwrap();
        let record =
            fs::read_to_string(p.0.join(".kineti/fault-observation.json")).unwrap_or_default();
        println!("isolated-{mode}: {record}");
        assert!(
            output.status.success(),
            "{mode}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_str(&record).unwrap()
    }
    #[test]
    fn live_syscall_faults_fail_closed_and_reap_owned_child() {
        for mode in ["setup", "read", "poll-invalid", "poll-syscall", "status"] {
            isolated_proof(mode);
        }
    }
    #[test]
    fn genuine_directed_signal_interrupts_live_poll_without_extending_deadline() {
        isolated_proof("eintr");
    }
    #[test]
    fn transient_poll_eagain_remains_pending_for_live_completion() {
        isolated_proof("poll-again");
    }

    #[test]
    fn repeated_poll_eagain_cannot_reset_deadline_or_busy_loop() {
        isolated_proof("poll-again-deadline");
    }

    #[test]
    fn group_loss_leader_fixture() {
        let root = std::env::current_dir().unwrap();
        if !root.join(".kineti/group-loss-mode").exists() {
            return;
        }
        let ready = fs::OpenOptions::new()
            .write(true)
            .open(root.join(".kineti/live-ready"))
            .unwrap();
        let release = fs::File::open(root.join(".kineti/group-release")).unwrap();
        unsafe {
            assert_eq!(libc::getpgrp(), libc::getpid());
            let supervisor_group = libc::getpgid(libc::getppid());
            assert!(supervisor_group > 0 && supervisor_group != libc::getpid());
            assert_eq!(libc::setpgid(0, supervisor_group), 0);
            assert_eq!(libc::getpgrp(), supervisor_group);
            assert_eq!(libc::write(ready.as_raw_fd(), b"R".as_ptr().cast(), 1), 1);
            let mut event = libc::pollfd {
                fd: release.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // Independent lifetime if supervisor/regression test malfunctions.
            libc::poll(&mut event, 1, 5000);
        }
        std::process::exit(0);
    }
    #[test]
    fn real_group_loss_cleanup_fixture() {
        use std::os::unix::{ffi::OsStrExt, fs::OpenOptionsExt};
        let root = std::env::current_dir().unwrap();
        let Ok(mode) = fs::read_to_string(root.join(".kineti/live-fault-mode")) else {
            return;
        };
        if mode != "cleanup-unconfirmed" {
            return;
        }
        fs::write(root.join(".kineti/group-loss-mode"), b"fixture only").unwrap();
        for name in ["live-ready", "group-release"] {
            let name =
                std::ffi::CString::new(root.join(".kineti").join(name).as_os_str().as_bytes())
                    .unwrap();
            assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        }
        let ready = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(root.join(".kineti/live-ready"))
            .unwrap();
        let mut release = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(root.join(".kineti/group-release"))
            .unwrap();
        fault_proof::install(fault_proof::Plan {
            mode: mode.clone(),
            ready,
            pid: 0,
            descriptors: [-1; 2],
            injected: false,
            observations: vec![],
            poll_calls: 0,
            interrupted_polls: 0,
            poll_enter: None,
            cleanup_start: None,
            cleanup_pending_count: 0,
            cleanup_available: None,
        });
        let start = Instant::now();
        let result = execute(
            &root,
            &[
                std::env::current_exe()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned(),
                "--exact".into(),
                "receipt::supervision_tests::group_loss_leader_fixture".into(),
                "--nocapture".into(),
            ],
            1,
        );
        let elapsed = start.elapsed();
        let plan = fault_proof::take();
        let cleanup_elapsed = plan.cleanup_start.unwrap().elapsed();
        let mut status = 0;
        let pending = unsafe { libc::waitpid(plan.pid as i32, &mut status, libc::WNOHANG) } == 0;
        let closed = plan.descriptors.iter().all(|fd| unsafe { libc::fcntl(*fd, libc::F_GETFD) } == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EBADF));
        // The known direct child is still waitable: release it through the FIFO,
        // then reap this exact owned PID. Never signal the supervisor's group.
        release.write_all(b"Q").unwrap();
        let until = Instant::now() + Duration::from_secs(2);
        let mut reaped = false;
        while Instant::now() < until {
            let waited = unsafe { libc::waitpid(plan.pid as i32, &mut status, libc::WNOHANG) };
            if waited == plan.pid as i32 {
                reaped = true;
                break;
            }
            assert_eq!(waited, 0);
            std::thread::sleep(Duration::from_millis(5));
        }
        let error = result.unwrap_err();
        let observation = json!({"mode":mode,"error_code":error.0,"diagnostic":error.1,"elapsed_seconds":elapsed.as_secs_f64(),"cleanup_elapsed_seconds":cleanup_elapsed.as_secs_f64(),"cleanup_available_seconds":plan.cleanup_available.unwrap().as_secs_f64(),"pending_after_helper":pending,"independently_released_and_reaped":reaped,"owned_descriptors_closed":closed,"cleanup_pending_count":plan.cleanup_pending_count,"observations":plan.observations,"fault_origin":"actual child changed its own group; actual waitpid remained pending; no injected syscall result","scope":"not kernel refusal after successful SIGKILL; not escaped-process containment"});
        fs::write(
            root.join(".kineti/fault-observation.json"),
            observation.to_string(),
        )
        .unwrap();
        println!("group-loss-observation: {observation}");
        assert_eq!(error.0, 3);
        assert!(error.1.contains("cleanup unconfirmed") && error.1.contains(&plan.pid.to_string()));
        assert!(
            pending && reaped && closed && plan.cleanup_pending_count > 1,
            "{observation}"
        );
        assert!(
            elapsed >= Duration::from_millis(1900) && elapsed < Duration::from_millis(2400),
            "{observation}"
        );
        assert!(
            cleanup_elapsed + Duration::from_millis(10) >= plan.cleanup_available.unwrap()
                && cleanup_elapsed < Duration::from_millis(1300),
            "{observation}"
        );
        assert!(!plan.injected);
        assert!(plan
            .observations
            .iter()
            .any(|v| v["event"] == "live-child-ready"
                && v["observed_group"] == v["supervisor_group"]
                && v["observed_group"] != plan.pid));
        assert!(plan.observations.iter().any(|v| v["syscall"] == "killpg"
            && (v["errno"] == libc::ESRCH || v["errno"] == libc::EPERM)));
    }
    #[test]
    fn actual_group_loss_keeps_child_pending_for_full_cleanup_bound() {
        isolated_proof("cleanup-unconfirmed");
    }

    extern "C" fn child_signal_handler(_: i32) {}
    #[test]
    fn unsupported_sigchld_fixture() {
        let root = std::env::current_dir().unwrap();
        let Ok(mode) = fs::read_to_string(root.join(".kineti/sigchld-mode")) else {
            return;
        };
        let mut action = unsafe { std::mem::zeroed::<libc::sigaction>() };
        action.sa_sigaction = match mode.as_str() {
            "ignore" => libc::SIG_IGN,
            "handler" => child_signal_handler as *const () as usize,
            "no-wait" => libc::SIG_DFL,
            _ => panic!("unknown fixture mode"),
        };
        if mode == "no-wait" {
            action.sa_flags = libc::SA_NOCLDWAIT;
        }
        unsafe {
            libc::sigemptyset(&mut action.sa_mask);
            assert_eq!(
                libc::sigaction(libc::SIGCHLD, &action, std::ptr::null_mut()),
                0
            );
        }
        let error = execute(
            &root,
            &[
                "/bin/sh".into(),
                "-c".into(),
                "printf forbidden > .kineti/forbidden".into(),
            ],
            1,
        )
        .unwrap_err();
        assert_eq!(error.0, 3);
        assert!(error.1.contains("SIGCHLD"));
        assert!(!root.join(".kineti/forbidden").exists());
    }
    #[test]
    fn non_waitable_and_custom_sigchld_dispositions_deny_before_spawn() {
        for mode in ["ignore", "no-wait", "handler"] {
            let p = Fixture::new();
            fs::write(p.0.join(".kineti/sigchld-mode"), mode).unwrap();
            let output = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "receipt::supervision_tests::unsupported_sigchld_fixture",
                    "--nocapture",
                ])
                .current_dir(&p.0)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{mode}: {} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(!p.0.join(".kineti/forbidden").exists());
        }
    }
}

#[cfg(test)]
mod parser_and_signing_tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.kineti/runner-tests")
                .join(&store::nonce().unwrap()[..12]);
            fs::create_dir_all(root.join(".kineti")).unwrap();
            let root = root.canonicalize().unwrap();
            store::write(
                &root.join(".kineti/state.json"),
                &json!({"root_goal": "Receipt parser and signing test"}),
            )
            .unwrap();
            fs::write(root.join("test.txt"), "fixture").unwrap();
            Self(root)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn summary_parses_multi_suite_bun_and_cargo_with_strict_modes() {
        assert_eq!(
            summary("2 pass\n0 fail\n\n1 pass\n1 fail\n", "bun"),
            Some((3, 1, 0))
        );
        assert_eq!(
            summary("2 pass\n0 fail\n1 skip\n1 pass\n0 fail\n2 skip\n", "auto"),
            Some((3, 0, 3))
        );
        assert_eq!(summary("2 fail\n", "bun"), Some((0, 2, 0)));
        assert_eq!(summary("all good\n", "auto"), None);
        assert_eq!(summary("1 pass\n0 fail\n", "cargo"), None);
        assert_eq!(
            summary(
                "test result: ok. 2 passed; 0 failed; 1 ignored\ntest result: ok. 3 passed; 0 failed; 0 ignored\n",
                "cargo"
            ),
            Some((5, 0, 1))
        );
        assert_eq!(
            summary("test result: ok. 2 passed; 0 failed; 0 ignored\n", "bun"),
            None
        );
    }

    #[test]
    fn sign_object_and_verify_object_signature_round_trip() {
        let f = Fixture::new();
        let mut audit = json!({
            "schema": "kineti-pr-audit-v1",
            "verdict": "passed",
            "findings": [],
        });
        let sig = sign_object(&f.0, &mut audit).unwrap();
        assert!(!sig.is_empty());
        assert_eq!(audit["encoding"], "kineti-canonical-v2");
        let trusted = fs::read_to_string(f.0.join(".kineti/evidence-issuer.pem")).unwrap();
        assert!(verify_object_signature(&audit, &trusted));

        let mut tampered = audit.clone();
        tampered["verdict"] = json!("blocked");
        assert!(!verify_object_signature(&tampered, &trusted));
        assert!(!verify_object_signature(&audit, "untrusted-key"));
    }

    #[test]
    fn resolve_uses_project_root_for_relative_executable() {
        let f = Fixture::new();
        let script_dir = f.0.join("scripts");
        fs::create_dir_all(&script_dir).unwrap();
        let script = script_dir.join("check.sh");
        fs::write(&script, "#!/bin/sh\nexit 0\n").unwrap();
        let resolved = resolve(&f.0, "./scripts/check.sh").unwrap();
        assert_eq!(resolved, script.canonicalize().unwrap());
    }

    #[test]
    fn two_tier_parsers_and_constant_size_receipts_enforce_policy() {
        let f = Fixture::new();

        // Invalid parser is rejected before execution with exit 2.
        let err = run(
            &f.0,
            Run {
                label: "bad-parser".into(),
                argv: vec!["/bin/sh".into(), "-c".into(), "exit 0".into()],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "nope".into(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 2);

        // Multi-suite bun failure in suite 2 must fail even with exit 0.
        let err = run(
            &f.0,
            Run {
                label: "bun-multi".into(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf '2 pass\\n0 fail\\n1 pass\\n1 fail\\n'".into(),
                ],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "bun".into(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);

        // Ignored rust tests fail by default, pass with allow_ignored, and respect min_passed.
        let err = run(
            &f.0,
            Run {
                label: "cargo-ign".into(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf 'test result: ok. 2 passed; 0 failed; 1 ignored\\n'".into(),
                ],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "cargo".into(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);

        let err = run(
            &f.0,
            Run {
                label: "cargo-min".into(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf 'test result: ok. 2 passed; 0 failed; 1 ignored\\n'".into(),
                ],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "cargo".into(),
                allow_ignored: true,
                min_passed: Some(3),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);

        let r = run(
            &f.0,
            Run {
                label: "cargo-ok".into(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf 'test result: ok. 2 passed; 0 failed; 1 ignored\\n'".into(),
                ],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "cargo".into(),
                allow_ignored: true,
                min_passed: Some(2),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(r["manifest"].get("inputs").is_none());
        assert!(r["manifest"]["inventory_sha256"].is_string());
        assert!(r["manifest"]["inventory_count"].is_u64());
        assert_eq!(r["verification"]["parser"], "kineti-native-summary-v1");
        assert_eq!(r["verification"]["count_observed"], true);
        assert!(check(&f.0, "cargo-ok", 60, None).is_ok());

        // Exit-code mode succeeds on exit 0 with no test counts and records count_observed: false.
        let r = run(
            &f.0,
            Run {
                label: "pytest-exit".into(),
                argv: vec!["/bin/sh".into(), "-c".into(), "printf 'ok\\n'".into()],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "exit-code".into(),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(r["verification"]["parser"], "kineti-exit-code-v1");
        assert_eq!(r["verification"]["count_observed"], false);
        assert_eq!(r["verification"]["passed"], Value::Null);
        assert!(check(&f.0, "pytest-exit", 60, None).is_ok());
    }

    #[test]
    fn summary_parses_junit_xml_suites_and_rejects_malformed() {
        let single = r#"<testsuite name="pytest" tests="5" errors="0" failures="0" skipped="0"></testsuite>"#;
        assert_eq!(summary(single, "junit"), Some((5, 0, 0)));
        assert_eq!(summary(single, "auto"), Some((5, 0, 0)));

        let multi = r#"
<testsuites tests="10" failures="2" errors="1" skipped="1">
  <testsuite name="a" tests="4" failures="0" errors="0" skipped="0"/>
  <testsuite name="b" tests="6" failures="1" errors="1" skipped="1"/>
</testsuites>"#;
        assert_eq!(summary(multi, "junit"), Some((7, 2, 1)));

        let root_only = r#"<testsuites tests="3" failures="0" errors="0" disabled="1"/>"#;
        assert_eq!(summary(root_only, "junit"), Some((2, 0, 1)));

        // Malformed: failures + skipped > tests
        let invalid = r#"<testsuite tests="1" failures="2" errors="0" skipped="0"/>"#;
        assert_eq!(summary(invalid, "junit"), None);
        assert_eq!(summary("plain text", "junit"), None);

        let f = Fixture::new();
        let r = run(
            &f.0,
            Run {
                label: "junit-ok".into(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf '<testsuite name=\"py\" tests=\"3\" failures=\"0\" errors=\"0\" skipped=\"0\"/>\\n'".into(),
                ],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "junit".into(),
                min_passed: Some(3),
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(r["verification"]["parser"], "kineti-native-summary-v1");
        assert_eq!(r["verification"]["count_observed"], true);
        assert_eq!(r["verification"]["passed"], 3);
        assert!(check(&f.0, "junit-ok", 60, None).is_ok());

        // Zero tests in JUnit XML fails min_passed >= 1.
        let err = run(
            &f.0,
            Run {
                label: "junit-zero".into(),
                argv: vec![
                    "/bin/sh".into(),
                    "-c".into(),
                    "printf '<testsuite tests=\"0\" failures=\"0\" errors=\"0\" skipped=\"0\"/>\\n'".into(),
                ],
                tests: vec!["test.txt".into()],
                timeout: 5,
                kind: "test".into(),
                parser: "junit".into(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);
    }
}
