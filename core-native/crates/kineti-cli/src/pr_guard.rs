//! Base-branch policy enforcement, polyglot AI test-tampering detection, and signed PR audit bundles.

use crate::{
    compat::{canonical, digest, public_key},
    receipt::{self, sign_object},
    spend,
    store::{self, Error, Result},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::Signature;
use regex::Regex;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    os::unix::ffi::OsStrExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

/// Options for evaluating a pull request diff against protected base-branch policy.
#[derive(Debug, Default, Clone)]
pub struct PrCheckOpts {
    /// Base git ref or SHA (required).
    pub base_ref: String,
    /// Head git ref or SHA (defaults to `"HEAD"`).
    pub head_ref: String,
    /// Explicitly permit PR changes to protected policy/workflow files (e.g., CODEOWNER approved).
    pub allow_policy_change: bool,
    /// Explicitly permit net test-function deletions.
    pub allow_test_reduction: bool,
    /// Optional override for max allowed net deleted test functions (defaults to policy or 0).
    pub max_deleted_tests: Option<u64>,
    /// Optional path to write signed `kineti-pr-audit-v1` canonical JSON bundle.
    pub audit_out: Option<PathBuf>,
    /// Optional path to write Markdown PR / GitHub Step Summary table.
    pub summary_out: Option<PathBuf>,
    /// Optional evidence check labels to verify against base-branch commands and embed in the audit bundle.
    pub require_evidence: Vec<String>,
    /// Also enforce participating CI state/spend/stage checks so audit verdict matches `kineti ci`.
    pub require_ci: bool,
    /// Max evidence age in minutes when `require_evidence` is non-empty (default 240).
    pub max_age: u64,
}

#[derive(Debug, Default)]
struct BasePolicy {
    protected_paths: Vec<String>,
    required_tests: Vec<String>,
    protected_test_globs: Vec<String>,
    max_deleted_tests: Option<u64>,
    max_age_minutes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChangedFile {
    status: String,
    old_path: String,
    new_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TamperFinding {
    file: String,
    line: u64,
    rule: String,
    snippet: String,
}

impl TamperFinding {
    fn to_json(&self) -> Value {
        json!({
            "file": self.file,
            "line": self.line,
            "rule": self.rule,
            "snippet": self.snippet,
        })
    }
}

#[derive(Clone, Copy)]
enum TestLang {
    Rust,
    Python,
    JsTs,
    Go,
}

struct Patterns {
    hunk_header: Regex,
    rust_ignore: Regex,
    rust_cfg_attr_ignore: Regex,
    rust_test_def: Regex,
    python_skip: Regex,
    python_test_def: Regex,
    js_skip: Regex,
    js_test_def: Regex,
    go_skip: Regex,
    go_test_def: Regex,
    weakening_flag: Regex,
    pkg_test_script: Regex,
}

impl Patterns {
    fn new() -> Self {
        Self {
            hunk_header: Regex::new(r"^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@")
                .expect("valid hunk header regex"),
            rust_ignore: Regex::new(
                r"#\[\s*(?:[A-Za-z0-9_]+::)*ignore(?:\s*=|\s*\(|\s*\])",
            )
            .expect("valid rust ignore regex"),
            rust_cfg_attr_ignore: Regex::new(
                r"#\[\s*(?:[A-Za-z0-9_]+::)*cfg_attr\s*\([^)]*,\s*(?:[A-Za-z0-9_]+::)*ignore(?:\s*=|\s*\(|\s*\))",
            )
            .expect("valid rust cfg_attr ignore regex"),
            rust_test_def: Regex::new(
                r"#\[\s*(?:[A-Za-z0-9_]+::)*test(?:\s*\([^)]*\))?\s*\]",
            )
            .expect("valid rust test def regex"),
            python_skip: Regex::new(
                r"(?:@pytest\.mark\.(?:skip|skipif|xfail)\b|@unittest\.skip(?:If|Unless)?\b|\bpytest\.(?:skip|xfail)\s*\(|\bself\.skipTest\s*\()",
            )
            .expect("valid python skip regex"),
            python_test_def: Regex::new(r"^\s*(?:async\s+)?def\s+test_[A-Za-z0-9_]+\s*\(")
                .expect("valid python test def regex"),
            js_skip: Regex::new(
                r"(?:\b(?:test|it|describe)\.(?:skip|only|todo)(?:\.each)?\s*\(|\b(?:xit|xtest|xdescribe)\s*\()",
            )
            .expect("valid js skip regex"),
            js_test_def: Regex::new(r"^\s*(?:test|it)\s*\(").expect("valid js test def regex"),
            go_skip: Regex::new(r"\b(?:t|b)\.(?:Skip|Skipf|SkipNow)\s*\(")
                .expect("valid go skip regex"),
            go_test_def: Regex::new(r"^\s*func\s+Test[A-Za-z0-9_]+\s*\(")
                .expect("valid go test def regex"),
            weakening_flag: Regex::new(
                r#"(?:--passWithNoTests\b|--skip-tests\b|-DskipTests\b|(?:^|[\s"'=])-k\s*=?\s*(?:""|'')|(?:^|[\s"'])--ignore(?:=|\s|["']|$))"#,
            )
            .expect("valid weakening flag regex"),
            pkg_test_script: Regex::new(r#"^\s*"test"\s*:\s*"([^"]*)"\s*,?\s*$"#)
                .expect("valid package test script regex"),
        }
    }
}

struct DiffScanResult {
    findings: Vec<TamperFinding>,
    added_tests: u64,
    deleted_tests: u64,
}

struct DiffScanner<'a, F> {
    policy: &'a BasePolicy,
    patterns: &'a Patterns,
    rust_has_tests: F,
    findings: Vec<TamperFinding>,
    added_tests: u64,
    deleted_tests: u64,
}

impl<F> DiffScanner<'_, F>
where
    F: FnMut(&str) -> bool,
{
    fn is_test_file(&mut self, path: &str) -> bool {
        if matches_explicit_required_test(path, self.policy) || is_default_test_path(path) {
            return true;
        }
        if path.ends_with(".rs") {
            return (self.rust_has_tests)(path);
        }
        false
    }

    fn inspect_added_line(&mut self, file: &str, line_no: u64, content: &str) {
        let trimmed = content.trim();

        if is_ci_or_script_file(file) {
            let is_hash_comment = !is_package_json(file) && trimmed.starts_with('#');
            if !is_hash_comment && self.patterns.weakening_flag.is_match(content) {
                self.findings.push(TamperFinding {
                    file: file.to_owned(),
                    line: line_no,
                    rule: "added_weakening_test_flag".to_owned(),
                    snippet: truncate_snippet(content),
                });
            } else if is_package_json(file)
                && self
                    .patterns
                    .pkg_test_script
                    .captures(content)
                    .is_some_and(|caps| is_weakened_package_test_cmd(&caps[1]))
            {
                self.findings.push(TamperFinding {
                    file: file.to_owned(),
                    line: line_no,
                    rule: "weakened_package_test_script".to_owned(),
                    snippet: truncate_snippet(content),
                });
            }
        }

        if !self.is_test_file(file) {
            return;
        }

        match detect_test_lang(file) {
            Some(TestLang::Rust) => {
                let code = strip_slash_code(content);
                let ct = code.trim();
                if ct.is_empty() {
                    return;
                }
                if self.patterns.rust_ignore.is_match(ct)
                    || self.patterns.rust_cfg_attr_ignore.is_match(ct)
                {
                    self.findings.push(TamperFinding {
                        file: file.to_owned(),
                        line: line_no,
                        rule: "added_rust_ignore".to_owned(),
                        snippet: truncate_snippet(content),
                    });
                }
                if self.patterns.rust_test_def.is_match(ct) {
                    self.added_tests += 1;
                }
            }
            Some(TestLang::Python) if !trimmed.starts_with('#') => {
                if self.patterns.python_skip.is_match(content) {
                    self.findings.push(TamperFinding {
                        file: file.to_owned(),
                        line: line_no,
                        rule: "added_python_skip".to_owned(),
                        snippet: truncate_snippet(content),
                    });
                }
                if self.patterns.python_test_def.is_match(content) {
                    self.added_tests += 1;
                }
            }
            Some(TestLang::JsTs) => {
                let code = strip_slash_code(content);
                let ct = code.trim();
                if ct.is_empty() {
                    return;
                }
                if self.patterns.js_skip.is_match(ct) {
                    self.findings.push(TamperFinding {
                        file: file.to_owned(),
                        line: line_no,
                        rule: "added_js_skip_or_only".to_owned(),
                        snippet: truncate_snippet(content),
                    });
                }
                if self.patterns.js_test_def.is_match(ct) {
                    self.added_tests += 1;
                }
            }
            Some(TestLang::Go) => {
                let code = strip_slash_code(content);
                let ct = code.trim();
                if ct.is_empty() {
                    return;
                }
                if self.patterns.go_skip.is_match(ct) {
                    self.findings.push(TamperFinding {
                        file: file.to_owned(),
                        line: line_no,
                        rule: "added_go_skip".to_owned(),
                        snippet: truncate_snippet(content),
                    });
                }
                if self.patterns.go_test_def.is_match(ct) {
                    self.added_tests += 1;
                }
            }
            _ => {}
        }
    }

    fn inspect_deleted_line(&mut self, file: &str, content: &str) {
        if !self.is_test_file(file) {
            return;
        }
        let trimmed = content.trim();
        let matched = match detect_test_lang(file) {
            Some(TestLang::Rust) => {
                let code = strip_slash_code(content);
                let ct = code.trim();
                !ct.is_empty() && self.patterns.rust_test_def.is_match(ct)
            }
            Some(TestLang::Python) if !trimmed.starts_with('#') => {
                self.patterns.python_test_def.is_match(content)
            }
            Some(TestLang::JsTs) => {
                let code = strip_slash_code(content);
                let ct = code.trim();
                !ct.is_empty() && self.patterns.js_test_def.is_match(ct)
            }
            Some(TestLang::Go) => {
                let code = strip_slash_code(content);
                let ct = code.trim();
                !ct.is_empty() && self.patterns.go_test_def.is_match(ct)
            }
            _ => false,
        };
        if matched {
            self.deleted_tests += 1;
        }
    }
}

/// Verify the Ed25519 signature of a `kineti-pr-audit-v1` bundle against a trusted SPKI PEM key.
///
/// A valid signature binds the bundle to the local runner's issuer key; it does not prove
/// independent test assertion quality.
pub fn verify_audit(audit: &Value, trusted_pem: &str) -> bool {
    let check = || -> Option<()> {
        let obj = audit.as_object()?;
        if obj.get("schema")?.as_str()? != "kineti-pr-audit-v1"
            || obj.get("encoding")?.as_str()? != "kineti-canonical-v2"
            || obj.get("issuer_key")?.as_str()? != trusted_pem
        {
            return None;
        }
        let sig_b64 = obj.get("signature")?.as_str()?;
        let sig_bytes = STANDARD.decode(sig_b64).ok()?;
        if sig_bytes.len() != 64 || STANDARD.encode(&sig_bytes) != sig_b64 {
            return None;
        }
        let mut unsigned = obj.clone();
        unsigned.shift_remove("signature");
        let payload = canonical(&Value::Object(unsigned)).ok()?;
        let key = public_key(trusted_pem).ok()?;
        let sig = Signature::from_slice(&sig_bytes).ok()?;
        key.verify_strict(payload.as_bytes(), &sig).ok()
    };
    check().is_some()
}

/// Evaluate a pull request against protected base-branch policy, scan the diff for
/// polyglot AI test tampering, optionally verify required evidence receipts against
/// base-branch commands, and emit a signed `kineti-pr-audit-v1` bundle and Markdown summary.
pub fn run(root: &Path, opts: PrCheckOpts) -> Result<Value> {
    validate_git_ref(&opts.base_ref, "base_ref")?;
    let head_ref = if opts.head_ref.is_empty() {
        "HEAD"
    } else {
        opts.head_ref.as_str()
    };
    validate_git_ref(head_ref, "head_ref")?;
    if opts.max_age > 5_256_000 {
        return store::fail(2, "max evidence age must be 1..=5256000 minutes");
    }
    for label in &opts.require_evidence {
        if label.is_empty() || label.contains('\0') {
            return store::fail(2, "evidence label must be nonempty without NUL bytes");
        }
    }

    let resolved_audit_out = opts
        .audit_out
        .as_deref()
        .map(|p| resolve_output_path(root, p))
        .transpose()?;
    let resolved_summary_out = opts
        .summary_out
        .as_deref()
        .map(|p| resolve_output_path(root, p))
        .transpose()?;

    let base_sha = git_rev_parse_commit(root, &opts.base_ref, "base_ref")?;
    let head_sha = git_rev_parse_commit(root, head_ref, "head_ref")?;
    let diff_base_sha =
        git_merge_base(root, &base_sha, &head_sha).unwrap_or_else(|| base_sha.clone());

    let (base_policy_value, base_policy_sha256, policy) = load_base_policy(root, &base_sha)?;
    let changed_files = git_diff_name_status(root, &diff_base_sha, &head_sha)?;

    let mut protected_touched = BTreeSet::new();
    for entry in &changed_files {
        if !entry.status.starts_with('C')
            && is_protected_path(&entry.old_path, &policy.protected_paths)
        {
            protected_touched.insert(entry.old_path.clone());
        }
        if let Some(new_path) = entry
            .new_path
            .as_deref()
            .filter(|p| is_protected_path(p, &policy.protected_paths))
        {
            protected_touched.insert(new_path.to_owned());
        }
    }
    let policy_tamper_files: Vec<String> = protected_touched.into_iter().collect();
    let policy_tamper_detected = !policy_tamper_files.is_empty() && !opts.allow_policy_change;

    let mut findings =
        detect_deleted_test_files(&changed_files, &policy, opts.allow_test_reduction);

    let diff_text = git_diff_unified_zero(root, &diff_base_sha, &head_sha)?;
    let patterns = Patterns::new();
    let mut rust_cache: BTreeMap<String, bool> = BTreeMap::new();
    let scan = scan_unified_diff(&diff_text, &policy, &patterns, |path| {
        if let Some(&cached) = rust_cache.get(path) {
            return cached;
        }
        let has_tests = rust_file_has_tests(root, &head_sha, path)
            || rust_file_has_tests(root, &diff_base_sha, path);
        rust_cache.insert(path.to_owned(), has_tests);
        has_tests
    });
    findings.extend(scan.findings);

    let net_deleted_tests = scan.deleted_tests.saturating_sub(scan.added_tests);
    let max_deleted_tests = opts
        .max_deleted_tests
        .or(policy.max_deleted_tests)
        .unwrap_or(0);
    if net_deleted_tests > max_deleted_tests && !opts.allow_test_reduction {
        findings.push(TamperFinding {
            file: "*".to_owned(),
            line: 0,
            rule: "net_test_deletion".to_owned(),
            snippet: format!(
                "deleted {} test definition(s) vs added {} (net deleted {}, max allowed {})",
                scan.deleted_tests, scan.added_tests, net_deleted_tests, max_deleted_tests
            ),
        });
    }

    let effective_max_age = if opts.max_age > 0 {
        opts.max_age
    } else {
        policy.max_age_minutes.unwrap_or(240)
    };
    let mut evidence_checks = Vec::new();
    let mut evidence_failures = Vec::new();
    let base_commands = base_policy_value.get("commands").and_then(Value::as_object);
    for label in &opts.require_evidence {
        let Some(expected) = base_commands.and_then(|map| map.get(label)) else {
            evidence_failures.push(json!({
                "label": label,
                "code": 1,
                "error": "required command mapping missing in base-branch .kineti/gate.json",
            }));
            continue;
        };
        if !validate_argv(expected) {
            evidence_failures.push(json!({
                "label": label,
                "code": 1,
                "error": "base-branch command mapping must be a nonempty argv string array",
            }));
            continue;
        }
        match receipt::check_with_policy(
            root,
            label,
            effective_max_age,
            Some(expected),
            &base_policy_value,
        ) {
            Ok(check_val) => {
                let r = &check_val["receipt"];
                let ver = &r["verification"];
                let count_observed = ver["count_observed"]
                    .as_bool()
                    .unwrap_or_else(|| ver["parser"].as_str() != Some("kineti-exit-code-v1"));
                evidence_checks.push(json!({
                    "label": label,
                    "run_id": r["run_id"],
                    "status": check_val["status"],
                    "parser": ver["parser"],
                    "passed": ver["passed"],
                    "failed": ver["failed"],
                    "skipped": ver["skipped"],
                    "count_observed": count_observed,
                    "command": r["manifest"]["command"],
                    "signature": r["signature"],
                    "at": r["at"],
                }));
            }
            Err(err) => {
                evidence_failures.push(json!({
                    "label": label,
                    "code": err.0,
                    "error": err.1,
                }));
            }
        }
    }

    if opts.require_ci {
        match store::state(root) {
            Err(err) => {
                evidence_failures.push(json!({
                    "label": "ci:state",
                    "code": err.0,
                    "error": err.1,
                }));
            }
            Ok(state) => {
                if let Err(err) = spend::inspect(root, true) {
                    evidence_failures.push(json!({
                        "label": "ci:spend",
                        "code": err.0,
                        "error": err.1,
                    }));
                }
                let stage = state["stage"]
                    .as_u64()
                    .or_else(|| state["current_stage"].as_u64());
                match stage {
                    None => {
                        evidence_failures.push(json!({
                            "label": "ci:stage",
                            "code": 1,
                            "error": "project stage missing; preview init for missing fields",
                        }));
                    }
                    Some(stage) => {
                        if stage >= 7 && state["gates"]["spec"] == "fail" {
                            evidence_failures.push(json!({
                                "label": "ci:spec",
                                "code": 1,
                                "error": "spec approval failed",
                            }));
                        }
                        if state["gates"]["security"] == "fail"
                            || stage >= 11 && state["gates"]["security"] != "pass"
                        {
                            evidence_failures.push(json!({
                                "label": "ci:security",
                                "code": 1,
                                "error": "security gate must pass before ship",
                            }));
                        }
                    }
                }
            }
        }
    }

    let blocked = policy_tamper_detected || !findings.is_empty() || !evidence_failures.is_empty();
    let finding_values: Vec<Value> = findings.iter().map(TamperFinding::to_json).collect();

    let mut audit = json!({
        "schema": "kineti-pr-audit-v1",
        "encoding": "kineti-canonical-v2",
        "at": store::now(),
        "base_ref": opts.base_ref,
        "head_ref": head_ref,
        "base_sha": base_sha,
        "head_sha": head_sha,
        "diff_base_sha": diff_base_sha,
        "base_policy_sha256": base_policy_sha256,
        "policy_tamper_detected": policy_tamper_detected,
        "policy_change_allowed": opts.allow_policy_change,
        "policy_tamper_files": policy_tamper_files,
        "test_tamper_findings": finding_values,
        "test_counts": {
            "added": scan.added_tests,
            "deleted": scan.deleted_tests,
            "net_deleted": net_deleted_tests,
            "max_deleted_tests": max_deleted_tests,
            "test_reduction_allowed": opts.allow_test_reduction,
        },
        "evidence_checks": evidence_checks,
        "evidence_failures": evidence_failures,
        "verdict": if blocked { "blocked" } else { "passed" },
        "ci_runner_trust_boundary": true,
        "independent_assertion_quality_proven": false,
        "automatic_interception": false,
        "provider_billing": "unknown",
    });

    let _ = sign_object(root, &mut audit)?;

    if let Some(ref out_path) = resolved_audit_out {
        let pretty = format!("{}\n", serde_json::to_string_pretty(&audit)?);
        store::atomic(out_path, pretty.as_bytes())?;
    }
    if let Some(ref out_path) = resolved_summary_out {
        let summary_md = render_summary(&audit);
        store::atomic(out_path, summary_md.as_bytes())?;
    }

    if blocked {
        return store::fail(
            1,
            format!(
                "PR check blocked: policy_tamper_detected={policy_tamper_detected}, tamper_findings={}, evidence_failures={}",
                findings.len(),
                evidence_failures.len()
            ),
        );
    }

    Ok(audit)
}

fn validate_git_ref(value: &str, name: &str) -> Result<()> {
    if value.is_empty() || value.len() > 256 {
        return store::fail(2, format!("{name} must be 1..=256 characters"));
    }
    if value.contains('\0')
        || value.starts_with('-')
        || value.contains("..")
        || value.starts_with('/')
        || value.ends_with('/')
        || value.contains("//")
        || value.ends_with('.')
    {
        return store::fail(2, format!("invalid git ref for {name}"));
    }
    if !value.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'/' | b'@' | b'^' | b'~' | b'-')
    }) {
        return store::fail(2, format!("invalid characters in git ref for {name}"));
    }
    Ok(())
}

