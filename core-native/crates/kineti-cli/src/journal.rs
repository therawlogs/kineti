//! Read-only integrity of legacy accounting notes. No memory product operations.
use crate::{
    compat::{canonical, digest, legacy_json},
    store::{self, Error, Result},
};
use serde_json::{Map, Value};
use std::{fs, path::Path};

// Original Number.toFixed(6), rounding the exact binary value, with ties upward
// in magnitude. Rust's decimal formatter uses a different tie rule.
fn fixed6(value: &serde_json::Number) -> String {
    let x = value.as_f64().unwrap();
    if x.abs() >= 1e21 {
        return legacy_json(&Value::Number(value.clone()));
    }
    let bits = x.abs().to_bits();
    let exp = ((bits >> 52) & 0x7ff) as i32;
    let mantissa = if exp == 0 {
        bits & ((1u64 << 52) - 1)
    } else {
        (bits & ((1u64 << 52) - 1)) | (1u64 << 52)
    };
    let e = if exp == 0 { -1074 } else { exp - 1023 - 52 };
    let numerator = mantissa as u128 * 1_000_000;
    let rounded = if e >= 0 {
        numerator << e
    } else if -e >= 128 {
        0
    } else {
        let denominator = 1u128 << (-e);
        let quotient = numerator / denominator;
        let remainder = numerator % denominator;
        quotient + u128::from(remainder >= denominator - remainder)
    };
    format!(
        "{}{}.{:06}",
        if x < 0.0 { "-" } else { "" },
        rounded / 1_000_000,
        rounded % 1_000_000
    )
}
fn normalized(v: &Value, stable: bool) -> Value {
    match v {
        Value::Number(n) if stable => Value::String(fixed6(n)),
        Value::Array(a) => Value::Array(a.iter().map(|v| normalized(v, stable)).collect()),
        Value::Object(m) => {
            let mut keys = m.keys().collect::<Vec<_>>();
            keys.sort_by(|a, b| a.encode_utf16().cmp(b.encode_utf16()));
            let mut out = Map::new();
            for key in keys {
                out.insert(key.clone(), normalized(&m[key], stable));
            }
            Value::Object(out)
        }
        _ => v.clone(),
    }
}
fn delimited(parts: &[&str]) -> String {
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend_from_slice(&(part.len() as u32).to_be_bytes());
        bytes.extend_from_slice(part.as_bytes());
        bytes.push(0);
    }
    digest(bytes)
}
/// Validate original checkpoints and all four historical hash encodings, without writes.
pub fn read(root: &Path) -> Result<Vec<Value>> {
    let file = root.join(".kineti/journal.jsonl");
    let checkpoint = root.join(".kineti/journal.jsonl.checkpoint.json");
    if checkpoint.exists() {
        let c = store::read(&checkpoint)?;
        let bytes = fs::read(&file)
            .map_err(|_| Error(3, "initialized accounting journal missing".into()))?;
        if c["version"] != 1
            || c["bytes"].as_u64() != Some(bytes.len() as u64)
            || c["sha256"] != digest(&bytes)
        {
            return store::fail(
                3,
                "accounting journal checkpoint differs; inspected recovery required",
            );
        }
    } else if root.join(".kineti/journal.jsonl.initialized").exists() {
        return store::fail(3, "initialized accounting journal checkpoint missing");
    }
    let rows = store::lines(&file)?;
    let mut previous = "GENESIS";
    for row in &rows {
        let id = row["id"].as_str().unwrap_or("unknown-record");
        let fail = || {
            Error(
                3,
                format!("accounting journal integrity failed at {id}; inspect original history"),
            )
        };
        if row["id"].as_str().is_none() {
            return Err(fail());
        }
        let at = row["at"].as_str().ok_or_else(fail)?;
        if row["project"].as_str().is_none()
            || !(row["data"].is_object() || row["data"].is_array())
            || row["prev_hash"] != previous
            || time::OffsetDateTime::parse(at, &time::format_description::well_known::Rfc3339)
                .is_err()
        {
            return Err(fail());
        }
        canonical(&row["data"]).map_err(|_| fail())?;
        let hash = row["hash"].as_str().ok_or_else(fail)?;
        let mut valid = false;
        for stable in [true, false] {
            let data = legacy_json(&normalized(&row["data"], stable));
            valid |= delimited(&[previous, at, id, &data]) == hash
                || digest(format!("{previous}{at}{id}{data}")) == hash;
        }
        if !valid {
            return Err(fail());
        }
        previous = hash;
    }
    Ok(rows)
}
