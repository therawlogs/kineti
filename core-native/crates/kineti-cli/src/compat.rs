//! Original JavaScript signing bytes and the distinct native canonical encoding.
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

/// SHA-256 hexadecimal digest.
pub fn digest(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}

fn js_number(n: &serde_json::Number) -> String {
    let x = n.as_f64().expect("JSON number");
    if x == 0.0 {
        return "0".into();
    }
    let negative = x < 0.0;
    let raw = x.abs().to_string();
    let (mantissa, exponent) = raw
        .split_once(['e', 'E'])
        .map_or((raw.as_str(), 0), |(m, e)| {
            (m, e.parse::<i32>().expect("exponent"))
        });
    let decimal = mantissa.find('.').unwrap_or(mantissa.len()) as i32 + exponent;
    let mut digits = mantissa.replace('.', "");
    let leading = digits.bytes().take_while(|b| *b == b'0').count();
    digits.drain(..leading);
    let point = decimal - leading as i32;
    while digits.ends_with('0') && digits.len() > 1 {
        digits.pop();
    }
    let sign = if negative { "-" } else { "" };
    if point > 21 || point <= -6 {
        let exp = point - 1;
        let frac = if digits.len() > 1 {
            format!(".{}", &digits[1..])
        } else {
            String::new()
        };
        format!(
            "{sign}{}{frac}e{}{exp}",
            &digits[..1],
            if exp >= 0 { "+" } else { "" }
        )
    } else if point <= 0 {
        format!("{sign}0.{}{}", "0".repeat((-point) as usize), digits)
    } else if point as usize >= digits.len() {
        format!(
            "{sign}{digits}{}",
            "0".repeat(point as usize - digits.len())
        )
    } else {
        format!(
            "{sign}{}.{}",
            &digits[..point as usize],
            &digits[point as usize..]
        )
    }
}

