//! Phase 8 acceptance: PKCE S256 matches the official RFC 7636 vector,
//! tokens persist 0600, expiry logic honors clock skew, bearer resolution
//! prefers valid OAuth over env keys (with refresh + fallback), and the
//! full login flow completes offline against a fake IdP.

use std::io::{Read, Write};
use std::path::Path;
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::path::PathBuf;
use std::time::Duration;

use kineti::auth::{
    challenge_s256, exchange_code, is_expired, load_token, new_verifier, prepare_login,
    resolve_bearer, save_token, status_all, StoredToken,
};
use kineti::config::{OAuthCfg, ProviderCfg};

// ── PKCE primitives ──────────────────────────────────────────────────────────

#[test]
fn s256_matches_rfc7636_appendix_b_vector() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    assert_eq!(
        challenge_s256(verifier),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
}

#[test]
fn verifier_is_in_rfc_range_and_charset() {
    for _ in 0..20 {
        let v = new_verifier();
        assert!((43..=128).contains(&v.len()), "len {}", v.len());
        assert!(
            v.chars()
                .all(|c| c.is_ascii_alphanumeric() || "-._~".contains(c)),
            "illegal char in {v}"
        );
    }
}

// ── token storage ────────────────────────────────────────────────────────────

fn home_guard() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let d = std::env::temp_dir().join(format!("kp8-home-{}-{nanos}", std::process::id()));
    std::fs::create_dir_all(d.join(".kineti/auth")).unwrap();
    d
}

/// Serializes every test that redirects HOME (process-global env).
static STORE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn with_home<T>(f: impl FnOnce(&Path) -> T) -> T {
    let _g = STORE_LOCK.lock().unwrap();
    let fake = home_guard();
    let old = std::env::var("HOME").ok();
    std::env::set_var("HOME", &fake);
    let out = f(&fake);
    match old {
        Some(v) => std::env::set_var("HOME", v),
        None => std::env::remove_var("HOME"),
    }
    out
}

#[test]
fn token_store_roundtrip_with_0600() {
    with_home(|home| {
        let t = StoredToken {
            provider: "grok".into(),
            access_token: "secret-access".into(),
            refresh_token: Some("refresh-xyz".into()),
            expires_at: Some(kineti::auth::now_unix() + 3600),
        };
        save_token(&t).unwrap();

        let path = home.join(".kineti/auth/grok.json");
        assert!(path.exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o600, "bearer secret must be owner-only");
        }

        let loaded = load_token("grok").unwrap();
        assert_eq!(loaded.access_token, "secret-access");
        assert_eq!(loaded.refresh_token.as_deref(), Some("refresh-xyz"));
        assert!(status_all().iter().any(|s| s.provider == "grok" && !s.expired));
    });
}

#[test]
fn expiry_honors_clock_skew() {
    let now = kineti::auth::now_unix();
    let fresh = StoredToken { provider: "p".into(), access_token: String::new(), refresh_token: None, expires_at: Some(now + 3600) };
    let dying = StoredToken { provider: "p".into(), access_token: String::new(), refresh_token: None, expires_at: Some(now + 30) }; // inside 60s skew
    let dead = StoredToken { provider: "p".into(), access_token: String::new(), refresh_token: None, expires_at: Some(now - 10) };
    let eternal = StoredToken { provider: "p".into(), access_token: String::new(), refresh_token: None, expires_at: None };
    assert!(!is_expired(&fresh, now));
    assert!(is_expired(&dying, now), "30s left must count as expired (skew)");
    assert!(is_expired(&dead, now));
    assert!(!is_expired(&eternal, now));
}

// ── bearer resolution precedence ─────────────────────────────────────────────

fn prov(name: &str, key_env: &str) -> ProviderCfg {
    ProviderCfg {
        base_url: "http://localhost:9".into(),
        api_key_env: key_env.into(),
        default_model: "m".into(),
        name: name.into(),
        auth: None,
        price_per_1m_input: 0.0,
        price_per_1m_output: 0.0,
    }
}

#[test]
fn bearer_prefers_valid_oauth_then_falls_back_to_env() {
    with_home(|_h| {
        // no token, no env → error names the env var
        std::env::remove_var("KINETI_P8_KEY");
        let err = resolve_bearer("none", &prov("none", "KINETI_P8_KEY")).unwrap_err();
        assert!(err.contains("KINETI_P8_KEY"), "{err}");

        // env present → used
        std::env::set_var("KINETI_P8_KEY", "env-secret");
        assert_eq!(resolve_bearer("none", &prov("none", "KINETI_P8_KEY")).unwrap(), "env-secret");

        // valid stored token WINS over env
        save_token(&StoredToken {
            provider: "winner".into(),
            access_token: "oauth-token".into(),
            refresh_token: None,
            expires_at: Some(kineti::auth::now_unix() + 3600),
        })
        .unwrap();
        std::env::set_var("KINETI_WIN_KEY", "should-not-be-used");
        assert_eq!(
            resolve_bearer("winner", &prov("winner", "KINETI_WIN_KEY")).unwrap(),
            "oauth-token"
        );

        // expired token without refresh → warns and falls back to env
        save_token(&StoredToken {
            provider: "stale".into(),
            access_token: "old".into(),
            refresh_token: None,
            expires_at: Some(kineti::auth::now_unix() - 100),
        })
        .unwrap();
        std::env::set_var("KINETI_STALE_KEY", "env-fallback");
        assert_eq!(
            resolve_bearer("stale", &prov("stale", "KINETI_STALE_KEY")).unwrap(),
            "env-fallback"
        );

        std::env::remove_var("KINETI_P8_KEY");
        std::env::remove_var("KINETI_WIN_KEY");
        std::env::remove_var("KINETI_STALE_KEY");
    });
}

