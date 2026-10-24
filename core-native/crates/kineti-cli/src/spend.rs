//! Locked integer accounting. No provider-bill verification or guessed costs.
use crate::{
    compat::{canonical, digest},
    store::{self, Error, Lock, Result},
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};

/// Parse USD to legacy-compatible millionths of a dollar using decimal arithmetic.
pub fn money(text: &str) -> Result<u64> {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if whole.is_empty()
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || fraction.len() > 6
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return store::fail(2, "USD must be nonnegative decimal with at most six places");
    }
    let f = if fraction.is_empty() {
        0
    } else {
        fraction
            .parse::<u64>()
            .map_err(|_| Error(2, "invalid USD fraction".into()))?
    };
    let n = whole
        .parse::<u64>()
        .ok()
        .and_then(|w| w.checked_mul(1_000_000))
        .and_then(|w| w.checked_add(f * 10u64.pow(6 - fraction.len() as u32)))
        .ok_or_else(|| Error(2, "money out of range".into()))?;
    if n > 9_007_199_254_740_991 {
        return store::fail(2, "money exceeds exact supported integer range");
    }
    Ok(n)
}
fn amount(v: &Value) -> Result<u64> {
    money(&v.to_string()).map_err(|_| Error(3, "invalid historical monetary amount".into()))
}
fn integer(v: &Value) -> Result<u64> {
    v.as_u64()
        .filter(|n| *n <= 9_007_199_254_740_991)
        .ok_or_else(|| Error(3, "invalid integer accounting".into()))
}

