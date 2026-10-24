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
fn legacy_usd(v: &Value) -> Result<f64> {
    v.as_f64()
        .filter(|n| n.is_finite() && *n >= 0.0)
        .ok_or_else(|| Error(3, "invalid historical monetary amount".into()))
}
fn amount(v: &Value) -> Result<u64> {
    // Original usdToMicrocents: Math.round(usd * 1_000_000).
    // Legacy JSON numbers may use exponent notation; new CLI amounts remain decimal-only.
    let n = (legacy_usd(v)? * 1_000_000.0).round();
    if !n.is_finite() || n > 9_007_199_254_740_991.0 {
        return store::fail(3, "historical money exceeds exact supported integer range");
    }
    Ok(n as u64)
}
fn legacy_matches(micro: u64, usd: &Value) -> Result<bool> {
    // Preserve the original four-decimal IEEE-754 rounding sequence exactly.
    let round = |v: f64| (v * 1e4).round() / 1e4;
    Ok(round(micro as f64 / 1e6) == round(legacy_usd(usd)?))
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
    /// Configured factor can tighten, but never loosen, the 95% stop.
    pub safety_factor_ppm: u64,
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
    if s.get("reason")
        .is_none_or(|v| !v.is_null() && !v.is_string())
    {
        return store::fail(3, "legacy trip reason malformed");
    }
    let entries = store::lines(&log)?;
    let mut total = 0u64;
    let mut stages = BTreeMap::<String, u64>::new();
    for e in &entries {
        let stage = e["stage"]
            .as_str()
            .filter(|s| !s.is_empty())
            .ok_or_else(|| Error(3, "accounting stage missing".into()))?;
        if !e["model"].is_string() {
            return store::fail(3, "legacy accounting model missing");
        }
        for k in ["tokens_in", "tokens_out"] {
            if e[k].as_f64().is_none_or(|v| !v.is_finite() || v < 0.0) {
                return store::fail(3, "invalid historical token count");
            }
        }
        legacy_usd(&e["usd"])?;
        let m = if e.get("microcents").is_some() {
            integer(&e["microcents"])?
        } else {
            amount(&e["usd"])?
        };
        total = total
            .checked_add(m)
            .ok_or_else(|| Error(3, "accounting overflow".into()))?;
        let st = stages.entry(stage.into()).or_default();
        *st = st
            .checked_add(m)
            .ok_or_else(|| Error(3, "stage overflow".into()))?;
    }
    if s["entries"].as_u64() != Some(entries.len() as u64)
        || !legacy_matches(total, &s["total_usd"])?
        || match s.get("total_microcents") {
            Some(v) => integer(v)?,
            None => amount(&s["total_usd"])?,
        } != total
    {
        return store::fail(3, "ledger/state totals differ; reviewed recovery required");
    }
    let by = s["by_stage"]
        .as_object()
        .ok_or_else(|| Error(3, "stage totals missing".into()))?;
    for (k, v) in by {
        if !legacy_matches(stages.get(k).copied().unwrap_or(0), v)? {
            return store::fail(3, "stage accounting differs");
        }
    }
    for (k, v) in &stages {
        if !legacy_matches(*v, by.get(k).unwrap_or(&json!(0)))? {
            return store::fail(3, "stage missing from state");
        }
    }
    let mut expected_stages = BTreeMap::new();
    if let Some(raw) = s.get("by_stage_microcents") {
        for (k, v) in raw
            .as_object()
            .ok_or_else(|| Error(3, "malformed integer stages".into()))?
        {
            expected_stages.insert(k.clone(), integer(v)?);
        }
    } else {
        for (k, v) in by {
            expected_stages.insert(k.clone(), amount(v)?);
        }
    }
    for k in stages.keys().chain(expected_stages.keys()).chain(by.keys()) {
        if expected_stages.get(k).copied().unwrap_or(0) != stages.get(k).copied().unwrap_or(0) {
            return store::fail(3, "integer stage totals differ");
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
fn recovery_guard(root: &Path) -> Result<()> {
    let base = root.join(".kineti");
    let log = base.join("accounting-recovery.jsonl");
    let guard = base.join("accounting-recovery.guard.json");
    let marker = base.join("accounting-recovery.initialized");
    let view = base.join("spend.recovery-view.json");
    if !log.exists() && !guard.exists() && !marker.exists() && !view.exists() {
        return Ok(());
    }
    if !marker.exists()
        || !log.exists()
        || !guard.exists()
        || store::read(&guard)?["sha256"] != digest(fs::read(&log)?)
    {
        return store::fail(
            3,
            "recovery history changed or was interrupted; exact inspection required",
        );
    }
    let mut pending = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    let mut last = None;
    for row in store::lines(&log)? {
        let id = row["id"]
            .as_str()
            .ok_or_else(|| Error(3, "recovery id missing".into()))?
            .to_owned();
        match row["kind"].as_str() {
            Some("recovery-intent") => {
                if !seen.insert(id.clone()) {
                    return store::fail(3, "duplicate recovery intent");
                }
                pending.insert(id, row);
            }
            Some("recovery-completion") => {
                let intent = pending
                    .remove(&id)
                    .ok_or_else(|| Error(3, "recovery completion lacks intent".into()))?;
                if row["proposal_sha256"] != intent["proposal"]["proposal_sha256"] {
                    return store::fail(3, "recovery proposal identity differs");
                }
                last = Some(row);
            }
            _ => return store::fail(3, "unknown recovery operation"),
        }
    }
    if !pending.is_empty() {
        return store::fail(
            3,
            "unfinished recovery intent; inspect before resuming accounting",
        );
    }
    let last = last.ok_or_else(|| Error(3, "recovery completion missing".into()))?;
    if !view.exists()
        || last["view_sha256"] != digest(fs::read(&view)?)
        || last["proposal_sha256"] != store::read(&view)?["proposal_sha256"]
    {
        return store::fail(3, "reviewed recovery view changed or is missing");
    }
    Ok(())
}
fn recovery_append(root: &Path, row: &Value) -> Result<()> {
    let log = root.join(".kineti/accounting-recovery.jsonl");
    store::append(&log, row)?;
    store::write(
        &root.join(".kineti/accounting-recovery.guard.json"),
        &json!({"sha256":digest(fs::read(log)?)}),
    )
}
/// Inspect under the caller's spend lock. Every monetary reservation/unknown blocks.
pub fn load(root: &Path) -> Result<Account> {
    store::state(root)?;
    recovery_guard(root)?;
    for record in crate::journal::read(root)? {
        let data = &record["data"];
        if record["type"] == "run-record"
            || data.get("costs_usd").is_some()
            || data.get("root_goal").is_some() && data.get("stage_outcomes").is_some()
        {
            let unknown = data["costs_usd"].is_null()
                || data["costs_usd"] == 0
                    && data["cost_basis"].as_str().is_some_and(|s| {
                        ["unavailable", "unknown", "unresolved", "not recorded"]
                            .iter()
                            .any(|word| s.to_lowercase().contains(word))
                    });
            if unknown {
                return store::fail(3,format!("unresolved historical cost {} remains recorded; any implementation-only subscription exception does not authorize publication",record["id"]));
            }
            if data["costs_usd"]
                .as_f64()
                .is_none_or(|n| !n.is_finite() || n < 0.0)
            {
                return store::fail(3, "historical accounting note has invalid monetary fields");
            }
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
    let safety_factor_ppm = match config.pointer("/settings/spend_limit_usd/safety_factor") {
        Some(v) => {
            let raw = legacy_usd(v)?;
            let n = (raw * 1_000_000.0).floor() as u64;
            if n == 0 || raw > 1.0 {
                return store::fail(3, "invalid accounting safety factor");
            }
            n.min(950_000)
        }
        None => 950_000,
    };
    let mut a = Account {
        total,
        stages,
        head: base_digest(root)?,
        count: 0,
        global,
        stage_default,
        safety_factor_ppm,
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
fn threshold(limit: u64, factor: u64) -> u64 {
    if factor == 950_000 {
        (limit as u128 * 95).div_ceil(100) as u64
    } else {
        (limit as u128 * factor as u128 / 1_000_000) as u64
    }
}
/// Require remaining reported headroom.
pub fn headroom(a: &Account) -> Result<()> {
    if a.tripped
        || a.total >= threshold(a.global, a.safety_factor_ppm)
        || a.stages.iter().any(|(k, v)| {
            *v >= threshold(
                *a.stage_limits.get(k).unwrap_or(&a.stage_default),
                a.safety_factor_ppm,
            )
        })
    {
        return store::fail(
            3,
            "reported spending reached configured stop (at most 95%); reset without headroom cannot pass",
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
        json!({"total_microcents":a.total,"global_microcents":a.global,"stop_microcents":threshold(a.global,a.safety_factor_ppm),"headroom_microcents":threshold(a.global,a.safety_factor_ppm).saturating_sub(a.total),"by_stage_microcents":a.stages,"provider_billing":"unknown","safety_factor_ppm":a.safety_factor_ppm,"tripped":a.tripped}),
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
        "spend.recovery-view.json",
        "accounting-recovery.jsonl",
        "accounting-recovery.guard.json",
        "accounting-recovery.initialized",
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
    recovery_guard(root)?;
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
    store::atomic(
        &root.join(".kineti/accounting-recovery.initialized"),
        b"1\n",
    )?;
    recovery_append(
        root,
        &json!({"at":store::now(),"kind":"recovery-intent","id":id,"proposal":proposal,"supporting_evidence_sha256":digest(&evidence_bytes),"identity_proven":false,"unknowns_resolved":false}),
    )?;
    let view = json!({"version":1,"verified_source":proposal["verified_source"],"original_sources":originals,"candidate_sources":proposal["candidate_sources"],"proposal_sha256":expected});
    store::write(&root.join(".kineti/spend.recovery-view.json"), &view)?;
    // Verify installed sources before completion; public accounting remains blocked by intent.
    let verified = legacy(root)?;
    recovery_append(
        root,
        &json!({"at":store::now(),"kind":"recovery-completion","id":id,"proposal_sha256":expected,"view_sha256":digest(fs::read(root.join(".kineti/spend.recovery-view.json"))?),"total_microcents":verified.0,"unknowns_resolved":false}),
    )?;
    let verified = load(root)?;
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
