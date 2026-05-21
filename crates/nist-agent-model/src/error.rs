//! `ModelError` — typed errors for the model resolver.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ModelError {
    /// No model source in the configured resolution order was
    /// reachable. The harness MUST NOT start without a model.
    #[error("no model available: tried {tried}")]
    NoModelAvailable { tried: String },

    /// HTTP transport error (timeout, refused, malformed response).
    #[error("transport: {0}")]
    Transport(String),

    /// The Ollama / llama.cpp server returned an unexpected
    /// response shape (missing field, wrong type).
    #[error("protocol: {0}")]
    Protocol(String),

    /// NIST SI-7 — model file hash does not match the manifest's
    /// declared sha256. The harness refuses to load the file.
    #[error("SI-7: model hash mismatch — expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    /// File I/O during hash verification.
    #[error("io: {0}")]
    Io(String),

    /// Egress is forbidden by policy and the requested resolution
    /// step required network access. Surfaced before any DNS lookup
    /// or connect attempt.
    #[error("egress forbidden by policy for {0}")]
    EgressForbidden(&'static str),
}
