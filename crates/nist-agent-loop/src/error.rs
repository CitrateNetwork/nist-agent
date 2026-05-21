//! `AgentLoopError` — typed errors for the agent loop.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AgentLoopError {
    /// The underlying model failed during inference.
    #[error("model: {0}")]
    Model(#[from] nist_agent_model::ModelError),

    /// The agent proposed an action but the operator surface
    /// (Slint, CLI, daemon RPC) did not resume within the operator-
    /// configured deadline. Causes the loop to abort the proposal,
    /// not the agent (per RFC §5.4 — the interrupt is recoverable).
    #[error("HITL timeout after {0:?}")]
    HitlTimeout(std::time::Duration),

    /// Checkpoint persistence failed. The loop refuses to propose
    /// the action because a lost checkpoint would break Rule 10
    /// (audit chain integrity).
    #[error("checkpoint store: {0}")]
    Checkpoint(String),

    /// The approval payload disagrees with the checkpoint it
    /// references — e.g. the checkpoint id doesn't match, or the
    /// signatures don't cover the proposal hash. Surfaced as a
    /// hard error; the harness MUST NOT proceed.
    #[error("approval mismatch: {0}")]
    ApprovalMismatch(String),
}
