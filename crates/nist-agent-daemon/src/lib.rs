//! nist-agent-daemon — long-running process state + event loop
//! + IPC surface.
//!
//! Composes upstream's `citrate_agent_core::hitl::ApprovalQueue`
//! with this workspace's `nist-agent-audit-sinks` and a future
//! `dyn ChainClient` (S-4) into a single `Daemon` struct that:
//!
//! 1. Loads a TOML config (`DaemonConfig`).
//! 2. Holds the live `ApprovalQueue` + `AuditChain` handles.
//! 3. Runs an anchor-cadence ticker per the configured
//!    strategy (nightly / per-install / hybrid).
//! 4. Listens on a Unix-domain socket for status queries.
//!
//! The CLI's `daemon` subcommand instantiates this crate; the
//! `status` subcommand connects to the IPC socket from the
//! operator side.
//!
//! IPC wire form: line-delimited JSON. One request per line, one
//! response per line. Schema versioned via `protocol_version`
//! in the response so a future gRPC migration (v1.1+) can
//! coexist.

pub mod config;
pub mod daemon;
pub mod error;
pub mod ipc;

pub use config::{AnchorStrategy, DaemonConfig};
pub use daemon::Daemon;
pub use error::DaemonError;
pub use ipc::{IpcRequest, IpcResponse, IPC_PROTOCOL_VERSION};
