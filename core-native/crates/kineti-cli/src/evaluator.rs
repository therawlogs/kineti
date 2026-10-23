//! Linux fake-publish evaluator. Only service-owned assertions can authorize export.
use ed25519_dalek::{Signer, SigningKey};
use rand_core::OsRng;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::{
    fs::PermissionsExt,
    net::{UnixListener, UnixStream},
};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const MAX_REQUEST: u64 = 160_000;
const MAX_ARTIFACT: usize = 65_536;
const HELPER: &str = "/usr/local/libexec/kineti-candidate";
const CASES: [(&str, &str, &str); 4] = [
    ("2", "3", "5"),
    ("-7", "2", "-5"),
    ("0", "0", "0"),
    ("123", "456", "579"),
];
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn digest(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}
fn runtime_digest() -> Result<String> {
    let mut parts = Vec::new();
    for name in [
        "/bin/sh",
        "/bin/dash",
        "/usr/bin/sudo",
        "/usr/bin/setpriv",
        "/usr/bin/timeout",
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib64/ld-linux-x86-64.so.2",
    ] {
        parts.push(json!({"path":name,"sha256":digest(&fs::read(name)?)}));
    }
    Ok(digest(&serde_json::to_vec(&parts)?))
}
fn decode(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid hex".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| Ok(u8::from_str_radix(&s[i..i + 2], 16)?))
        .collect()
}
fn field<'a>(v: &'a Value, name: &str) -> Result<&'a str> {
    v[name]
        .as_str()
        .ok_or_else(|| format!("missing {name}").into())
}
fn uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit() && !c.is_ascii_uppercase()
            }
        })
}
fn sync_json(path: &Path, value: &Value) -> Result<()> {
    let tmp = path.with_extension("new");
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
    f.write_all(&serde_json::to_vec(value)?)?;
    f.sync_all()?;
    fs::rename(tmp, path)?;
    File::open(path.parent().ok_or("missing parent")?)?.sync_all()?;
    Ok(())
}
use std::os::unix::fs::OpenOptionsExt;
fn signed(mut decision: Value, key: &SigningKey) -> Result<Value> {
    decision["issuer"] = json!(hex(key.verifying_key().as_bytes()));
    let signature = key.sign(&serde_json::to_vec(&decision)?);
    decision["signature"] = json!(hex(&signature.to_bytes()));
    Ok(decision)
}
fn policy(state: &Path, req: &Value, key: &SigningKey) -> Result<Value> {
    let p: Value = serde_json::from_slice(&fs::read(state.join("policy.json"))?)?;
    if p["version"] != 1 || p["inventory"] != "sum-v1" {
        return Err("unsupported owner policy".into());
    }
    for (list, value) in [
        (
            "revoked_operations",
            field(req, "operation_id")?.to_string(),
        ),
        ("revoked_runs", field(req, "run_id")?.to_string()),
        ("revoked_issuers", hex(key.verifying_key().as_bytes())),
    ] {
        let entries = p[list].as_array().ok_or("malformed revocation policy")?;
        if !entries.iter().all(Value::is_string) {
            return Err("malformed revocation list".into());
        }
        if entries.iter().any(|v| v.as_str() == Some(value.as_str())) {
            return Err("authorization revoked".into());
        }
    }
    Ok(p)
}
fn check_owner(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || metadata.permissions().mode() & 0o077 != 0 {
        return Err("service state must be private and not a symlink".into());
    }
    Ok(())
}
fn handle(state: &Path, request: &Value) -> Result<Value> {
    check_owner(state)?;
    let obj = request.as_object().ok_or("request must be an object")?;
    let fields = [
        "operation_id",
        "run_id",
        "session_id",
        "artifact_hex",
        "artifact_sha256",
        "expires_at",
    ];
    if obj.len() != fields.len() || !obj.keys().all(|k| fields.contains(&k.as_str())) {
        return Err("unknown or missing request fields".into());
    }
    for name in ["operation_id", "run_id", "session_id"] {
        if !uuid(field(request, name)?) {
            return Err("invalid operation/execution/session identity".into());
        }
    }
    let id = field(request, "operation_id")?;
    let bytes = decode(field(request, "artifact_hex")?)?;
    if bytes.is_empty()
        || bytes.len() > MAX_ARTIFACT
        || digest(&bytes) != field(request, "artifact_sha256")?
    {
        return Err("artifact size or digest mismatch".into());
    }
    let request_digest = digest(&serde_json::to_vec(request)?);
    let expiry = request["expires_at"].as_u64().ok_or("invalid expiry")?;
    let keybytes: [u8; 32] = fs::read(state.join("signing.key"))?
        .try_into()
        .map_err(|_| "invalid service key")?;
    let key = SigningKey::from_bytes(&keybytes);
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(state.join("operation.lock"))?;
    lock.lock()?;
    let journal = state.join("operations").join(format!("{id}.json"));
    let destination = state.join("exports").join(format!("{id}.artifact"));
    if journal.exists() {
        let mut old: Value = serde_json::from_slice(&fs::read(&journal)?)?;
        if old["request_digest"] != request_digest {
            return Err("operation identity substitution".into());
        }
        // Reconcile only. An interrupted operation is never silently re-executed.
        if old["status"] == "pending" {
            if destination.exists()
                && digest(&fs::read(&destination)?) == field(request, "artifact_sha256")?
            {
                let previous = old.clone();
                old["previous"] = previous;
                old["status"] = json!("applied");
                old.as_object_mut()
                    .ok_or("invalid journal")?
                    .remove("signature");
                old = signed(old, &key)?;
                sync_json(&journal, &old)?;
            } else {
                return Ok(
                    json!({"status":"unknown","operation_id":id,"reason":"interrupted export; owner reconciliation required; do not launch another operation"}),
                );
            }
        }
        return Ok(old);
    }
    // Do not admit a new operation while an interrupted export remains unreconciled.
    for entry in fs::read_dir(state.join("operations"))? {
        let v: Value = serde_json::from_slice(&fs::read(entry?.path())?)?;
        if v["status"] == "pending" {
            return Err("unreconciled operation blocks further exports".into());
        }
    }
    if expiry <= now() || expiry > now() + 14_400 {
        return Err("authorization expired or exceeds maximum lifetime".into());
    }
    let p = policy(state, request, &key)?;
    let policy_digest = digest(&serde_json::to_vec(&p)?);
    let runtime = runtime_digest()?;
    let helper = digest(&fs::read(HELPER)?);
    let supervisor = digest(&fs::read(std::env::current_exe()?)?);
    // Retain the original input under the trusted account before running any candidate.
    // An interrupted export can then be investigated without candidate-supplied bytes.
    let retained_path = state.join("artifacts").join(format!("{id}.artifact"));
    let mut retained = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&retained_path)?;
    retained.write_all(&bytes)?;
    retained.sync_all()?;
    File::open(state.join("artifacts"))?.sync_all()?;
    let job = Path::new("/var/lib/kineti-jobs").join(format!("{id}.sh"));
    let mut artifact = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o444)
        .open(&job)?;
    artifact.write_all(&bytes)?;
    artifact.sync_all()?;
    drop(artifact);
    let mut checks = Vec::new();
    for (index, (a, b, expected)) in CASES.iter().enumerate() {
        let output_path = state.join(format!("{id}-{index}.output"));
        let output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&output_path)?;
        let status = Command::new("sudo")
            .args(["-n", HELPER, job.to_str().ok_or("invalid job path")?, a, b])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::from(output))
            .stderr(Stdio::null())
            .status()?;
        let observed = fs::read(&output_path)?;
        fs::remove_file(output_path)?;
        let passed = status.success()
            && observed.len() < 4096
            && String::from_utf8_lossy(&observed).trim() == *expected;
        checks.push(json!({"id":format!("sum-{index}"),"status":if passed {"pass"} else {"fail"},"exit_code":status.code(),"command":["/bin/sh",job.to_string_lossy(),a,b],"observed_sha256":digest(&observed)}));
    }
    fs::remove_file(&job)?;
    let accepted = checks.iter().all(|c| c["status"] == "pass");
    let mut result = json!({"schema_version":1,"operation_id":id,"run_id":request["run_id"],"session_id":request["session_id"],"request_digest":request_digest,"artifact_sha256":request["artifact_sha256"],"command":["/bin/sh",job.to_string_lossy()],"inventory":"sum-v1","runtime_sha256":runtime,"helper_sha256":helper,"supervisor_sha256":supervisor,"policy_sha256":policy_digest,"destination":format!("exports/{id}.artifact"),"expires_at":expiry,"checks":checks,"status":if accepted {"pending"} else {"denied"}});
    result = signed(result, &key)?;
    sync_json(&journal, &result)?;
    if !accepted {
        return Ok(result);
    }
    if std::env::var("KINETI_EVALUATOR_FAILPOINT").as_deref() == Ok("after-intent") {
        std::process::exit(86);
    }
    if expiry <= now()
        || policy_digest != digest(&serde_json::to_vec(&policy(state, request, &key)?)?)
        || runtime_digest()? != runtime
        || digest(&fs::read(HELPER)?) != helper
    {
        return Err("authorization inputs changed before export".into());
    }
    let stage = state.join("exports").join(format!(".{id}.stage"));
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&stage)?;
    f.write_all(&bytes)?;
    f.sync_all()?;
    drop(f);
    if destination.exists() {
        return Err("destination already exists without a matching operation".into());
    }
    fs::rename(stage, &destination)?;
    File::open(state.join("exports"))?.sync_all()?;
    if std::env::var("KINETI_EVALUATOR_FAILPOINT").as_deref() == Ok("after-rename") {
        std::process::exit(86);
    }
    let previous = result.clone();
    result["previous"] = previous;
    result["status"] = json!("applied");
    result
        .as_object_mut()
        .ok_or("invalid result")?
        .remove("signature");
    result = signed(result, &key)?;
    sync_json(&journal, &result)?;
    Ok(result)
}

