//! Per-overlay mobile-signing eligibility + signature TTL.
//!
//! The feature scenario "Mobile eligibility by overlay" tabulates
//! the rule: CMMC-L3 / FERPA / HIPAA permit; FedRAMP-High / ITAR
//! forbid. ITAR is a post-v1.0 overlay; CMMC-L3 is the v1.0
//! baseline; the remaining four are toggleable.
//!
//! The TTL rule: "Mobile signatures have a 1-hour TTL by default".
//! This module pins both rules so the daemon, the desktop UI, and
//! the future native apps all read the same answer.

use std::time::Duration;

use nist_agent_prelude::Overlay;
use serde::{Deserialize, Serialize};

use crate::error::MobilePairingError;

/// Mobile-signature TTL. Wrapped so callers don't trip over
/// `Duration` arithmetic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SignatureTtl(pub Duration);

impl SignatureTtl {
    /// The RFC §5.6 default of one hour. The feature scenario
    /// "Reviewer signs from Mobile at time T → expires at T + 1
    /// hour" pins this value.
    pub const DEFAULT: SignatureTtl = SignatureTtl(Duration::from_secs(60 * 60));

    pub fn secs(self) -> u64 {
        self.0.as_secs()
    }
}

/// The eligibility verdict for a given active overlay set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MobileEligibility {
    pub allowed: bool,
    pub ttl: SignatureTtl,
    /// The overlay (if any) that forbade mobile signing. Empty
    /// when `allowed = true`. Surfaced so the desktop UI can show
    /// the operator *why* the mobile tab is disabled.
    pub forbidden_by: Option<Overlay>,
}

impl MobileEligibility {
    /// Compute eligibility from the active overlay set. If any
    /// overlay forbids mobile signing (`Overlay::forbids_mobile_signing()`
    /// from `nist-agent-prelude`), the verdict is `allowed = false`
    /// and `forbidden_by` carries the first such overlay (stable
    /// ordering because the caller passes a slice).
    pub fn from_active(active: &[Overlay], ttl: SignatureTtl) -> Self {
        let blocker = active.iter().copied().find(|o| o.forbids_mobile_signing());
        Self {
            allowed: blocker.is_none(),
            ttl,
            forbidden_by: blocker,
        }
    }

    /// Convenience: eligibility with the default 1-hour TTL.
    pub fn from_active_default(active: &[Overlay]) -> Self {
        Self::from_active(active, SignatureTtl::DEFAULT)
    }

    /// Validate a signature timestamp + ttl + "now" triple. The
    /// daemon calls this when a `SignedDecision` arrives, before
    /// routing it to the `ApprovalQueue`.
    pub fn check_not_expired(
        signed_at_unix: u64,
        ttl: SignatureTtl,
        now_unix: u64,
        signed_at_iso: &str,
    ) -> Result<(), MobilePairingError> {
        let expires = signed_at_unix.saturating_add(ttl.secs());
        if now_unix > expires {
            return Err(MobilePairingError::SignatureExpired {
                signed_at_iso: signed_at_iso.to_string(),
                ttl_secs: ttl.secs(),
            });
        }
        Ok(())
    }

    /// Validate a fresh sign attempt's eligibility. Used at the
    /// daemon's pre-route gate; the desktop UI uses
    /// `Self::from_active` directly to display the disable reason.
    pub fn require_allowed(&self) -> Result<(), MobilePairingError> {
        if !self.allowed {
            return Err(MobilePairingError::MobileSigningForbidden);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmmc_only_allows_mobile() {
        let e = MobileEligibility::from_active_default(&[Overlay::CmmcL3]);
        assert!(e.allowed);
        assert!(e.forbidden_by.is_none());
        e.require_allowed()
            .expect("CMMC-L3 baseline permits mobile");
    }

    #[test]
    fn ferpa_hipaa_allow_mobile_per_feature_table() {
        for o in [
            Overlay::Ferpa,
            Overlay::HipaaHitech,
            Overlay::Coppa,
            Overlay::Cipa,
        ] {
            let e = MobileEligibility::from_active_default(&[Overlay::CmmcL3, o]);
            assert!(e.allowed, "{o:?} should permit mobile");
        }
    }

    #[test]
    fn fedramp_high_forbids_mobile_per_feature_table() {
        let e = MobileEligibility::from_active_default(&[Overlay::CmmcL3, Overlay::FedrampHigh]);
        assert!(!e.allowed);
        assert_eq!(e.forbidden_by, Some(Overlay::FedrampHigh));
        match e.require_allowed() {
            Err(MobilePairingError::MobileSigningForbidden) => {}
            other => panic!("expected MobileSigningForbidden, got {other:?}"),
        }
    }

    #[test]
    fn default_ttl_is_one_hour_per_rfc_section_5_6() {
        // The feature scenario "Mobile signatures have a 1-hour
        // TTL by default" pins this value; the daemon, the
        // desktop UI, and the native apps all read this constant.
        assert_eq!(SignatureTtl::DEFAULT.secs(), 3600);
    }

    #[test]
    fn signature_within_ttl_is_accepted() {
        // signed at t=0, ttl=3600, now=t+1800 → 30 min in, valid.
        MobileEligibility::check_not_expired(
            0,
            SignatureTtl::DEFAULT,
            1800,
            "1970-01-01T00:00:00Z",
        )
        .expect("30 min into a 1-hour TTL is still valid");
    }

    #[test]
    fn signature_at_ttl_boundary_is_accepted() {
        // The feature scenario says "expires at T + 1 hour" — at
        // exactly T+ttl the signature is still valid; strictly
        // after is when it expires.
        MobileEligibility::check_not_expired(
            0,
            SignatureTtl::DEFAULT,
            3600,
            "1970-01-01T00:00:00Z",
        )
        .expect("at-boundary signature is still valid");
    }

    #[test]
    fn signature_past_ttl_is_refused() {
        let err = MobileEligibility::check_not_expired(
            0,
            SignatureTtl::DEFAULT,
            3601,
            "1970-01-01T00:00:00Z",
        )
        .unwrap_err();
        match err {
            MobilePairingError::SignatureExpired { ttl_secs, .. } => {
                assert_eq!(ttl_secs, 3600);
            }
            other => panic!("expected SignatureExpired, got {other:?}"),
        }
    }

    #[test]
    fn ttl_serializes_as_seconds() {
        // The wire form carries TTL as seconds (Duration's serde
        // round-trip). Pin so a future serde rework can't drift
        // the contract.
        let s = serde_json::to_string(&SignatureTtl::DEFAULT).unwrap();
        // Duration serializes as {"secs":3600,"nanos":0} in
        // serde-json by default; we use #[serde(transparent)] so
        // it inherits that shape.
        assert!(s.contains("3600"));
    }
}
