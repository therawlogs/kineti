//! # Autonomous TOTP Authenticator Engine (`totp`)
//!
//! Full RFC 6238 (TOTP) and RFC 4226 (HOTP) implementation supporting:
//! - Standard Google Authenticator, Authy, and Microsoft Authenticator seeds.
//! - Base32 secret decoding with padding and whitespace tolerance.
//! - Standard `otpauth://totp/Provider:Account?secret=...` URI parsing.
//! - HMAC-SHA1 and HMAC-SHA256 hashing.
//! - Dynamic truncation and 6-digit or 8-digit formatting.
//! - Clock drift window tolerance ($\pm 1$ step, default $\pm 30$ seconds).
//! - Replay prevention caching last-consumed step.
//! - Full [`KinetiConnectorProtocol`] integration.

use crate::protocol::{
    get_str_property, ConnectorProtocolError, ConsequenceLevel, KinetiConnectorProtocol, Value,
};
use crate::vault::{CredentialVault, TotpSeedEntry};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};

/// Configuration parameters for generating or verifying a TOTP token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TotpParams {
    /// Service or issuer name.
    pub service_name: String,
    /// Account username or email.
    pub account_name: String,
    /// Shared secret in Base32 encoding.
    pub secret_base32: String,
    /// Number of output digits (default 6).
    pub digits: u32,
    /// Time step in seconds (default 30).
    pub period_secs: u64,
    /// Hash algorithm ("SHA1" or "SHA256").
    pub algorithm: String,
}

impl Default for TotpParams {
    fn default() -> Self {
        Self {
            service_name: "Kineti".to_string(),
            account_name: "user".to_string(),
            secret_base32: String::new(),
            digits: 6,
            period_secs: 30,
            algorithm: "SHA1".to_string(),
        }
    }
}

/// Parses an `otpauth://totp/...` URI into [`TotpParams`].
pub fn parse_otpauth_uri(uri: &str) -> Result<TotpParams, &'static str> {
    if !uri.starts_with("otpauth://totp/") {
        return Err("Invalid URI scheme; expected 'otpauth://totp/'");
    }

    let rest = &uri["otpauth://totp/".len()..];
    let (label_part, query_part) = match rest.find('?') {
        Some(pos) => (&rest[..pos], &rest[pos + 1..]),
        None => return Err("Missing query parameters in otpauth URI"),
    };

    let (mut service_name, account_name) = if let Some(pos) = label_part.find(':') {
        (url_decode(&label_part[..pos]), url_decode(&label_part[pos + 1..]))
    } else {
        (String::new(), url_decode(label_part))
    };

    let mut secret_base32 = String::new();
    let mut digits = 6;
    let mut period_secs = 30;
    let mut algorithm = "SHA1".to_string();

    for pair in query_part.split('&') {
        if let Some(pos) = pair.find('=') {
            let key = &pair[..pos];
            let val = url_decode(&pair[pos + 1..]);
            match key.to_lowercase().as_str() {
                "secret" => secret_base32 = val,
                "issuer" => {
                    if service_name.is_empty() {
                        service_name = val;
                    }
                }
                "digits" => digits = val.parse().unwrap_or(6),
                "period" => period_secs = val.parse().unwrap_or(30),
                "algorithm" => algorithm = val.to_uppercase(),
                _ => {}
            }
        }
    }

    if secret_base32.is_empty() {
        return Err("Missing 'secret' parameter in otpauth URI");
    }

    if service_name.is_empty() {
        service_name = "Authenticator".to_string();
    }

    Ok(TotpParams {
        service_name,
        account_name,
        secret_base32,
        digits,
        period_secs,
        algorithm,
    })
}

fn url_decode(s: &str) -> String {
    let mut res = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '%' {
            let h1 = chars.next().unwrap_or('0');
            let h2 = chars.next().unwrap_or('0');
            let hex = format!("{}{}", h1, h2);
            if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                res.push(byte as char);
            }
        } else if c == '+' {
            res.push(' ');
        } else {
            res.push(c);
        }
    }
    res
}