// ── full login flow, fully offline ───────────────────────────────────────────

/// Fake IdP: serves the token endpoint; returns a JSON grant once.
fn fake_token_server() -> (u16, std::sync::Arc<AtomicBool>) {
    use std::net::TcpListener;
    let l = TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = l.local_addr().unwrap().port();
    let hit = std::sync::Arc::new(AtomicBool::new(false));
    let hit2 = hit.clone();
    std::thread::spawn(move || {
        if let Ok((mut s, _)) = l.accept() {
            // frame like a real HTTP server: headers until blank line, then
            // exactly Content-Length bytes — read_to_string would deadlock
            // waiting for a FIN ureq never sends.
            let mut buf = Vec::new();
            let mut tmp = [0u8; 1024];
            loop {
                let n = s.read(&mut tmp).unwrap_or(0);
                if n == 0 {
                    break;
                }
                buf.extend_from_slice(&tmp[..n]);
                let done = buf.windows(4).any(|w| w == b"\r\n\r\n");
                if done {
                    let head = String::from_utf8_lossy(&buf).to_lowercase();
                    if let Some(cl) =
                        head.split("content-length:").nth(1).and_then(|r| {
                            r.split_whitespace().next().and_then(|v| v.parse::<usize>().ok())
                        })
                    {
                        let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n").unwrap() + 4;
                        if buf.len() >= header_end + cl {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
            hit2.store(true, std::sync::atomic::Ordering::SeqCst);
            let req = String::from_utf8_lossy(&buf).to_string();
            assert!(
                req.contains("grant_type=authorization_code"),
                "form body: {req}"
            );
            assert!(req.contains("code_verifier="), "PKCE verifier must be sent");
            let body =
                r#"{"access_token":"fake-at","refresh_token":"fake-rt","expires_in":3600,"token_type":"Bearer"}"#;
            let _ = s.write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                )
                .as_bytes(),
            );
        }
    });
    (port, hit)
}

#[test]
fn offline_login_flow_end_to_end() {
    with_home(|_h| {
        let cfg = OAuthCfg {
            client_id: "kineti-test".into(),
            authorize_url: "https://idp.invalid/authorize".into(),
            token_url: String::new(), // set after server binds
            scopes: "openid profile".into(),
        };
        let (token_port, hit) = fake_token_server();
        let mut cfg = cfg;
        cfg.token_url = format!("http://127.0.0.1:{token_port}/token");

        let sess = prepare_login("test-idp", &cfg).unwrap();
        // authorization URL carries every PKCE parameter
        assert!(sess.auth_url.contains("response_type=code"));
        assert!(sess.auth_url.contains("code_challenge_method=S256"));
        assert!(sess.auth_url.contains(&format!("state={}", sess.state)));

        // simulate the browser redirect from another thread
        let port = sess.port;
        let want_state = sess.state.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
            let req = format!(
                "GET /callback?code=THE-CODE&state={want_state} HTTP/1.1\r\nHost: x\r\n\r\n"
            );
            let _ = s.write_all(req.as_bytes());
            let _ = s.flush();
            let mut buf = String::new();
            let _ = s.read_to_string(&mut buf);
            assert!(buf.contains("200 OK"), "browser page reply");
        });

        let code = kineti::auth::await_callback(&sess, Duration::from_secs(5))
            .expect("callback captured");
        assert_eq!(code, "THE-CODE");

        let tok = exchange_code("test-idp", &cfg, &code, &sess.verifier, sess.port)
            .expect("exchange succeeds offline");
        assert_eq!(tok.access_token, "fake-at");
        assert!(hit.load(std::sync::atomic::Ordering::SeqCst), "token server was hit");

        save_token(&tok).unwrap();
        let st = load_token("test-idp").unwrap();
        assert_eq!(st.refresh_token.as_deref(), Some("fake-rt"));

        // state validation: wrong state must be rejected
        let sess2 = prepare_login("test-idp", &cfg).unwrap();
        let port2 = sess2.port;
        let bad_state = "WRONG";
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(120));
            if let Ok(mut s) = TcpStream::connect(("127.0.0.1", port2)) {
                let _ = s.write_all(
                    format!("GET /callback?code=X&state={bad_state} HTTP/1.1\r\nHost: x\r\n\r\n")
                        .as_bytes(),
                );
            }
        });
        let err = kineti::auth::await_callback(&sess2, Duration::from_secs(5)).unwrap_err();
        assert!(err.contains("state mismatch"), "{err}");
    });
}
