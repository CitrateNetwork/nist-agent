//! nist-agent-marketplace — operator UI for the capsule marketplace
//! browser and the interactive chat surface per RFC §8.3.
//!
//! Headless render models in the lib + an optional Slint UI behind
//! `feat-ui` (same pattern as `nist-agent-wizard` / S-10a and
//! `nist-agent-hitl` / S-10b; see ADR-008 and ADR-009).
//!
//! Two surfaces:
//!
//! - **Marketplace browser** — three tabs (Bundled / Site Mirror /
//!   On-Chain) listing capsules the operator can install. Listings
//!   are filtered by the active overlay set so a FERPA-only
//!   deployment never sees ITAR-only capsules. The Install button
//!   on a row hands the manifest off to the Capsule Inspector pane
//!   (S-10b) which is the load-bearing HIC gate.
//!
//! - **Chat** — interactive input that drives `Agent::step()`. The
//!   render model collects assistant tokens as they stream, pauses
//!   when the agent returns `AgentOutcome::Pending` (shows the
//!   action proposal inline), and resumes after the HIC queue
//!   returns a decision. Trajectory export hint surfaces only
//!   when the operator's overlay allows it (RFC §12 Q3).
//!
//! Per ADR-009, this crate stays in nist-agent permanently — same
//! product-vs-engine split that ADR-008 documents for
//! `nist-agent-wizard`.

pub mod browser;
pub mod chat;
pub mod error;

#[cfg(feature = "feat-ui")]
pub mod ui;

pub use browser::{
    CapsuleListing, MarketplaceAction, MarketplaceFilter, MarketplaceSource, MarketplaceView,
};
pub use chat::{ChatAction, ChatRole, ChatSessionView, ChatStreamState, ChatTurn};
pub use error::MarketplaceUiError;
