//! nist-agent-release — release manifest + signature verifier +
//! installer + model-integrity gate per RFC §§3.3, 8.2, 11.1.
//!
//! Per ADR-011, this crate ships the **Rust verification logic**
//! the operator runs on the target host. The producer side (HSM-
//! held signing key, CI workflow, GGUF mirror) is deferred to
//! S-12b / external infra.
//!
//! Five surfaces:
//!
//! - `manifest.rs` — `ReleaseManifest` (TOML), `ArtifactEntry`
//!   with sha256, signature envelope.
//! - `signature.rs` — Ed25519 detached-signature wrap (signer
//!   helper for fixtures; verifier for installs).
//! - `model.rs` — SI-7 GGUF hash check.
//! - `install.rs` — installer flow that verifies signature
//!   before extracting; exact refusal wording matches the
//!   `dist-signed-releases.feature` scenario.
//! - `egress.rs` — `EgressPosture::default() == Disabled` per
//!   RFC §3.3 G1; opt-in via signed SecurityOfficer directive.
//! - `daemon_hash.rs` — doctor-side comparison of the running
//!   daemon's sha256 against the release manifest.

pub mod daemon_hash;
pub mod egress;
pub mod error;
pub mod install;
pub mod manifest;
pub mod model;
pub mod signature;

pub use daemon_hash::{DaemonHashCheck, DaemonHashVerdict};
pub use egress::{EgressDirective, EgressPosture};
pub use error::ReleaseError;
pub use install::{InstallVerdict, Installer};
pub use manifest::{ArtifactEntry, ArtifactKind, ReleaseManifest, SignatureEnvelope};
pub use model::ModelIntegrity;
pub use signature::{ReleaseSigner, ReleaseVerifier};
