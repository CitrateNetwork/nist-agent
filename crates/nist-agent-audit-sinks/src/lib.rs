//! nist-agent-audit-sinks — `AuditSink` impls for the storage
//! backends RFC §6.2 names that upstream's `FilesystemSink`
//! (CIT-AGENT-5a) doesn't cover.
//!
//! S-9 ships:
//!
//! - [`WormFilesystemSink`] — one record per file, created with
//!   `O_CREAT | O_EXCL` so a rewrite of an existing record errors
//!   at the syscall layer. Required for FedRAMP High deployments
//!   per RFC §6.2 / overlay-fedramp-high.feature; the CJIS overlay
//!   (v1.1) reuses it.
//!
//! S-9b ships (planned, not in this sprint):
//!
//! - `NfsSink` — uses `O_DIRECT` + an NFS-specific advisory lock
//!   to serialize writes across multiple harness instances sharing
//!   one log directory.
//! - `S3Sink` — single-PUT per record against an S3-compatible
//!   bucket. Multi-region replication via the storage's own
//!   replication; nist-agent does NOT cross-region itself.
//!
//! Both deferred backends bind to the same `AuditSink` trait. The
//! local landing follows the ALIGNMENT.md exception-clause
//! precedent (ADR-002 / 004 / 005 / 006 / 007 — see ADR-007 in
//! this sprint).

pub mod worm;

pub use worm::WormFilesystemSink;

// Re-export the upstream trait so consumers don't need to know
// about citrate-agent-core to use our sinks.
pub use citrate_agent_core::audit::AuditSink;
