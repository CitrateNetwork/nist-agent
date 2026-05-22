//! Unix-socket IPC surface.
//!
//! Wire form: one JSON object per line. Request/response shapes
//! are versioned via the `protocol_version` field on the
//! response so a future gRPC v2 migration can coexist (operator
//! reads the version field; client falls back to JSON if it's
//! talking to a v1 daemon).
//!
//! Today's request/response surface is intentionally small:
//!
//! - `Status` → human-readable banner + protocol version.
//! - `QueueDepth` → number of pending HITL approvals.
//! - `RecentAudit` → last N audit records' summaries.
//! - `Shutdown` → graceful shutdown (reserved; not yet wired to
//!   the harness's signal handler).
//!
//! The CLI's `status` subcommand calls `Status`; future
//! subcommands (queue, audit-tail) attach to the same socket.

use serde::{Deserialize, Serialize};

/// Bump on every wire-format change. Major bumps break
/// compatibility; minor bumps are additive (new variants /
/// fields). Today: `1.0`.
pub const IPC_PROTOCOL_VERSION: &str = "1.0";

/// Request the CLI sends to the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum IpcRequest {
    /// Identity readout + protocol version.
    Status,
    /// Current HITL approval queue depth.
    QueueDepth,
    /// Last `count` audit records' summaries.
    RecentAudit {
        #[serde(default = "default_count")]
        count: usize,
    },
    /// Reserved.
    Shutdown,
}

fn default_count() -> usize {
    10
}

/// Daemon reply. Every variant carries the protocol version so a
/// CLI talking to an older daemon can detect mismatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum IpcResponse {
    Status {
        protocol_version: String,
        crate_version: String,
        queue_depth: usize,
        last_anchor_unix: Option<i64>,
    },
    QueueDepth {
        protocol_version: String,
        depth: usize,
    },
    RecentAudit {
        protocol_version: String,
        entries: Vec<AuditSummary>,
    },
    Error {
        protocol_version: String,
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditSummary {
    pub at_iso: String,
    pub kind: String,
    pub message: String,
}

impl IpcRequest {
    /// Encode for the wire (one line, no trailing newline).
    pub fn to_line(&self) -> String {
        // unwrap: serializing our own enum can't fail
        serde_json::to_string(self).expect("ipc request serialize")
    }

    /// Decode from a single wire line.
    pub fn from_line(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s.trim())
    }
}

impl IpcResponse {
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).expect("ipc response serialize")
    }
    pub fn from_line(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s.trim())
    }
    pub fn protocol_version(&self) -> &str {
        match self {
            Self::Status {
                protocol_version, ..
            }
            | Self::QueueDepth {
                protocol_version, ..
            }
            | Self::RecentAudit {
                protocol_version, ..
            }
            | Self::Error {
                protocol_version, ..
            } => protocol_version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_request_round_trips() {
        let req = IpcRequest::Status;
        let line = req.to_line();
        assert!(line.contains("\"kind\":\"status\""));
        let round = IpcRequest::from_line(&line).unwrap();
        assert_eq!(round, req);
    }

    #[test]
    fn recent_audit_default_count_is_ten() {
        let line = r#"{"kind":"recent-audit"}"#;
        let req = IpcRequest::from_line(line).unwrap();
        match req {
            IpcRequest::RecentAudit { count } => assert_eq!(count, 10),
            other => panic!("expected RecentAudit, got {other:?}"),
        }
    }

    #[test]
    fn status_response_carries_protocol_version() {
        let resp = IpcResponse::Status {
            protocol_version: IPC_PROTOCOL_VERSION.into(),
            crate_version: "0.1.0".into(),
            queue_depth: 0,
            last_anchor_unix: None,
        };
        assert_eq!(resp.protocol_version(), IPC_PROTOCOL_VERSION);
        let line = resp.to_line();
        let round = IpcResponse::from_line(&line).unwrap();
        assert_eq!(round, resp);
    }

    #[test]
    fn error_response_shape_is_stable() {
        // The CLI's status subcommand pattern-matches Error; pin the wire form.
        let r = IpcResponse::Error {
            protocol_version: IPC_PROTOCOL_VERSION.into(),
            message: "queue not initialized".into(),
        };
        let line = r.to_line();
        assert!(line.contains("\"kind\":\"error\""));
        assert!(line.contains("queue not initialized"));
    }

    #[test]
    fn unknown_kind_returns_serde_error() {
        let bad = r#"{"kind":"not-a-request"}"#;
        assert!(IpcRequest::from_line(bad).is_err());
    }
}
