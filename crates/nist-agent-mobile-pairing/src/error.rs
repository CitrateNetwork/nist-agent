//! `MobilePairingError` — typed errors for the pairing,
//! attestation, eligibility, and wire layers.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MobilePairingError {
    /// State-machine transition was attempted from the wrong
    /// state. The state machine refuses the transition and surfaces
    /// what was expected vs. observed; the harness should treat
    /// this as a programmer / replay error and not retry.
    #[error("pairing transition: expected {expected}, observed {observed}")]
    InvalidPairingTransition {
        expected: &'static str,
        observed: &'static str,
    },

    /// The device's attestation chain id is not on the operator's
    /// allowlist. Maps directly to the feature scenario "Samsung
    /// Knox attestation refused".
    #[error("device attestation not on allowlist: {chain_id}")]
    AttestationNotAllowlisted { chain_id: String },

    /// The attestation envelope was not cryptographically verified
    /// (NA2-B-005). The allowlist check confirms the *claimed* chain
    /// id is permitted, but a device is only admitted once its
    /// attestation chain actually verifies. A self-asserted chain id
    /// with a degenerate or unverifiable envelope is refused.
    #[error("device attestation failed verification: {reason}")]
    AttestationUnverified { reason: String },

    /// A sign attempt arrived from a paired device but the active
    /// overlay set forbids mobile signing. RFC §5.6 — FedRAMP
    /// High deployments must refuse.
    #[error("mobile signing forbidden by active overlay set")]
    MobileSigningForbidden,

    /// A signed decision arrived but the signing timestamp is
    /// older than the active TTL — the daemon must refuse.
    #[error("signature expired: signed_at + ttl ({signed_at_iso} + {ttl_secs}s) is in the past")]
    SignatureExpired {
        signed_at_iso: String,
        ttl_secs: u64,
    },

    /// Wire-form decode failure. Surfaced so the daemon can return
    /// an unambiguous error to a misbehaving device rather than a
    /// generic 400.
    #[error("wire decode: {0}")]
    WireDecode(String),

    /// The pairing token presented by the device did not match
    /// the daemon's outstanding token. Could be replay or expired.
    #[error("pairing token mismatch or expired")]
    PairingTokenMismatch,

    /// The pairing's short-lived token window has elapsed
    /// (NA2-B-006). The pairing must be re-initiated.
    #[error("pairing token expired at {expires_at_unix} (now {now_unix})")]
    PairingExpired {
        expires_at_unix: i64,
        now_unix: i64,
    },

    /// Too many failed token presentations; the pairing is
    /// terminated to stop unlimited brute-force retries (NA2-B-006).
    #[error("pairing terminated after {attempts} failed token attempts")]
    PairingTooManyAttempts { attempts: u32 },
}