/// Reconciled accounting view. Unknown entries remain explicitly blocking.
pub struct Account {
    /// Total in millionths of USD.
    pub total: u64,
    /// Per-stage totals.
    pub stages: BTreeMap<String, u64>,
    /// Ledger head digest.
    pub head: String,
    /// Count of native operations.
    pub count: u64,
    /// Global ceiling.
    pub global: u64,
    /// Default stage ceiling.
    pub stage_default: u64,
    /// Stage overrides.
    pub stage_limits: BTreeMap<String, u64>,
    /// Legacy trip flag.
    pub tripped: bool,
}
fn legacy(root: &Path) -> Result<(u64, BTreeMap<String, u64>, bool)> {
    let recovered = root.join(".kineti/spend.recovery-view.json");
    if recovered.exists() {
        let view = store::read(&recovered)?;
        for key in ["original_sources", "candidate_sources"] {
            for src in view[key]
                .as_array()
                .ok_or_else(|| Error(3, "recovery source inventory missing".into()))?
            {
                let p = store::input(
                    root,
                    src["path"]
                        .as_str()
                        .ok_or_else(|| Error(3, "recovery path missing".into()))?,
                )?;
                if digest(fs::read(p)?) != src["sha256"].as_str().unwrap_or("") {
                    return store::fail(3, "accounting source changed after reviewed recovery");
                }
            }
        }
        let backup = store::input(
            root,
            view["verified_source"]
                .as_str()
                .ok_or_else(|| Error(3, "recovery source missing".into()))?,
        )?;
        if backup == root || backup.join(".kineti/spend.recovery-view.json").exists() {
            return store::fail(3, "recursive recovery source");
        }
        return legacy(&backup);
    }
    let p = root.join(".kineti/spend.json");
    let log = root.join(".kineti/spend.log.jsonl");
    if !p.exists() {
        if log.exists() || root.join(".kineti/spend.initialized.json").exists() {
            return store::fail(
                3,
                "missing initialized accounting; reviewed recovery required",
            );
        }
        return Ok((0, BTreeMap::new(), false));
    }
    let s = store::read(&p)?;
    let entries = store::lines(&log)?;
    let mut total = 0u64;
    let mut stages = BTreeMap::<String, u64>::new();
    for e in &entries {
        let stage = e["stage"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error(3, "accounting stage missing".into()))?;
        for k in ["tokens_in", "tokens_out"] {
            if e[k].as_f64().is_none_or(|v| !v.is_finite() || v < 0.0) {
                return store::fail(3, "invalid historical token count");
            }
        }
        let usd = amount(&e["usd"])?;
        let m = if e.get("microcents").is_some() {
            integer(&e["microcents"])?
        } else {
            usd
        };
        if (m as u128 + 5_000) / 10_000 != (usd as u128 + 5_000) / 10_000 {
            return store::fail(3, "ledger integer/USD mismatch");
        }
        total = total
            .checked_add(m)
            .ok_or_else(|| Error(3, "accounting overflow".into()))?;
        let st = stages.entry(stage.into()).or_default();
        *st = st
            .checked_add(m)
            .ok_or_else(|| Error(3, "stage overflow".into()))?;
    }
    if s["entries"].as_u64() != Some(entries.len() as u64)
        || (amount(&s["total_usd"])? as u128 + 5_000) / 10_000 != (total as u128 + 5_000) / 10_000
        || s.get("total_microcents")
            .is_some_and(|v| v.as_u64() != Some(total))
    {
        return store::fail(3, "ledger/state totals differ; reviewed recovery required");
    }
    let by = s["by_stage"]
        .as_object()
        .ok_or_else(|| Error(3, "stage totals missing".into()))?;
    for (k, v) in by {
        if (stages.get(k).copied().unwrap_or(0) as u128 + 5_000) / 10_000
            != (amount(v)? as u128 + 5_000) / 10_000
        {
            return store::fail(3, "stage accounting differs");
        }
    }
    for (k, v) in &stages {
        if (by.get(k).map(amount).transpose()?.unwrap_or(0) as u128 + 5_000) / 10_000
            != (*v as u128 + 5_000) / 10_000
        {
            return store::fail(3, "stage missing from state");
        }
    }
    if let Some(by) = s.get("by_stage_microcents") {
        for (k, v) in by
            .as_object()
            .ok_or_else(|| Error(3, "malformed integer stages".into()))?
        {
            if integer(v)? != stages.get(k).copied().unwrap_or(0) {
                return store::fail(3, "integer stage totals differ");
            }
        }
    }
    Ok((
        total,
        stages,
        s["tripped"]
            .as_bool()
            .ok_or_else(|| Error(3, "trip state malformed".into()))?,
    ))
}
fn base_digest(root: &Path) -> Result<String> {
    let mut a = vec![];
    for p in ["spend.json", "spend.log.jsonl", "spend.initialized.json"] {
        let p = root.join(".kineti").join(p);
        a.push(if p.exists() {
            digest(fs::read(p)?)
        } else {
            "missing".into()
        });
    }
    Ok(digest(a.join(":")))
}
/// Inspect under the caller's spend lock. Every monetary reservation/unknown blocks.
pub fn load(root: &Path) -> Result<Account> {
    store::state(root)?;
    for record in store::lines(&root.join(".kineti/journal.jsonl"))? {
        if record["type"] == "run-record"
            && (record["data"]["costs_usd"].is_null()
                || record["data"]["costs_usd"] == 0
                    && record["data"]["cost_basis"]
                        .as_str()
                        .is_some_and(|s| s.contains("unavailable")))
        {
            return store::fail(3,format!("unresolved historical cost {} remains recorded; any implementation-only subscription exception does not authorize publication",record["id"]));
        }
    }
    for p in ["spend.reservations.jsonl", "spend.unknown.jsonl"] {
        let entries = store::lines(&root.join(".kineti").join(p))?;
        let mut unresolved = BTreeMap::new();
        for e in entries {
            let id = e["id"]
                .as_str()
                .ok_or_else(|| Error(3, "accounting record id missing".into()))?
                .to_owned();
            unresolved.insert(id, e);
        }
        if unresolved.values().any(|e| e["status"] != "resolved") {
            return store::fail(3, "unresolved accounting reservation or unknown cost");
        }
    }
    let (total, stages, tripped) = legacy(root)?;
    let config = if root.join("kineti.config.json").exists() {
        store::read(&root.join("kineti.config.json"))?
    } else {
        json!({})
    };
    let mut global = match config.pointer("/settings/spend_limit_usd/global") {
        Some(v) => amount(v)?,
        None => 50_000_000,
    };
    let mirror = root.join(".kineti/mirror.json");
    if mirror.exists() {
        global = amount(&store::read(&mirror)?["ceiling"])?;
    }
    let stage_default = match config.pointer("/settings/spend_limit_usd/per_stage_default") {
        Some(v) => amount(v)?,
        None => 10_000_000,
    };
    if global == 0 || stage_default == 0 {
        return store::fail(3, "zero accounting ceiling refused");
    }
    let mut stage_limits = BTreeMap::new();
    if let Some(m) = config.pointer("/settings/spend_limit_usd/per_stage") {
        for (k, v) in m
            .as_object()
            .ok_or_else(|| Error(3, "stage limits malformed".into()))?
        {
            let n = amount(v)?;
            if n == 0 {
                return store::fail(3, "zero stage ceiling refused");
            }
            stage_limits.insert(k.clone(), n);
        }
    }
    let mut a = Account {
        total,
        stages,
        head: base_digest(root)?,
        count: 0,
        global,
        stage_default,
        stage_limits,
        tripped,
    };
    let log = root.join(".kineti/spend.native.jsonl");
    let anchor = root.join(".kineti/spend.native.anchor.json");
    for e in store::lines(&log)? {
        let mut plain = e.clone();
        plain
            .as_object_mut()
            .ok_or_else(|| Error(3, "invalid native accounting record".into()))?
            .shift_remove("hash");
        if e["prev_hash"] != a.head
            || e["hash"] != digest(canonical(&plain).map_err(|e| Error(3, e))?)
        {
            return store::fail(3, "native accounting chain damaged");
        }
        if e["kind"] == "charge" {
            let n = integer(&e["microcents"])?;
            let stage = e["stage"]
                .as_str()
                .ok_or_else(|| Error(3, "stage missing".into()))?;
            a.total = a
                .total
                .checked_add(n)
                .filter(|n| *n <= 9_007_199_254_740_991)
                .ok_or_else(|| Error(3, "accounting overflow".into()))?;
            let st = a.stages.entry(stage.into()).or_default();
            *st = st
                .checked_add(n)
                .ok_or_else(|| Error(3, "stage overflow".into()))?;
        } else if e["kind"] == "reset" {
            a.global = integer(&e["global_microcents"])?;
            a.tripped = false;
        } else {
            return store::fail(3, "unknown native accounting operation");
        }
        a.head = e["hash"].as_str().unwrap().into();
        a.count += 1;
    }
    if root.join(".kineti/spend.native.initialized").exists() && (!log.exists() || !anchor.exists())
    {
        return store::fail(
            3,
            "native accounting history missing; reviewed recovery required",
        );
    }
    if log.exists() || anchor.exists() {
        let marker = store::read(&anchor)?;
        if marker["head"] != a.head || marker["count"] != a.count {
            return store::fail(
                3,
                "interrupted accounting commit; inspect and use reviewed recovery",
            );
        }
    }
    Ok(a)
}
fn threshold(limit: u64) -> u64 {
    (limit as u128 * 95).div_ceil(100) as u64
}
/// Require remaining reported headroom.
pub fn headroom(a: &Account) -> Result<()> {
    if a.tripped
        || a.total >= threshold(a.global)
        || a.stages
            .iter()
            .any(|(k, v)| *v >= threshold(*a.stage_limits.get(k).unwrap_or(&a.stage_default)))
    {
        return store::fail(
            3,
            "reported spending reached 95% stop; reset without headroom cannot pass",
        );
    }
    Ok(())
}
/// A locked status/check, including effective ceilings.
pub fn inspect(root: &Path, check: bool) -> Result<Value> {
    let _lock = Lock::take(&root.join(".kineti/spend.lock"))?;
    let a = load(root)?;
    if check {
        headroom(&a)?;
    }
    Ok(
        json!({"total_microcents":a.total,"global_microcents":a.global,"stop_microcents":threshold(a.global),"headroom_microcents":threshold(a.global).saturating_sub(a.total),"by_stage_microcents":a.stages,"provider_billing":"unknown","tripped":a.tripped}),
    )
}
fn commit(root: &Path, a: &Account, mut e: Value) -> Result<()> {
    if !root.join(".kineti/spend.native.initialized").exists() {
        store::atomic(&root.join(".kineti/spend.native.initialized"), b"1\n")?;
    }
    e["prev_hash"] = json!(a.head);
    e["at"] = json!(store::now());
    e["hash"] = json!(digest(canonical(&e).map_err(|e| Error(3, e))?));
    store::append(&root.join(".kineti/spend.native.jsonl"), &e)?;
    store::write(
        &root.join(".kineti/spend.native.anchor.json"),
        &json!({"head":e["hash"],"count":a.count+1}),
    )
}
/// Record an explicitly reported amount, never infer provider prices.
pub fn charge(
    root: &Path,
    stage: &str,
    model: &str,
    tin: u64,
    tout: u64,
    usd: &str,
) -> Result<Value> {
    let n = money(usd)?;
    if stage.is_empty()
        || model.is_empty()
        || tin > 9_007_199_254_740_991
        || tout > 9_007_199_254_740_991
    {
        return store::fail(2, "stage and model required");
    }
    let _lock = Lock::take(&root.join(".kineti/spend.lock"))?;
    let a = load(root)?;
    headroom(&a)?;
    if a.total
        .checked_add(n)
        .is_none_or(|v| v > 9_007_199_254_740_991)
    {
        return store::fail(
            2,
            "reported amount exceeds exact accounting range; no entry written",
        );
    }
    commit(
        root,
        &a,
        json!({"kind":"charge","stage":stage,"model":model,"tokens_in":tin,"tokens_out":tout,"microcents":n,"pricing_mode":"reported"}),
    )?;
    let after = load(root)?;
    headroom(&after)?;
    Ok(json!({"recorded_microcents":n,"total_microcents":after.total}))
}
/// Preserve confirmation interface without claiming agents cannot supply it.
pub fn reset(root: &Path, confirmed: bool, extend: &str) -> Result<Value> {
    if !confirmed {
        return store::fail(2,"reset requires --i-am-human audited confirmation; this flag does not prove human identity");
    }
    let increase = money(extend)?;
    let _lock = Lock::take(&root.join(".kineti/spend.lock"))?;
    let mut a = load(root)?;
    let global = a
        .global
        .checked_add(increase)
        .filter(|n| *n <= 1_000_000_000)
        .ok_or_else(|| Error(2, "ceiling outside supported range".into()))?;
    a.global = global;
    a.tripped = false;
    headroom(&a)?;
    commit(
        root,
        &a,
        json!({"kind":"reset","global_microcents":global,"confirmation":"--i-am-human","identity_proven":false,"previous_global_microcents":global-increase}),
    )?;
    Ok(json!({"global_microcents":global,"identity_proven":false}))
}
/// A non-mutating exact recovery proposal. Never invent reconstructed entries.
pub fn recovery_proposal(root: &Path) -> Result<Value> {
    let mut sources = vec![];
    for name in [
        "spend.json",
        "spend.log.jsonl",
        "spend.initialized.json",
        "spend.native.jsonl",
        "spend.native.anchor.json",
        "spend.reservations.jsonl",
        "spend.unknown.jsonl",
        "journal.jsonl",
    ] {
        let p = root.join(".kineti").join(name);
        if p.exists() {
            sources.push(json!({"path":format!(".kineti/{name}"),"sha256":digest(fs::read(p)?)}));
        }
    }
    let result = load(root);
    Ok(
        json!({"version":1,"kind":"accounting-recovery-proposal","sources":sources,"error":result.as_ref().err().map(|e|e.1.clone()),"known_total_microcents":result.ok().map(|a|a.total),"proposed_adjustments":[],"unknown_costs":"not-valued","apply_allowed":false,"next_step":"Supply exact evidence-backed reconstructed entries and review a resolution; originals must be backed up byte-for-byte. No automatic repair or guessed totals."}),
    )
}

