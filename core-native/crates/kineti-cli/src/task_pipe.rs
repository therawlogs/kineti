//! Inherited Unix endpoints are task capabilities, never owner authority.
//! The guest owner retains its endpoints before changing the task child UID.
//! macOS getpeereid authenticates the owner; pipe stat ownership cannot do so.
use crate::store::{self, Error, Result};
use serde_json::{json, Value};
use std::{
    fs::File,
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::FileTypeExt,
    },
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
static SEQUENCE: AtomicU64 = AtomicU64::new(0);
/// Fixed, inherited task transport. It cannot mint or enlarge a grant.
pub struct Channel {
    reader: File,
    writer: File,
}
fn duplicate(fd: i32, _expected_uid: u32) -> Result<File> {
    if !(3..1024).contains(&fd) {
        return store::fail(
            2,
            "task channel requires a nonstandard inherited descriptor",
        );
    }
    let copy = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 64) };
    if copy < 0 {
        return store::fail(3, "task channel descriptor unavailable");
    }
    let file = unsafe { File::from_raw_fd(copy) };
    if !file.metadata()?.file_type().is_socket() {
        return store::fail(3, "task channel must be an inherited connected Unix stream");
    }
    let mut kind: libc::c_int = 0;
    let mut len = std::mem::size_of_val(&kind) as libc::socklen_t;
    if unsafe {
        libc::getsockopt(
            copy,
            libc::SOL_SOCKET,
            libc::SO_TYPE,
            (&mut kind as *mut libc::c_int).cast(),
            &mut len,
        )
    } != 0
        || kind != libc::SOCK_STREAM
    {
        return store::fail(3, "task channel must use an ordered Unix stream");
    }
    #[cfg(target_os = "macos")]
    {
        let mut uid = 0;
        let mut gid = 0;
        if unsafe { libc::getpeereid(copy, &mut uid, &mut gid) } != 0 || uid != _expected_uid {
            return store::fail(3, "deny: unexpected kernel-authenticated task channel peer");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        store::fail(
            3,
            "inherited task channel peer verification is not accepted on this platform",
        )
    }
    #[cfg(target_os = "macos")]
    {
        let flags = unsafe { libc::fcntl(copy, libc::F_GETFL) };
        if flags < 0 || unsafe { libc::fcntl(copy, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
            return store::fail(3, "cannot bound task channel IO");
        }
        Ok(file)
    }
}

fn ready(fd: i32, events: i16, until: Instant) -> Result<()> {
    loop {
        if Instant::now() >= until {
            return store::fail(
                3,
                "inspect-uncertain: task pipe timed out; inspect owner state before retry",
            );
        }
        let mut p = libc::pollfd {
            fd,
            events,
            revents: 0,
        };
        let n = unsafe { libc::poll(&mut p, 1, 100) };
        if n < 0 {
            if std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            return store::fail(3, "inspect-uncertain: task pipe poll failed");
        }
        if p.revents & events != 0 {
            return Ok(());
        }
        if p.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0 {
            return store::fail(3, "inspect-uncertain: task owner pipe closed");
        }
    }
}
impl Channel {
    /// Inherited Unix peer credentials are checked by the kernel, not caller flags.
    pub fn inherited(read_fd: i32, write_fd: i32) -> Result<Self> {
        if read_fd == write_fd {
            return store::fail(2, "task pipe directions must be separate");
        }
        Ok(Self {
            reader: duplicate(read_fd, 0)?,
            writer: duplicate(write_fd, 0)?,
        })
    }
    /// Connect to a guest-owner endpoint without relying on MCP descriptor inheritance.
    #[cfg(target_os = "macos")]
    pub fn connect(path: &Path) -> Result<Self> {
        use std::os::unix::net::UnixStream;
        let stream = UnixStream::connect(path)?;
        let reader = duplicate(stream.as_raw_fd(), 0)?;
        let writer = duplicate(stream.as_raw_fd(), 0)?;
        Ok(Self { reader, writer })
    }
    /// Only the five fixed agent operations; no argv, scope or approval fields.
    pub fn request(&mut self, id: &str, session: &str, method: &str, root: &Path) -> Result<Value> {
        crate::task_grant::id(id)?;
        crate::task_grant::id(session)?;
        if !["status", "run", "check", "recover", "cancel"].contains(&method) {
            return store::fail(3, "deny: owner operation is not a task pipe capability");
        }
        let request_id = format!(
            "{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::SeqCst)
        );
        let root = root.canonicalize()?;
        let mut bytes = serde_json::to_vec(
            &json!({"schema":"kineti-task-pipe-request-v1","request_id":request_id,"grant_id":id,"session":session,"method":method,"workspace":root}),
        )?;
        bytes.push(b'\n');
        if bytes.len() > 8192 {
            return store::fail(2, "task pipe request exceeds fixed scope bound");
        }
        let until =
            Instant::now() + Duration::from_secs(if method == "status" { 10 } else { 86410 });
        let mut sent = 0;
        while sent < bytes.len() {
            ready(self.writer.as_raw_fd(), libc::POLLOUT, until)?;
            match self.writer.write(&bytes[sent..]) {
                Ok(0) => return store::fail(3, "inspect-uncertain: task owner pipe write closed"),
                Ok(n) => sent += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return store::fail(3, "inspect-uncertain: task owner pipe write failed"),
            }
        }
        let mut response = Vec::new();
        loop {
            ready(self.reader.as_raw_fd(), libc::POLLIN, until)?;
            let mut chunk = [0; 4096];
            let n = match self.reader.read(&mut chunk) {
                Ok(0) => {
                    return store::fail(3, "inspect-uncertain: task owner response incomplete")
                }
                Ok(n) => n,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) =>
                {
                    continue
                }
                Err(_) => return store::fail(3, "inspect-uncertain: task owner response failed"),
            };
            response.extend_from_slice(&chunk[..n]);
            if response.len() > 4_000_000 {
                return store::fail(3, "owner response exceeds task pipe bound");
            }
            if response.contains(&b'\n') {
                if response.iter().position(|c| *c == b'\n') != Some(response.len() - 1) {
                    return store::fail(3, "inspect-uncertain: extra task pipe response data");
                }
                let v: Value = serde_json::from_slice(&response)
                    .map_err(|_| Error(3, "invalid task owner response".into()))?;
                crate::task_grant::keys(
                    &v,
                    &[
                        "schema",
                        "request_id",
                        "grant_id",
                        "session",
                        "workspace",
                        "result",
                    ],
                )?;
                if v["schema"] != "kineti-task-pipe-response-v1"
                    || v["request_id"] != request_id
                    || v["grant_id"] != id
                    || v["session"] != session
                    || v["workspace"] != json!(root)
                    || v["result"]["schema"] != "kineti-task-decision-v1"
                    || v["result"]["release_authority"] != false
                    || ![
                        "allow",
                        "deny",
                        "needs-scope",
                        "needs-action-approval",
                        "inspect-uncertain",
                    ]
                    .contains(&v["result"]["decision"].as_str().unwrap_or(""))
                {
                    return store::fail(3, "deny: task owner response binding differs");
                }
                return Ok(v["result"].clone());
            }
        }
    }
}

