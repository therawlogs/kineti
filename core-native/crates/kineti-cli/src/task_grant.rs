//! Strict, reviewed task requests. Parsing a project file never grants authority.
use crate::{
    compat::digest,
    store::{self, Error, Result},
};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    path::{Component, Path, PathBuf},
};

/// One declared verification command and its conservative inputs.
#[derive(Clone, Debug)]
pub struct Check {
    /// Human-readable evidence label.
    pub label: String,
    /// Exact argv, never reconstructed from a command string.
    pub argv: Vec<String>,
    /// Test evidence requires observed positive passing counts; checks are exit-only.
    pub kind: String,
    /// Project-relative inputs; dot selects the conservative full work inventory.
    pub inputs: Vec<String>,
}
/// Explicit, bounded model repair authority; never a credential value.
#[derive(Clone, Debug)]
pub struct ModelRepair {
    /// Exact approved model, without provider fallback.
    pub model: String,
    /// Owner-reviewed synthetic/project repair instructions.
    pub prompt: String,
    /// Persistent aggregate turn ceiling, not a per-resume allowance.
    pub max_turns: u64,
    /// Digest of the official matching sibling code-mode helper.
    pub helper_sha256: String,
}
/// Validated request; only a separately protected owner approval can activate it.
#[derive(Clone, Debug)]
pub struct Grant {
    /// Canonical JSON request, without any credential values.
    pub request: Value,
    /// Unique task identifier.
    pub id: String,
    /// Host/session binding.
    pub session: String,
    /// Original canonical project; the service never writes to it.
    pub project: PathBuf,
    /// Locked project identity.
    pub identity: String,
    /// Separate unprivileged task account.
    pub uid: u32,
    /// Task account name, checked against the OS by the owner.
    pub user: String,
    /// Pinned, independently owned app-server executable.
    pub host: PathBuf,
    /// App-server executable digest.
    pub host_hash: String,
    /// Checks approved once by the owner.
    pub checks: Vec<Check>,
    /// Approved repair program; no implicit discovery/trust.
    pub repair: Vec<String>,
    /// Optional explicitly approved subscription repair mode.
    pub model_repair: Option<ModelRepair>,
    /// Explicit snapshot and host read/write exclusions.
    pub excludes: Vec<String>,
    /// Public system runtime inputs, hashed before reuse.
    pub runtime: Vec<String>,
    /// Absolute expiry in Unix seconds.
    pub expires: i64,
    /// Whole-task elapsed-time limit, never reset on resume.
    pub wall: u64,
    /// Per-command bound.
    pub command_seconds: u64,
    /// Aggregate repair quota.
    pub repairs: u64,
    /// Aggregate verification-attempt quota.
    pub attempts: u64,
    /// Initial snapshot byte limit.
    pub snapshot_bytes: u64,
    /// Kernel task memory bound.
    pub memory: u64,
    /// Kernel process-count bound.
    pub pids: u64,
    /// Preauthorized known local rollback only.
    pub rollback: bool,
}
/// Reject unknown fields rather than silently accepting unenforced restrictions.
pub(crate) fn keys(v: &Value, allowed: &[&str]) -> Result<()> {
    let o = v
        .as_object()
        .ok_or_else(|| Error(2, "expected JSON object".into()))?;
    if let Some(k) = o.keys().find(|k| !allowed.contains(&k.as_str())) {
        return store::fail(2, format!("unsupported task field: {k}"));
    }
    Ok(())
}
fn text(v: &Value, k: &str) -> Result<String> {
    v[k].as_str()
        .filter(|s| !s.is_empty() && s.len() <= 4096 && !s.contains('\0'))
        .map(String::from)
        .ok_or_else(|| Error(2, format!("required task string: {k}")))
}
fn number(v: &Value, k: &str, lo: u64, hi: u64) -> Result<u64> {
    v[k].as_u64()
        .filter(|n| (lo..=hi).contains(n))
        .ok_or_else(|| Error(2, format!("task limit {k} must be {lo}..{hi}")))
}
fn strings(v: &Value, k: &str) -> Result<Vec<String>> {
    v[k].as_array()
        .filter(|a| a.len() <= 128)
        .ok_or_else(|| Error(2, format!("required task array: {k}")))?
        .iter()
        .map(|x| {
            x.as_str()
                .filter(|s| s.len() <= 4096 && !s.contains('\0'))
                .map(String::from)
                .ok_or_else(|| Error(2, format!("invalid task array: {k}")))
        })
        .collect()
}
/// A bounded ASCII identifier, not an approval token.
pub fn id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 48
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
    {
        return store::fail(
            2,
            "task identifiers require 1..48 lowercase ASCII letters, digits or hyphens",
        );
    }
    Ok(())
}
/// Only relative, unambiguous paths; callers must also reject symlink parents.
pub fn relative(value: &str, dot: bool) -> Result<()> {
    if dot && value == "." {
        return Ok(());
    }
    if value.is_empty()
        || value.len() > 4096
        || value.contains('\0')
        || Path::new(value)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || value.split('/').any(|c| c.is_empty() || c == ".")
    {
        return store::fail(2,"task paths must be normal relative paths; no traversal, absolute paths or empty components");
    }
    Ok(())
}
fn argv(v: &Value, k: &str) -> Result<Vec<String>> {
    let a = strings(v, k)?;
    if a.is_empty() || a[0].is_empty() {
        return store::fail(2, "task argv must name an executable");
    }
    // Bounded hygiene, not universal secret detection. Credential handles are unsupported.
    for arg in &a {
        let s = arg.to_ascii_lowercase();
        if [
            "--password",
            "--token",
            "--api-key",
            "authorization:",
            "bearer ",
            ".pem.private",
        ]
        .iter()
        .any(|p| s.contains(p))
        {
            return store::fail(2,"secret-bearing invocation unsupported; use a separately approved credential broker");
        }
    }
    let program = Path::new(&a[0])
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("");
    if [
        "git", "cargo", "npm", "gh", "wrangler", "vercel", "fly", "kubectl",
    ]
    .contains(&program)
        && a.iter()
            .skip(1)
            .any(|s| ["push", "merge", "publish", "deploy", "release"].contains(&s.as_str()))
    {
        return store::fail(2,"release actions cannot be included in a task grant; separate exact approval is required");
    }
    Ok(a)
}
/// Bind the canonical project to its existing locked goal, without changing it.
pub fn project_identity(project: &Path) -> Result<String> {
    if project.canonicalize()? != project || !project.is_dir() {
        return store::fail(2, "project root must be an existing canonical directory");
    }
    let state = store::state(project)?;
    let goal = state["root_goal"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error(3, "locked goal unavailable".into()))?;
    let locked = state["root_goal_locked_at"]
        .as_str()
        .ok_or_else(|| Error(3, "locked goal identity unavailable".into()))?;
    Ok(digest(format!("{}\0{goal}\0{locked}", project.display())))
}
impl Grant {
    /// Parse an informational request with explicit capability restrictions.
    pub fn parse(v: Value) -> Result<Self> {
        keys(
            &v,
            &[
                "schema",
                "id",
                "session",
                "project_root",
                "project_identity",
                "purpose",
                "task_uid",
                "task_user",
                "host",
                "checks",
                "repair_argv",
                "model_repair",
                "exclusions",
                "runtime_inputs",
                "limits",
                "network_destinations",
                "credential_handles",
                "resource_class",
                "automatic_rollback",
            ],
        )?;
        if v["schema"] != "kineti-task-request-v1" {
            return store::fail(2, "unsupported task request schema");
        }
        let task_id = text(&v, "id")?;
        id(&task_id)?;
        let session = text(&v, "session")?;
        id(&session)?;
        text(&v, "purpose")?;
        let project = PathBuf::from(text(&v, "project_root")?);
        let identity = text(&v, "project_identity")?;
        if project_identity(&project)? != identity {
            return store::fail(3, "needs-scope: task project/locked-goal identity differs");
        }
        let model_repair = match v.get("model_repair") {
            None => None,
            Some(m) => {
                keys(
                    m,
                    &[
                        "model",
                        "prompt",
                        "max_turns",
                        "provider",
                        "data_scope",
                        "helper_sha256",
                    ],
                )?;
                if m["provider"] != "codex-chatgpt-subscription"
                    || m["data_scope"] != "approved-task-workspace"
                {
                    return store::fail(3,"model repair requires explicit subscription provider and copied-workspace outbound data approval");
                }
                let model = text(m, "model")?;
                if !model
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-._".contains(&c))
                {
                    return store::fail(2, "model name must be an exact provider model identifier");
                }
                let helper_sha256 = text(m, "helper_sha256")?;
                if helper_sha256.len() != 64
                    || !helper_sha256
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                {
                    return store::fail(
                        2,
                        "matching code-mode helper digest must be lowercase SHA-256",
                    );
                }
                Some(ModelRepair {
                    model,
                    prompt: text(m, "prompt")?,
                    max_turns: number(m, "max_turns", 1, 16)?,
                    helper_sha256,
                })
            }
        };
        let expected_resource = if model_repair.is_some() {
            "included-subscription"
        } else {
            "owned-local"
        };
        if v["resource_class"] != expected_resource
            || !strings(&v, "network_destinations")?.is_empty()
            || !strings(&v, "credential_handles")?.is_empty()
        {
            return store::fail(3,"needs-scope: this host mode supports owned local resources, no task network and no credential handles; other resource classes require a verified adapter");
        }
        let uid = number(&v, "task_uid", 1, u32::MAX as u64)? as u32;
        let user = text(&v, "task_user")?;
        id(&user)?;
        keys(&v["host"], &["path", "sha256", "version"])?;
        let host = PathBuf::from(text(&v["host"], "path")?);
        let host_hash = text(&v["host"], "sha256")?;
        if !host.is_absolute()
            || host_hash.len() != 64
            || !host_hash.bytes().all(|c| c.is_ascii_hexdigit())
            || v["host"]["version"] != "0.166.0"
        {
            return store::fail(2, "require pinned Linux Codex 0.166.0 path and SHA-256");
        }
        let mut checks = vec![];
        let mut labels = BTreeSet::new();
        for c in v["checks"]
            .as_array()
            .filter(|a| !a.is_empty() && a.len() <= 8)
            .ok_or_else(|| Error(2, "require 1..8 task checks".into()))?
        {
            keys(c, &["label", "argv", "inputs", "kind"])?;
            let kind = text(c, "kind")?;
            if !["test", "check"].contains(&kind.as_str()) {
                return store::fail(2, "check kind must be test or check");
            }
            let label = text(c, "label")?;
            id(&label)?;
            if !labels.insert(label.clone()) {
                return store::fail(2, "duplicate task check label");
            }
            let inputs = strings(c, "inputs")?;
            if inputs.is_empty() {
                return store::fail(2, "verification inputs cannot be empty");
            }
            for p in &inputs {
                relative(p, true)?;
            }
            checks.push(Check {
                label,
                kind,
                argv: argv(c, "argv")?,
                inputs,
            });
        }
        let repair = if model_repair.is_some() {
            let a = strings(&v, "repair_argv")?;
            if !a.is_empty() {
                return store::fail(2, "model repair cannot silently fall back to a command");
            }
            a
        } else {
            argv(&v, "repair_argv")?
        };
        let mut excludes = strings(&v, "exclusions")?;
        for p in &excludes {
            relative(p, false)?;
            if p.contains('/') {
                return store::fail(
                    3,
                    "needs-scope: this adapter supports top-level exclusion roots only",
                );
            }
        }
        for p in [".git", ".kineti", ".env", ".ssh", ".aws", ".codex"] {
            if !excludes.iter().any(|x| x == p) {
                excludes.push(p.into());
            }
        }
        excludes.sort();
        excludes.dedup();
        let runtime = strings(&v, "runtime_inputs")?;
        if runtime.is_empty() {
            return store::fail(
                2,
                "declare public system runtime inputs for evidence freshness",
            );
        }
        for p in &runtime {
            if !["/usr/", "/lib/", "/lib64/", "/bin/", "/sbin/"]
                .iter()
                .any(|s| p.starts_with(s))
                || Path::new(p)
                    .components()
                    .any(|x| matches!(x, Component::ParentDir))
            {
                return store::fail(2,"runtime inputs must be public system runtime paths, not credentials or project authority");
            }
        }
        let l = &v["limits"];
        keys(
            l,
            &[
                "expires_at",
                "wall_seconds",
                "command_seconds",
                "repair_attempts",
                "verification_attempts",
                "snapshot_bytes",
                "concurrency",
                "new_paid_usd_micros",
                "memory_bytes",
                "pids",
            ],
        )?;
        if l["concurrency"] != 1 || l["new_paid_usd_micros"] != 0 {
            return store::fail(
                3,
                "deny: one task worker and zero new paid spending are required",
            );
        }
        let expires = l["expires_at"]
            .as_i64()
            .filter(|n| *n > 0)
            .ok_or_else(|| Error(2, "explicit task expiry required".into()))?;
        let rollback = v["automatic_rollback"]
            .as_bool()
            .ok_or_else(|| Error(2, "explicit recovery policy required".into()))?;
        Ok(Self {
            request: v.clone(),
            id: task_id,
            session,
            project,
            identity,
            uid,
            user,
            host,
            host_hash,
            checks,
            repair,
            model_repair,
            excludes,
            runtime,
            expires,
            wall: number(l, "wall_seconds", 1, 86_400)?,
            command_seconds: number(l, "command_seconds", 1, 3600)?,
            repairs: number(l, "repair_attempts", 0, 16)?,
            attempts: number(l, "verification_attempts", 1, 64)?,
            snapshot_bytes: number(l, "snapshot_bytes", 1, 1_000_000_000)?,
            memory: number(l, "memory_bytes", 134_217_728, 8_589_934_592)?,
            pids: number(l, "pids", 16, 256)?,
            rollback,
        })
    }
    /// Stable authority binding; possession of this digest confers no authority.
    pub fn digest(&self) -> Result<String> {
        Ok(digest(
            crate::compat::canonical(&self.request).map_err(|s| Error(2, s))?,
        ))
    }
    /// Validate immutable identity and host bytes on every use and resume.
    pub fn current(&self) -> Result<()> {
        if let Some(mode) = &self.model_repair {
            if crate::task_snapshot::artifact_digest(
                &self.host.with_file_name("codex-code-mode-host"),
            )? != mode.helper_sha256
            {
                return store::fail(
                    3,
                    "deny: matching code-mode helper changed or is unavailable",
                );
            }
        }

        if project_identity(&self.project)? != self.identity {
            return store::fail(3, "needs-scope: project identity changed");
        }
        if crate::task_snapshot::artifact_digest(&self.host)? != self.host_hash {
            return store::fail(3, "deny: pinned host artifact changed");
        }
        if time::OffsetDateTime::now_utc().unix_timestamp() >= self.expires {
            return store::fail(3, "deny: task grant expired; agents cannot renew it");
        }
        Ok(())
    }
    /// Readable initial approval summary, without a command/hash typing ritual.
    pub fn summary(&self) -> Value {
        json!({"schema":"kineti-task-plan-v1","id":self.id,"purpose":self.request["purpose"],"project":self.project,"session":self.session,"project_identity":self.identity,"task_uid":self.uid,"host":self.request["host"],"checks":self.request["checks"],"repair_argv":self.repair,"model_repair":self.request.get("model_repair"),"login_authority":false,"exclusions":self.excludes,"limits":self.request["limits"],"automatic_known_local_rollback":self.rollback,"authority":false,"release_authority":false,"unsupported":["task network","credential handles","paid fallback","transparent interception","authenticated model acceptance"]})
    }
}

