//! `HitlUiError` — typed errors for the render-model machinery.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HitlUiError {
    /// An approval row's signature collection was attempted but
    /// the row's `current_signatures` already satisfies the
    /// required-roles multiset. The harness should remove the
    /// row from the queue, not add another signature.
    #[error("row {call_id} already satisfies quorum")]
    AlreadyApproved { call_id: String },

    /// Install attempt before the operator has viewed every
    /// Capsule Inspector field. RFC §8.3 explicit gate.
    #[error("install gate: {unseen} of {total} fields not yet viewed")]
    InstallGateUnseenFields { unseen: usize, total: usize },

    /// JSON args failed to pretty-format for display.
    #[error("args pretty-print: {0}")]
    ArgsPretty(String),
}
