//! `DaemonError` — typed errors for the daemon's config load,
//! IPC bring-up, anchor-cadence ticker, and shutdown.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    /// TOML decode failed for the operator-supplied config.
    #[error("config decode: {0}")]
    ConfigDecode(String),

    /// Config field had an invalid value (e.g. empty audit-sink
    /// path, malformed Unix-socket path, anchor cadence out of
    /// range).
    #[error("config invalid: {field}: {reason}")]
    ConfigInvalid { field: String, reason: String },

    /// File-system error preparing the audit sink directory or
    /// the IPC socket path.
    #[error("filesystem: {0}")]
    Filesystem(String),

    /// Tokio runtime / IPC listen / bind / accept error.
    #[error("io: {0}")]
    Io(String),

    /// IPC client sent a request the daemon didn't understand.
    /// The response carries the original line + the parser
    /// diagnostic for operator-side debugging.
    #[error("ipc decode: {0}")]
    IpcDecode(String),

    /// Anchor cadence ticker failed to write to the configured
    /// audit sink. The daemon surfaces this and keeps running
    /// (it does NOT crash on a single anchor-write failure;
    /// rule-6 requires availability over strict-best-effort).
    #[error("anchor write: {0}")]
    AnchorWrite(String),

    /// The configured PolicyBundle failed to read, verify against
    /// the SecurityOfficer trust root, or activate. The daemon
    /// refuses to start (fail-closed) rather than booting without
    /// the operator's signed policy
    /// (NIST_AGENT-2026-05-31-001).
    #[error("policy bundle rejected: {0}")]
    PolicyRejected(String),
}