pub fn cli(args: &[String]) -> Result<()> {
    if !cfg!(target_arch = "x86_64") {
        return Err("trusted evaluator runtime manifest supports x86-64 Linux only".into());
    }
    match args.first().map(String::as_str) {
        Some("init") if args.len() == 2 => {
            let state = Path::new(&args[1]);
            if state.exists() { return Err("refusing to replace existing evaluator state".into()); }
            fs::create_dir(state)?; fs::set_permissions(state,fs::Permissions::from_mode(0o700))?;
            for dir in ["operations","exports","artifacts"] { fs::create_dir(state.join(dir))?; }
            let key = SigningKey::generate(&mut OsRng);
            let mut f = OpenOptions::new().write(true).create_new(true).mode(0o600).open(state.join("signing.key"))?;
            f.write_all(&key.to_bytes())?; f.sync_all()?;
            sync_json(&state.join("policy.json"),&json!({"version":1,"inventory":"sum-v1","revoked_operations":[],"revoked_runs":[],"revoked_issuers":[]}))?;
            println!("{}",json!({"status":"initialized","issuer":hex(key.verifying_key().as_bytes())}));
        }
        Some("revoke") if args.len() == 4 => {
            let state = Path::new(&args[1]); check_owner(state)?;
            let list = match args[2].as_str() { "operation" => "revoked_operations", "run" => "revoked_runs", "issuer" => "revoked_issuers", _ => return Err("revoke kind must be operation, run or issuer".into()) };
            if list != "revoked_issuers" && !uuid(&args[3]) { return Err("invalid identity".into()); }
            if list == "revoked_issuers" && decode(&args[3])?.len() != 32 { return Err("invalid issuer".into()); }
            let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(state.join("operation.lock"))?; lock.lock()?;
            let mut p: Value = serde_json::from_slice(&fs::read(state.join("policy.json"))?)?;
            let entries = p[list].as_array_mut().ok_or("invalid revocation policy")?;
            if !entries.contains(&json!(args[3])) { entries.push(json!(args[3])); }
            sync_json(&state.join("policy.json"), &p)?;
            println!("{}", json!({"status":"revoked","kind":args[2],"identity":args[3]}));
        }
        Some("reconcile") if args.len() == 3 => {
            let state = Path::new(&args[1]); check_owner(state)?;
            if !uuid(&args[2]) { return Err("invalid operation identity".into()); }
            let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false).mode(0o600).open(state.join("operation.lock"))?; lock.lock()?;
            let journal = state.join("operations").join(format!("{}.json",args[2]));
            let mut record: Value = serde_json::from_slice(&fs::read(&journal)?)?;
            if record["status"] != "pending" { println!("{record}"); return Ok(()); }
            let destination = state.join("exports").join(format!("{}.artifact",args[2]));
            let previous = record.clone();
            if destination.exists() {
                if digest(&fs::read(&destination)?) != field(&record,"artifact_sha256")? { return Err("destination differs; manual investigation required".into()); }
                record["status"] = json!("applied");
            } else {
                let stage = state.join("exports").join(format!(".{}.stage",args[2]));
                if stage.exists() { fs::remove_file(stage)?; File::open(state.join("exports"))?.sync_all()?; }
                record["status"] = json!("denied"); record["recovery"] = json!("owner confirmed no export; original authorization consumed");
            }
            record["previous"] = previous;
            record.as_object_mut().ok_or("invalid journal")?.remove("signature");
            let bytes: [u8;32] = fs::read(state.join("signing.key"))?.try_into().map_err(|_| "invalid service key")?;
            record = signed(record,&SigningKey::from_bytes(&bytes))?;sync_json(&journal,&record)?;println!("{record}");
        }
        Some("serve") if args.len() == 3 => {
            let state = Path::new(&args[1]); check_owner(state)?;
            let listener = UnixListener::bind(&args[2])?;
            fs::set_permissions(&args[2],fs::Permissions::from_mode(0o666))?;
            for stream in listener.incoming() {
                let mut stream = stream?; stream.set_read_timeout(Some(Duration::from_secs(2)))?;
                let mut line = String::new();
                let response = (|| -> Result<Value> {
                    let n = BufReader::new((&stream).take(MAX_REQUEST+1)).read_line(&mut line)?;
                    if n as u64 > MAX_REQUEST || !line.ends_with('\n') { return Err("oversize or truncated request".into()); }
                    handle(state,&serde_json::from_str::<Value>(&line)?)
                })().unwrap_or_else(|e| json!({"status":"denied","reason":e.to_string()}));
                // A disconnected client must not kill the service after a durable effect.
                let _ = writeln!(stream,"{response}");
            }
        }
        Some("submit") if args.len() == 3 => {
            let request = fs::read(&args[2])?;
            if request.len() as u64 > MAX_REQUEST { return Err("oversize request".into()); }
            let value: Value = serde_json::from_slice(&request)?;
            let mut stream = UnixStream::connect(&args[1])?; stream.set_read_timeout(Some(Duration::from_secs(30)))?;
            writeln!(stream,"{value}")?;
            let mut response = String::new(); BufReader::new(stream).take(MAX_REQUEST).read_line(&mut response)?;
            let result: Value = serde_json::from_str(&response)?; println!("{result}");
            if result["status"] != "applied" { std::process::exit(1); }
        }
        _ => return Err("usage: kineti-cli evaluator init STATE | serve STATE SOCKET | submit SOCKET REQUEST.json | revoke STATE KIND ID | reconcile STATE OPERATION_ID".into()),
    }
    Ok(())
}
