//! kinetid — the governance daemon (Phase 1).
//!
//! std-only: `UnixListener` + thread-per-connection, no async runtime.
//! Owns ONE `DirectBackend` (the same types direct mode uses — §R1), so the
//! spend pool is authoritative process-wide and journal appends are
//! serialized through a single chain-validating writer. Full O(N) chain
//! verification stays OUT of the hot path and remains mandatory at the ship
//! gate / `kineti verify`.

use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::Arc;

use crate::config::Config;
use crate::ipc::{
    dto,
    {clean_stale_socket, restrict_socket_perms, socket_path, DirectBackend},
};
use crate::memory::journal::Record;

const DAEMON_JSON: &str = ".kineti/daemon.json";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usd(micro: u64) -> String {
    format!("${:.2}", micro as f64 / 1_000_000.0)
}

/// Run the daemon against `root` until a Shutdown request arrives.
pub fn serve(root: &Path, _foreground: bool) -> i32 {
    let cfg = Config::load();
    let ceilings = crate::ipc::pool::Ceilings {
        global_micro: (cfg.limits.global_usd.max(0.0) * 1_000_000.0).round() as u64,
        stage_micro: (cfg.limits.per_stage_usd > 0.0)
            .then(|| (cfg.limits.per_stage_usd * 1_000_000.0).round() as u64),
        worker_micro: (cfg.limits.max_worker_usd > 0.0)
            .then(|| (cfg.limits.max_worker_usd * 1_000_000.0).round() as u64),
    };

    clean_stale_socket(root);
    let sock = socket_path(root);
    if let Some(dir) = sock.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let listener = match UnixListener::bind(&sock) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("kinetid: cannot bind {}: {e}", sock.display());
            return 1;
        }
    };
    restrict_socket_perms(&sock);
    stamp(root, &sock);

    let backend = match DirectBackend::new(root, ceilings.clone()) {
        Ok(b) => Arc::new(b),
        Err(e) => {
            eprintln!("kinetid: {e}");
            let _ = std::fs::remove_file(&sock);
            let _ = std::fs::remove_file(root.join(DAEMON_JSON));
            return 1;
        }
    };
    println!(
        "kinetid v{VERSION} listening on {} (cap {}{}, per-stage {}, per-worker {})",
        sock.display(),
        usd(ceilings.global_micro),
        if ceilings.stage_micro.is_some() { ", stage cap enforced" } else { "" },
        ceilings.stage_micro.map(usd).unwrap_or_else(|| "off".into()),
        ceilings.worker_micro.map(usd).unwrap_or_else(|| "off".into()),
    );

    for conn in listener.incoming() {
        let Ok(stream) = conn else { continue };
        let be = backend.clone();
        let rootp = root.to_path_buf();
        std::thread::spawn(move || {
            handle_connection(be, &rootp, stream);
        });
    }
    0
}

fn stamp(root: &Path, sock: &Path) {
    let doc = serde_json::json!({
        "pid": std::process::id(),
        "version": VERSION,
        "sock": sock.display().to_string(),
        "started_at": crate::memory::journal::now_iso(),
    });
    let _ = std::fs::write(root.join(DAEMON_JSON), doc.to_string());
}

fn unstamp(root: &Path) {
    let _ = std::fs::remove_file(root.join(DAEMON_JSON));
}

fn handle_connection(be: Arc<DirectBackend>, root: &Path, stream: UnixStream) {
    let mut reader = BufReader::new(match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    });
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) | Err(_) => return,
        Ok(_) => {}
    }
    let req: dto::Req = match serde_json::from_str(line.trim()) {
        Ok(r) => r,
        Err(e) => {
            reply(&stream, &dto::Resp::Err(format!("bad request: {e}")));
            return;
        }
    };

    // A panicking handler must degrade to an Err frame, never kill silently.
    let was_shutdown = matches!(req, dto::Req::Shutdown);
    let resp = std::panic::catch_unwind(AssertUnwindSafe(|| dispatch(&be, root, req)))
        .unwrap_or_else(|_| dto::Resp::Err("internal error".into()));

    reply(&stream, &resp);
    if was_shutdown {
        unstamp(root);
        let _ = std::fs::remove_file(socket_path(root));
        // Response already flushed; exiting here is safe and simple.
        std::process::exit(0);
    }
}

fn reply(mut stream: &UnixStream, resp: &dto::Resp) {
    if let Ok(line) = serde_json::to_string(resp) {
        use std::io::Write;
        let _ = writeln!(stream, "{line}");
        let _ = stream.flush();
    }
}

fn dispatch(be: &DirectBackend, root: &Path, req: dto::Req) -> dto::Resp {
    use crate::ipc::{JournalWriter, SpendService};
    match req {
        dto::Req::Ping => dto::Resp::Pong,
        dto::Req::SpendReserve { ctx } => match be.reserve(&ctx) {
            Ok(r) => dto::Resp::Reserved(r),
            Err(e) => dto::Resp::Err(e),
        },
        dto::Req::SpendSettle { res, actual_micro_usd } => match be.settle(&res, actual_micro_usd)
        {
            Ok(total) => dto::Resp::Settled { total_micro_usd: total },
            Err(e) => dto::Resp::Err(e),
        },
        dto::Req::SpendSnapshot => match be.snapshot() {
            Ok(s) => dto::Resp::Snapshot(s),
            Err(e) => dto::Resp::Err(e),
        },
        dto::Req::SpendResetIfRequested => match be.reset_if_human_requested(root) {
            Ok(b) => dto::Resp::Reset(b),
            Err(e) => dto::Resp::Err(e),
        },
        dto::Req::JournalHead { branch } => match be.head(&branch) {
            Ok(h) => dto::Resp::Head(h),
            Err(e) => dto::Resp::Err(e),
        },
        dto::Req::AppendBatch { branch, records } => {
            let mut parsed = Vec::with_capacity(records.len());
            for l in records {
                match serde_json::from_str::<Record>(&l) {
                    Ok(r) => parsed.push(r),
                    Err(e) => return dto::Resp::Err(format!("bad record: {e}")),
                }
            }
            match be.append_batch(&branch, parsed) {
                Ok(()) => dto::Resp::Appended,
                Err(e) => dto::Resp::Err(e),
            }
        }
        dto::Req::HaltStatus => dto::Resp::Halted { tripped: be.pool.is_tripped() },
        dto::Req::Shutdown => {
            dto::Resp::Pong
        }
    }
}