/// Decodes RFC 4648 Base32 alphabet into raw bytes.
pub fn decode_base32(input: &str) -> Result<Vec<u8>, &'static str> {
    let mut clean: Vec<u8> = Vec::with_capacity(input.len());
    for b in input.bytes() {
        if b.is_ascii_whitespace() || b == b'-' {
            continue;
        }
        if b == b'=' {
            break; // Padding
        }
        let val = match b.to_ascii_uppercase() {
            b'A'..=b'Z' => b.to_ascii_uppercase() - b'A',
            b'2'..=b'7' => b - b'2' + 26,
            _ => return Err("Invalid Base32 character"),
        };
        clean.push(val);
    }

    let mut out = Vec::with_capacity((clean.len() * 5) / 8);
    let mut buffer: u32 = 0;
    let mut bits_left: u8 = 0;

    for val in clean {
        buffer = (buffer << 5) | (val as u32);
        bits_left += 5;
        if bits_left >= 8 {
            bits_left -= 8;
            out.push(((buffer >> bits_left) & 0xff) as u8);
        }
    }

    Ok(out)
}

/// Pure Safe Rust HMAC-SHA1 and HMAC-SHA256 implementations.
mod hmac {
    pub fn hmac_sha1(key: &[u8], data: &[u8]) -> [u8; 20] {
        let mut k = [0u8; 64];
        if key.len() > 64 {
            let hash = sha1(key);
            k[..20].copy_from_slice(&hash);
        } else {
            k[..key.len()].copy_from_slice(key);
        }

        let mut ipad = [0x36u8; 64];
        let mut opad = [0x5cu8; 64];
        for i in 0..64 {
            ipad[i] ^= k[i];
            opad[i] ^= k[i];
        }

        let mut inner = Vec::with_capacity(64 + data.len());
        inner.extend_from_slice(&ipad);
        inner.extend_from_slice(data);
        let inner_hash = sha1(&inner);

        let mut outer = Vec::with_capacity(64 + 20);
        outer.extend_from_slice(&opad);
        outer.extend_from_slice(&inner_hash);
        sha1(&outer)
    }

    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
        use kineti_core::kernel::sha256;
        let mut k = [0u8; 64];
        if key.len() > 64 {
            let hash = sha256(key);
            k[..32].copy_from_slice(&hash);
        } else {
            k[..key.len()].copy_from_slice(key);
        }

        let mut ipad = [0x36u8; 64];
        let mut opad = [0x5cu8; 64];
        for i in 0..64 {
            ipad[i] ^= k[i];
            opad[i] ^= k[i];
        }

        let mut inner = Vec::with_capacity(64 + data.len());
        inner.extend_from_slice(&ipad);
        inner.extend_from_slice(data);
        let inner_hash = sha256(&inner);

        let mut outer = Vec::with_capacity(64 + 32);
        outer.extend_from_slice(&opad);
        outer.extend_from_slice(&inner_hash);
        sha256(&outer)
    }

    /// Pure Safe Rust SHA-1 (FIPS 180-1).
    pub fn sha1(data: &[u8]) -> [u8; 20] {
        let mut h0: u32 = 0x67452301;
        let mut h1: u32 = 0xEFCDAB89;
        let mut h2: u32 = 0x98BADCFE;
        let mut h3: u32 = 0x10325476;
        let mut h4: u32 = 0xC3D2E1F0;

        let ml = (data.len() as u64) * 8;
        let mut msg = data.to_vec();
        msg.push(0x80);
        while (msg.len() % 64) != 56 {
            msg.push(0x00);
        }
        msg.extend_from_slice(&ml.to_be_bytes());

        for chunk in msg.chunks_exact(64) {
            let mut w = [0u32; 80];
            for i in 0..16 {
                w[i] = u32::from_be_bytes([
                    chunk[i * 4],
                    chunk[i * 4 + 1],
                    chunk[i * 4 + 2],
                    chunk[i * 4 + 3],
                ]);
            }
            for i in 16..80 {
                w[i] = (w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16]).rotate_left(1);
            }

            let mut a = h0;
            let mut b = h1;
            let mut c = h2;
            let mut d = h3;
            let mut e = h4;

            for i in 0..80 {
                let (f, k) = match i {
                    0..=19 => ((b & c) | ((!b) & d), 0x5A827999u32),
                    20..=39 => (b ^ c ^ d, 0x6ED9EBA1u32),
                    40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDCu32),
                    _ => (b ^ c ^ d, 0xCA62C1D6u32),
                };
                let temp = a
                    .rotate_left(5)
                    .wrapping_add(f)
                    .wrapping_add(e)
                    .wrapping_add(k)
                    .wrapping_add(w[i]);
                e = d;
                d = c;
                c = b.rotate_left(30);
                b = a;
                a = temp;
            }

            h0 = h0.wrapping_add(a);
            h1 = h1.wrapping_add(b);
            h2 = h2.wrapping_add(c);
            h3 = h3.wrapping_add(d);
            h4 = h4.wrapping_add(e);
        }

        let mut out = [0u8; 20];
        out[..4].copy_from_slice(&h0.to_be_bytes());
        out[4..8].copy_from_slice(&h1.to_be_bytes());
        out[8..12].copy_from_slice(&h2.to_be_bytes());
        out[12..16].copy_from_slice(&h3.to_be_bytes());
        out[16..20].copy_from_slice(&h4.to_be_bytes());
        out
    }
}

