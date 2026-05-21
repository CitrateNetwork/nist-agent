//! `WizardError` — typed errors for the wizard state machine.

use nist_agent_policy::types::Role;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WizardError {
    /// Called a step-specific transition out of order. Each
    /// transition asserts `current_step == expected_step` and
    /// errors here on mismatch — protects against a UI that
    /// double-fires a transition or skips a step.
    #[error("wrong step: at {actual:?}, expected {expected:?}")]
    WrongStep {
        actual: super::wizard::Step,
        expected: super::wizard::Step,
    },

    /// Role-assignment step submitted with a role missing or
    /// empty. Mirror of `PolicyError::RoleUnassigned` for the
    /// pre-signing check.
    #[error("role {0:?} has no identity assigned")]
    RoleUnassigned(Role),

    /// Org identity name was empty / whitespace-only.
    #[error("org identity name cannot be empty")]
    EmptyOrgName,

    /// Hardware key enrollment supplied an identifier shorter than
    /// the per-surface minimum (FIDO2 attestation IDs are ≥32 bytes;
    /// PIV cert SHA-256 is 64 hex chars).
    #[error("hardware-key id too short for {role:?}: got {got} bytes")]
    HardwareKeyTooShort { role: Role, got: usize },

    /// Sign step received an Ed25519 key that fails the
    /// SecurityOfficer identity check (operator passed in a key
    /// that doesn't match the previously enrolled SO).
    #[error("signing key does not match the enrolled SecurityOfficer identity")]
    SigningKeyMismatch,

    /// Bundle emission failed because the configured output path
    /// already has a v1 bundle. The wizard refuses to clobber.
    #[error("output path {0} already has a bundle; refusing to overwrite")]
    OutputPathExists(String),

    /// IO failure during bundle write.
    #[error("io: {0}")]
    Io(String),

    /// Underlying PolicyBundle activation failed (validity window,
    /// version overflow, etc.).
    #[error("policy: {0}")]
    Policy(#[from] nist_agent_policy::PolicyError),
}