fn receive(file: &mut File, until: Instant, bound: usize) -> Result<Value> {
    let mut bytes = Vec::new();
    loop {
        ready(file.as_raw_fd(), libc::POLLIN, until)?;
        let mut chunk = [0; 4096];
        let n = match file.read(&mut chunk) {
            Ok(0) => return store::fail(3, "inspect-uncertain: owner task channel closed"),
            Ok(n) => n,
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                continue
            }
            Err(_) => return store::fail(3, "inspect-uncertain: owner task channel read failed"),
        };
        bytes.extend_from_slice(&chunk[..n]);
        if bytes.len() > bound {
            return store::fail(3, "owner task channel frame exceeds bound");
        }
        if bytes.contains(&b'\n') {
            if bytes.iter().position(|x| *x == b'\n') != Some(bytes.len() - 1) {
                return store::fail(3, "deny: task channel does not permit pipelined frames");
            }
            return serde_json::from_slice(&bytes)
                .map_err(|_| Error(3, "invalid task channel frame".into()));
        }
    }
}
fn fixed_method<'a>(q: &'a Value, id: &str, session: &str, root: &Path) -> Result<&'a str> {
    crate::task_grant::keys(
        q,
        &[
            "schema",
            "request_id",
            "grant_id",
            "session",
            "method",
            "workspace",
        ],
    )?;
    if q["schema"] != "kineti-task-pipe-request-v1"
        || q["grant_id"] != id
        || q["session"] != session
        || q["workspace"] != json!(root)
        || !q["request_id"].as_str().is_some_and(|s| {
            !s.is_empty()
                && s.len() <= 80
                && s.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
    {
        return store::fail(3, "deny: task channel binding differs");
    }
    q["method"]
        .as_str()
        .filter(|m| ["status", "run", "check", "recover", "cancel"].contains(m))
        .ok_or_else(|| {
            Error(
                3,
                "deny: owner authority is not a task channel operation".into(),
            )
        })
}
/// Serve a fixed owner-created task capability. The handler receives only an
/// allowed operation, never caller-selected argv, identity, scope or grant data.
/// The caller must bind the grant, task UID, resource controls and external VM
/// lifetime before handing endpoints to its unprivileged child.
pub fn serve_owned(
    read_fd: i32,
    write_fd: i32,
    id: &str,
    session: &str,
    root: &Path,
    expires: i64,
    mut handler: impl FnMut(&str) -> Result<Value>,
) -> Result<()> {
    if unsafe { libc::geteuid() } != 0 {
        return store::fail(
            3,
            "task channel server requires the independent guest owner role",
        );
    }
    crate::task_grant::id(id)?;
    crate::task_grant::id(session)?;
    let root = root.canonicalize()?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if expires <= now || expires > now + 86400 {
        return store::fail(3, "owner task channel requires an unexpired bounded grant");
    }
    let channel = Channel::inherited(read_fd, write_fd)?;
    serve_channel(channel, id, session, &root, expires, &mut handler)
}
fn serve_channel(
    mut channel: Channel,
    id: &str,
    session: &str,
    root: &Path,
    expires: i64,
    handler: &mut impl FnMut(&str) -> Result<Value>,
) -> Result<()> {
    loop {
        let remaining = expires - time::OffsetDateTime::now_utc().unix_timestamp();
        if remaining <= 0 {
            return store::fail(3, "task channel grant expired; stop dependent execution");
        }
        let q = receive(
            &mut channel.reader,
            Instant::now() + Duration::from_secs(remaining as u64),
            8192,
        )?;
        let result = match fixed_method(&q, id, session, root) {
            Ok(method) => match handler(method) {
                Ok(result) => result,
                Err(_) => crate::task::decision(
                    "inspect-uncertain",
                    method,
                    "owner-handler-failed-inspect-before-retry",
                ),
            },
            Err(_) => crate::task::decision(
                "deny",
                "task-request",
                "owner-grant-binding-or-operation-differs",
            ),
        };
        if result["schema"] != "kineti-task-decision-v1"
            || result["release_authority"] != false
            || ![
                "allow",
                "deny",
                "needs-scope",
                "needs-action-approval",
                "inspect-uncertain",
            ]
            .contains(&result["decision"].as_str().unwrap_or(""))
        {
            return store::fail(3, "invalid owner decision; stop dependent execution");
        }
        if time::OffsetDateTime::now_utc().unix_timestamp() >= expires {
            return store::fail(
                3,
                "task grant expired during operation; inspect owner state",
            );
        }
        let mut response = serde_json::to_vec(
            &json!({"schema":"kineti-task-pipe-response-v1","request_id":q["request_id"],"grant_id":id,"session":session,"workspace":root,"result":result}),
        )?;
        response.push(b'\n');
        if response.len() > 4_000_000 {
            return store::fail(3, "owner task response exceeds bound");
        }
        let until = Instant::now() + Duration::from_secs(10);
        let mut sent = 0;
        while sent < response.len() {
            ready(channel.writer.as_raw_fd(), libc::POLLOUT, until)?;
            match channel.writer.write(&response[sent..]) {
                Ok(0) => return store::fail(3, "owner task channel write closed"),
                Ok(n) => sent += n,
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                    ) => {}
                Err(_) => return store::fail(3, "owner task channel write failed"),
            }
        }
    }
}

