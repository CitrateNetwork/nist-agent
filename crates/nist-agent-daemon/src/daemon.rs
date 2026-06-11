//! `Daemon` — the long-running process state + tokio event loop.
//!
//! Holds:
//!
//! - The IPC listener (Unix socket).
//! - Shared state behind `Arc<DaemonState>` so per-connection
//!   tasks can read without contention.
//! - The anchor-cadence ticker (sleeps until next nightly /
//!   per-install event + writes to the audit sink).
//!
//! Lifecycle: `Daemon::bind()` → `Daemon::run_until_signal()` (or
//! `Daemon::serve_once()` for tests). Bring-up is split from
//! the run loop so the CLI's `--smoke` path can confirm the
//! socket binds + the audit-sink dir exists without entering
//! the run loop.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Mutex;

use nist_agent_policy::PolicyBundle;

use crate::config::DaemonConfig;
use crate::error::DaemonError;
use crate::ipc::{AuditSummary, IpcRequest, IpcResponse, IPC_PROTOCOL_VERSION};

/// State the daemon shares across IPC connections + the
/// anchor-cadence ticker. Wrapped in `Arc` at construction.
#[derive(Debug)]
pub struct DaemonState {
    pub config: DaemonConfig,
    /// The verified-and-activated PolicyBundle when the operator
    /// configured one. Loaded fail-closed at `Daemon::prepare`
    /// (NIST_AGENT-2026-05-31-001): a configured bundle that
    /// fails to read/verify/activate refuses startup. `None` only
    /// in the documented minimal/air-gap smoke mode (no
    /// `policy.bundle_path` set). Runtime surfaces gate on this.
    pub policy: Option<PolicyBundle>,
    /// Wall-clock unix-time of the last successful anchor write.
    /// Surfaced via `IpcResponse::Status::last_anchor_unix`.
    pub last_anchor_unix: Mutex<Option<i64>>,
    /// In-memory ring buffer of recent audit summaries. The
    /// daemon writes here in lockstep with the WORM audit sink
    /// so IPC can answer `RecentAudit` without re-reading the
    /// filesystem.
    pub recent_audit: Mutex<Vec<AuditSummary>>,
    /// Pending-queue depth as the daemon last observed it. The
    /// real binding to `citrate_agent_core::hitl::ApprovalQueue`
    /// arrives once the binding upstream goes through the
    /// federation manifest bump; today the field is operator-
    /// driven via `Daemon::set_queue_depth` (e.g. test hooks +
    /// the future HITL wiring).
    pub queue_depth: Mutex<usize>,
}

impl DaemonState {
    fn new(config: DaemonConfig, policy: Option<PolicyBundle>) -> Arc<Self> {
        Arc::new(Self {
            config,
            policy,
            last_anchor_unix: Mutex::new(None),
            recent_audit: Mutex::new(Vec::new()),
            queue_depth: Mutex::new(0),
        })
    }
}

pub struct Daemon {
    state: Arc<DaemonState>,
    listener: Option<UnixListener>,
}

impl Daemon {
    /// Validate config, load + verify + activate the configured
    /// PolicyBundle (fail-closed; NIST_AGENT-2026-05-31-001), and
    /// prepare the audit-sink directory. Does not bind the IPC
    /// socket — that happens in `bind`.
    pub fn prepare(config: DaemonConfig) -> Result<Self, DaemonError> {
        config.validate()?;
        let policy = crate::policy::load_policy(&config)?;
        std::fs::create_dir_all(&config.daemon.audit_sink_path)
            .map_err(|e| DaemonError::Filesystem(format!("mkdir audit_sink_path: {e}")))?;
        let state = DaemonState::new(config, policy);
        Ok(Self {
            state,
            listener: None,
        })
    }