/// Generates a numeric TOTP code at a specific time step (RFC 4226 Section 5.4).
pub fn generate_hotp_code(secret_bytes: &[u8], counter: u64, digits: u32, use_sha256: bool) -> String {
    let counter_bytes = counter.to_be_bytes();
    let hash: Vec<u8> = if use_sha256 {
        hmac::hmac_sha256(secret_bytes, &counter_bytes).to_vec()
    } else {
        hmac::hmac_sha1(secret_bytes, &counter_bytes).to_vec()
    };

    // Dynamic Truncation
    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let binary = (((hash[offset] & 0x7f) as u32) << 24)
        | (((hash[offset + 1] & 0xff) as u32) << 16)
        | (((hash[offset + 2] & 0xff) as u32) << 8)
        | ((hash[offset + 3] & 0xff) as u32);

    let modulo = 10u32.pow(digits);
    let code_num = binary % modulo;
    format!("{:0width$}", code_num, width = digits as usize)
}

/// Autonomous TOTP Authenticator Engine.
#[derive(Debug)]
pub struct TotpAuthenticator {
    vault: Arc<CredentialVault>,
    last_consumed_step: RwLock<HashMap<(String, String), u64>>,
}

impl TotpAuthenticator {
    /// Creates a new TOTP authenticator backed by the given vault.
    pub fn new(vault: Arc<CredentialVault>) -> Self {
        Self {
            vault,
            last_consumed_step: RwLock::new(HashMap::new()),
        }
    }

    /// Generates current TOTP code and seconds remaining in time window.
    pub fn current_code(
        &self,
        user_id: &str,
        service_name: &str,
        epoch_seconds: u64,
    ) -> Result<(String, u64), &'static str> {
        let seed = self
            .vault
            .retrieve_totp_seed(user_id, service_name)
            .ok_or("No TOTP seed found for service")?;

        let secret_bytes = decode_base32(&seed.secret_base32)?;
        let step = epoch_seconds / seed.period_secs;
        let use_sha256 = seed.algorithm.eq_ignore_ascii_case("SHA256");
        let code = generate_hotp_code(&secret_bytes, step, seed.digits, use_sha256);
        let seconds_remaining = seed.period_secs - (epoch_seconds % seed.period_secs);

