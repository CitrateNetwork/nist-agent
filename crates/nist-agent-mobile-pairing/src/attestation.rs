//! Device-attestation envelope + policy allowlist.
//!
//! The feature scenario "Mobile device attestation chains are
//! policy-allowlisted" pins the contract: a device presents a
//! chain envelope; the daemon checks the chain's id against the
//! operator-configured allowlist; non-allowlisted chains refuse
//! the signing attempt with a stable error message.
//!
//! The chain id is the user-facing handle (e.g. "apple-sep",
//! "google-strongbox"). The cryptographic chain verification is
//! deferred to S-12 / the runtime-side crypto crate — this module
//! routes the envelope to the verifier, but treats the verifier's
//! decision as an external concern.

use serde::{Deserialize, Serialize};

use crate::error::MobilePairingError;

/// Stable identifier for a device-attestation chain. Operator-
/// configurable allowlist keys on this exact string. The
/// well-known set in v1.0:
///
/// - `"apple-sep"`         — Apple Secure Enclave attestation.
/// - `"google-strongbox"`  — Google StrongBox attestation.
/// - `"samsung-knox"`      — Samsung Knox attestation (not in
///   default allowlist; operators must opt in).
///
/// Free-form so an operator can allowlist a future vendor without
/// a Rust release.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttestationChainId(pub String);

impl AttestationChainId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Coarse vendor tag for display purposes. The chain id is the
/// load-bearing field; this enum exists so the desktop UI can
/// render a vendor icon without re-deriving it from the chain id.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceVendor {
    Apple,
    Google,
    Samsung,
    Other,
}

/// One device-attestation envelope as the device presents it to
/// the daemon. The opaque chain bytes are routed to the underlying
/// crypto verifier (deferred to S-12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceAttestation {
    pub chain_id: AttestationChainId,
    pub vendor: DeviceVendor,
    /// Hex-encoded chain bytes — opaque to this module; the
    /// crypto verifier consumes them.
    pub chain_bytes_hex: String,
    /// Display label the daemon shows in the SecurityOfficer's
    /// approval row when the pairing is initiated.
    pub display_label: String,
}

/// Operator-configured policy allowlist. Persisted as part of the
/// `PolicyBundle` (S-6); this module just owns the predicate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AttestationAllowlist {
    pub allowed_chain_ids: Vec<AttestationChainId>,
}

impl AttestationAllowlist {
    pub fn new(ids: impl IntoIterator<Item = AttestationChainId>) -> Self {
        Self {
            allowed_chain_ids: ids.into_iter().collect(),
        }
    }

    /// The v1.0-default allowlist per RFC §5.6: Apple SEP + Google
    /// StrongBox. Samsung Knox is opt-in (must be added by the
    /// operator), matching the feature scenario.
    pub fn v1_default() -> Self {
        Self::new([
            AttestationChainId::new("apple-sep"),
            AttestationChainId::new("google-strongbox"),
        ])
    }

    /// Refuse the attestation if its chain id isn't allowlisted.
    /// Maps directly to the feature scenario "Samsung Knox is
    /// refused with 'device attestation not on allowlist'".
    pub fn validate_chain(
        &self,
        attestation: &DeviceAttestation,
    ) -> Result<(), MobilePairingError> {
        if self
            .allowed_chain_ids
            .iter()
            .any(|a| a == &attestation.chain_id)
        {
            Ok(())
        } else {
            Err(MobilePairingError::AttestationNotAllowlisted {
                chain_id: attestation.chain_id.0.clone(),
            })
        }
    }

    /// Whether a chain id is allowlisted. Cheaper helper for the
    /// desktop UI which renders an "allowlisted" badge inline in
    /// the pairing approval row.
    pub fn permits(&self, chain_id: &AttestationChainId) -> bool {
        self.allowed_chain_ids.contains(chain_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn att(chain: &str, vendor: DeviceVendor) -> DeviceAttestation {
        DeviceAttestation {
            chain_id: AttestationChainId::new(chain),
            vendor,
            chain_bytes_hex: "00".into(),
            display_label: chain.into(),
        }
    }

    #[test]
    fn default_allowlist_permits_apple_and_google_only() {
        let a = AttestationAllowlist::v1_default();
        a.validate_chain(&att("apple-sep", DeviceVendor::Apple))
            .expect("Apple SEP permitted by default");
        a.validate_chain(&att("google-strongbox", DeviceVendor::Google))
            .expect("Google StrongBox permitted by default");
    }

    #[test]
    fn samsung_knox_refused_by_default_per_feature_scenario() {
        // Feature: "Samsung Knox attestation is refused with
        // 'device attestation not on allowlist'".
        let a = AttestationAllowlist::v1_default();
        let err = a
            .validate_chain(&att("samsung-knox", DeviceVendor::Samsung))
            .unwrap_err();
        match err {
            MobilePairingError::AttestationNotAllowlisted { chain_id } => {
                assert_eq!(chain_id, "samsung-knox");
            }
            other => panic!("expected AttestationNotAllowlisted, got {other:?}"),
        }
    }

    #[test]
    fn operator_can_extend_allowlist() {
        // The allowlist is operator-configurable per the ADR —
        // adding Samsung Knox is a config change, not a code change.
        let a = AttestationAllowlist::new([
            AttestationChainId::new("apple-sep"),
            AttestationChainId::new("samsung-knox"),
        ]);
        a.validate_chain(&att("samsung-knox", DeviceVendor::Samsung))
            .expect("operator opted in to Samsung Knox");
    }

    #[test]
    fn permits_check_is_consistent_with_validate() {
        let a = AttestationAllowlist::v1_default();
        assert!(a.permits(&AttestationChainId::new("apple-sep")));
        assert!(!a.permits(&AttestationChainId::new("samsung-knox")));
    }

    #[test]
    fn chain_id_serializes_transparently() {
        let id = AttestationChainId::new("apple-sep");
        let s = serde_json::to_string(&id).unwrap();
        assert_eq!(s, "\"apple-sep\"");
        let round: AttestationChainId = serde_json::from_str(&s).unwrap();
        assert_eq!(round, id);
    }

    #[test]
    fn vendor_serializes_kebab_case() {
        let s = serde_json::to_string(&DeviceVendor::Google).unwrap();
        assert_eq!(s, "\"google\"");
    }
}
