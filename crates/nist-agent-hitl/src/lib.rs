//! nist-agent-hitl, operator UI for the HIC approval queue and
//! the Capsule Inspector pane per RFC §8.3.
//!
//! Headless render models in the lib + an optional Slint UI
//! behind `feat-ui` (same pattern as `nist-agent-wizard` S-10a /
//! ADR-008).
//!
//! Two surfaces:
//!
//! - **Approval queue** — list of pending `ToolCall` proposals
//!   from `citrate_agent_core::hitl::ApprovalQueue::pending()`,
//!   each rendered as an `ApprovalRow` the UI iterates over.
//!   Sign / Reject actions route back through the queue's
//!   `approve()` / `reject()` methods at the call site (this
//!   crate exposes the typed action; integration with the
//!   queue's tokio handles is a thin wrapper).
//!
//! - **Capsule Inspector** — every field per
//!   `features/surfaces/surface-slint-capsule-inspector.feature`,
//!   plus the scroll-tracking gate that disables Install until
//!   all fields have been viewed (AC-6 audit artifact).
//!
//! Per ADR-009, the crate stays in nist-agent permanently — same
//! product-vs-engine split that ADR-008 documented for
//! `nist-agent-wizard`.

pub mod approval;
pub mod error;
pub mod inspector;

#[cfg(feature = "feat-ui")]
pub mod ui;

pub use approval::{ApprovalAction, ApprovalQueueView, ApprovalRow, ApprovalRowSeverity};
pub use error::HitlUiError;
pub use inspector::{
    CapabilityRow, CapsuleInspectorView, InspectorField, OverlayCertification, SigningTierBadge,
};