        Ok((code, seconds_remaining))
    }

    /// Verifies a submitted TOTP code allowing $\pm 1$ step clock drift and enforcing replay prevention.
    pub fn verify_code(
        &self,
        user_id: &str,
        service_name: &str,
        submitted_code: &str,
        epoch_seconds: u64,
    ) -> Result<bool, &'static str> {
        let seed = self
            .vault
            .retrieve_totp_seed(user_id, service_name)
            .ok_or("No TOTP seed found for service")?;

        let secret_bytes = decode_base32(&seed.secret_base32)?;
        let current_step = epoch_seconds / seed.period_secs;
        let use_sha256 = seed.algorithm.eq_ignore_ascii_case("SHA256");

        // Windows: -1, 0, +1
        let windows = [
            current_step.saturating_sub(1),
            current_step,
            current_step.saturating_add(1),
        ];

        let mut consumed_map = self.last_consumed_step.write().unwrap();
        let key = (user_id.to_string(), service_name.to_string());
        let last_step = consumed_map.get(&key).copied().unwrap_or(0);

        for &step in &windows {
            if step <= last_step && last_step != 0 {
                continue; // Prevent replay of same or older time step
            }
            let candidate = generate_hotp_code(&secret_bytes, step, seed.digits, use_sha256);
            if candidate.trim() == submitted_code.trim() {
                consumed_map.insert(key, step);
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Registers a new TOTP seed from params.
    pub fn register_seed(&self, user_id: &str, params: &TotpParams) -> Result<(), &'static str> {
        let _ = decode_base32(&params.secret_base32)?;
        let entry = TotpSeedEntry {
            service_name: params.service_name.clone(),
            secret_base32: params.secret_base32.clone(),
            digits: params.digits,
            period_secs: params.period_secs,
            algorithm: params.algorithm.clone(),
        };
        self.vault.store_totp_seed(user_id, &entry);
        Ok(())
    }

    /// Registers a new TOTP seed from an `otpauth://totp/...` URI.
    pub fn register_from_uri(&self, user_id: &str, uri: &str) -> Result<TotpParams, &'static str> {
        let params = parse_otpauth_uri(uri)?;
        self.register_seed(user_id, &params)?;
        Ok(params)
    }
}

/// Protocol wrapper for the TOTP authenticator.
#[derive(Debug, Clone)]
pub struct TotpAuthenticatorConnector {
    authenticator: Arc<TotpAuthenticator>,
}

impl TotpAuthenticatorConnector {
    /// Creates a connector wrapper for TOTP authenticator.
    pub fn new(authenticator: Arc<TotpAuthenticator>) -> Self {
        Self { authenticator }
    }
}