fn resolve_output_path(root: &Path, path: &Path) -> Result<PathBuf> {
    let raw = path.as_os_str();
    if raw.is_empty() || raw.as_bytes().contains(&0) {
        return store::fail(2, "output path must be nonempty without NUL bytes");
    }
    let canon_root = root.canonicalize()?;
    let rel: PathBuf = if path.is_absolute() {
        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return store::fail(2, "output path must not traverse parent directories");
        }
        if let Ok(stripped) = path.strip_prefix(&canon_root) {
            stripped.to_path_buf()
        } else if let Ok(stripped) = path.strip_prefix(root) {
            stripped.to_path_buf()
        } else {
            return store::fail(2, "output path must remain inside the project root");
        }
    } else {
        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return store::fail(
                2,
                "relative output path must not traverse parent directories",
            );
        }
        path.to_path_buf()
    };
    if rel.as_os_str().is_empty() {
        return store::fail(2, "output path must name a file inside the project root");
    }
    let mut cur = canon_root.clone();
    for comp in rel.components() {
        match comp {
            Component::CurDir => continue,
            Component::Normal(seg) => {
                cur.push(seg);
                if let Ok(meta) = fs::symlink_metadata(&cur) {
                    if meta.file_type().is_symlink() {
                        return store::fail(3, "symbolic-link output destination refused");
                    }
                }
            }
            _ => return store::fail(2, "invalid output path component"),
        }
    }
    if let Some(parent) = cur.parent() {
        let mut probe = parent.to_path_buf();
        while !probe.exists() {
            let Some(next) = probe.parent() else { break };
            probe = next.to_path_buf();
        }
        if probe.exists() && !probe.canonicalize()?.starts_with(&canon_root) {
            return store::fail(3, "output path resolves outside the project root");
        }
    }
    store::safe_destination(&cur)?;
    Ok(cur)
}

