//! Previewable state initialization and ownership-aware Codex installation.
use crate::{
    compat::digest,
    store::{self, Error, Lock, Result},
};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
};
const BEGIN: &str = "# kineti gate owned begin\n";
const END: &str = "# kineti gate owned end\n";
const SKILL:&str="---\nname: kineti-gate\ndescription: Record checks and check reported spending before participating release commands.\n---\n\nUse native kineti test run with exact argv and declared tests, then kineti ci.\nReport missing, stale, invalid evidence or spending-policy failures.\nUse registered undo only after inspection and explicit confirmation.\nA signature does not prove honest tests. This is not automatic interception.\n";
fn bytes(p: &Path) -> Result<Option<String>> {
    store::safe_destination(p)?;
    if p.exists() {
        Ok(Some(fs::read_to_string(p)?))
    } else {
        Ok(None)
    }
}
fn block_range(text: &str) -> Result<Option<std::ops::Range<usize>>> {
    let begins = text.match_indices(BEGIN).collect::<Vec<_>>();
    let ends = text.match_indices(END).collect::<Vec<_>>();
    if begins.is_empty() && ends.is_empty() {
        return Ok(None);
    }
    if begins.len() != 1 || ends.len() != 1 || ends[0].0 < begins[0].0 {
        return store::fail(
            3,
            "damaged installation ownership markers; preserve and inspect",
        );
    }
    Ok(Some(begins[0].0..ends[0].0 + END.len()))
}
/// Preview by default. Applying makes backups first and never repairs damaged records.
pub fn run(
    root: &Path,
    goal: Option<&str>,
    home: Option<&Path>,
    apply: bool,
    uninstall: bool,
) -> Result<Value> {
    let statepath = root.join(".kineti/state.json");
    let manifestpath = root.join(".kineti/install.json");
    let manifest = if manifestpath.exists() {
        store::read(&manifestpath)?
    } else {
        json!({})
    };
    let mut operations: Vec<(PathBuf, Option<String>, Option<String>)> = vec![];
    if !uninstall {
        let state = if statepath.exists() {
            let mut s = store::state(root)?;
            if let Some(g) = goal {
                let existing = s["root_goal"]
                    .as_str()
                    .or_else(|| s["root_goal"]["description"].as_str())
                    .unwrap();
                if g != existing {
                    return store::fail(2, "locked goal cannot be changed");
                }
            }
            let m = s.as_object_mut().unwrap();
            m.entry("version").or_insert(json!(1));
            m.entry("stage").or_insert(json!(1));
            m.entry("gates")
                .or_insert(json!({"spec":"pending","security":"pending"}));
            s
        } else {
            let g = goal.filter(|g| !g.trim().is_empty()).ok_or_else(|| {
                Error(2, "new project requires --goal; no goal is invented".into())
            })?;
            if root.join(".kineti/evidence.jsonl").exists()
                || root.join(".kineti/spend.json").exists()
            {
                return store::fail(
                    3,
                    "missing state alongside historical records; reviewed recovery required",
                );
            }
            json!({"version":1,"project":root.file_name().and_then(|s|s.to_str()).unwrap_or("project"),"root_goal":g,"root_goal_locked_at":store::now(),"stage":1,"gates":{"spec":"pending","security":"pending"},"history":[{"at":store::now(),"event":"native gate initialization; explicit goal"}]})
        };
        let original = bytes(&statepath)?;
        if original
            .as_ref()
            .map(|s| serde_json::from_str::<Value>(s))
            .transpose()?
            .as_ref()
            != Some(&state)
        {
            operations.push((
                statepath,
                original,
                Some(serde_json::to_string_pretty(&state)? + "\n"),
            ));
        }
        let cfg = root.join(".kineti/gate.json");
        if !cfg.exists() {
            operations.push((cfg,None,Some(json!({"version":1,"required_checks":[],"max_age_minutes":240,"note":"Declare required checks explicitly before CI can pass."}).to_string()+"\n")));
        } else {
            store::read(&cfg)?;
        }
        // Inspect accounting without creating or rewriting its state.
        if root.join(".kineti/spend.json").exists() {
            store::read(&root.join(".kineti/spend.json"))?;
            store::lines(&root.join(".kineti/spend.log.jsonl"))?;
        }
    }
    let mut owned_block = None;
    if let Some(home) = home {
        store::safe_destination(home)?;
        let cfg = home.join("config.toml");
        let before = bytes(&cfg)?.unwrap_or_default();
        if !before.is_empty() {
            toml::from_str::<toml::Value>(&before)
                .map_err(|e| Error(3, format!("damaged Codex configuration: {e}")))?;
        }
        let range = block_range(&before)?;
        if let Some(ref r) = range {
            if manifest["block"].as_str() != Some(&before[r.clone()]) {
                return store::fail(
                    3,
                    "Codex registration is edited or owned by another installation; preserved",
                );
            }
        } else if toml::from_str::<toml::Value>(&before)
            .ok()
            .and_then(|v| v.get("mcp_servers").and_then(|m| m.get("kineti")).cloned())
            .is_some()
        {
            return store::fail(
                3,
                "existing unowned kineti registration preserved; review migration explicitly",
            );
        }
        let block = format!(
            "{BEGIN}[mcp_servers.kineti]\ncommand = {}\nargs = [\"mcp\", \"--root\", {}]\n{END}",
            json!(std::env::current_exe()?),
            json!(root)
        );
        let after = if let Some(r) = range {
            let mut s = before.clone();
            s.replace_range(r, if uninstall { "" } else { &block });
            s
        } else if uninstall {
            before.clone()
        } else {
            format!(
                "{}{}{}",
                before,
                if before.ends_with('\n') || before.is_empty() {
                    ""
                } else {
                    "\n"
                },
                block
            )
        };
        if after != before {
            operations.push((
                cfg,
                if before.is_empty() {
                    None
                } else {
                    Some(before)
                },
                Some(after),
            ));
        }
        let skill = home.join("skills/kineti-gate/SKILL.md");
        let before = bytes(&skill)?;
        if let Some(ref b) = before {
            if b != SKILL && manifest["skill_sha256"] != digest(b) {
                return store::fail(3, "modified or unowned gate skill preserved");
            }
        }
        let after = if uninstall { None } else { Some(SKILL.into()) };
        if before != after {
            operations.push((skill, before, after));
        }
        if !uninstall {
            owned_block = Some(block);
        }
    }
    let changes=operations.iter().map(|(p,b,a)|json!({"destination":p,"before":b,"after":a,"backup":root.join(".kineti/backups").join(digest(format!("{}:{}",p.display(),b.as_deref().unwrap_or("missing"))))})).collect::<Vec<_>>();
    let preview = json!({"changes":changes,"applied":false,"codex_detected":home.is_some(),"restart_required":home.is_some()&&!operations.is_empty(),"automatic_interception":false,"historical_records_rewritten":false});
    if !apply {
        return Ok(preview);
    }
    let _lock = Lock::take(&root.join(".kineti/init.lock"))?;
    for (p, b, _) in &operations {
        if bytes(p)? != *b {
            return store::fail(3, "configuration changed since preview; retry inspection");
        }
    }
    for ((_, b, _), change) in operations.iter().zip(&changes) {
        if let Some(b) = b {
            let backup = PathBuf::from(change["backup"].as_str().unwrap());
            if !backup.exists() {
                store::atomic(&backup, b.as_bytes())?;
            } else if fs::read(&backup)? != b.as_bytes() {
                return store::fail(3, "backup collision; original preserved");
            }
        }
    }
    store::append(
        &root.join(".kineti/install-audit.jsonl"),
        &json!({"at":store::now(),"kind":"intent","uninstall":uninstall,"preview":preview}),
    )?;
    for (p, _, after) in operations {
        if let Some(s) = after {
            store::atomic(&p, s.as_bytes())?;
        } else if p.exists() {
            fs::remove_file(p)?;
        }
    }
    if let Some(block) = owned_block {
        store::write(
            &manifestpath,
            &json!({"version":1,"block":block,"skill_sha256":digest(SKILL),"codex_home":home}),
        )?;
    } else if uninstall && manifestpath.exists() {
        fs::remove_file(manifestpath)?;
    }
    store::append(
        &root.join(".kineti/install-audit.jsonl"),
        &json!({"at":store::now(),"kind":"completion","uninstall":uninstall}),
    )?;
    let mut result = preview;
    result["applied"] = json!(true);
    Ok(result)
}
