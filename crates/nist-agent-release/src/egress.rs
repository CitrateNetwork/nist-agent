//! Egress posture defaults + the signed-directive activation gate.
//!
//! `dist-airgap-install.feature` pins the rule: "Egress remains
//! disabled after install"; "any capsule with network capability
//! is greyed-out in the marketplace pane". RFC §3.3 G1 makes the
//! global posture explicit: off by default, opt-in by signed
//! SecurityOfficer policy directive.
//!
//! Activation flow (NIST_AGENT-2026-05-31-004 fix — the
//! signature check lives IN THIS CRATE; no injectable verifier):
//!
//! 1. Operator's SecurityOfficer signs an [`EgressDirective`]
//!    (Ed25519 over [`EgressDirective::signing_payload`]).
//! 2. [`EgressPosture::apply_directive`] verifies the signature
//!    against the SecurityOfficer trust root, refuses expired
//!    directives, and refuses any nonce not strictly greater
//!    than the last consumed nonce (replay binding).
//! 3. On success the harness records the activation as an
//!    AuditRecord, persists the consumed nonce, and flips egress
//!    to `Enabled`. Disabling never needs a signature
//!    ([`EgressPosture::disable`]) — fail-closed is always free.
//!
//! The PolicyBundle's 3-state posture
//! (`nist_agent_policy::types::EgressPosture`) is the signed
//! source of truth; this crate's 2-state runtime gate derives
//! from it via the exhaustive `From` impl below
//! (NIST_AGENT-2026-05-31-009). `BrokerOnly` maps to `Disabled`
//! here: the 2-state gate governs DIRECT sockets only, and
//! broker-only traffic must never become unrestricted egress.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

use crate::error::ReleaseError;

/// Whether the harness will permit outbound network calls.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EgressPosture {
    /// Off by default per RFC §3.3 G1.
    #[default]
    Disabled,
    /// Activated by a signed SecurityOfficer directive.
    Enabled,
}

/// Total mapping from the PolicyBundle's signed 3-state posture
/// onto the 2-state runtime gate (NIST_AGENT-2026-05-31-009).
/// Exhaustive match — adding a policy variant forces a decision
/// here. Pinned by test: `BrokerOnly` is NOT direct egress.
impl From<nist_agent_policy::types::EgressPosture> for EgressPosture {
    fn from(p: nist_agent_policy::types::EgressPosture) -> Self {
        use nist_agent_policy::types::EgressPosture as Policy;
        match p {
            Policy::Disabled => Self::Disabled,
            // Broker-only forbids direct sockets; the direct-
            // egress gate therefore stays Disabled.
            Policy::BrokerOnly => Self::Disabled,
            Policy::Allowed => Self::Enabled,
        }
    }
}

impl EgressPosture {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// Fail-closed transition. Disabling egress never requires a
    /// signature.
    pub fn disable(self) -> Self {
        Self::Disabled
    }
}

/// A signed SecurityOfficer directive that flips egress to
/// `Enabled`. Self-verifying: the Ed25519 check, expiry, and
/// replay binding all live in [`EgressPosture::apply_directive`]
/// — there is no caller-injectable verifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressDirective {
    /// Operator-friendly reason field; persisted in the
    /// AuditRecord this activation generates.
    pub reason: String,
    /// ISO-8601 timestamp the SecurityOfficer asserts as the
    /// activation moment. Surfaced for audit.
    pub asserted_at_iso: String,
    /// Strictly-increasing sequence number. The harness persists
    /// the last consumed nonce; a directive whose nonce is not
    /// greater is refused (replay binding).
    pub nonce: u64,
    /// Unix-epoch seconds after which this directive is dead. A
    /// captured directive cannot be banked indefinitely.
    pub expires_at_unix: i64,
    /// Hex-encoded Ed25519 signature over
    /// [`Self::signing_payload`].
    pub signature_hex: String,
}

