//! Native PKCE OAuth2 (RFC 7636, Phase 8): std-only implementation.
//!
//! Flow: `prepare_login` builds an authorization URL with an S256 challenge
//! and binds an ephemeral loopback listener; the user's browser redirects to
//! it; `await_callback` captures `code` (validating `state`); the code is
//! exchanged for a token saved to `~/.kineti/auth/<provider>.json` (mode
//! 0600). Providers then accept a bearer token OR the classic env key —
//! valid tokens win, expired ones attempt one refresh, then fall back to
//! the env key with a warning.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

// ── primitives ───────────────────────────────────────────────────────────────

const UNRESERVED: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~";

/// Cryptographically-random PKCE verifier (64 chars, RFC 7636 §4.1 range).
pub fn new_verifier() -> String {
    rand_chars(64)
}

pub fn rand_state() -> String {
    rand_chars(24)
}

fn rand_chars(n: usize) -> String {
    let mut buf = vec![0u8; n * 2];
    {
        use std::io::Read;
        let mut f = std::fs::File::open("/dev/urandom")
            .expect("/dev/urandom unavailable");
        f.read_exact(&mut buf).expect("short read from /dev/urandom");
    }
    buf.iter()
        .filter_map(|b| {
            // rejection sampling keeps the mapping uniform over UNRESERVED
            let idx = (*b % 64) as usize;
            if idx < UNRESERVED.len() {
                Some(UNRESERVED[idx] as char)
            } else {
                None
            }
        })
        .take(n)
        .collect()
}

/// base64url without padding (RFC 7636 §4.2 alphabet).
pub fn b64url(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(TABLE[(n >> 18) as usize & 63] as char);
        out.push(TABLE[(n >> 12) as usize & 63] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(n >> 6) as usize & 63] as char);
        }
        if chunk.len() > 2 {
            out.push(TABLE[n as usize & 63] as char);
        }
    }
    out
}

/// S256 code challenge: BASE64URL(SHA256(verifier)), no padding.
pub fn challenge_s256(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}

// ── token storage ────────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct StoredToken {
    pub provider: String,
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: Option<String>,
    /// unix seconds; None = non-expiring
    #[serde(default)]
    pub expires_at: Option<u64>,
}

/// $HOME resolution without the `dirs` crate — pulling it in dragged
/// CoreFoundation + libiconv onto every macOS process launch (startup I/O
/// and dyld cost for zero benefit).
fn home_dir() -> PathBuf {
    match std::env::var("HOME") {
        Ok(h) if !h.is_empty() => PathBuf::from(h),
        _ => PathBuf::from("."),
    }
}

pub fn auth_dir() -> PathBuf {
    home_dir().join(".kineti/auth")
}

pub fn store_path(provider: &str) -> PathBuf {
    auth_dir().join(format!("{provider}.json"))
}

pub fn save_token(t: &StoredToken) -> Result<(), String> {
    let dir = auth_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {e}", dir.display()))?;
    let path = store_path(&t.provider);
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_string_pretty(t).unwrap())
        .map_err(|e| format!("token write: {e}"))?;
    // 0700 dir + 0600 file: bearer secrets never world-readable
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700));
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o600));
    }
    std::fs::rename(&tmp, &path).map_err(|e| format!("token rename: {e}"))
}

pub fn load_token(provider: &str) -> Option<StoredToken> {
    serde_json::from_str(
        &std::fs::read_to_string(store_path(provider)).ok()?,
    )
    .ok()
}