    /// Bind the IPC Unix socket. Removes any stale socket at the
    /// path first (a previous run that crashed leaves a stale
    /// file). The caller's `--smoke` path stops here.
    pub fn bind(&mut self) -> Result<PathBuf, DaemonError> {
        let path = self.state.config.daemon.ipc_socket_path.clone();
        // Stale-socket clean-up.
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| DaemonError::Filesystem(format!("remove stale socket: {e}")))?;
        }
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DaemonError::Filesystem(format!(
                        "mkdir socket parent {}: {e}",
                        parent.display()
                    ))
                })?;
            }
        }
        let listener = UnixListener::bind(&path)
            .map_err(|e| DaemonError::Io(format!("bind socket {}: {e}", path.display())))?;
        // Owner-only socket (NIST_AGENT-2026-05-31-003): without
        // this the umask default leaves the socket world-
        // connectable and any local user can read audit/queue
        // metadata. Fail closed if the chmod doesn't take.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
                .map_err(|e| DaemonError::Filesystem(format!("chmod socket 0600: {e}")))?;
        }
        self.listener = Some(listener);
        Ok(path)
    }

    /// Shared state pointer. Test hooks + the anchor ticker
    /// hold this.
    pub fn state(&self) -> Arc<DaemonState> {
        Arc::clone(&self.state)
    }

    /// Set the queue-depth observation. The future HITL wiring
    /// drives this from `ApprovalQueue::pending().len()`; today
    /// it's exposed so tests + the CLI can poke a value.
    pub async fn set_queue_depth(&self, depth: usize) {
        let mut g = self.state.queue_depth.lock().await;
        *g = depth;
    }

    /// Record an anchor write. Updates the timestamp + appends
    /// a summary entry. The real anchor-cadence ticker calls
    /// this after a successful WORM + chain write.
    pub async fn record_anchor(&self, summary: AuditSummary) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        *self.state.last_anchor_unix.lock().await = Some(now);
        let mut buf = self.state.recent_audit.lock().await;
        buf.push(summary);
        // Cap the ring at 256 entries; ample for `RecentAudit`
        // queries which top out at maybe ~16.
        if buf.len() > 256 {
            let drop = buf.len() - 256;
            buf.drain(0..drop);
        }
    }

    /// Accept one connection + dispatch one request. Tests use
    /// this to exercise the IPC path without running the full
    /// loop.
    pub async fn serve_once(&self) -> Result<(), DaemonError> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| DaemonError::Io("listener not bound; call bind() first".into()))?;
        let (stream, _addr) = listener
            .accept()
            .await
            .map_err(|e| DaemonError::Io(format!("accept: {e}")))?;
        handle_connection(stream, Arc::clone(&self.state)).await
    }

    /// Run the IPC accept loop until SIGINT / SIGTERM. Spawns
    /// one task per connection.
    pub async fn run_until_signal(&self) -> Result<(), DaemonError> {
        let listener = self
            .listener
            .as_ref()
            .ok_or_else(|| DaemonError::Io("listener not bound; call bind() first".into()))?;

        let shutdown = tokio::signal::ctrl_c();
        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    tracing::info!("shutdown signal received");
                    return Ok(());
                }
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            let state = Arc::clone(&self.state);
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, state).await {
                                    tracing::warn!("ipc connection: {e}");
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!("accept: {e}");
                        }
                    }
                }
            }
        }
    }
}

/// The daemon's own effective uid — IPC peers must match it (or
/// be root) to be served.
fn process_euid() -> u32 {
    // SAFETY: geteuid() has no failure modes and touches no
    // memory; it always returns the process's effective uid.
    unsafe { libc::geteuid() }
}

/// Peer-credential gate (NIST_AGENT-2026-05-31-003): only the
/// daemon's own uid or root may query the socket.
fn peer_authorized(peer_uid: u32) -> bool {
    peer_uid == 0 || peer_uid == process_euid()
}