/// Reproduce JSON.stringify property order and number spelling, not sorted JSON.
pub fn legacy_json(value: &Value) -> String {
    match value {
        Value::Number(n) => js_number(n),
        Value::Array(items) => format!(
            "[{}]",
            items.iter().map(legacy_json).collect::<Vec<_>>().join(",")
        ),
        Value::Object(map) => {
            let index = |key: &str| {
                key.parse::<u32>()
                    .ok()
                    .filter(|n| *n != u32::MAX && n.to_string() == key)
            };
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by(|(a, _), (b, _)| match (index(a), index(b)) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            });
            format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(k, v)| format!(
                        "{}:{}",
                        serde_json::to_string(k).unwrap(),
                        legacy_json(v)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
        _ => serde_json::to_string(value).expect("JSON"),
    }
}

/// Remove only the signature; retain original insertion ordering.
pub fn legacy_unsigned(record: &Value) -> Result<String, String> {
    let mut unsigned = record
        .as_object()
        .ok_or("receipt must be an object")?
        .clone();
    unsigned.shift_remove("signature");
    Ok(legacy_json(&Value::Object(unsigned)))
}

/// Verify Ed25519 against an externally supplied trusted key, including issuer identity.
pub fn verify_legacy(record: &Value, trusted: &str) -> bool {
    let verify = || -> Option<()> {
        if record.get("issuer_key")?.as_str()? != trusted {
            return None;
        }
        let text = record.get("signature")?.as_str()?;
        let bytes = STANDARD.decode(text).ok()?;
        if bytes.len() != 64 || STANDARD.encode(&bytes) != text {
            return None;
        }
        let signature = Signature::from_slice(&bytes).ok()?;
        let key = public_key(trusted).ok()?;
        key.verify_strict(legacy_unsigned(record).ok()?.as_bytes(), &signature)
            .ok()
    };
    verify().is_some()
}

/// Read the original Ed25519 SPKI PEM public-key format without changing its identity.
pub fn public_key(pem: &str) -> Result<VerifyingKey, String> {
    let body = pem
        .trim()
        .strip_prefix("-----BEGIN PUBLIC KEY-----")
        .ok_or("public key header")?
        .strip_suffix("-----END PUBLIC KEY-----")
        .ok_or("public key footer")?;
    let der = STANDARD
        .decode(body.split_whitespace().collect::<String>())
        .map_err(|e| e.to_string())?;
    if der.len() != 44 || der[..12] != [0x30, 0x2a, 0x30, 5, 6, 3, 0x2b, 0x65, 0x70, 3, 0x21, 0] {
        return Err("not an Ed25519 SPKI key".into());
    }
    VerifyingKey::from_bytes(der[12..].try_into().unwrap()).map_err(|e| e.to_string())
}

/// kineti-canonical-v2 typed ASCII tree; mirrors the existing cross-language code.
pub fn canonical(value: &Value) -> Result<String, String> {
    fn encode(v: &Value, depth: usize) -> Result<Value, String> {
        if depth > 128 {
            return Err("JSON nesting limit exceeded".into());
        }
        let hex = |s: &str| {
            s.as_bytes()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        Ok(match v {
            Value::Null => json!(["null"]),
            Value::Bool(b) => json!(["bool", b]),
            Value::String(s) => json!(["string", hex(s)]),
            Value::Number(n) => {
                let mut x = n.as_f64().ok_or("invalid number")?;
                if !x.is_finite() || (x.fract() == 0.0 && x.abs() > 9_007_199_254_740_991.0) {
                    return Err("unsafe JSON number".into());
                }
                if x == 0.0 {
                    x = 0.0;
                }
                json!(["f64", format!("{:016x}", x.to_bits())])
            }
            Value::Array(a) => json!([
                "array",
                a.iter()
                    .map(|v| encode(v, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?
            ]),
            Value::Object(m) => {
                let mut entries: Vec<_> = m.iter().collect();
                entries.sort_by(|(a, _), (b, _)| a.encode_utf16().cmp(b.encode_utf16()));
                json!([
                    "object",
                    entries
                        .iter()
                        .map(|(k, v)| Ok(json!([hex(k), encode(v, depth + 1)?])))
                        .collect::<Result<Vec<Value>, String>>()?
                ])
            }
        })
    }
    Ok(json!(["kineti-canonical-v2", encode(value, 0)?]).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixtures() -> Value {
        serde_json::from_str(include_str!("../tests/fixtures/legacy-golden.json")).unwrap()
    }
    #[test]
    fn genuine_legacy_signatures_and_exact_original_bytes() {
        let all = fixtures();
        for f in all["fixtures"].as_array().unwrap() {
            assert_eq!(
                legacy_unsigned(&f["record"]).unwrap(),
                f["unsigned"].as_str().unwrap(),
                "{}",
                f["name"]
            );
            assert!(
                verify_legacy(&f["record"], all["public_key"].as_str().unwrap()),
                "{}",
                f["name"]
            );
        }
    }
    #[test]
    fn altered_receipts_and_substituted_issuers_are_rejected() {
        let all = fixtures();
        for f in all["fixtures"].as_array().unwrap() {
            let mut r = f["record"].clone();
            r["extra"] = json!("altered");
            assert!(!verify_legacy(&r, all["public_key"].as_str().unwrap()));
            assert!(!verify_legacy(&f["record"], "untrusted issuer"));
            r = f["record"].clone();
            r["issuer_key"] = json!("substitution");
            assert!(!verify_legacy(&r, all["public_key"].as_str().unwrap()));
        }
    }
    #[test]
    fn native_canonical_matches_cross_language_golden_bytes() {
        for f in fixtures()["canonical"].as_array().unwrap() {
            assert_eq!(
                canonical(&f["value"]).unwrap(),
                f["expected"].as_str().unwrap()
            );
        }
        assert!(canonical(&json!(9_007_199_254_740_992u64)).is_err());
    }
}