/// Fixed owner-side socket parameters. Only the independent root owner may bind.
#[cfg(target_os = "macos")]
pub struct SocketBinding<'a> {
    /// Root-owned socket destination in a protected directory.
    pub socket: &'a Path,
    /// Owner-approved grant identifier.
    pub id: &'a str,
    /// Exact task session.
    pub session: &'a str,
    /// Copied workspace, canonicalized before comparison.
    pub workspace: &'a Path,
    /// Absolute UTC expiry, at most one day away.
    pub expires: i64,
    /// Dedicated unprivileged task UID.
    pub task_uid: u32,
    /// Dedicated task socket access group.
    pub task_gid: u32,
}
/// Authenticate each accepted connection before reading its request. This is a
/// transport, not a grant store: the owner must bind resource/lifecycle controls.
#[cfg(target_os = "macos")]
pub fn serve_socket(
    binding: SocketBinding<'_>,
    alive: impl Fn() -> bool,
    mut handler: impl FnMut(&str) -> Result<Value>,
) -> Result<()> {
    use std::{
        fs,
        os::unix::{
            fs::{MetadataExt, PermissionsExt},
            net::UnixListener,
        },
    };
    let SocketBinding {
        socket,
        id,
        session,
        workspace,
        expires,
        task_uid,
        task_gid,
    } = binding;
    if unsafe { libc::geteuid() } != 0 || task_uid == 0 || task_gid == 0 {
        return store::fail(
            3,
            "socket service requires separate owner and task identities",
        );
    }
    crate::task_grant::id(id)?;
    crate::task_grant::id(session)?;
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    if expires <= now || expires > now + 86400 {
        return store::fail(3, "socket service requires an unexpired bounded grant");
    }
    let root = workspace.canonicalize()?;
    let parent = socket
        .parent()
        .ok_or_else(|| Error(2, "socket requires owner directory".into()))?;
    // Every directory is root-controlled; reject symlinks and task-writable parents.
    for path in parent.ancestors() {
        let m = fs::symlink_metadata(path)?;
        if !m.is_dir() || m.uid() != 0 || m.mode() & 0o022 != 0 {
            return store::fail(3, "socket parent is not protected by the guest owner");
        }
    }
    if socket.file_name().is_none() || fs::symlink_metadata(socket).is_ok() {
        return store::fail(3, "socket destination already exists or is invalid");
    }
    let listener = UnixListener::bind(socket)?;
    let name = std::ffi::CString::new(socket.as_os_str().as_encoded_bytes())
        .map_err(|_| Error(2, "invalid socket path".into()))?;
    if unsafe { libc::chown(name.as_ptr(), 0, task_gid) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    fs::set_permissions(socket, fs::Permissions::from_mode(0o660))?;
    listener.set_nonblocking(true)?;
    let deadline = Instant::now() + Duration::from_secs((expires - now) as u64);
    for _ in 0..128 {
        loop {
            if !alive() {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return store::fail(3, "socket grant expired");
            }
            let mut poll = libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            let n = unsafe { libc::poll(&mut poll, 1, 100) };
            if n > 0 && poll.revents & libc::POLLIN != 0 {
                break;
            }
            if n < 0 && std::io::Error::last_os_error().kind() != std::io::ErrorKind::Interrupted {
                return Err(std::io::Error::last_os_error().into());
            }
        }
        let stream = match listener.accept() {
            Ok((s, _)) => s,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => return Err(e.into()),
        };
        let reader = match duplicate(stream.as_raw_fd(), task_uid) {
            Ok(file) => file,
            Err(_) => continue,
        };
        let writer = duplicate(stream.as_raw_fd(), task_uid)?;
        // A disconnected client cannot change the grant or enlarge the catalog.
        // The handler retains durable operation state on uncertain completion.
        let _ = serve_channel(
            Channel { reader, writer },
            id,
            session,
            &root,
            expires,
            &mut handler,
        );
        if time::OffsetDateTime::now_utc().unix_timestamp() >= expires {
            return store::fail(3, "socket grant expired; stop dependent execution");
        }
    }
    store::fail(3, "socket request connection limit reached")
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pipe() -> (File, File) {
        let mut fd = [-1; 2];
        assert_eq!(unsafe { libc::pipe(fd.as_mut_ptr()) }, 0);
        unsafe { (File::from_raw_fd(fd[0]), File::from_raw_fd(fd[1])) }
    }
    #[test]
    fn server_rejects_forged_bindings_and_extra_authority_before_dispatch() {
        let root = std::env::current_dir().unwrap();
        let good = json!({"schema":"kineti-task-pipe-request-v1","request_id":"raw-proof", "grant_id":"task-test","session":"task-session","workspace":root,"method":"status"});
        assert_eq!(
            fixed_method(&good, "task-test", "task-session", &root).unwrap(),
            "status"
        );
        for (field, value) in [
            ("schema", json!("future")),
            ("request_id", json!("")),
            ("request_id", json!("../escape")),
            ("grant_id", json!("another-task")),
            ("session", json!("another-session")),
            ("workspace", json!("/")),
            ("method", json!("approve")),
            ("method", json!("revoke")),
            ("method", json!("pause")),
            ("method", json!("publish")),
            ("argv", json!(["/bin/false"])),
            ("task_uid", json!(0)),
            ("paid_limit", json!(1)),
        ] {
            let mut q = good.clone();
            q[field] = value;
            assert!(
                fixed_method(&q, "task-test", "task-session", &root).is_err(),
                "accepted {field}"
            );
        }
        for field in [
            "schema",
            "request_id",
            "grant_id",
            "session",
            "workspace",
            "method",
        ] {
            let mut q = good.clone();
            q.as_object_mut().unwrap().remove(field);
            assert!(fixed_method(&q, "task-test", "task-session", &root).is_err());
        }
    }
    #[test]
    fn pipelined_requests_are_rejected_without_dispatch() {
        let (mut reader, mut writer) = pipe();
        writer.write_all(b"{}\n{}\n").unwrap();
        assert!(receive(&mut reader, Instant::now() + Duration::from_secs(1), 8192).is_err());
    }
    // Protocol-only tests construct the private endpoints directly. Actual root
    // ownership and UID dropping are covered by the separate guest fixture.
    #[test]
    fn response_binding_and_owner_operations_cannot_expand_task_authority() {
        for valid in [true, false] {
            let (reader, mut server_write) = pipe();
            let (server_read, writer) = pipe();
            let mut channel = Channel { reader, writer };
            let server = std::thread::spawn(move || {
                use std::io::BufRead;
                let mut text = String::new();
                std::io::BufReader::new(server_read)
                    .read_line(&mut text)
                    .unwrap();
                let q: Value = serde_json::from_str(&text).unwrap();
                let result = crate::task::decision("allow", "read-status", "unit protocol fixture");
                writeln!(server_write,"{}",json!({"schema":"kineti-task-pipe-response-v1","request_id":q["request_id"],"grant_id":q["grant_id"],"session":if valid {q["session"].clone()} else {json!("different")},"workspace":q["workspace"],"result":result})).unwrap();
            });
            let root = std::env::current_dir().unwrap();
            assert_eq!(
                channel
                    .request("task-test", "task-session", "status", &root)
                    .is_ok(),
                valid
            );
            server.join().unwrap();
        }
        let (reader, _server_write) = pipe();
        let (_server_read, writer) = pipe();
        let mut channel = Channel { reader, writer };
        for action in ["revoke", "pause", "approve", "publish", "run-argv"] {
            assert_eq!(
                channel
                    .request(
                        "task-test",
                        "task-session",
                        action,
                        &std::env::current_dir().unwrap()
                    )
                    .unwrap_err()
                    .0,
                3
            );
        }
    }
}