impl EgressDirective {
    /// Canonical signing payload: u64-BE length-prefixed `reason`
    /// and `asserted_at_iso`, then `nonce` and `expires_at_unix`
    /// as fixed-width big-endian words. Length-prefixing keeps
    /// field concatenation unambiguous.
    pub fn signing_payload(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.reason.len() as u64).to_be_bytes());
        out.extend_from_slice(self.reason.as_bytes());
        out.extend_from_slice(&(self.asserted_at_iso.len() as u64).to_be_bytes());
        out.extend_from_slice(self.asserted_at_iso.as_bytes());
        out.extend_from_slice(&self.nonce.to_be_bytes());
        out.extend_from_slice(&self.expires_at_unix.to_be_bytes());
        out
    }

    fn verify(&self, so_pubkey: &VerifyingKey) -> Result<(), ReleaseError> {
        let sig_bytes =
            hex::decode(&self.signature_hex).map_err(|_| ReleaseError::EgressDirectiveInvalid)?;
        let sig =
            Signature::from_slice(&sig_bytes).map_err(|_| ReleaseError::EgressDirectiveInvalid)?;
        so_pubkey
            .verify(&self.signing_payload(), &sig)
            .map_err(|_| ReleaseError::EgressDirectiveInvalid)
    }
}

impl EgressPosture {
    /// Apply a directive. Verifies the SecurityOfficer signature
    /// in-crate, refuses expired directives, and refuses any
    /// nonce not strictly greater than `last_consumed_nonce`.
    /// On success returns `(Enabled, consumed_nonce)`; the caller
    /// MUST persist the consumed nonce and pass it back on the
    /// next application. On any failure the posture is unchanged
    /// (the error is returned and `self` is dropped unmodified —
    /// `Copy`).
    pub fn apply_directive(
        self,
        directive: &EgressDirective,
        so_pubkey: &VerifyingKey,
        last_consumed_nonce: Option<u64>,
        now_unix: i64,
    ) -> Result<(Self, u64), ReleaseError> {
        directive.verify(so_pubkey)?;
        if now_unix > directive.expires_at_unix {
            return Err(ReleaseError::EgressDirectiveInvalid);
        }
        if let Some(last) = last_consumed_nonce {
            if directive.nonce <= last {
                return Err(ReleaseError::EgressDirectiveReplayed {
                    nonce: directive.nonce,
                    last_consumed: last,
                });
            }
        }
        Ok((Self::Enabled, directive.nonce))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    const NOW: i64 = 1_780_000_000;

    fn so_key() -> SigningKey {
        SigningKey::from_bytes(&[9u8; 32])
    }

    fn dir(nonce: u64) -> EgressDirective {
        let sk = so_key();
        let mut d = EgressDirective {
            reason: "operator approved on 2026-06-01".into(),
            asserted_at_iso: "2026-06-01T10:00:00Z".into(),
            nonce,
            expires_at_unix: NOW + 3600,
            signature_hex: String::new(),
        };
        d.signature_hex = hex::encode(sk.sign(&d.signing_payload()).to_bytes());
        d
    }

    #[test]
    fn default_posture_is_disabled_per_rfc_section_3_3_g1() {
        // RFC §3.3 G1 pin.
        let p: EgressPosture = Default::default();
        assert_eq!(p, EgressPosture::Disabled);
        assert!(!p.is_enabled());
    }

    #[test]
    fn valid_directive_flips_posture_to_enabled() {
        let (p, consumed) = EgressPosture::Disabled
            .apply_directive(&dir(1), &so_key().verifying_key(), None, NOW)
            .expect("valid directive accepted");
        assert_eq!(p, EgressPosture::Enabled);
        assert!(p.is_enabled());
        assert_eq!(consumed, 1);
    }

    #[test]
    fn invalid_directive_keeps_posture_disabled() {
        let mut d = dir(1);
        d.signature_hex = "deadbeef".into();
        let err = EgressPosture::Disabled
            .apply_directive(&d, &so_key().verifying_key(), None, NOW)
            .unwrap_err();
        assert_eq!(err, ReleaseError::EgressDirectiveInvalid);
    }

    #[test]
    fn signing_payload_is_deterministic_and_field_bound() {
        let a = dir(1).signing_payload();
        let b = dir(1).signing_payload();
        assert_eq!(a, b);
        assert_ne!(dir(1).signing_payload(), dir(2).signing_payload());
    }

    #[test]
    fn posture_serializes_kebab_case() {
        let s = serde_json::to_string(&EgressPosture::Disabled).unwrap();
        assert_eq!(s, "\"disabled\"");
        let s = serde_json::to_string(&EgressPosture::Enabled).unwrap();
        assert_eq!(s, "\"enabled\"");
    }
}
