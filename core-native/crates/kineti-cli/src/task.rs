//! Task requests and an explicit owner IPC channel; no project-local self-approval.
use crate::{
    store::{self, Error, Result},
    task_grant::Grant,
};
use serde_json::{json, Value};
use std::{fs, io::Read, path::Path};
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "task_owner.rs"]
mod owner;

/// Stable decisions always identify the action; allow is never a blanket grant.
pub fn decision(kind: &str, action: &str, reason: &str) -> Value {
    json!({"schema":"kineti-task-decision-v1","decision":kind,"action":action,"reason":reason,"release_authority":false})
}
/// Decisions that require stopping dependent actions.
pub fn blocked(v: &Value) -> bool {
    matches!(
        v["decision"].as_str(),
        Some("deny" | "needs-scope" | "needs-action-approval" | "inspect-uncertain")
    )
}
/// Bounded JSON input; errors never print raw credential-bearing input.
pub fn read_request(path: &Path) -> Result<Value> {
    let mut bytes = vec![];
    fs::File::open(path)?.take(65537).read_to_end(&mut bytes)?;
    if bytes.len() > 65536 {
        return store::fail(2, "task request exceeds 64 KiB");
    }
    serde_json::from_slice(&bytes).map_err(|_| Error(2, "invalid task request JSON".into()))
}
/// Plan only: metadata discovery never approves a new command.
pub fn plan(path: &Path) -> Result<Value> {
    Ok(Grant::parse(read_request(path)?)?.summary())
}
/// Read only protected owner records. Never starts, approves or resumes a task.
pub fn inspect(base: &Path, id: &str) -> Result<Value> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        owner::inspect(base, id)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (base, id);
        store::fail(3, "owner inspection unsupported on this platform")
    }
}
/// Owner approval requires the separate Linux owner role and its interactive TTY.
pub fn approve(base: &Path, request: &Path) -> Result<Value> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        owner::approve(base, request)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (base, request);
        store::fail(
            3,
            "Linux owner service unsupported on this platform; no approval fallback",
        )
    }
}
/// Serve an already approved, externally protected grant. This does not approve it.
pub fn serve(base: &Path, id: &str) -> Result<Value> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        owner::serve(base, id)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (base, id);
        store::fail(3, "Linux owner service unsupported on this platform")
    }
}
/// Separate owner-console entry point; arms one fresh memory-only login only.
pub fn serve_model(base: &Path, id: &str) -> Result<Value> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        owner::serve_model(base, id)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (base, id);
        store::fail(3, "Linux model owner service unsupported")
    }
}
/// Kernel-authenticated local client. Agent-supplied flags cannot assume UID 0.
pub fn request(socket: &Path, id: &str, session: &str, method: &str, root: &Path) -> Result<Value> {
    crate::task_grant::id(id)?;
    crate::task_grant::id(session)?;
    if ![
        "status", "run", "check", "recover", "cancel", "revoke", "pause",
    ]
    .contains(&method)
    {
        return store::fail(2, "unknown task operation");
    }
    #[cfg(target_os = "linux")]
    {
        owner::request(socket, id, session, method, root)
    }
    #[cfg(target_os = "macos")]
    {
        crate::task_pipe::Channel::connect(socket)?.request(id, session, method, root)
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (socket, root);
        store::fail(
            3,
            "Linux owner IPC unavailable; no project-local authority fallback",
        )
    }
}