/// Reviewable recovery from an evidence-supplied valid legacy snapshot. Native
/// chain damage requires separate inspection; this never rewrites old files.
pub fn propose_from(root: &Path, source: &Path) -> Result<Value> {
    let source = source.canonicalize()?;
    if !source.starts_with(root.canonicalize()?) || source == root.canonicalize()? {
        return store::fail(
            2,
            "recovery source must be a preserved snapshot inside the project",
        );
    }
    if source.join(".kineti/spend.recovery-view.json").exists() {
        return store::fail(3, "nested recovery views are not supported");
    }
    let (total, stages, tripped) = legacy(&source)?;
    if root.join(".kineti/spend.native.jsonl").exists() {
        return store::fail(3,"native history exists; inspect its exact interrupted operation before recovery; no automatic legacy replacement");
    }
    let mut proposal = recovery_proposal(root)?;
    let mut snapshots = vec![];
    for name in ["spend.json", "spend.log.jsonl", "spend.initialized.json"] {
        let p = source.join(".kineti").join(name);
        if p.exists() {
            snapshots
                .push(json!({"path":p.strip_prefix(root).unwrap(),"sha256":digest(fs::read(p)?)}));
        }
    }
    proposal["verified_source"] = json!(source.strip_prefix(root).unwrap());
    proposal["candidate_sources"] = json!(snapshots);
    proposal["candidate_total_microcents"] = json!(total);
    proposal["candidate_by_stage_microcents"] = json!(stages);
    proposal["candidate_tripped"] = json!(tripped);
    proposal["review_required"]=json!("Confirm the snapshot covers all affected costs using external evidence. Unknown/reserved costs are not resolved by selecting a snapshot.");
    proposal["proposal_sha256"] = json!(digest(canonical(&proposal).map_err(|e| Error(3, e))?));
    Ok(proposal)
}
/// Apply exactly the reviewed source digest, preserving original bytes and all
/// unknown/reservation records. Confirmation is not proof of human identity.
pub fn recover_from(
    root: &Path,
    source: &Path,
    expected: &str,
    evidence: &Path,
    confirmed: bool,
) -> Result<Value> {
    if !confirmed || expected.len() != 64 {
        return store::fail(
            2,
            "recovery requires --i-am-human, --proposal-sha256 and --evidence",
        );
    }
    let evidence = store::input(
        root,
        evidence
            .to_str()
            .ok_or_else(|| Error(2, "invalid evidence path".into()))?,
    )?;
    let evidence_bytes = fs::read(&evidence)?;
    if evidence_bytes.is_empty() {
        return store::fail(2, "recovery supporting evidence must not be empty");
    }
    let _lock = Lock::take(&root.join(".kineti/spend.lock"))?;
    let proposal = propose_review(
        root,
        source,
        evidence
            .strip_prefix(root)
            .map_err(|e| Error(2, e.to_string()))?,
    )?;
    if proposal["proposal_sha256"] != expected {
        return store::fail(
            3,
            "recovery proposal or evidence changed; review again before applying",
        );
    }
    let backup = root.join(".kineti/accounting-recovery").join(expected);
    fs::create_dir_all(&backup)?;
    let mut originals = vec![];
    for src in proposal["sources"].as_array().unwrap() {
        let rel = src["path"].as_str().unwrap();
        let original = store::input(root, rel)?;
        let bytes = fs::read(original)?;
        let dest = backup.join(digest(rel));
        if dest.exists() {
            if fs::read(&dest)? != bytes {
                return store::fail(3, "recovery backup collision");
            }
        } else {
            store::atomic(&dest, &bytes)?;
        }
        if rel == ".kineti/spend.json"
            || rel == ".kineti/spend.log.jsonl"
            || rel == ".kineti/spend.initialized.json"
        {
            originals.push(src.clone());
        }
    }
    let id = store::nonce()?;
    store::append(
        &root.join(".kineti/accounting-recovery.jsonl"),
        &json!({"at":store::now(),"kind":"recovery-intent","id":id,"proposal":proposal,"supporting_evidence_sha256":digest(&evidence_bytes),"identity_proven":false,"unknowns_resolved":false}),
    )?;
    let view = json!({"version":1,"verified_source":proposal["verified_source"],"original_sources":originals,"candidate_sources":proposal["candidate_sources"],"proposal_sha256":expected});
    store::write(&root.join(".kineti/spend.recovery-view.json"), &view)?;
    let verified = load(root)?;
    store::append(
        &root.join(".kineti/accounting-recovery.jsonl"),
        &json!({"at":store::now(),"kind":"recovery-completion","id":id,"total_microcents":verified.total,"unknowns_resolved":false}),
    )?;
    Ok(
        json!({"recovered_view":true,"total_microcents":verified.total,"originals_preserved":true,"unknowns_resolved":false}),
    )
}

/// Bind supporting evidence into the exact proposal reviewed before apply.
pub fn propose_review(root: &Path, source: &Path, evidence: &Path) -> Result<Value> {
    let full = store::input(
        root,
        evidence
            .to_str()
            .ok_or_else(|| Error(2, "invalid evidence path".into()))?,
    )?;
    let bytes = fs::read(full)?;
    if bytes.is_empty() {
        return store::fail(2, "supporting evidence is empty");
    }
    let mut proposal = propose_from(root, source)?;
    proposal
        .as_object_mut()
        .unwrap()
        .shift_remove("proposal_sha256");
    proposal["supporting_evidence"] = json!({"path":evidence,"sha256":digest(bytes)});
    proposal["proposal_sha256"] = json!(digest(canonical(&proposal).map_err(|e| Error(3, e))?));
    Ok(proposal)
}
