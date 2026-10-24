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
    if !pubpath.exists() || !private.exists() {
        return store::fail(
            3,
            "incomplete issuer identity; preserve originals and use reviewed recovery",
        );
    }
    let raw = fs::read_to_string(private)?;
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
    let public = fs::read_to_string(pubpath)?;
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
/// Verify signature without treating parsed output as independent observation.
pub fn signature(v: &Value, trusted: &str) -> bool {
    if v["receipt_version"] != 3 {
        return verify_legacy(v, trusted);
    }
    let check = || -> Option<()> {
        if v["issuer_key"].as_str()? != trusted || v["encoding"] != "kineti-canonical-v2" {
            return None;
        }
        let text = v["signature"].as_str()?;
        let bytes = STANDARD.decode(text).ok()?;
        if STANDARD.encode(&bytes) != text {
            return None;
        }
        public_key(trusted)
            .ok()?
            .verify_strict(
                unsigned(v).ok()?.as_bytes(),
                &Signature::from_slice(&bytes).ok()?,
            )
            .ok()
    };
    check().is_some()
}
fn resolve(command: &str) -> Result<PathBuf> {
    if command.contains('/') {
        return Ok(Path::new(command).canonicalize()?);
    }
    for dir in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        let p = dir.join(command);
        if p.is_file() {
            return Ok(p.canonicalize()?);
        }
    }
    store::fail(2, format!("executable missing: {command}"))
}
fn runtime(argv: &[String]) -> Result<Value> {
    let p = resolve(&argv[0])?;
    Ok(
        json!({"kineti_version":"0.6.0","gate_sha256":digest(fs::read(std::env::current_exe()?)?),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"executable":p,"executable_sha256":digest(fs::read(&p)?)}),
    )
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
            es.sort_by_key(|e| e.file_name());
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
            out.push(json!({"path":p.strip_prefix(root).map_err(|e|Error(2,e.to_string()))?,"sha256":digest(fs::read(resolved)?)}));
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
fn summary(output: &str) -> Option<(u64, u64, u64)> {
    let rust = regex::Regex::new(
        r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",
    )
    .unwrap();
    if rust.is_match(output) {
        let mut sum = (0, 0, 0);
        for c in rust.captures_iter(output) {
            sum.0 += c[1].parse::<u64>().ok()?;
            sum.1 += c[2].parse::<u64>().ok()?;
            sum.2 += c[3].parse::<u64>().ok()?;
        }
        return Some(sum);
    }
    let pass = regex::Regex::new(r"(?m)^\s*(\d+) pass\s*$").unwrap();
    let fail = regex::Regex::new(r"(?m)^\s*(\d+) fail\s*$").unwrap();
    let skip = regex::Regex::new(r"(?m)^\s*(\d+) skip\s*$").unwrap();
    Some((
        pass.captures(output)?[1].parse().ok()?,
        fail.captures(output)?[1].parse().ok()?,
        skip.captures(output)
            .map(|c| c[1].parse::<u64>())
            .transpose()
            .ok()?
            .unwrap_or(0),
    ))
}
/// Execute without a shell. Timeout kills the process group; output is bounded but drained.
pub fn execute(root: &Path, argv: &[String], timeout: u64) -> Result<(Option<i32>, String, bool)> {
    use std::os::unix::process::CommandExt;
    let child = Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(root)
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();
    // A spawn failure is definite: the requested executable did not start.
    let mut child = match child {
        Ok(child) => child,
        Err(error) => return Ok((Some(127), format!("command did not start: {error}"), false)),
    };
    fn drain(mut f: impl Read) -> (Vec<u8>, bool) {
        let mut bytes = vec![];
        let mut chunk = [0; 8192];
        let mut truncated = false;
        loop {
            match f.read(&mut chunk) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let keep = n.min(8_000_000usize.saturating_sub(bytes.len()));
                    bytes.extend_from_slice(&chunk[..keep]);
                    truncated |= keep < n;
                }
            }
        }
        (bytes, truncated)
    }
    let group_id = child.id();
    let out = child.stdout.take().unwrap();
    let err = child.stderr.take().unwrap();
    let a = std::thread::spawn(move || drain(out));
    let b = std::thread::spawn(move || drain(err));
    let deadline = Instant::now() + Duration::from_secs(timeout);
    let status;
    loop {
        if let Some(s) = child.try_wait()? {
            status = Some(s);
            break;
        }
        if Instant::now() >= deadline {
            unsafe {
                libc::kill(-(child.id() as i32), libc::SIGKILL);
            };
            child.wait()?;
            status = None;
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        libc::kill(-(group_id as i32), libc::SIGKILL);
    };
    let (a, ta) = a
        .join()
        .map_err(|_| Error(1, "output reader failed".into()))?;
    let (b, tb) = b
        .join()
        .map_err(|_| Error(1, "output reader failed".into()))?;
    Ok((
        status.and_then(|s| s.code()),
        format!(
            "{}\n{}",
            String::from_utf8_lossy(&a),
            String::from_utf8_lossy(&b)
        ),
        ta || tb,
    ))
}
/// Record an observed command and bind it to exact declared/native inputs.
pub fn run(root: &Path, opts: Run) -> Result<Value> {
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
    let tests = files(root, &opts.tests)?;
    if opts.kind == "test" && tests.as_array().unwrap().is_empty() {
        return store::fail(2, "test inventory is empty");
    }
    let artifacts = files(root, &opts.artifacts)?;
    let before = store::inventory(root)?;
    let runtime_before = runtime(&opts.argv)?;
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
    let parsed = summary(&output);
    let stable = store::inventory(root)? == before
        && runtime(&opts.argv)? == runtime_before
        && environment
            .as_object()
            .unwrap()
            .iter()
            .all(|(k, v)| env_digest(k).ok().as_deref() == v.as_str());
    let observed = opts.kind != "test" || parsed.is_some_and(|(p, f, s)| p > 0 && f == 0 && s == 0);
    let passed = exit == Some(0) && stable && observed && !truncated;
    let mut receipt = json!({"receipt_version":3,"encoding":"kineti-canonical-v2","run_id":store::nonce()?,"at":store::now(),"label":opts.label,"cmd":opts.argv.join(" "),"exit_code":exit,"result_kind":opts.kind,"outcome":if passed{"passed"}else{"failed"},"fingerprint":store::inventory_digest(&before)?,"manifest":{"command":opts.argv,"inputs":before,"dependencies":files(root,&["Cargo.lock","core-native/Cargo.lock","package-lock.json","bun.lock"].iter().filter(|p|root.join(p).is_file()).map(|p|p.to_string()).collect::<Vec<_>>())?,"environment":environment,"runtime":runtime_before,"artifacts":artifacts,"session_ref":std::env::var("KINETI_SESSION_ID").ok()},"verification":{"parser":"kineti-native-summary-v1","status":if observed{"valid"}else{"unsupported"},"passed":parsed.map(|v|v.0),"failed":parsed.map(|v|v.1),"skipped":parsed.map(|v|v.2),"inventory":tests,"output_sha256":digest(output.as_bytes()),"truncated":truncated},"mutated_during_run":!stable,"issuer_key":public});
    receipt["signature"] =
        json!(STANDARD.encode(key.sign(unsigned(&receipt)?.as_bytes()).to_bytes()));
    let _lock = Lock::take(&root.join(".kineti/evidence.lock"))?;
    store::lines(&root.join(".kineti/evidence.jsonl"))?;
    store::append(&root.join(".kineti/evidence.jsonl"), &receipt)?;
    if !passed {
        return Err(Error(1,format!("command evidence recorded but failed: exit={exit:?}, stable={stable}, output_supported={observed}")));
    }
    Ok(receipt)
}
/// Signature and freshness are separate. Legacy evidence remains readable without upgraded authority.
pub fn check(root: &Path, label: &str, max_age: u64, expected: Option<&Value>) -> Result<Value> {
    if max_age == 0 || max_age > 5_256_000 {
        return store::fail(2, "receipt age must be 1..5256000 minutes");
    }
    store::state(root)?;
    let records = store::lines(&root.join(".kineti/evidence.jsonl"))?;
    let r = records
        .iter()
        .rev()
        .find(|v| v["label"] == label)
        .ok_or_else(|| Error(5, format!("missing evidence: {label}")))?;
    let trusted = fs::read_to_string(root.join(".kineti/evidence-issuer.pem"))
        .map_err(|_| Error(7, "trusted issuer missing".into()))?;
    if !signature(r, &trusted) {
        return store::fail(7, "altered or untrusted receipt");
    }
    if r["receipt_version"] != 3 {
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
    if r["result_kind"] != "test"
        || r["outcome"] != "passed"
        || r["exit_code"] != 0
        || r["mutated_during_run"] != false
        || r["verification"]["status"] != "valid"
        || r["verification"]["passed"].as_u64().unwrap_or(0) == 0
        || r["verification"]["failed"] != 0
        || r["verification"]["skipped"] != 0
        || r["verification"]["inventory"]
            .as_array()
            .is_none_or(|a| a.is_empty())
    {
        return store::fail(7, "receipt does not record supported passing test evidence");
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
    if argv.is_empty() || expected.is_some_and(|e| e != &m["command"]) {
        return store::fail(7, "command does not match required exact argv");
    }
    if store::inventory(root)? != m["inputs"] || runtime(&argv)? != m["runtime"] {
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
            let p =
                store::input(root, rel).map_err(|_| Error(4, "declared file missing".into()))?;
            if digest(fs::read(p)?) != item["sha256"].as_str().unwrap_or("") {
                return store::fail(4, "declared file changed");
            }
        }
    }
    Ok(
        json!({"status":"fresh","signature_valid":true,"independent_test_execution":false,"receipt":r}),
    )
}
