//! nist-agent-policy — signed PolicyBundle, data-class lattice,
//! and overlay activation ratchet.
//!
//! Implements four RFC-CIT-AGENT-0001 surfaces in one crate
//! because they share data structures:
//!
//! - **§2.3 Overlay Profiles** — signed policy bundles that
//!   augment the baseline. Activation is a one-way ratchet within
//!   a deployment lifetime.
//! - **§3.1 PolicyBundle** — the signed, versioned configuration
//!   object containing risk-tier mappings, role lattice, overlay
//!   activations, anchor strategy, and egress posture.
//! - **§5.2 Risk tiers + §5.3 Five-role lattice** — Low/Medium/
//!   High/Critical mapped to required-role multisets over the
//!   Operator/Reviewer/ComplianceOfficer/SecurityOfficer/Auditor
//!   lattice.
//! - **§7.2 Bell-LaPadula data-class lattice** — PUBLIC < CUI <
//!   PHI < FERPA < ITAR, with dominance checks for capsule install
//!   and per-call data-class enforcement.
//!
//! Per ADR-005, this crate is the local landing for the empty
//! `citrate_agent_core::policy` slot. The upstream PR migrates
//! the entire surface in one move; same exception-clause pattern
//! as S-5 / ADR-004.
//!
//! Canonical CBOR (RFC 8949 §4.2.1 deterministic encoding rules)
//! is used for everything that gets signed or hashed. ciborium
//! produces deterministic output by default.

pub mod bundle;
pub mod error;
pub mod lattice;
pub mod overlay_state;
pub mod types;

pub use bundle::{PolicyBundle, RawSignedBundle};
pub use error::PolicyError;
pub use lattice::DataClass;
pub use overlay_state::ActiveOverlays;
pub use types::{AnchorStrategy, EgressPosture, RiskTier, Role};

// Re-export Overlay from prelude so PolicyBundle consumers don't
// have to know about the prelude crate to use the bundle. Rule 9
// — one source of truth (Overlay lives in prelude), this is just
// the consumer-facing surface.
pub use nist_agent_prelude::Overlay;
