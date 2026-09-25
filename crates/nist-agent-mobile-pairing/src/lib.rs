//! nist-agent-mobile-pairing — protocol, eligibility, and render
//! models for the out-of-band mobile signing surface per RFC §5.6.
//!
//! This crate owns four pieces:
//!
//! - **Pairing** — the state machine an operator walks through to
//!   pair a phone with the daemon. HIC-gated: a new pairing is
//!   itself a proposal that requires a SecurityOfficer signature
//!   before transitioning to Active. The persisted record is an
//!   `AuditRecord`.
//! - **Attestation** — `DeviceAttestation` envelopes the operator
//!   policy will allowlist by `chain_id`. The feature scenario
//!   "Samsung Knox refused when not on allowlist" maps to a
//!   `validate_chain()` call here.
//! - **Eligibility** — per-overlay enable/disable + TTL math. The
//!   eligibility rule is the canonical home of "FedRAMP High
//!   forbids mobile signing"; both the desktop UI and the future
//!   native apps read from this module.
//! - **Wire** — the serde wire form the daemon ships to a paired
//!   device (`ApprovalSnapshot`) and the form the device returns
//!   after the operator signs (`SignedDecision`).
//!
//! The actual mTLS transport, the native iOS / Android apps, and
//! the device-attestation cryptography are deferred — see
//! ADR-010 for the placement rationale.

pub mod attestation;
pub mod eligibility;
pub mod error;
pub mod pairing;
pub mod wire;

pub use attestation::{AttestationAllowlist, AttestationChainId, DeviceAttestation, DeviceVendor};
pub use eligibility::{MobileEligibility, SignatureTtl};
pub use error::MobilePairingError;
pub use pairing::{PairingId, PairingRecord, PairingState, PairingToken};
pub use wire::{ApprovalSnapshot, ApprovalSnapshotRow, SignedDecision, SignedDecisionKind};
