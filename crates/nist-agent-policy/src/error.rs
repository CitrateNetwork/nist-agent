//! `PolicyError` — typed errors for bundle parsing, signature
//! verification, lattice operations, and overlay activation.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PolicyError {
    /// The bundle's CBOR encoding was malformed.
    #[error("cbor decode: {0}")]
    CborDecode(String),

    /// The bundle's CBOR encoding wasn't canonical per RFC 8949
    /// §4.2.1. We re-encode the parsed bundle and compare bytes;
    /// any divergence is rejected.
    #[error("non-canonical CBOR: re-encoding produces different bytes")]
    NonCanonicalCbor,

    /// Ed25519 verification of the SecurityOfficer signature failed.
    #[error("signature: SecurityOfficer signature does not verify")]
    SignatureInvalid,

    /// The SecurityOfficer signature's public key isn't on the
    /// operator's configured trust roots.
    #[error("signer: SecurityOfficer key not in trust roots")]
    UntrustedSigner,

    /// Bundle validity window has expired (current time > expires_at).
    /// Operators MUST rotate before expiration; doctor flags
    /// imminent expiration as WARN.
    #[error("validity: bundle expired at {0}")]
    Expired(String),

    /// Bundle's not-before is in the future (clock skew or premature
    /// activation).
    #[error("validity: bundle not-before is {0}")]
    NotYetValid(String),

    /// Risk-tier mapping attempts to de-escalate a capsule's
    /// declared tier (Rule 1, RFC §5.2: tier never de-escalates).
    #[error("risk-tier: cannot de-escalate {capsule} from {declared:?} to {proposed:?}")]
    TierDeEscalation {
        capsule: String,
        declared: super::types::RiskTier,
        proposed: super::types::RiskTier,
    },

    /// Overlay removal attempted without a documented decommissioning
    /// workflow (RFC §2.3 one-way ratchet).
    #[error("overlay: removal of {0:?} requires decommissioning workflow")]
    OverlayRemovalForbidden(super::Overlay),

    /// Bell-LaPadula no-read-up violation: operator clearance does
    /// not dominate the capsule's declared reads.
    #[error("lattice: clearance {clearance:?} does not dominate read {read:?}")]
    LatticeNoReadUp {
        clearance: super::lattice::DataClass,
        read: super::lattice::DataClass,
    },

    /// The five base roles MUST all be assigned. Activation fails
    /// if any role has zero identities (RFC §5.3).
    #[error("roles: required base role {0:?} has no identities assigned")]
    RoleUnassigned(super::types::Role),
}
