//! `MarketplaceUiError` — typed errors for the marketplace + chat
//! render-model machinery.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MarketplaceUiError {
    /// Operator attempted to enable the On-Chain tab but the active
    /// overlay's egress posture forbids it. The UI displays this as
    /// a disabled tab with the reason inline; the harness should
    /// not route around it.
    #[error("on-chain tab disabled: {reason}")]
    OnChainTabDisabled { reason: String },

    /// Operator tried to install a capsule whose certified-overlay
    /// set is incompatible with the active deployment overlay set.
    /// The marketplace pre-filters listings so this only fires when
    /// a downstream caller constructs an Install action by id.
    #[error("capsule {capsule} not certified for any active overlay")]
    CapsuleOverlayMismatch { capsule: String },

    /// Chat surface received a resume signal but the session is not
    /// in a paused state. Indicates a programmer error in the
    /// harness wiring.
    #[error("chat resume called but session is not paused")]
    ResumeWithoutPause,

    /// Trajectory export was requested but the active overlay
    /// forbids it (e.g. FedRAMP High treats trajectories as
    /// covered data). RFC §12 Q3.
    #[error("trajectory export forbidden by overlay")]
    TrajectoryExportForbidden,
}
