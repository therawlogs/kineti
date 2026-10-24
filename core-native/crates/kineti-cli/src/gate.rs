//! Participating CI checks and declared command trust.
use crate::{
    compat::digest,
    receipt, spend,
    store::{self, Error, Result},
};
use serde_json::{json, Value};
use std::{
    fs,
    io::{self, IsTerminal, Write},
    path::Path,
};
/// Check state, reported spending and every required exact-command receipt.
pub fn ci(
    root: &Path,
    required: Vec<String>,
    commands: Option<Value>,
    max_age: u64,
) -> Result<Value> {
    let state = store::state(root).map_err(|e| Error(1, e.1))?;
    spend::inspect(root, true).map_err(|e| Error(1, e.1))?;
    let config = if root.join(".kineti/gate.json").exists() {
        store::read(&root.join(".kineti/gate.json"))?
    } else {
        json!({})
    };
    let labels = if required.is_empty() {
        config["required_checks"]
            .as_array()
            .ok_or_else(|| {
                Error(
                    1,
                    "required checks missing; configure them explicitly".into(),
                )
            })?
            .iter()
            .map(|v| {
                v.as_str()
                    .map(String::from)
                    .ok_or_else(|| Error(1, "required check label invalid".into()))
            })
            .collect::<Result<Vec<_>>>()?
    } else {
        required
    };
    if labels.is_empty() {
        return store::fail(
            1,
            "required checks empty; configure at least one required label",
        );
    }
    let stage = state["stage"]
        .as_u64()
        .or_else(|| state["current_stage"].as_u64())
        .ok_or_else(|| {
            Error(
                1,
                "project stage missing; preview init for missing fields".into(),
            )
        })?;
    if stage >= 7 && state["gates"]["spec"] == "fail" {
        return store::fail(1, "spec approval failed");
    }
    if state["gates"]["security"] == "fail" || stage >= 11 && state["gates"]["security"] != "pass" {
        return store::fail(1, "security gate must pass before ship");
    }
    let mut checks = vec![];
    for label in labels {
        let expected = commands
            .as_ref()
            .and_then(|m| m.get(&label))
            .or_else(|| config.get("commands").and_then(|m| m.get(&label)));
        if let Some(v) = expected {
            if v.as_array()
                .is_none_or(|a| a.is_empty() || a.iter().any(|v| !v.is_string()))
            {
                return store::fail(2, "required command must be a nonempty argv array");
            }
        }
        if commands.is_some() && expected.is_none() {
            return store::fail(1, "required command mapping missing");
        }
        let check = receipt::check(root, &label, max_age, expected).map_err(|e| {
            Error(
                1,
                format!("required check {label}: {} (evidence code {})", e.1, e.0),
            )
        })?;
        checks.push(json!({"label":label,"status":check["status"]}));
    }
    Ok(
        json!({"checks_passed":true,"checks":checks,"participating_ci_only":true,"provider_billing":"unknown"}),
    )
}
fn declared(root: &Path) -> Result<Vec<String>> {
    let config = store::read(&root.join("kineti.config.json")).map_err(|_| {
        Error(
            9,
            "verify declaration missing; configure settings.verify_command".into(),
        )
    })?;
    let value = &config["settings"]["verify_command"];
    let argv: Vec<String> = if let Some(s) = value.as_str() {
        shell_words::split(s).map_err(|e| Error(2, e.to_string()))?
    } else {
        serde_json::from_value(value.clone())
            .map_err(|_| Error(9, "verify command missing or malformed".into()))?
    };
    if argv.is_empty() || argv[0].is_empty() {
        return store::fail(9, "verify command missing");
    }
    Ok(argv)
}
/// Trust is scoped to the project and exact declared argv, not universal interception.
pub fn verify(root: &Path, mode: &str) -> Result<Value> {
    store::state(root)?;
    let argv = declared(root)?;
    let hash = digest(serde_json::to_vec(&argv)?);
    let p = root.join(".kineti/verify-trust.json");
    if mode == "trust" {
        if !io::stdin().is_terminal() {
            return store::fail(
                2,
                "verify --trust requires an interactive terminal and exact hash confirmation",
            );
        }
        eprintln!(
            "Declared argv: {}\nType full hash to confirm: {hash}",
            json!(argv)
        );
        io::stderr().flush()?;
        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        if s.trim() != hash {
            return store::fail(2, "trust confirmation did not match");
        }
        store::write(
            &p,
            &json!({"argv":argv,"sha256":hash,"at":store::now(),"identity_proven":false}),
        )?;
        return Ok(json!({"trusted":true,"identity_proven":false}));
    }
    let trusted = fs::read(&p)
        .ok()
        .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        .is_some_and(|v| v["sha256"] == hash && v["argv"] == json!(argv));
    if mode == "status" {
        return Ok(json!({"declared":argv,"trusted":trusted}));
    }
    if !trusted {
        return store::fail(
            9,
            "verify command not trusted; review and run kineti verify --trust",
        );
    }
    spend::inspect(root, true)?;
    let (code, _, truncated) = receipt::execute(root, &argv, 600)?;
    if code != Some(0) || truncated {
        return store::fail(1, "declared verification command failed");
    }
    Ok(json!({"verify":"passed","argv":argv}))
}
