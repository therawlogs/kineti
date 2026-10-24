//! Disposable demonstrations including the deliberately retained fabricated-output limit.
use crate::{
    gate, init, receipt,
    store::{self, Result},
};
use serde_json::{json, Value};
use std::{fs, path::Path};
/// Run without source-checkout or Node/Bun dependencies.
pub fn run(root: &Path) -> Result<Value> {
    let dir = root.join(".kineti/self-test").join(store::nonce()?);
    fs::create_dir_all(&dir)?;
    let result = (|| {
        init::run(
            &dir,
            Some("Disposable native gate self-test"),
            None,
            true,
            false,
        )?;
        fs::write(
            dir.join("declared-test.txt"),
            "Declared fixture: passing output is intentionally fabricated.",
        )?;
        let record = || {
            receipt::run(
                &dir,
                receipt::Run {
                    label: "release".into(),
                    argv: vec![
                        "/bin/sh".into(),
                        "-c".into(),
                        "printf '1 pass\\n0 fail\\n'".into(),
                    ],
                    tests: vec!["declared-test.txt".into()],
                    timeout: 5,
                    kind: "test".into(),
                    ..Default::default()
                },
            )
        };
        record()?;
        gate::ci(&dir, vec!["release".into()], None, 240)?;
        fs::write(dir.join("declared-test.txt"), "changed")?;
        if gate::ci(&dir, vec!["release".into()], None, 240).is_ok() {
            return store::fail(
                1,
                "self-test failed: stale evidence did not block simulated release",
            );
        }
        record()?;
        gate::ci(&dir, vec!["release".into()], None, 240)?;
        let log = dir.join(".kineti/evidence.jsonl");
        let mut records = store::lines(&log)?;
        records.last_mut().unwrap()["signature"] = json!("invalid");
        let bytes = records
            .iter()
            .map(|r| r.to_string() + "\n")
            .collect::<String>();
        fs::write(log, bytes)?;
        if receipt::check(&dir, "release", 240, None).is_ok() {
            return store::fail(1, "self-test failed: invalid signature accepted");
        }
        Ok(
            json!({"stale_release_blocked":true,"fresh_release_passed":true,"invalid_signature_rejected":true,"fabricated_output_accepted":true,"limitation":"An executed command can fabricate a passing summary. Signatures attest recorded bytes, not honest tests.","disposable_fixture":true}),
        )
    })();
    fs::remove_dir_all(&dir)?;
    result
}
