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

/// Fail-closed fallback wire line for a response that failed to
/// serialize (NIST_AGENT-2026-05-31-008: the connection path must
/// never panic). `serde_json::Value::to_string` cannot fail, so
/// this line always decodes as a well-formed `Error` response.
pub(crate) fn serialize_error_line(detail: &str) -> String {
    serde_json::json!({
        "kind": "error",
        "protocol_version": IPC_PROTOCOL_VERSION,
        "message": format!("response serialize failed: {detail}"),
    })
    .to_string()
}

impl IpcRequest {
    /// Encode for the wire (one line, no trailing newline).
    /// Serializing this enum can't fail in practice; if it ever
    /// does, emit a line the daemon will refuse rather than
    /// panicking on the connection path (HYG-UNWRAP,
    /// NIST_AGENT-2026-05-31-008).
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| {
            serde_json::json!({
                "kind": "unserializable-request",
                "error": e.to_string(),
            })
            .to_string()
        })
    }

    /// Decode from a single wire line.
    pub fn from_line(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s.trim())
    }
}

impl IpcResponse {
    /// Encode for the wire. Never panics: a serialization failure
    /// degrades to [`serialize_error_line`], which decodes as an
    /// `Error` response.
    pub fn to_line(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|e| serialize_error_line(&e.to_string()))
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

    #[test]
    fn wire_encoding_has_no_panic_path() {
        // RED for NIST_AGENT-2026-05-31-008: `to_line` used a
        // panic-on-error unwrap on the per-connection path. The
        // wire encoders must degrade to a decodable error line,
        // never panic. Source pin: no expect-on-serialize left in
        // this module. (Needle is assembled so this test's own
        // source can't satisfy it.)
        let src = include_str!("ipc.rs");
        let needle = [".exp", "ect("].concat();
        assert!(
            !src.contains(&needle),
            "ipc.rs must not panic-on-error in wire encoding (HYG-UNWRAP)"
        );
    }

    #[test]
    fn response_serialize_fallback_decodes_as_error() {
        // The fail-closed fallback line emitted when response
        // serialization fails must itself decode as a valid
        // Error response so clients aren't handed garbage.
        let line = serialize_error_line("fixture failure");
        match IpcResponse::from_line(&line).unwrap() {
            IpcResponse::Error { message, .. } => assert!(message.contains("fixture failure")),
            other => panic!("expected Error, got {other:?}"),
        }
    }
}