fn run_git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .stdin(Stdio::null())
        .output()
        .map_err(|e| Error(3, format!("failed to invoke git: {e}")))
}

fn is_hex_sha(text: &str) -> bool {
    (text.len() == 40 || text.len() == 64)
        && text
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn git_rev_parse_commit(root: &Path, git_ref: &str, name: &str) -> Result<String> {
    let spec = format!("{git_ref}^{{commit}}");
    let out = run_git(root, &["rev-parse", "--verify", &spec])?;
    if !out.status.success() {
        return store::fail(2, format!("unable to resolve {name} commit '{git_ref}'"));
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if !is_hex_sha(&sha) {
        return store::fail(2, format!("unexpected commit digest for {name}"));
    }
    Ok(sha)
}

fn git_merge_base(root: &Path, base_sha: &str, head_sha: &str) -> Option<String> {
    let out = run_git(root, &["merge-base", base_sha, head_sha]).ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    is_hex_sha(&sha).then_some(sha)
}

fn load_base_policy(root: &Path, base_sha: &str) -> Result<(Value, String, BasePolicy)> {
    let ls = run_git(root, &["ls-tree", base_sha, "--", ".kineti/gate.json"])?;
    if !ls.status.success() {
        return store::fail(1, "failed to inspect base-branch .kineti/gate.json");
    }
    let ls_text = String::from_utf8_lossy(&ls.stdout);
    let ls_line = ls_text.trim();
    if ls_line.is_empty() {
        return store::fail(1, "base-branch .kineti/gate.json is missing");
    }
    if ls_line.starts_with("120000 ") {
        return store::fail(1, "base-branch .kineti/gate.json must not be a symlink");
    }

    let spec = format!("{base_sha}:.kineti/gate.json");
    let out = run_git(root, &["show", &spec])?;
    if !out.status.success() {
        return store::fail(1, "base-branch .kineti/gate.json could not be read");
    }
    let policy_sha = digest(&out.stdout);
    let value: Value = serde_json::from_slice(&out.stdout).map_err(|e| {
        Error(
            1,
            format!("invalid JSON in base-branch .kineti/gate.json: {e}"),
        )
    })?;
    if !value.is_object() {
        return store::fail(1, "base-branch .kineti/gate.json must be a JSON object");
    }
    let parsed = parse_base_policy(&value)?;
    Ok((value, policy_sha, parsed))
}

fn parse_string_list(value: &Value, field: &str) -> Result<Vec<String>> {
    let Some(raw) = value.get(field) else {
        return Ok(Vec::new());
    };
    if raw.is_null() {
        return Ok(Vec::new());
    }
    let arr = raw.as_array().ok_or_else(|| {
        Error(
            1,
            format!("base policy '{field}' must be an array of strings"),
        )
    })?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        let s = item
            .as_str()
            .filter(|s| !s.is_empty() && !s.contains('\0'))
            .ok_or_else(|| {
                Error(
                    1,
                    format!("base policy '{field}' entries must be nonempty strings"),
                )
            })?;
        out.push(normalize_rel_path(s));
    }
    Ok(out)
}

fn parse_required_tests(value: &Value) -> Result<Vec<String>> {
    let Some(raw) = value.get("required_tests") else {
        return Ok(Vec::new());
    };
    if raw.is_null() {
        return Ok(Vec::new());
    }
    if raw.is_array() {
        return parse_string_list(value, "required_tests");
    }
    if let Some(map) = raw.as_object() {
        let mut set = BTreeSet::new();
        for entry in map.values() {
            if let Some(single) = entry.as_str() {
                if single.is_empty() || single.contains('\0') {
                    return store::fail(1, "base policy 'required_tests' entry invalid");
                }
                set.insert(normalize_rel_path(single));
            } else if let Some(arr) = entry.as_array() {
                for item in arr {
                    let s = item
                        .as_str()
                        .filter(|s| !s.is_empty() && !s.contains('\0'))
                        .ok_or_else(|| {
                            Error(
                                1,
                                "base policy 'required_tests' entry must be nonempty string".into(),
                            )
                        })?;
                    set.insert(normalize_rel_path(s));
                }
            } else {
                return store::fail(
                    1,
                    "base policy 'required_tests' map values must be strings or string arrays",
                );
            }
        }
        return Ok(set.into_iter().collect());
    }
    store::fail(1, "base policy 'required_tests' must be an array or object")
}

fn parse_base_policy(value: &Value) -> Result<BasePolicy> {
    let protected_paths = parse_string_list(value, "protected_paths")?;
    let required_tests = parse_required_tests(value)?;
    let protected_test_globs = parse_string_list(value, "protected_test_globs")?;

    let max_deleted_tests = match value.get("max_deleted_tests") {
        None | Some(Value::Null) => None,
        Some(v) => Some(v.as_u64().ok_or_else(|| {
            Error(
                1,
                "base policy 'max_deleted_tests' must be an integer".into(),
            )
        })?),
    };
    let max_age_minutes = match value.get("max_age_minutes") {
        None | Some(Value::Null) => None,
        Some(v) => {
            let n = v.as_u64().ok_or_else(|| {
                Error(1, "base policy 'max_age_minutes' must be an integer".into())
            })?;
            if n == 0 || n > 5_256_000 {
                return store::fail(1, "base policy 'max_age_minutes' out of range 1..=5256000");
            }
            Some(n)
        }
    };

    Ok(BasePolicy {
        protected_paths,
        required_tests,
        protected_test_globs,
        max_deleted_tests,
        max_age_minutes,
    })
}

fn git_diff_name_status(
    root: &Path,
    diff_base_sha: &str,
    head_sha: &str,
) -> Result<Vec<ChangedFile>> {
    let range = format!("{diff_base_sha}..{head_sha}");
    let out = run_git(
        root,
        &[
            "-c",
            "core.quotepath=false",
            "diff",
            "--name-status",
            "-M",
            "-z",
            &range,
            "--",
        ],
    )?;
    if !out.status.success() {
        return store::fail(3, "git diff --name-status failed");
    }
    Ok(parse_name_status_z(&out.stdout))
}

fn parse_name_status_z(raw: &[u8]) -> Vec<ChangedFile> {
    let mut out = Vec::new();
    let mut iter = raw.split(|&b| b == 0);
    while let Some(status_bytes) = iter.next() {
        if status_bytes.is_empty() {
            continue;
        }
        let status = String::from_utf8_lossy(status_bytes).into_owned();
        let Some(old_bytes) = iter.next() else {
            break;
        };
        let old_path = normalize_rel_path(&String::from_utf8_lossy(old_bytes));
        let new_path = if status.starts_with('R') || status.starts_with('C') {
            iter.next()
                .map(|b| normalize_rel_path(&String::from_utf8_lossy(b)))
        } else {
            None
        };
        out.push(ChangedFile {
            status,
            old_path,
            new_path,
        });
    }
    out
}

fn git_diff_unified_zero(root: &Path, diff_base_sha: &str, head_sha: &str) -> Result<String> {
    let range = format!("{diff_base_sha}..{head_sha}");
    let out = run_git(
        root,
        &[
            "-c",
            "core.quotepath=false",
            "-c",
            "diff.noprefix=false",
            "diff",
            "--no-ext-diff",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            "--unified=0",
            "--no-color",
            "-M",
            &range,
            "--",
        ],
    )?;
    if !out.status.success() {
        return store::fail(3, "git diff --unified=0 failed");
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

fn rust_file_has_tests(root: &Path, sha: &str, path: &str) -> bool {
    let spec = format!("{sha}:{path}");
    let Ok(out) = run_git(root, &["show", &spec]) else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    text.contains("#[test]") || text.contains("#[cfg(test)]") || text.contains("::test]")
}

fn normalize_rel_path(path: &str) -> String {
    let mut s = path.trim_end_matches('\r');
    while let Some(rest) = s.strip_prefix("./") {
        s = rest;
    }
    s.to_owned()
}

fn is_protected_path(path: &str, custom_protected: &[String]) -> bool {
    let normalized = normalize_rel_path(path);
    let lower = normalized.to_ascii_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(lower.as_str());

    if lower == ".kineti/gate.json"
        || lower.starts_with(".github/workflows/")
        || basename == "pytest.ini"
        || basename.starts_with("jest.config.")
        || basename.starts_with("vitest.config.")
    {
        return true;
    }

    for entry in custom_protected {
        if matches_path_rule(&normalized, entry) {
            return true;
        }
    }
    false
}

fn matches_path_rule(path: &str, rule: &str) -> bool {
    let normalized_rule = normalize_rel_path(rule);
    if normalized_rule.is_empty() {
        return false;
    }
    if normalized_rule.ends_with('/') {
        return path.starts_with(&normalized_rule);
    }
    if normalized_rule.contains('*') || normalized_rule.contains('?') {
        return glob_match(&normalized_rule, path);
    }
    path == normalized_rule
        || path
            .strip_prefix(&normalized_rule)
            .is_some_and(|rest| rest.starts_with('/'))
}

#[derive(Clone, Copy)]
enum GlobTok {
    DoubleStarSlash,
    DoubleStar,
    Star,
    Question,
    Literal(char),
}

fn glob_match(pattern: &str, text: &str) -> bool {
    let pchars: Vec<char> = pattern.chars().collect();
    let mut tokens = Vec::with_capacity(pchars.len());
    let mut i = 0;
    while i < pchars.len() {
        if pchars[i] == '*' && i + 1 < pchars.len() && pchars[i + 1] == '*' {
            i += 2;
            while i < pchars.len() && pchars[i] == '*' {
                i += 1;
            }
            if i < pchars.len() && pchars[i] == '/' {
                i += 1;
                tokens.push(GlobTok::DoubleStarSlash);
            } else {
                tokens.push(GlobTok::DoubleStar);
            }
        } else if pchars[i] == '*' {
            i += 1;
            tokens.push(GlobTok::Star);
        } else if pchars[i] == '?' {
            i += 1;
            tokens.push(GlobTok::Question);
        } else {
            tokens.push(GlobTok::Literal(pchars[i]));
            i += 1;
        }
    }

    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut prev = vec![false; n + 1];
    let mut cur = vec![false; n + 1];
    prev[0] = true;

    for tok in tokens {
        cur.fill(false);
        match tok {
            GlobTok::Literal(c) => {
                for (j, &ch) in chars.iter().enumerate() {
                    cur[j + 1] = prev[j] && ch == c;
                }
            }
            GlobTok::Question => {
                for (j, &ch) in chars.iter().enumerate() {
                    cur[j + 1] = prev[j] && ch != '/';
                }
            }
            GlobTok::Star => {
                cur[0] = prev[0];
                for (j, &ch) in chars.iter().enumerate() {
                    cur[j + 1] = prev[j + 1] || (ch != '/' && cur[j]);
                }
            }
            GlobTok::DoubleStar => {
                let mut seen = prev[0];
                cur[0] = seen;
                for (c, &p) in cur[1..].iter_mut().zip(&prev[1..]) {
                    seen |= p;
                    *c = seen;
                }
            }
            GlobTok::DoubleStarSlash => {
                let mut seen_prev = prev[0];
                cur[0] = prev[0];
                for (j, &ch) in chars.iter().enumerate() {
                    cur[j + 1] = prev[j + 1] || (ch == '/' && seen_prev);
                    seen_prev |= prev[j + 1];
                }
            }
        }
        std::mem::swap(&mut prev, &mut cur);
    }

    prev[n]
}

fn matches_explicit_required_test(path: &str, policy: &BasePolicy) -> bool {
    let normalized = normalize_rel_path(path);
    policy
        .required_tests
        .iter()
        .chain(policy.protected_test_globs.iter())
        .any(|rule| matches_path_rule(&normalized, rule))
}

fn is_default_test_path(path: &str) -> bool {
    let normalized = normalize_rel_path(path);
    let lower = normalized.to_ascii_lowercase();
    let segments: Vec<&str> = lower.split('/').collect();
    if segments.len() > 1
        && segments[..segments.len() - 1]
            .iter()
            .any(|seg| matches!(*seg, "test" | "tests" | "__tests__" | "spec" | "specs"))
    {
        return true;
    }
    let basename = segments.last().copied().unwrap_or("");
    (basename.starts_with("test_") && basename.ends_with(".py"))
        || basename.ends_with("_test.py")
        || basename.ends_with("_test.go")
        || basename.ends_with("_test.rs")
        || basename.contains(".test.")
        || basename.contains(".spec.")
        || basename.ends_with("_spec.py")
        || basename.ends_with("_spec.js")
        || basename.ends_with("_spec.ts")
        || matches!(basename, "tests.rs" | "test.rs" | "tests.py" | "test.py")
}

fn detect_deleted_test_files(
    changed_files: &[ChangedFile],
    policy: &BasePolicy,
    allow_test_reduction: bool,
) -> Vec<TamperFinding> {
    let mut out = Vec::new();
    for entry in changed_files {
        if entry.status == "D" {
            let explicit = matches_explicit_required_test(&entry.old_path, policy);
            if explicit || (!allow_test_reduction && is_default_test_path(&entry.old_path)) {
                out.push(TamperFinding {
                    file: entry.old_path.clone(),
                    line: 0,
                    rule: "deleted_required_test_file".to_owned(),
                    snippet: "required or protected test file deleted".to_owned(),
                });
            }
        } else if let Some(new_path) = entry
            .new_path
            .as_deref()
            .filter(|_| entry.status.starts_with('R'))
        {
            let old_explicit = matches_explicit_required_test(&entry.old_path, policy);
            let new_explicit = matches_explicit_required_test(new_path, policy);
            if old_explicit && !new_explicit {
                out.push(TamperFinding {
                    file: entry.old_path.clone(),
                    line: 0,
                    rule: "deleted_required_test_file".to_owned(),
                    snippet: format!("required test file renamed away to {new_path}"),
                });
            } else if !old_explicit
                && !allow_test_reduction
                && is_default_test_path(&entry.old_path)
                && !is_default_test_path(new_path)
                && !new_explicit
            {
                out.push(TamperFinding {
                    file: entry.old_path.clone(),
                    line: 0,
                    rule: "deleted_required_test_file".to_owned(),
                    snippet: format!("test file renamed outside test paths to {new_path}"),
                });
            }
        } else if entry.status == "T" && matches_explicit_required_test(&entry.old_path, policy) {
            out.push(TamperFinding {
                file: entry.old_path.clone(),
                line: 0,
                rule: "deleted_required_test_file".to_owned(),
                snippet: "required test file type changed".to_owned(),
            });
        }
    }
    out
}

fn detect_test_lang(path: &str) -> Option<TestLang> {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".rs") {
        Some(TestLang::Rust)
    } else if lower.ends_with(".py") {
        Some(TestLang::Python)
    } else if [".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs", ".mts", ".cts"]
        .iter()
        .any(|ext| lower.ends_with(ext))
    {
        Some(TestLang::JsTs)
    } else if lower.ends_with(".go") {
        Some(TestLang::Go)
    } else {
        None
    }
}

fn is_package_json(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower == "package.json" || lower.ends_with("/package.json")
}

fn is_ci_or_script_file(path: &str) -> bool {
    let lower = normalize_rel_path(path).to_ascii_lowercase();
    let basename = lower.rsplit('/').next().unwrap_or(lower.as_str());
    is_package_json(&lower)
        || lower == ".kineti/gate.json"
        || lower.starts_with(".github/workflows/")
        || basename == "makefile"
        || basename == "justfile"
        || basename == "pytest.ini"
        || basename == "pyproject.toml"
        || basename.starts_with("jest.config.")
        || basename.starts_with("vitest.config.")
        || lower.ends_with(".sh")
        || lower.ends_with(".bash")
        || lower.ends_with(".mk")
}

fn is_weakened_package_test_cmd(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    if trimmed.is_empty()
        || matches!(trimmed, "true" | ":" | "exit 0")
        || trimmed.ends_with("|| true")
        || trimmed.ends_with("|| exit 0")
        || trimmed.ends_with("; true")
        || trimmed.ends_with("; exit 0")
        || trimmed.ends_with("&& exit 0")
    {
        return true;
    }
    trimmed.starts_with("echo ") && !trimmed.contains("&&") && !trimmed.contains(';')
}

/// Strip unquoted `/* ... */` inline block comments and trailing `// ...` comments from one line.
fn strip_slash_comments(line: &str) -> String {
    let bytes = line.as_bytes();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    let mut quote: Option<u8> = None;
    let mut escaped = false;
    while i < bytes.len() {
        let b = bytes[i];
        if let Some(q) = quote {
            let ch = line[i..].chars().next().unwrap();
            out.push(ch);
            if escaped {
                escaped = false;
            } else if b == b'\\' && q != b'`' {
                escaped = true;
            } else if b == q {
                quote = None;
            }
            i += ch.len_utf8();
            continue;
        }
        if b == b'"' || b == b'\'' || b == b'`' {
            quote = Some(b);
            out.push(b as char);
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            break;
        }
        if b == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i += 2;
            let mut closed = false;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                break;
            }
            out.push(' ');
            continue;
        }
        // Preserve multi-byte UTF-8 characters safely outside ASCII syntax tokens.
        let ch = line[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn strip_slash_code(line: &str) -> String {
    let trimmed = line.trim_start();
    let effective = if let Some(pos) = trimmed.find("*/") {
        let before = &trimmed[..pos];
        if !before.contains("/*")
            && !before.contains('"')
            && !before.contains('\'')
            && !before.contains('`')
        {
            &trimmed[pos + 2..]
        } else {
            line
        }
    } else if trimmed.starts_with('*') {
        return String::new();
    } else {
        line
    };
    strip_slash_comments(effective)
}

fn normalize_diff_header_path(rest: &str) -> String {
    let trimmed = rest.trim_end_matches('\r').trim_end_matches('\t');
    normalize_rel_path(trimmed)
}

fn parse_hunk_header(re: &Regex, line: &str) -> Option<(u64, u64, u64)> {
    let caps = re.captures(line)?;
    let _old_start = caps.get(1)?.as_str().parse::<u64>().ok()?;
    let old_count = caps
        .get(2)
        .map_or(Ok(1u64), |m| m.as_str().parse::<u64>())
        .ok()?;
    let new_start = caps.get(3)?.as_str().parse::<u64>().ok()?;
    let new_count = caps
        .get(4)
        .map_or(Ok(1u64), |m| m.as_str().parse::<u64>())
        .ok()?;
    Some((old_count, new_start, new_count))
}

fn scan_unified_diff<F>(
    diff_text: &str,
    policy: &BasePolicy,
    patterns: &Patterns,
    rust_has_tests: F,
) -> DiffScanResult
where
    F: FnMut(&str) -> bool,
{
    let mut scanner = DiffScanner {
        policy,
        patterns,
        rust_has_tests,
        findings: Vec::new(),
        added_tests: 0,
        deleted_tests: 0,
    };

    let mut old_path: Option<String> = None;
    let mut new_path: Option<String> = None;
    let mut in_hunk = false;
    let mut old_remaining: u64 = 0;
    let mut new_line: u64 = 0;
    let mut new_remaining: u64 = 0;

    for line in diff_text.lines() {
        if in_hunk {
            if line.starts_with('\\') {
                continue;
            }
            if let Some(raw_content) = line.strip_prefix('+') {
                let content = raw_content.trim_end_matches('\r');
                let line_no = new_line;
                new_line += 1;
                new_remaining = new_remaining.saturating_sub(1);
                if let Some(ref file) = new_path {
                    scanner.inspect_added_line(file, line_no, content);
                }
                if old_remaining == 0 && new_remaining == 0 {
                    in_hunk = false;
                }
                continue;
            }
            if let Some(raw_content) = line.strip_prefix('-') {
                let content = raw_content.trim_end_matches('\r');
                old_remaining = old_remaining.saturating_sub(1);
                if let Some(ref file) = old_path {
                    scanner.inspect_deleted_line(file, content);
                }
                if old_remaining == 0 && new_remaining == 0 {
                    in_hunk = false;
                }
                continue;
            }
            if line.starts_with(' ') {
                new_line += 1;
                old_remaining = old_remaining.saturating_sub(1);
                new_remaining = new_remaining.saturating_sub(1);
                if old_remaining == 0 && new_remaining == 0 {
                    in_hunk = false;
                }
                continue;
            }
            in_hunk = false;
        }

        if line.starts_with("diff --git ") {
            old_path = None;
            new_path = None;
            in_hunk = false;
        } else if line == "--- /dev/null" {
            old_path = None;
        } else if let Some(rest) = line.strip_prefix("--- a/") {
            old_path = Some(normalize_diff_header_path(rest));
        } else if line == "+++ /dev/null" {
            new_path = None;
        } else if let Some(rest) = line.strip_prefix("+++ b/") {
            new_path = Some(normalize_diff_header_path(rest));
        } else if let Some((old_count, new_start, new_count)) =
            parse_hunk_header(&patterns.hunk_header, line)
        {
            old_remaining = old_count;
            new_line = new_start;
            new_remaining = new_count;
            in_hunk = old_remaining > 0 || new_remaining > 0;
        }
    }

    DiffScanResult {
        findings: scanner.findings,
        added_tests: scanner.added_tests,
        deleted_tests: scanner.deleted_tests,
    }
}

fn validate_argv(value: &Value) -> bool {
    value.as_array().is_some_and(|arr| {
        !arr.is_empty()
            && arr[0].as_str().is_some_and(|s| !s.is_empty())
            && arr
                .iter()
                .all(|item| item.as_str().is_some_and(|s| !s.contains('\0')))
    })
}

fn truncate_snippet(line: &str) -> String {
    let trimmed = line.trim();
    const MAX_CHARS: usize = 160;
    if trimmed.chars().count() <= MAX_CHARS {
        return trimmed.to_owned();
    }
    let mut out: String = trimmed.chars().take(MAX_CHARS).collect();
    out.push_str("...");
    out
}

fn short_sha(sha: &str) -> &str {
    sha.get(..12).unwrap_or(sha)
}

fn md_cell(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace(['\n', '\r'], " ")
        .replace('`', "'")
}

fn render_summary(audit: &Value) -> String {
    let verdict = audit["verdict"].as_str().unwrap_or("blocked");
    let badge = if verdict == "passed" {
        "PASSED"
    } else {
        "BLOCKED"
    };
    let base_sha = audit["base_sha"].as_str().unwrap_or("");
    let head_sha = audit["head_sha"].as_str().unwrap_or("");
    let diff_base_sha = audit["diff_base_sha"].as_str().unwrap_or("");
    let policy_sha = audit["base_policy_sha256"].as_str().unwrap_or("");
    let at = audit["at"].as_str().unwrap_or("");
    let policy_tamper = audit["policy_tamper_detected"].as_bool().unwrap_or(false);
    let policy_allowed = audit["policy_change_allowed"].as_bool().unwrap_or(false);
    let policy_files: &[Value] = audit["policy_tamper_files"]
        .as_array()
        .map_or(&[], Vec::as_slice);
    let findings: &[Value] = audit["test_tamper_findings"]
        .as_array()
        .map_or(&[], Vec::as_slice);
    let ev_checks: &[Value] = audit["evidence_checks"]
        .as_array()
        .map_or(&[], Vec::as_slice);
    let ev_failures: &[Value] = audit["evidence_failures"]
        .as_array()
        .map_or(&[], Vec::as_slice);

    let added = audit["test_counts"]["added"].as_u64().unwrap_or(0);
    let deleted = audit["test_counts"]["deleted"].as_u64().unwrap_or(0);
    let net_deleted = audit["test_counts"]["net_deleted"].as_u64().unwrap_or(0);
    let max_deleted = audit["test_counts"]["max_deleted_tests"]
        .as_u64()
        .unwrap_or(0);

    let policy_status = if policy_files.is_empty() {
        "Clean (0 protected files touched)".to_owned()
    } else if policy_tamper {
        format!("BLOCKED ({} protected file(s) touched)", policy_files.len())
    } else if policy_allowed {
        format!(
            "Allowed override ({} protected file(s) touched)",
            policy_files.len()
        )
    } else {
        format!("{} protected file(s) touched", policy_files.len())
    };

    let total_evidence = ev_checks.len() + ev_failures.len();
    let evidence_status = if total_evidence == 0 {
        "Not requested".to_owned()
    } else {
        format!("{}/{} passed", ev_checks.len(), total_evidence)
    };

    let mut out = String::new();
    let _ = writeln!(out, "## Kineti PR Verification Gate — {badge}\n");
    let _ = writeln!(out, "| Check | Result |");
    let _ = writeln!(out, "| --- | --- |");
    let _ = writeln!(out, "| Verdict | **{badge}** |");
    let _ = writeln!(
        out,
        "| Base Commit | `{}` (`{}`) |",
        short_sha(base_sha),
        md_cell(base_sha)
    );
    let _ = writeln!(
        out,
        "| Head Commit | `{}` (`{}`) |",
        short_sha(head_sha),
        md_cell(head_sha)
    );
    let _ = writeln!(out, "| Diff Base | `{}` |", short_sha(diff_base_sha));
    let _ = writeln!(out, "| Base Policy SHA-256 | `{}` |", md_cell(policy_sha));
    let _ = writeln!(
        out,
        "| Protected Policy Status | {} |",
        md_cell(&policy_status)
    );
    let _ = writeln!(out, "| Test Tampering Findings | {} |", findings.len());
    let _ = writeln!(
        out,
        "| Test Definition Delta | +{added} / -{deleted} (net deleted: {net_deleted}, max allowed: {max_deleted}) |"
    );
    let _ = writeln!(out, "| Evidence Checks | {} |", md_cell(&evidence_status));
    let _ = writeln!(out, "| Evaluated At (UTC) | `{}` |", md_cell(at));

    if !policy_files.is_empty() {
        let _ = writeln!(out, "\n### Protected Policy & Workflow Files Touched\n");
        for path in policy_files {
            if let Some(p) = path.as_str() {
                let _ = writeln!(out, "- `{}`", md_cell(p));
            }
        }
    }

    if !findings.is_empty() {
        let _ = writeln!(out, "\n### Test Tampering Findings\n");
        let _ = writeln!(out, "| File | Line | Rule | Snippet |");
        let _ = writeln!(out, "| --- | --- | --- | --- |");
        for item in findings {
            let file = item["file"].as_str().unwrap_or("");
            let line = item["line"].as_u64().unwrap_or(0);
            let line_str = if line == 0 {
                "-".to_owned()
            } else {
                line.to_string()
            };
            let rule = item["rule"].as_str().unwrap_or("");
            let snippet = item["snippet"].as_str().unwrap_or("");
            let _ = writeln!(
                out,
                "| `{}` | {} | `{}` | `{}` |",
                md_cell(file),
                line_str,
                md_cell(rule),
                md_cell(snippet)
            );
        }
    }

    if total_evidence > 0 {
        let _ = writeln!(out, "\n### Linked Evidence Checks\n");
        let _ = writeln!(
            out,
            "| Label | Status | Parser | Observed Counts | Details |"
        );
        let _ = writeln!(out, "| --- | --- | --- | --- | --- |");
        for ev in ev_checks {
            let label = ev["label"].as_str().unwrap_or("");
            let status = ev["status"].as_str().unwrap_or("fresh");
            let parser = ev["parser"].as_str().unwrap_or("unknown");
            let count_observed = ev["count_observed"].as_bool().unwrap_or(false);
            let counts = if count_observed {
                format!(
                    "{} passed / {} failed / {} skipped",
                    ev["passed"].as_u64().unwrap_or(0),
                    ev["failed"].as_u64().unwrap_or(0),
                    ev["skipped"].as_u64().unwrap_or(0)
                )
            } else {
                "exit-code only (count not observed)".to_owned()
            };
            let run_id = ev["run_id"].as_str().unwrap_or("");
            let _ = writeln!(
                out,
                "| `{}` | `{}` | `{}` | {} | run `{}` |",
                md_cell(label),
                md_cell(status),
                md_cell(parser),
                md_cell(&counts),
                md_cell(short_sha(run_id))
            );
        }
        for fail in ev_failures {
            let label = fail["label"].as_str().unwrap_or("");
            let code = fail["code"].as_i64().unwrap_or(1);
            let err = fail["error"].as_str().unwrap_or("evidence check failed");
            let _ = writeln!(
                out,
                "| `{}` | `FAILED (code {code})` | - | - | {} |",
                md_cell(label),
                md_cell(err)
            );
        }
    }

    let _ = writeln!(
        out,
        "\n> **Verification Boundary & Limits:** Policy was loaded from the base-branch `.kineti/gate.json` (`ci_runner_trust_boundary: true`). Signatures bind this audit bundle and linked receipts to this runner's issuer key; they do **not** prove independent test assertion quality (`independent_assertion_quality_proven: false`), automatic local interception (`automatic_interception: false`), or provider billing (`provider_billing: \"unknown\"`)."
    );

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../../.kineti/native-gate-tests")
                .join(store::nonce().unwrap());
            fs::create_dir_all(&path).unwrap();
            Self(path.canonicalize().unwrap())
        }

        fn write(&self, rel: &str, content: &str) {
            let full = self.0.join(rel);
            if let Some(parent) = full.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(full, content).unwrap();
        }

        fn git(&self, args: &[&str]) -> String {
            let out = Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args(args)
                .stdin(Stdio::null())
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&out.stderr)
            );
            String::from_utf8_lossy(&out.stdout).trim().to_owned()
        }

        fn commit(&self, message: &str) -> String {
            self.git(&["add", "-A"]);
            self.git(&[
                "-c",
                "user.name=Kineti Test",
                "-c",
                "user.email=test@example.invalid",
                "-c",
                "commit.gpgsign=false",
                "commit",
                "-m",
                message,
            ]);
            self.git(&["rev-parse", "HEAD"])
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn validates_and_rejects_git_refs() {
        for valid in [
            "main",
            "HEAD",
            "HEAD~1",
            "HEAD^2",
            "origin/main",
            "refs/heads/feat/path-a-pr-gate",
            "3ce824cb747c3fbbb0cd78d07a577b5c7374061d",
            "v0.7.0-pilot",
        ] {
            assert!(validate_git_ref(valid, "base_ref").is_ok(), "{valid}");
        }
        for invalid in [
            "",
            "-u",
            "--output=/tmp/pwn",
            "main..HEAD",
            "main:evil",
            "main HEAD",
            "HEAD\0foo",
            "/leading",
            "trailing/",
            "double//slash",
            "trailing.",
        ] {
            let err = validate_git_ref(invalid, "base_ref").unwrap_err();
            assert_eq!(err.0, 2, "{invalid}");
        }
    }

    #[test]
    fn glob_matching_supports_wildcards_and_double_star() {
        assert!(glob_match(
            ".github/workflows/**",
            ".github/workflows/ci.yml"
        ));
        assert!(glob_match(
            ".github/workflows/**",
            ".github/workflows/sub/release.yaml"
        ));
        assert!(glob_match("tests/**/*.py", "tests/auth_test.py"));
        assert!(glob_match("tests/**/*.py", "tests/unit/auth_test.py"));
        assert!(glob_match("tests/**/*.py", "tests/a/b/c_test.py"));
        assert!(!glob_match("tests/**/*.py", "tests_other/auth_test.py"));
        assert!(!glob_match("tests/*.py", "tests/sub/auth_test.py"));
        assert!(glob_match("**/jest.config.*", "jest.config.ts"));
        assert!(glob_match("**/jest.config.*", "apps/web/jest.config.mjs"));
        assert!(glob_match("test_?.py", "test_1.py"));
        assert!(!glob_match("test_?.py", "test_12.py"));
    }

    #[test]
    fn protected_path_detection_handles_defaults_and_custom_rules() {
        let custom = vec!["ci/scripts/".to_owned(), "configs/*.toml".to_owned()];
        assert!(is_protected_path(".kineti/gate.json", &custom));
        assert!(is_protected_path(".github/workflows/ci.yml", &custom));
        assert!(is_protected_path("pytest.ini", &custom));
        assert!(is_protected_path("services/api/pytest.ini", &custom));
        assert!(is_protected_path("jest.config.js", &custom));
        assert!(is_protected_path("packages/web/vitest.config.ts", &custom));
        assert!(is_protected_path("ci/scripts/gate.sh", &custom));
        assert!(is_protected_path("configs/runner.toml", &custom));

        assert!(!is_protected_path("Cargo.toml", &custom));
        assert!(!is_protected_path("package.json", &custom));
        assert!(!is_protected_path("src/lib.rs", &custom));
    }

    #[test]
    fn diff_scanner_detects_polyglot_skips_and_weakening_flags() {
        let diff = "\
diff --git a/tests/auth_test.rs b/tests/auth_test.rs
--- a/tests/auth_test.rs
+++ b/tests/auth_test.rs
@@ -10,0 +11,3 @@
+#[ignore]
+#[ignore = \"flaky\"]
+// #[ignore] comment should not trigger
diff --git a/tests/test_auth.py b/tests/test_auth.py
--- a/tests/test_auth.py
+++ b/tests/test_auth.py
@@ -4,0 +5,6 @@
+@pytest.mark.skip(reason=\"later\")
+@pytest.mark.skipif(True, reason=\"ci\")
+@pytest.mark.xfail
+@unittest.skip(\"todo\")
+    pytest.skip(\"dynamic\")
+# @pytest.mark.skip comment ignored
diff --git a/src/auth.test.ts b/src/auth.test.ts
--- a/src/auth.test.ts
+++ b/src/auth.test.ts
@@ -8,0 +9,5 @@
+test.skip(\"login\", () => {});
+it.only(\"focused\", () => {});
+describe.skip(\"suite\", () => {});
+it.todo(\"later\");
+// it.skip(\"comment\")
diff --git a/pkg/auth_test.go b/pkg/auth_test.go
--- a/pkg/auth_test.go
+++ b/pkg/auth_test.go
@@ -12,0 +13,4 @@
+\tt.Skip(\"skipping\")
+\tt.Skipf(\"skipping %s\", \"now\")
+\tt.SkipNow()
+\t// t.Skip(\"comment\")
diff --git a/package.json b/package.json
--- a/package.json
+++ b/package.json
@@ -5,1 +5,1 @@
-    \"test\": \"vitest run\"
+    \"test\": \"jest --passWithNoTests\"
diff --git a/scripts/run.sh b/scripts/run.sh
--- a/scripts/run.sh
+++ b/scripts/run.sh
@@ -2,0 +3,3 @@
+pytest -k \"\"
+pytest --ignore=tests/test_auth.py
+# pytest --passWithNoTests
";
        let policy = BasePolicy::default();
        let patterns = Patterns::new();
        let scan = scan_unified_diff(diff, &policy, &patterns, |_| false);

        let rules: Vec<(&str, u64, &str)> = scan
            .findings
            .iter()
            .map(|f| (f.file.as_str(), f.line, f.rule.as_str()))
            .collect();

        assert_eq!(
            rules,
            vec![
                ("tests/auth_test.rs", 11, "added_rust_ignore"),
                ("tests/auth_test.rs", 12, "added_rust_ignore"),
                ("tests/test_auth.py", 5, "added_python_skip"),
                ("tests/test_auth.py", 6, "added_python_skip"),
                ("tests/test_auth.py", 7, "added_python_skip"),
                ("tests/test_auth.py", 8, "added_python_skip"),
                ("tests/test_auth.py", 9, "added_python_skip"),
                ("src/auth.test.ts", 9, "added_js_skip_or_only"),
                ("src/auth.test.ts", 10, "added_js_skip_or_only"),
                ("src/auth.test.ts", 11, "added_js_skip_or_only"),
                ("src/auth.test.ts", 12, "added_js_skip_or_only"),
                ("pkg/auth_test.go", 13, "added_go_skip"),
                ("pkg/auth_test.go", 14, "added_go_skip"),
                ("pkg/auth_test.go", 15, "added_go_skip"),
                ("package.json", 5, "added_weakening_test_flag"),
                ("scripts/run.sh", 3, "added_weakening_test_flag"),
                ("scripts/run.sh", 4, "added_weakening_test_flag"),
            ]
        );
    }

    #[test]
    fn diff_scanner_detects_weakened_package_json_and_tracks_net_test_deletions() {
        let diff = "\
diff --git a/package.json b/package.json
--- a/package.json
+++ b/package.json
@@ -4,1 +4,1 @@
-    \"test\": \"vitest run\"
+    \"test\": \"vitest run || true\"
diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -20,3 +20,0 @@
-#[test]
-fn deleted_rust_test() {}
-#[tokio::test]
diff --git a/tests/test_api.py b/tests/test_api.py
--- a/tests/test_api.py
+++ b/tests/test_api.py
@@ -1,2 +1,1 @@
-def test_one():
-async def test_two():
+def test_replacement():
diff --git a/src/app.test.js b/src/app.test.js
--- a/src/app.test.js
+++ b/src/app.test.js
@@ -1,2 +1,1 @@
-it(\"a\", () => {});
-test(\"b\", () => {});
+test(\"c\", () => {});
diff --git a/pkg/api_test.go b/pkg/api_test.go
--- a/pkg/api_test.go
+++ b/pkg/api_test.go
@@ -1,1 +1,0 @@
-func TestEndpoint(t *testing.T) {}
";
        let policy = BasePolicy::default();
        let patterns = Patterns::new();
        let scan = scan_unified_diff(diff, &policy, &patterns, |p| p == "src/lib.rs");

        assert_eq!(scan.findings.len(), 1);
        assert_eq!(scan.findings[0].rule, "weakened_package_test_script");
        assert_eq!(scan.deleted_tests, 7);
        assert_eq!(scan.added_tests, 2);
        assert_eq!(scan.deleted_tests.saturating_sub(scan.added_tests), 5);
    }

    #[test]
    fn removing_skip_markers_is_allowed() {
        let diff = "\
diff --git a/tests/test_auth.py b/tests/test_auth.py
--- a/tests/test_auth.py
+++ b/tests/test_auth.py
@@ -5,1 +5,0 @@
-@pytest.mark.skip(reason=\"was flaky\")
diff --git a/tests/auth_test.rs b/tests/auth_test.rs
--- a/tests/auth_test.rs
+++ b/tests/auth_test.rs
@@ -8,1 +8,0 @@
-#[ignore]
";
        let policy = BasePolicy::default();
        let patterns = Patterns::new();
        let scan = scan_unified_diff(diff, &policy, &patterns, |_| false);
        assert!(scan.findings.is_empty());
        assert_eq!(scan.deleted_tests, 0);
        assert_eq!(scan.added_tests, 0);
    }

    #[test]
    fn missing_base_policy_fails_closed() {
        let fx = Fixture::new();
        fx.git(&["init"]);
        fx.git(&["checkout", "-B", "main"]);
        fx.write("README.md", "hello\n");
        let base_sha = fx.commit("no gate json");

        let err = run(
            &fx.0,
            PrCheckOpts {
                base_ref: base_sha,
                head_ref: "HEAD".to_owned(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);
        assert!(err.1.contains(".kineti/gate.json"));
    }

    #[test]
    fn end_to_end_pr_check_enforces_base_policy_tamper_and_signs_audit() {
        let fx = Fixture::new();
        fx.git(&["init"]);
        fx.git(&["checkout", "-B", "main"]);
        fx.write(
            ".kineti/gate.json",
            r#"{
  "version": 1,
  "required_checks": ["unit"],
  "commands": { "unit": ["pytest", "-q"] },
  "required_tests": ["tests/test_auth.py"],
  "max_deleted_tests": 0
}
"#,
        );
        fx.write(
            "tests/test_auth.py",
            "def test_login():\n    assert True\n\ndef test_logout():\n    assert True\n",
        );
        fx.write("src/auth.py", "def login():\n    return True\n");
        let base_sha = fx.commit("base commit");

        // 1. Clean PR passes and emits verifiable signed audit pack + summary.
        fx.git(&["checkout", "-b", "pr-clean"]);
        fx.write("src/auth.py", "def login():\n    return 'ok'\n");
        fx.write(
            "tests/test_auth.py",
            "def test_login():\n    assert True\n\ndef test_logout():\n    assert True\n\ndef test_refresh():\n    assert True\n",
        );
        let clean_head = fx.commit("clean pr");

        // Advance main with an unrelated workflow edit to confirm merge-base scoping.
        fx.git(&["checkout", "main"]);
        fx.write(".github/workflows/ci.yml", "name: ci\n");
        let advanced_main = fx.commit("main updates workflow");

        let audit = run(
            &fx.0,
            PrCheckOpts {
                base_ref: advanced_main.clone(),
                head_ref: clean_head,
                audit_out: Some(PathBuf::from(".kineti/pr-audit.json")),
                summary_out: Some(PathBuf::from(".kineti/pr-summary.md")),
                ..Default::default()
            },
        )
        .expect("clean PR should pass even after main advances");
        assert_eq!(audit["schema"], "kineti-pr-audit-v1");
        assert_eq!(audit["verdict"], "passed");
        assert_eq!(audit["policy_tamper_detected"], false);
        assert_eq!(audit["diff_base_sha"], base_sha);
        assert_eq!(audit["test_counts"]["added"], 1);
        assert_eq!(audit["test_counts"]["net_deleted"], 0);
        assert_eq!(audit["independent_assertion_quality_proven"], false);

        let pub_pem = fs::read_to_string(fx.0.join(".kineti/evidence-issuer.pem")).unwrap();
        let written_audit: Value =
            serde_json::from_str(&fs::read_to_string(fx.0.join(".kineti/pr-audit.json")).unwrap())
                .unwrap();
        assert!(verify_audit(&written_audit, &pub_pem));

        let mut tampered = written_audit.clone();
        tampered["verdict"] = json!("forged");
        assert!(!verify_audit(&tampered, &pub_pem));

        let summary = fs::read_to_string(fx.0.join(".kineti/pr-summary.md")).unwrap();
        assert!(summary.contains("PASSED"));
        assert!(summary.contains("independent_assertion_quality_proven: false"));

        // 2. Policy tamper PR is blocked even when PR tries to weaken .kineti/gate.json.
        fx.git(&["checkout", "main"]);
        fx.git(&["checkout", "-b", "pr-policy-tamper"]);
        fx.write(
            ".kineti/gate.json",
            r#"{"version":1,"required_checks":[],"max_deleted_tests":99}"#,
        );
        let policy_head = fx.commit("weaken gate");

        let err = run(
            &fx.0,
            PrCheckOpts {
                base_ref: advanced_main.clone(),
                head_ref: policy_head.clone(),
                audit_out: Some(PathBuf::from(".kineti/tamper-audit.json")),
                summary_out: Some(PathBuf::from(".kineti/tamper-summary.md")),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);
        assert!(err.1.contains("policy_tamper_detected=true"));
        let tamper_audit: Value = serde_json::from_str(
            &fs::read_to_string(fx.0.join(".kineti/tamper-audit.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(tamper_audit["verdict"], "blocked");
        assert_eq!(tamper_audit["policy_tamper_detected"], true);
        assert_eq!(
            tamper_audit["policy_tamper_files"],
            json!([".kineti/gate.json"])
        );
        assert!(verify_audit(&tamper_audit, &pub_pem));

        // Explicit --allow-policy-change permits the policy-only change.
        let allowed = run(
            &fx.0,
            PrCheckOpts {
                base_ref: advanced_main.clone(),
                head_ref: policy_head,
                allow_policy_change: true,
                ..Default::default()
            },
        )
        .expect("allow_policy_change should permit policy edit");
        assert_eq!(allowed["verdict"], "passed");
        assert_eq!(allowed["policy_tamper_detected"], false);
        assert_eq!(allowed["policy_tamper_files"], json!([".kineti/gate.json"]));

        // 3. Test deletion PR is blocked unless allow_test_reduction is set.
        fx.git(&["checkout", "main"]);
        fx.git(&["checkout", "-b", "pr-delete-test"]);
        fx.write("tests/test_auth.py", "def test_login():\n    assert True\n");
        let delete_head = fx.commit("delete test");

        let err = run(
            &fx.0,
            PrCheckOpts {
                base_ref: advanced_main.clone(),
                head_ref: delete_head.clone(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);

        let allowed_del = run(
            &fx.0,
            PrCheckOpts {
                base_ref: advanced_main.clone(),
                head_ref: delete_head,
                allow_test_reduction: true,
                ..Default::default()
            },
        )
        .expect("allow_test_reduction should permit net deletion");
        assert_eq!(allowed_del["verdict"], "passed");
        assert_eq!(allowed_del["test_counts"]["net_deleted"], 1);

        // 4. Deleting required test file is always blocked.
        fx.git(&["checkout", "main"]);
        fx.git(&["checkout", "-b", "pr-delete-file"]);
        fs::remove_file(fx.0.join("tests/test_auth.py")).unwrap();
        let rm_head = fx.commit("remove required test file");
        let err = run(
            &fx.0,
            PrCheckOpts {
                base_ref: advanced_main,
                head_ref: rm_head,
                allow_test_reduction: true,
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);
    }

    #[test]
    fn red_team_bypasses_are_blocked() {
        use std::os::unix::fs::symlink;
        // 1. Inline block comments + spaced/qualified Rust ignore attributes (Flaws #3 & #6)
        let diff = "\
diff --git a/tests/x.rs b/tests/x.rs
--- a/tests/x.rs
+++ b/tests/x.rs
@@ -1,0 +1,4 @@
+/* note */ #[ignore]
+#[ ignore ]
+#[core::prelude::v1::ignore]
+fn a() {}
diff --git a/tests/a.test.ts b/tests/a.test.ts
--- a/tests/a.test.ts
+++ b/tests/a.test.ts
@@ -1,0 +1,2 @@
+/* note */ it.only(\"x\", () => {});
+/* note */ test.skip(\"y\", () => {});
diff --git a/tests/foo_test.go b/tests/foo_test.go
--- a/tests/foo_test.go
+++ b/tests/foo_test.go
@@ -1,0 +1,1 @@
+/* note */ t.Skip(\"flaky\")
diff --git a/tests/my test.py b/tests/my test.py
--- a/tests/my test.py\t
+++ b/tests/my test.py\t
@@ -1,1 +1,2 @@
-def test_old():
+@pytest.mark.skip
+def test_new():
";
        let res = scan_unified_diff(diff, &BasePolicy::default(), &Patterns::new(), |_| true);
        let rules: Vec<&str> = res.findings.iter().map(|f| f.rule.as_str()).collect();
        assert_eq!(
            rules.iter().filter(|&&r| r == "added_rust_ignore").count(),
            3,
            "rules={rules:?}"
        );
        assert_eq!(
            rules
                .iter()
                .filter(|&&r| r == "added_js_skip_or_only")
                .count(),
            2,
            "rules={rules:?}"
        );
        assert!(rules.contains(&"added_go_skip"), "rules={rules:?}");
        assert!(rules.contains(&"added_python_skip"), "rules={rules:?}");
        assert_eq!(res.deleted_tests, 1);
        assert_eq!(res.added_tests, 1);

        // 2. diff.noprefix=true & output path escape via symlink / outside absolute path (Flaws #2 & #4)
        let fx = Fixture::new();
        fx.git(&["init", "-b", "main"]);
        fx.write(
            ".kineti/gate.json",
            &json!({"version":1,"required_checks":["unit"],"commands":{"unit":["/bin/sh","-c","exit 0"]}})
                .to_string(),
        );
        fx.write("tests/my test.py", "def test_a():\n    assert True\n");
        let base = fx.commit("base");
        fx.git(&["config", "diff.noprefix", "true"]);
        fx.git(&["checkout", "-b", "tamper-noprefix"]);
        fx.write(
            "tests/my test.py",
            "@pytest.mark.skip\ndef test_a():\n    assert True\n",
        );
        let head = fx.commit("tamper with noprefix and spaced filename");

        let err = run(
            &fx.0,
            PrCheckOpts {
                base_ref: base.clone(),
                head_ref: head.clone(),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err.0, 1);
        assert!(err.1.contains("tamper_findings=1"), "err={}", err.1);

        // Symlinked output directory inside repo -> MUST fail closed (exit 3)
        let outside = Fixture::new();
        symlink(&outside.0, fx.0.join("outlink")).unwrap();
        let err_link = run(
            &fx.0,
            PrCheckOpts {
                base_ref: base.clone(),
                head_ref: base.clone(),
                audit_out: Some(PathBuf::from("outlink/escaped.json")),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err_link.0, 3);
        assert!(!outside.0.join("escaped.json").exists());

        // Absolute output path outside repo -> MUST fail closed (exit 2)
        let err_abs = run(
            &fx.0,
            PrCheckOpts {
                base_ref: base.clone(),
                head_ref: base,
                audit_out: Some(outside.0.join("abs.json")),
                ..Default::default()
            },
        )
        .unwrap_err();
        assert_eq!(err_abs.0, 2);
        assert!(!outside.0.join("abs.json").exists());
    }

    #[test]
    fn block_comment_closing_line_tamper_is_detected() {
        let diff = "\
diff --git a/tests/block_test.rs b/tests/block_test.rs
--- a/tests/block_test.rs
+++ b/tests/block_test.rs
@@ -1,0 +1,4 @@
+ * doc comment mentioning #[ignore] only
+ */ #[ignore]
+ * end */ #[ignore]
+fn test_a() {}
diff --git a/tests/block.test.js b/tests/block.test.js
--- a/tests/block.test.js
+++ b/tests/block.test.js
@@ -1,0 +1,3 @@
+ * doc comment mentioning it.skip('a')
+ */ it.skip('hidden', () => {});
+it.skip('*/', () => {});
diff --git a/pkg/block_test.go b/pkg/block_test.go
--- a/pkg/block_test.go
+++ b/pkg/block_test.go
@@ -1,0 +1,2 @@
+ * doc t.Skip(\"x\")
+ */ t.Skip(\"hidden\")
";
        let res = scan_unified_diff(diff, &BasePolicy::default(), &Patterns::new(), |_| true);
        let rules: Vec<&str> = res.findings.iter().map(|f| f.rule.as_str()).collect();
        assert_eq!(
            rules.iter().filter(|&&r| r == "added_rust_ignore").count(),
            2,
            "rules={rules:?}"
        );
        assert_eq!(
            rules
                .iter()
                .filter(|&&r| r == "added_js_skip_or_only")
                .count(),
            2,
            "rules={rules:?}"
        );
        assert_eq!(
            rules.iter().filter(|&&r| r == "added_go_skip").count(),
            1,
            "rules={rules:?}"
        );
    }
}