/// One IPC connection — peer-cred gate, read line, dispatch,
/// write line. The daemon serves one request per connection in
/// v1.0; long-lived streaming subscriptions are reserved for
/// v1.1.
async fn handle_connection(stream: UnixStream, state: Arc<DaemonState>) -> Result<(), DaemonError> {
    // Authorization before any request byte is read. Fail closed:
    // a platform that won't disclose the peer credential gets
    // refused, not trusted.
    let authorized = match stream.peer_cred() {
        Ok(cred) => peer_authorized(cred.uid()),
        Err(_) => false,
    };
    let (reader, mut writer) = stream.into_split();

    let response = if !authorized {
        IpcResponse::Error {
            protocol_version: IPC_PROTOCOL_VERSION.into(),
            message: "unauthorized peer".into(),
        }
    } else {
        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        buf.read_line(&mut line)
            .await
            .map_err(|e| DaemonError::Io(format!("read line: {e}")))?;
        match IpcRequest::from_line(&line) {
            Ok(req) => dispatch(req, &state).await,
            Err(e) => IpcResponse::Error {
                protocol_version: IPC_PROTOCOL_VERSION.into(),
                message: format!("decode: {e}"),
            },
        }
    };
    let mut out = response.to_line();
    out.push('\n');
    writer
        .write_all(out.as_bytes())
        .await
        .map_err(|e| DaemonError::Io(format!("write line: {e}")))?;
    writer
        .shutdown()
        .await
        .map_err(|e| DaemonError::Io(format!("shutdown: {e}")))?;
    Ok(())
}