pub fn logout(provider: &str) -> bool {
    std::fs::remove_file(store_path(provider)).is_ok()
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// True when the token cannot be trusted anymore (60 s clock skew).
pub fn is_expired(t: &StoredToken, now: u64) -> bool {
    matches!(t.expires_at, Some(exp) if now + 60 >= exp)
}

#[derive(Clone, Debug)]
pub struct TokenStatus {
    pub provider: String,
    pub expired: bool,
    pub expires_at: Option<u64>,
    pub has_refresh: bool,
}

pub fn status_all() -> Vec<TokenStatus> {
    let now = now_unix();
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(auth_dir()) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if let Some(provider) = name.strip_suffix(".json") {
                if let Some(t) = load_token(provider) {
                    out.push(TokenStatus {
                        expired: is_expired(&t, now),
                        provider: t.provider,
                        expires_at: t.expires_at,
                        has_refresh: t.refresh_token.is_some(),
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.provider.cmp(&b.provider));
    out
}

// ── login flow ───────────────────────────────────────────────────────────────

pub struct LoginSession {
    pub provider: String,
    pub auth_url: String,
    pub verifier: String,
    pub state: String,
    pub port: u16,
    listener: std::net::TcpListener,
}

/// Build the authorization URL + bind the loopback callback listener.
pub fn prepare_login(provider: &str, cfg: &crate::config::OAuthCfg) -> Result<LoginSession, String> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))
        .map_err(|e| format!("bind loopback: {e}"))?;
    let port = listener.local_addr().map_err(|e| format!("local addr: {e}"))?.port();
    let verifier = new_verifier();
    let state = rand_state();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        cfg.authorize_url.trim_end_matches('/'),
        urlencode(&cfg.client_id),
        urlencode(&redirect_uri),
        urlencode(&cfg.scopes),
        urlencode(&state),
        challenge_s256(&verifier),
    );
    Ok(LoginSession {
        provider: provider.to_string(),
        auth_url,
        verifier,
        state,
        port,
        listener,
    })
}

fn urlencode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Block until the browser redirect arrives; validates state; returns code.
pub fn await_callback(sess: &LoginSession, timeout: Duration) -> Result<String, String> {
    sess.listener
        .set_nonblocking(true)
        .map_err(|e| format!("nonblocking: {e}"))?;
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if std::time::Instant::now() > deadline {
            return Err("timed out waiting for authorization redirect".into());
        }
        match sess.listener.accept() {
            Ok((stream, _)) => {
                // BSD/Darwin: accepted sockets inherit the listener's
                // non-blocking mode — force blocking IO or every read/write
                // silently no-ops with WouldBlock.
                stream.set_nonblocking(false).ok();
                let mut reader = BufReader::new(stream.try_clone().map_err(|e| e.to_string())?);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() || line.is_empty() {
                    continue;
                }
                // drain remaining headers so the browser isn't left hanging
                loop {
                    let mut h = String::new();
                    match reader.read_line(&mut h) {
                        Ok(0) | Err(_) => break,
                        Ok(_) if h.trim().is_empty() => break,
                        Ok(_) => {}
                    }
                }
                let target = line.split_whitespace().nth(1).unwrap_or("").to_string();
                // reply regardless of validity — browser shows a friendly page
                let mut resp = stream.try_clone().map_err(|e| e.to_string())?;
                let _ = writeln!(resp, "HTTP/1.1 200 OK\r");
                let _ = writeln!(resp, "Content-Type: text/html\r");
                let _ = writeln!(resp, "Connection: close\r");
                let _ = writeln!(resp, "\r");
                let _ = writeln!(
                    resp,
                    "<html><body><h3>Kineti</h3>Authorization received — you may close this tab.</body></html>"
                );
                let _ = resp.flush();
                use std::net::Shutdown;
                let _ = resp.shutdown(Shutdown::Both); // deliver FIN after body

                let q = target.split('?').nth(1).unwrap_or("");
                let params = parse_query(q);
                if let Some(err) = params.get("error") {
                    return Err(format!("provider returned error: {err}"));
                }
                let got_state = params.get("state").cloned().unwrap_or_default();
                if got_state != sess.state {
                    return Err("state mismatch — possible CSRF, aborting".into());
                }
                if let Some(code) = params.get("code") {
                    return Ok(code.clone());
                }
                // keep listening (favicon noise etc.)
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(format!("accept: {e}")),
        }
    }
}