/// Permit only Codex-generated trust markers for fixed owner-store workspaces.
/// No provider, credential, tool or instruction settings may be inherited.
pub(crate) fn validate_model_host_config(text: &str) -> Result<()> {
    let parsed: toml::Value = toml::from_str(text).map_err(|_| {
        Error(
            3,
            "model task configuration is not a bounded trust-only document".into(),
        )
    })?;
    let root = parsed
        .as_table()
        .ok_or_else(|| Error(3, "model task configuration must be a table".into()))?;
    if root.keys().any(|k| k != "projects") {
        return store::fail(
            3,
            "needs-scope: model task configuration contains non-trust settings",
        );
    }
    if let Some(projects) = root.get("projects") {
        for (path, value) in projects
            .as_table()
            .ok_or_else(|| Error(3, "invalid trust markers".into()))?
        {
            let p = Path::new(path);
            if p.parent() != Some(Path::new("/var/lib/kineti-owner-native-v1/work")) {
                return store::fail(3, "trust marker outside owner work store");
            }
            id(p.file_name().and_then(|s| s.to_str()).unwrap_or(""))?;
            let t = value
                .as_table()
                .ok_or_else(|| Error(3, "invalid trust marker".into()))?;
            if t.len() != 1
                || !matches!(
                    t.get("trust_level").and_then(|x| x.as_str()),
                    Some("trusted" | "untrusted")
                )
            {
                return store::fail(3, "unexpected trust marker settings");
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod model_host_config_tests {
    use super::*;
    #[test]
    fn generated_trust_does_not_grant_provider_or_tool_configuration() {
        assert!(validate_model_host_config("").is_ok());
        assert!(validate_model_host_config(
            "[projects.\"/var/lib/kineti-owner-native-v1/work/task-a\"]\ntrust_level='trusted'\n"
        )
        .is_ok());
        for bad in ["model_provider='elsewhere'", "[projects.\"/tmp/other\"]\ntrust_level='trusted'", "[projects.\"/var/lib/kineti-owner-native-v1/work/task-a\"]\ntrust_level='trusted'\ncommand='outside'", "[model_providers.evil]\napi_key='fixture-secret'"] {
            assert!(validate_model_host_config(bad).is_err());
        }
    }
}