async fn dispatch(req: IpcRequest, state: &Arc<DaemonState>) -> IpcResponse {
    match req {
        IpcRequest::Status => IpcResponse::Status {
            protocol_version: IPC_PROTOCOL_VERSION.into(),
            crate_version: env!("CARGO_PKG_VERSION").into(),
            queue_depth: *state.queue_depth.lock().await,
            last_anchor_unix: *state.last_anchor_unix.lock().await,
        },
        IpcRequest::QueueDepth => IpcResponse::QueueDepth {
            protocol_version: IPC_PROTOCOL_VERSION.into(),
            depth: *state.queue_depth.lock().await,
        },
        IpcRequest::RecentAudit { count } => {
            let buf = state.recent_audit.lock().await;
            let n = count.min(buf.len());
            let entries: Vec<_> = buf[buf.len() - n..].to_vec();
            IpcResponse::RecentAudit {
                protocol_version: IPC_PROTOCOL_VERSION.into(),
                entries,
            }
        }
        IpcRequest::Shutdown => IpcResponse::Error {
            protocol_version: IPC_PROTOCOL_VERSION.into(),
            message: "shutdown via IPC reserved for v1.1; use SIGINT".into(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn fixture_daemon() -> (TempDir, Daemon) {
        let tmp = tempfile::tempdir().unwrap();
        let cfg = DaemonConfig::fixture(tmp.path());
        let daemon = Daemon::prepare(cfg).expect("prepare");
        (tmp, daemon)
    }

    #[tokio::test]
    async fn prepare_creates_audit_sink_dir() {
        let (tmp, _daemon) = fixture_daemon();
        assert!(tmp.path().join("audit").is_dir());
    }

    #[tokio::test]
    async fn bind_creates_socket_and_remove_stale() {
        let (tmp, mut daemon) = fixture_daemon();
        let path = daemon.bind().expect("bind");
        assert!(path.exists());
        // Drop + re-bind: should remove stale + succeed.
        drop(daemon);
        let cfg = DaemonConfig::fixture(tmp.path());
        let mut d2 = Daemon::prepare(cfg).unwrap();
        d2.bind().expect("re-bind after stale");
    }

    #[tokio::test]
    async fn status_request_round_trip_via_socket() {
        let (_tmp, mut daemon) = fixture_daemon();
        let path = daemon.bind().unwrap();
        let server = tokio::spawn(async move {
            daemon.serve_once().await.unwrap();
        });
        // Give the listener a tick to be ready.
        let mut stream = UnixStream::connect(&path).await.expect("connect");
        let req = IpcRequest::Status;
        let mut line = req.to_line();
        line.push('\n');
        stream.write_all(line.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();

        let (r, _w) = stream.into_split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        let resp = IpcResponse::from_line(&buf).unwrap();
        match resp {
            IpcResponse::Status {
                protocol_version,
                queue_depth,
                ..
            } => {
                assert_eq!(protocol_version, IPC_PROTOCOL_VERSION);
                assert_eq!(queue_depth, 0);
            }
            other => panic!("expected Status, got {other:?}"),
        }
        server.await.unwrap();
    }

    #[tokio::test]
    async fn queue_depth_reflects_set_value() {
        let (_tmp, mut daemon) = fixture_daemon();
        daemon.set_queue_depth(7).await;
        let path = daemon.bind().unwrap();
        let server = tokio::spawn(async move {
            daemon.serve_once().await.unwrap();
        });
        let mut stream = UnixStream::connect(&path).await.unwrap();
        let mut line = IpcRequest::QueueDepth.to_line();
        line.push('\n');
        stream.write_all(line.as_bytes()).await.unwrap();
        stream.shutdown().await.unwrap();
        let (r, _w) = stream.into_split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        let resp = IpcResponse::from_line(&buf).unwrap();
        match resp {
            IpcResponse::QueueDepth { depth, .. } => assert_eq!(depth, 7),
            other => panic!("expected QueueDepth, got {other:?}"),
        }
        server.await.unwrap();
    }

    #[tokio::test]
    async fn record_anchor_updates_state() {
        let (_tmp, daemon) = fixture_daemon();
        assert!(daemon.state().last_anchor_unix.lock().await.is_none());
        daemon
            .record_anchor(AuditSummary {
                at_iso: "2026-05-21T10:00:00Z".into(),
                kind: "merkle-root".into(),
                message: "nightly anchor written".into(),
            })
            .await;
        assert!(daemon.state().last_anchor_unix.lock().await.is_some());
        assert_eq!(daemon.state().recent_audit.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn recent_audit_ring_caps_at_256() {
        let (_tmp, daemon) = fixture_daemon();
        for i in 0..300 {
            daemon
                .record_anchor(AuditSummary {
                    at_iso: format!("2026-05-21T10:{i:02}:00Z"),
                    kind: "test".into(),
                    message: format!("entry {i}"),
                })
                .await;
        }
        assert_eq!(daemon.state().recent_audit.lock().await.len(), 256);
    }

    #[tokio::test]
    async fn bound_socket_mode_excludes_group_and_other() {
        // RED for NIST_AGENT-2026-05-31-003: the IPC socket must
        // not be world/group-connectable. Pre-fix, bind() left
        // the socket at the umask default (~0755) so any local
        // user could read audit summaries / queue depth.
        let (_tmp, mut daemon) = fixture_daemon();
        let path = daemon.bind().expect("bind");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(
                mode & 0o077,
                0,
                "IPC socket is group/world-accessible (mode {:o}); must be 0600",
                mode
            );
        }
    }

    #[test]
    fn peer_gate_refuses_foreign_uid() {
        // NIST_AGENT-2026-05-31-003: connections from any uid
        // other than the daemon's own (or root) are refused.
        let me = process_euid();
        assert!(peer_authorized(me), "daemon's own uid must be authorized");
        assert!(peer_authorized(0), "root must be authorized");
        assert!(
            !peer_authorized(me.wrapping_add(1)),
            "a foreign uid must be refused"
        );
    }

    #[tokio::test]
    async fn malformed_request_returns_error_response() {
        let (_tmp, mut daemon) = fixture_daemon();
        let path = daemon.bind().unwrap();
        let server = tokio::spawn(async move {
            daemon.serve_once().await.unwrap();
        });
        let mut stream = UnixStream::connect(&path).await.unwrap();
        stream.write_all(b"not a request\n").await.unwrap();
        stream.shutdown().await.unwrap();
        let (r, _w) = stream.into_split();
        let mut reader = BufReader::new(r);
        let mut buf = String::new();
        reader.read_line(&mut buf).await.unwrap();
        let resp = IpcResponse::from_line(&buf).unwrap();
        match resp {
            IpcResponse::Error { message, .. } => assert!(message.contains("decode")),
            other => panic!("expected Error, got {other:?}"),
        }
        server.await.unwrap();
    }
}
