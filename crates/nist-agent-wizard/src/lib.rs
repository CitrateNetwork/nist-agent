//! nist-agent-wizard — the first-run concierge per RFC §8.1.
//!
//! This crate is shaped as a **headless state machine** with an
//! optional Slint UI binding behind the `feat-ui` feature. The
//! split keeps the lib testable in CI without pulling in
//! libfontconfig + libxkbcommon for every workspace build.
//!
//! Lifecycle:
//!
//!   Idle  →  OrgIdentity  →  Roles  →  HardwareKeys  →  Overlays
//!         →  Review  →  Signed
//!
//! Each forward transition takes typed input and writes a
//! per-step audit record (RFC §6.1). The final `Signed` step
//! emits a `RawSignedBundle` consumable by every other crate
//! (S-7 doctor, S-8/S-9 overlay factories, the daemon).
//!
//! Per ADR-008, the wizard lives canonically in nist-agent;
//! unlike the loop / model / policy / doctor crates, this one
//! does NOT migrate upstream — it's product-shaped, not
//! engine-shaped. The upstream PR pattern (ADRs 004/005/006/007)
//! does not apply.

pub mod audit;
pub mod error;
pub mod state;
pub mod wizard;

#[cfg(feature = "feat-ui")]
pub mod ui;

pub use audit::{StepAudit, StepAuditKind};
pub use error::WizardError;
pub use state::{HardwareKey, RoleAssignment, WizardState};
pub use wizard::{Step, Wizard};
