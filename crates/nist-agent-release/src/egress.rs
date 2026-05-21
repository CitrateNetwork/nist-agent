//! Egress posture defaults.
//!
//! `dist-airgap-install.feature` pins the rule: "Egress remains
//! disabled after install"; "any capsule with network capability
//! is greyed-out in the marketplace pane". RFC §3.3 G1 makes the
//! global posture explicit: off by default, opt-in by signed
//! SecurityOfficer policy directive.
//!
//! Activation flow:
//!
//! 1. Operator's SecurityOfficer signs an `EgressDirective`
//!    (Ed25519 over the directive's canonical encoding).
//! 2. The harness verifies the signature against the configured
//!    SecurityOfficer trust root.
//! 3. On valid signature, the harness records the activation as
//!    an AuditRecord and flips egress to `Enabled`.
//!
//! This module owns the posture enum + the verification gate. The
//! actual canonical encoding + trust-root distribution live in
//! `nist-agent-policy` (PolicyBundle / SecurityOfficer key); we
//! consume them via a verifier callback so this crate stays
//! free of policy-bundle coupling.

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

impl EgressPosture {
    pub fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

/// A signed SecurityOfficer directive that flips egress to
/// `Enabled`. The verifier callback owns the signature check
/// (typically delegates to `nist-agent-policy`'s SecurityOfficer
/// verification path) — keeping it injectable means this crate
/// doesn't take a transitive dep on the full policy bundle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EgressDirective {
    /// Operator-friendly reason field; persisted in the
    /// AuditRecord this activation generates.
    pub reason: String,
    /// ISO-8601 timestamp the SecurityOfficer asserts as the
    /// activation moment. Surfaced for audit.
    pub asserted_at_iso: String,
    /// Hex-encoded Ed25519 signature over a canonical encoding
    /// of `(reason, asserted_at_iso)`. Opaque here; the verifier
    /// callback decides what counts as valid.
    pub signature_hex: String,
}

impl EgressPosture {
    /// Apply a directive. The caller passes a verifier that
    /// returns `true` when the directive's signature is valid
    /// under the operator's configured SecurityOfficer trust
    /// root. On `true`, the posture flips to `Enabled`; on
    /// `false`, the posture is unchanged and the error is
    /// returned.
    pub fn apply_directive(
        self,
        directive: &EgressDirective,
        verifier: impl FnOnce(&EgressDirective) -> bool,
    ) -> Result<Self, ReleaseError> {
        if !verifier(directive) {
            return Err(ReleaseError::EgressDirectiveInvalid);
        }
        Ok(Self::Enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> EgressDirective {
        EgressDirective {
            reason: "operator approved on 2026-06-01".into(),
            asserted_at_iso: "2026-06-01T10:00:00Z".into(),
            signature_hex: "deadbeef".into(),
        }
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
        let p = EgressPosture::Disabled
            .apply_directive(&dir(), |_| true)
            .expect("valid directive accepted");
        assert_eq!(p, EgressPosture::Enabled);
        assert!(p.is_enabled());
    }

    #[test]
    fn invalid_directive_keeps_posture_disabled() {
        let err = EgressPosture::Disabled
            .apply_directive(&dir(), |_| false)
            .unwrap_err();
        assert_eq!(err, ReleaseError::EgressDirectiveInvalid);
    }

    #[test]
    fn posture_serializes_kebab_case() {
        let s = serde_json::to_string(&EgressPosture::Disabled).unwrap();
        assert_eq!(s, "\"disabled\"");
        let s = serde_json::to_string(&EgressPosture::Enabled).unwrap();
        assert_eq!(s, "\"enabled\"");
    }
}
