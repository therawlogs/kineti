//! Durable undo intent and completion. An uncertain action is never replayed blindly.
use crate::{
    compat::digest,
    receipt,
    store::{self, Error, Lock, Result},
};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

fn records(root: &Path) -> Result<Vec<Value>> {
    let path = root.join(".kineti/saga.jsonl");
    let original = store::lines(&path)?;
    let guard = root.join(".kineti/saga.guard.json");
    if path.exists() && guard.exists() {
        let g = store::read(&guard)?;
        if let Some(hash) = g["sha256"].as_str() {
            if digest(fs::read(&path)?) != hash {
                return store::fail(3, "legacy undo guard differs; inspected recovery required");
            }
        }
    }
    for r in &original {
        if r["kind"] == "register"
            && r.get("inverse_hash").is_some()
            && r["inverse_hash"]
                != digest(
                    r["inverse"]
                        .as_str()
                        .ok_or_else(|| Error(3, "legacy inverse missing".into()))?,
                )
        {
            return store::fail(3, "legacy undo hash differs");
        }
    }
    let mut all = original;
    all.extend(store::lines(&root.join(".kineti/undo.native.jsonl"))?);
    Ok(all)
}
fn append(root: &Path, v: Value) -> Result<()> {
    store::append(&root.join(".kineti/undo.native.jsonl"), &v)
}
/// Open or seal a named run, retaining the saga interface.
pub fn transaction(root: &Path, run: &str, commit: bool) -> Result<Value> {
    if run.is_empty() {
        return store::fail(2, "--run-id required");
    }
    store::state(root)?;
    let _lock = Lock::take(&root.join(".kineti/undo.lock"))?;
    let all = records(root)?;
    if all
        .iter()
        .any(|r| r["run_id"] == run && r["kind"] == "commit")
    {
        return store::fail(2, "undo run already committed");
    }
    if commit && !all.iter().any(|r| r["run_id"] == run) {
        return store::fail(2, "unknown undo run");
    }
    append(
        root,
        json!({"kind":if commit{"commit"}else{"begin"},"at":store::now(),"run_id":run}),
    )?;
    Ok(json!({"run_id":run,"committed":commit}))
}
/// Register an exact argv inverse; string aliases are parsed without a shell.
pub fn push(root: &Path, label: &str, run: &str, argv: Vec<String>) -> Result<Value> {
    if label.is_empty()
        || run.is_empty()
        || argv.is_empty()
        || argv[0].is_empty()
        || argv.iter().any(|v| v.contains('\0'))
    {
        return store::fail(2, "undo registration requires label, run and exact argv");
    }
    store::state(root)?;
    let _lock = Lock::take(&root.join(".kineti/undo.lock"))?;
    let all = records(root)?;
    if all
        .iter()
        .any(|r| r["run_id"] == run && r["kind"] == "commit")
    {
        return store::fail(2, "undo run already committed");
    }
    if all
        .iter()
        .any(|r| r["run_id"] == run && r["label"] == label && r["kind"] == "register")
    {
        return store::fail(2, "undo label already registered in run");
    }
    let inverse = serde_json::to_string(&argv)?;
    append(
        root,
        json!({"kind":"register","at":store::now(),"run_id":run,"label":label,"inverse":inverse,"inverse_format":"argv-v1","inverse_hash":digest(&inverse)}),
    )?;
    Ok(json!({"registered":label,"run_id":run}))
}
/// Inspect or execute newest-first, continuing only after definite completion.
pub fn rollback(root: &Path, run: &str, confirmed: bool) -> Result<Value> {
    store::state(root)?;
    let _lock = Lock::take(&root.join(".kineti/undo.lock"))?;
    let all = records(root)?;
    let mut intent = BTreeSet::new();
    let mut complete = BTreeSet::new();
    let mut resolution = BTreeMap::new();
    let mut done = BTreeSet::new();
    for r in &all {
        if r["run_id"] != run {
            continue;
        }
        let id = r["action_id"].as_str().unwrap_or("").to_owned();
        match r["kind"].as_str() {
            Some("intent") => {
                intent.insert(id);
            }
            Some("completion") => {
                complete.insert(id);
                if r["exit_code"] == 0 {
                    done.insert(r["label"].as_str().unwrap_or("").to_owned());
                }
            }
            Some("inspection") => {
                resolution.insert(id, r["outcome"].as_str().unwrap_or("").to_owned());
                if r["outcome"] == "applied" {
                    done.insert(r["label"].as_str().unwrap_or("").to_owned());
                }
            }
            Some("rollback_step") if r["exit_code"] == 0 => {
                done.insert(r["label"].as_str().unwrap_or("").to_owned());
            }
            Some("commit") => return store::fail(2, "undo run is committed"),
            _ => {}
        }
    }
    for id in intent.difference(&complete) {
        if !resolution.contains_key(id) {
            return store::fail(
                3,
                format!("undo action {id} has uncertain completion; inspect before --resolve"),
            );
        }
    }
    let pending: Vec<_> = all
        .iter()
        .filter(|r| {
            r["run_id"] == run
                && r["kind"] == "register"
                && !done.contains(r["label"].as_str().unwrap_or(""))
        })
        .rev()
        .collect();
    if !confirmed {
        return Ok(
            json!({"preview":pending,"requires_confirmation":"--yes","identity_proven":false}),
        );
    }
    let mut failed = vec![];
    for r in pending {
        let inverse = r["inverse"]
            .as_str()
            .ok_or_else(|| Error(3, "inverse missing".into()))?;
        if r.get("inverse_hash").is_some() && r["inverse_hash"] != digest(inverse) {
            return store::fail(3, "undo inverse was altered");
        }
        let argv: Vec<String> = if r["inverse_format"] == "argv-v1" {
            serde_json::from_str(inverse)?
        } else {
            shell_words::split(inverse).map_err(|e| Error(3, e.to_string()))?
        };
        if argv.is_empty() || argv[0].is_empty() {
            return store::fail(3, "empty inverse");
        }
        let id = store::nonce()?;
        let label = r["label"].clone();
        append(
            root,
            json!({"kind":"intent","at":store::now(),"run_id":run,"label":label,"action_id":id,"inverse_hash":digest(inverse)}),
        )?;
        match receipt::execute(root, &argv, 60) {
            Ok((Some(code), _, _)) => {
                append(
                    root,
                    json!({"kind":"completion","at":store::now(),"run_id":run,"label":label,"action_id":id,"exit_code":code}),
                )?;
                if code != 0 {
                    failed.push(label);
                }
            }
            _ => {
                return store::fail(
                    3,
                    format!("undo action {id} completion uncertain; inspect before replay"),
                )
            }
        }
    }
    if !failed.is_empty() {
        return store::fail(
            1,
            format!("rollback incomplete; failed labels: {}", json!(failed)),
        );
    }
    Ok(json!({"rollback":"complete"}))
}
/// Explicit audited inspection resolution; never claim the confirmation proves a human.
pub fn resolve(root: &Path, run: &str, id: &str, outcome: &str, confirmed: bool) -> Result<Value> {
    if !confirmed || !["applied", "not-applied"].contains(&outcome) {
        return store::fail(
            2,
            "inspection requires --yes and --outcome applied|not-applied",
        );
    }
    let _lock = Lock::take(&root.join(".kineti/undo.lock"))?;
    let all = records(root)?;
    let intent = all
        .iter()
        .find(|r| r["kind"] == "intent" && r["run_id"] == run && r["action_id"] == id)
        .ok_or_else(|| Error(2, "unknown intent".into()))?;
    if all
        .iter()
        .any(|r| r["kind"] == "completion" && r["action_id"] == id)
    {
        return store::fail(2, "action already has definite completion");
    }
    append(
        root,
        json!({"kind":"inspection","at":store::now(),"run_id":run,"label":intent["label"],"action_id":id,"outcome":outcome,"identity_proven":false}),
    )?;
    Ok(json!({"resolved":id,"outcome":outcome}))
}