fn parse_query(q: &str) -> HashMap<String, String> {
    let mut m = HashMap::new();
    for pair in q.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            m.insert(urldecode(k), urldecode(v));
        }
    }
    m
}

fn urldecode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let hex = std::str::from_utf8(&b[i + 1..i + 3]).unwrap_or("");
                out.push(u8::from_str_radix(hex, 16).unwrap_or(b[i]));
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

use std::collections::HashMap;

/// Exchange the authorization code for tokens (PKCE verifier included).
pub fn exchange_code(
    provider: &str,
    cfg: &crate::config::OAuthCfg,
    code: &str,
    verifier: &str,
    port: u16,
) -> Result<StoredToken, String> {
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");
    let form = format!(
        "grant_type=authorization_code&code={}&client_id={}&redirect_uri={}&code_verifier={}",
        urlencode(code),
        urlencode(&cfg.client_id),
        urlencode(&redirect_uri),
        urlencode(verifier),
    );
    token_request(cfg.token_url.trim_end_matches('/'), &form, provider)
}

/// Refresh-grant flow for expiring tokens.
pub fn refresh(
    provider: &str,
    cfg: &crate::config::OAuthCfg,
    refresh_token: &str,
) -> Result<StoredToken, String> {
    let form = format!(
        "grant_type=refresh_token&refresh_token={}&client_id={}",
        urlencode(refresh_token),
        urlencode(&cfg.client_id),
    );
    token_request(cfg.token_url.trim_end_matches('/'), &form, provider)
}

fn token_request(url: &str, form: &str, provider: &str) -> Result<StoredToken, String> {
    let resp = ureq::post(url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .timeout(Duration::from_secs(30))
        .send_string(form);
    let body = match resp {
        Ok(r) => r.into_string().map_err(|e| format!("decode: {e}"))?,
        Err(ureq::Error::Status(code, r)) => {
            return Err(format!(
                "token endpoint http {code}: {}",
                truncate(&r.into_string().unwrap_or_default())
            ))
        }
        Err(e) => return Err(format!("http: {e}")),
    };
    let v: serde_json::Value = serde_json::from_str(&body).map_err(|e| format!("bad json: {e}"))?;
    let access = v["access_token"]
        .as_str()
        .ok_or_else(|| format!("no access_token in response: {}", truncate(&body)))?
        .to_string();
    let expires_in = v["expires_in"].as_u64();
    Ok(StoredToken {
        provider: provider.to_string(),
        access_token: access,
        refresh_token: v["refresh_token"].as_str().map(String::from),
        expires_at: expires_in.map(|s| now_unix() + s),
    })
}

fn truncate(s: &str) -> String {
    if s.len() > 200 { format!("{}…", &s[..200]) } else { s.to_string() }
}

// ── bearer resolution (the seam providers use) ──────────────────────────────

/// Valid stored token wins; expired attempts ONE refresh; otherwise fall
/// back to the classic env-var key. Never logs secret material.
pub fn resolve_bearer(provider: &str, p: &crate::config::ProviderCfg) -> Result<String, String> {
    if !provider.is_empty() {
        if let Some(t) = load_token(provider) {
            if !is_expired(&t, now_unix()) {
                return Ok(t.access_token);
            }
            if let (Some(rt), Some(oa)) = (&t.refresh_token, &p.auth) {
                match refresh(provider, oa, rt) {
                    Ok(new) => {
                        let _ = save_token(&new);
                        println!("   ⚙ oauth: refreshed token for '{provider}'");
                        return Ok(new.access_token);
                    }
                    Err(e) => eprintln!("⚠ oauth refresh failed ({e}) — falling back to env key"),
                }
            } else {
                eprintln!("⚠ oauth token expired for '{provider}' — falling back to env key");
            }
        }
    }
    std::env::var(&p.api_key_env).map_err(|_| format!("env var {} not set", p.api_key_env))
}