impl KinetiConnectorProtocol for TotpAuthenticatorConnector {
    fn connector_name(&self) -> &'static str {
        "totp_authenticator"
    }

    fn supported_actions(&self) -> &[&'static str] {
        &[
            "store_totp_seed",
            "get_current_code",
            "verify_totp_code",
            "list_services",
            "remove_totp_seed",
        ]
    }

    fn evaluate_consequence(&self, action: &str, _payload: &Value) -> ConsequenceLevel {
        match action {
            "get_current_code" => ConsequenceLevel::Operational,
            "verify_totp_code" | "list_services" => ConsequenceLevel::Trivial,
            "store_totp_seed" | "remove_totp_seed" => ConsequenceLevel::HighConsequence,
            _ => ConsequenceLevel::HighConsequence,
        }
    }

    fn execute_verified(
        &self,
        action: &str,
        payload: &Value,
    ) -> Result<Value, ConnectorProtocolError> {
        let user_id = get_str_property(payload, "user_id").unwrap_or("default");
        let service = get_str_property(payload, "service_name").unwrap_or("");
        let now_sec = kineti_core::current_epoch_millis() / 1000;

        match action {
            "get_current_code" => {
                match self.authenticator.current_code(user_id, service, now_sec) {
                    Ok((code, rem)) => {
                        let mut map = BTreeMap::new();
                        map.insert("code".to_string(), Value::String(code));
                        map.insert("seconds_remaining".to_string(), Value::from(rem));
                        map.insert("service".to_string(), Value::String(service.to_string()));
                        Ok(Value::Object(map))
                    }
                    Err(err) => Err(ConnectorProtocolError::ExecutionFailed(err.to_string())),
                }
            }
            "verify_totp_code" => {
                let submitted = get_str_property(payload, "code").unwrap_or("");
                match self.authenticator.verify_code(user_id, service, submitted, now_sec) {
                    Ok(verified) => {
                        let mut map = BTreeMap::new();
                        map.insert("verified".to_string(), Value::Bool(verified));
                        Ok(Value::Object(map))
                    }
                    Err(err) => Err(ConnectorProtocolError::ExecutionFailed(err.to_string())),
                }
            }
            "list_services" => {
                let services = self.authenticator.vault.list_totp_services(user_id);
                let val_arr: Vec<Value> = services.into_iter().map(Value::String).collect();
                let mut map = BTreeMap::new();
                map.insert("services".to_string(), Value::Array(val_arr));
                Ok(Value::Object(map))
            }
            "store_totp_seed" => {
                let secret = get_str_property(payload, "secret_base32").unwrap_or("");
                let digits = if let Value::Object(m) = payload {
                    m.get("digits").and_then(|v| match v {
                        Value::Number(n) => n.as_str().parse::<u32>().ok(),
                        _ => None,
                    }).unwrap_or(6)
                } else {
                    6
                };
                let params = TotpParams {
                    service_name: service.to_string(),
                    account_name: user_id.to_string(),
                    secret_base32: secret.to_string(),
                    digits,
                    period_secs: 30,
                    algorithm: "SHA1".to_string(),
                };
                match self.authenticator.register_seed(user_id, &params) {
                    Ok(()) => {
                        let mut map = BTreeMap::new();
                        map.insert("status".to_string(), Value::String("seed_stored".to_string()));
                        Ok(Value::Object(map))
                    }
                    Err(err) => Err(ConnectorProtocolError::ExecutionFailed(err.to_string())),
                }
            }
            "remove_totp_seed" => {
                let removed = self.authenticator.vault.delete_totp_seed(user_id, service);
                let mut map = BTreeMap::new();
                map.insert("removed".to_string(), Value::Bool(removed));
                Ok(Value::Object(map))
            }
            other => Err(ConnectorProtocolError::UnsupportedAction(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Standard RFC 6238 test vectors: Secret = "12345678901234567890" in ASCII
    // In Base32: "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ"
    const RFC_SECRET_BASE32: &str = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ";

    #[test]
    fn test_rfc6238_standard_vectors_sha1() {
        let secret_bytes = decode_base32(RFC_SECRET_BASE32).expect("Decodes Base32");
        assert_eq!(secret_bytes, b"12345678901234567890");

        // Test at epoch time T=59s (step = 1, counter = 1)
        let code_t1 = generate_hotp_code(&secret_bytes, 1, 8, false);
        assert_eq!(code_t1, "94287082");

        // 6-digit test
        let code_6 = generate_hotp_code(&secret_bytes, 1, 6, false);
        assert_eq!(code_6, "287082");
    }

    #[test]
    fn test_otpauth_uri_parser() {
        let uri = "otpauth://totp/Google:john%40example.com?secret=JBSWY3DPEHPK3PXP&issuer=Google&digits=6&period=30";
        let params = parse_otpauth_uri(uri).expect("Parses valid URI");
        assert_eq!(params.service_name, "Google");
        assert_eq!(params.account_name, "john@example.com");
        assert_eq!(params.secret_base32, "JBSWY3DPEHPK3PXP");
        assert_eq!(params.digits, 6);
        assert_eq!(params.period_secs, 30);
    }

    #[test]
    fn test_totp_authenticator_verification_and_drift() {
        let vault = Arc::new(CredentialVault::new("test_secret"));
        let auth = TotpAuthenticator::new(vault);

        let params = TotpParams {
            service_name: "GitHub".to_string(),
            account_name: "octocat".to_string(),
            secret_base32: RFC_SECRET_BASE32.to_string(),
            digits: 6,
            period_secs: 30,
            algorithm: "SHA1".to_string(),
        };
        auth.register_seed("user_01", &params).unwrap();

        let epoch = 1710000000u64; // arbitrary fixed epoch
        let (current_code, _) = auth.current_code("user_01", "GitHub", epoch).unwrap();

        // 1. Current code verifies
        let ok = auth.verify_code("user_01", "GitHub", &current_code, epoch).unwrap();
        assert!(ok);

        // 2. Replay prevention: Same code in same window fails on second consumption
        let replay = auth.verify_code("user_01", "GitHub", &current_code, epoch).unwrap();
        assert!(!replay);

        // 3. Next window (+30s) produces new code and verifies
        let epoch_next = epoch + 30;
        let (next_code, _) = auth.current_code("user_01", "GitHub", epoch_next).unwrap();
        assert_ne!(current_code, next_code);
        let ok_next = auth.verify_code("user_01", "GitHub", &next_code, epoch_next).unwrap();
        assert!(ok_next);
    }
}
