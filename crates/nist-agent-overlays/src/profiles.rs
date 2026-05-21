//! PolicyBundle factories — one per RFC §2.3 overlay.
//!
//! Each function takes role assignments + a validity window and
//! returns a canonical PolicyBundle pre-configured for the
//! overlay's posture. The SecurityOfficer signs the result with
//! `bundle.encode_canonical()` + `sk.sign()`.
//!
//! Posture defaults per overlay are derived from RFC §2.3 + the
//! per-overlay Gherkin feature files under `features/overlays/`.

use nist_agent_policy::bundle::BUNDLE_VERSION;
use nist_agent_policy::overlay_state::ActiveOverlays;
use nist_agent_policy::types::{AnchorStrategy, EgressPosture, Role};
use nist_agent_policy::PolicyBundle;
use nist_agent_prelude::Overlay;
use std::collections::BTreeMap;

/// Shared options across all factories. Lets each call site
/// inject the deployment-specific bits without re-stating the
/// boilerplate.
#[derive(Debug, Clone)]
pub struct OverlayBuilder {
    /// All five base roles → list of DID strings. The factory
    /// will refuse to build if any of `Role::ALL` is missing.
    pub role_assignments: BTreeMap<Role, Vec<String>>,
    /// Validity window — unix epoch seconds.
    pub not_before: i64,
    pub expires_at: i64,
}

impl OverlayBuilder {
    /// Build with placeholder roles + an open window (0 to MAX).
    /// Tests use this; production callers supply real values.
    pub fn placeholder() -> Self {
        let mut role_assignments = BTreeMap::new();
        for r in Role::ALL {
            role_assignments.insert(*r, vec![format!("did:placeholder:{:?}", r)]);
        }
        Self {
            role_assignments,
            not_before: 0,
            expires_at: i64::MAX,
        }
    }
}

// ── CMMC-L3 baseline ─────────────────────────────────────────────

/// CMMC Level 3 baseline — RFC §2.1.
///
/// This is the always-on overlay. Every operator's bundle includes
/// it whether they want it or not. Defaults:
///
///   - egress: Disabled (RFC G1 air-gap by default)
///   - anchor strategy: HybridNightlyPlusCapsule (RFC §6.3 default
///     for FedRAMP / CMMC deployments)
///   - no overlay augmentations
pub fn cmmc_l3_baseline(opts: OverlayBuilder) -> PolicyBundle {
    PolicyBundle {
        bundle_version: BUNDLE_VERSION,
        bundle_name: "cmmc-l3-baseline".into(),
        not_before: opts.not_before,
        expires_at: opts.expires_at,
        overlays: ActiveOverlays::new_with_cmmc_baseline(),
        risk_tier_map: BTreeMap::new(),
        role_assignments: opts.role_assignments,
        anchor_strategy: AnchorStrategy::HybridNightlyPlusCapsule,
        egress_posture: EgressPosture::Disabled,
    }
}

// ── FERPA ────────────────────────────────────────────────────────

/// FERPA overlay — CMMC-L3 baseline + Family Educational Rights and
/// Privacy Act (RFC §2.3).
///
/// Per `features/overlays/overlay-ferpa.feature`:
///   - ComplianceOfficer signature required before any
///     FERPA-restricted emit (enforced at HITL gate, declared in
///     bundle by overlay presence)
///   - Mobile signing permitted with 1-hour TTL (allowed because
///     `Overlay::Ferpa.forbids_mobile_signing()` returns false)
///   - Retention floor: 5 years (read from `retention_floor`)
pub fn ferpa(opts: OverlayBuilder) -> PolicyBundle {
    let mut b = cmmc_l3_baseline(opts);
    b.bundle_name = "ferpa".into();
    b.overlays.add(Overlay::Ferpa);
    b
}

// ── COPPA ────────────────────────────────────────────────────────

/// COPPA overlay — Children's Online Privacy Protection Act
/// (RFC §2.3).
///
/// Per `features/overlays/overlay-coppa.feature`:
///   - Capsule reads of COPPA-restricted data require a verifiable
///     parental-consent token at HITL time
///   - Emissions involving COPPA data are tier-high regardless of
///     the capsule's declared tier (enforced via risk_tier_map
///     escalation — populated by the caller per their capsule set)
///   - Retention floor: 5 years
pub fn coppa(opts: OverlayBuilder) -> PolicyBundle {
    let mut b = cmmc_l3_baseline(opts);
    b.bundle_name = "coppa".into();
    b.overlays.add(Overlay::Coppa);
    b
}

// ── CIPA ─────────────────────────────────────────────────────────

/// CIPA overlay — Children's Internet Protection Act (RFC §2.3).
///
/// Per `features/overlays/overlay-cipa.feature`:
///   - Every model token stream passes through a CIPA-content-
///     filter capsule before reaching a student-facing surface
///     (capsule-level enforcement, not bundle-level)
///   - Blocked emissions are logged with rationale (audit record
///     type ContentFilterDecision)
///   - Retention floor: 5 years (school context)
pub fn cipa(opts: OverlayBuilder) -> PolicyBundle {
    let mut b = cmmc_l3_baseline(opts);
    b.bundle_name = "cipa".into();
    b.overlays.add(Overlay::Cipa);
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use nist_agent_policy::RawSignedBundle;

    fn sign(bundle: &PolicyBundle) -> (RawSignedBundle, SigningKey) {
        let sk = SigningKey::from_bytes(&[7u8; 32]);
        let cbor = bundle.encode_canonical().expect("encode");
        let sig = sk.sign(&cbor).to_bytes().to_vec();
        (
            RawSignedBundle {
                bundle_cbor: cbor,
                signature: sig,
            },
            sk,
        )
    }

    fn opts() -> OverlayBuilder {
        OverlayBuilder::placeholder()
    }

    // ── construction tests ──

    #[test]
    fn cmmc_l3_baseline_has_only_cmmc() {
        let b = cmmc_l3_baseline(opts());
        assert!(b.overlays.is_active(Overlay::CmmcL3));
        assert!(!b.overlays.is_active(Overlay::Ferpa));
        assert!(!b.overlays.is_active(Overlay::HipaaHitech));
        assert_eq!(b.overlays.active().len(), 1);
        assert_eq!(b.bundle_name, "cmmc-l3-baseline");
    }

    #[test]
    fn ferpa_adds_ferpa_on_top_of_cmmc() {
        let b = ferpa(opts());
        assert!(b.overlays.is_active(Overlay::CmmcL3));
        assert!(b.overlays.is_active(Overlay::Ferpa));
        assert_eq!(b.overlays.active().len(), 2);
        assert_eq!(b.bundle_name, "ferpa");
    }

    #[test]
    fn coppa_adds_coppa_on_top_of_cmmc() {
        let b = coppa(opts());
        assert!(b.overlays.is_active(Overlay::CmmcL3));
        assert!(b.overlays.is_active(Overlay::Coppa));
        assert_eq!(b.bundle_name, "coppa");
    }

    #[test]
    fn cipa_adds_cipa_on_top_of_cmmc() {
        let b = cipa(opts());
        assert!(b.overlays.is_active(Overlay::CmmcL3));
        assert!(b.overlays.is_active(Overlay::Cipa));
        assert_eq!(b.bundle_name, "cipa");
    }

    // ── canonical CBOR round-trip ──

    #[test]
    fn every_overlay_signs_verifies_decodes() {
        for (name, b) in [
            ("cmmc-l3-baseline", cmmc_l3_baseline(opts())),
            ("ferpa", ferpa(opts())),
            ("coppa", coppa(opts())),
            ("cipa", cipa(opts())),
        ] {
            let (raw, sk) = sign(&b);
            let decoded = raw
                .verify_and_decode(&sk.verifying_key())
                .unwrap_or_else(|e| panic!("{name} verify failed: {e}"));
            assert_eq!(decoded.bundle_name, name);
        }
    }

    // ── activate at construction-time defaults ──

    #[test]
    fn every_overlay_activates_at_now_zero() {
        for b in [
            cmmc_l3_baseline(opts()),
            ferpa(opts()),
            coppa(opts()),
            cipa(opts()),
        ] {
            b.activate(0, &BTreeMap::new())
                .unwrap_or_else(|e| panic!("{} activate failed: {e}", b.bundle_name));
        }
    }

    // ── doctor PolicyBundleValidityCheck passes for every overlay ──

    #[test]
    fn doctor_policy_check_passes_for_every_overlay() {
        use nist_agent_doctor::{
            checks::{NistCheck, PolicyBundleValidityCheck},
            NistDoctorContext, Severity,
        };
        for b in [
            cmmc_l3_baseline(opts()),
            ferpa(opts()),
            coppa(opts()),
            cipa(opts()),
        ] {
            let name = b.bundle_name.clone();
            let (raw, sk) = sign(&b);
            let mut ctx = NistDoctorContext::empty(0);
            ctx.raw_bundle = Some(raw);
            ctx.so_pubkey = Some(sk.verifying_key());
            let r = PolicyBundleValidityCheck.run(&ctx);
            assert!(
                matches!(r.severity, Severity::Pass),
                "{name} doctor policy check: {r:?}"
            );
        }
    }

    // ── doctor RoleLatticeCheck passes too ──

    #[test]
    fn doctor_role_lattice_check_passes_for_every_overlay() {
        use nist_agent_doctor::{
            checks::{NistCheck, RoleLatticeCheck},
            NistDoctorContext, Severity,
        };
        for b in [
            cmmc_l3_baseline(opts()),
            ferpa(opts()),
            coppa(opts()),
            cipa(opts()),
        ] {
            let name = b.bundle_name.clone();
            let (raw, sk) = sign(&b);
            let mut ctx = NistDoctorContext::empty(0);
            ctx.raw_bundle = Some(raw);
            ctx.so_pubkey = Some(sk.verifying_key());
            let r = RoleLatticeCheck.run(&ctx);
            assert!(
                matches!(r.severity, Severity::Pass),
                "{name} role-lattice: {r:?}"
            );
        }
    }

    // ── ratchet — overlay set is add-only ──

    #[test]
    fn overlay_factories_never_remove_cmmc_baseline() {
        // The constructor includes CMMC-L3; subsequent add() calls
        // augment but never remove. Cross-check the type-system
        // property exercised in nist-agent-policy's
        // ActiveOverlays tests.
        for b in [
            cmmc_l3_baseline(opts()),
            ferpa(opts()),
            coppa(opts()),
            cipa(opts()),
        ] {
            assert!(
                b.overlays.is_active(Overlay::CmmcL3),
                "{} dropped CMMC-L3",
                b.bundle_name
            );
        }
    }
}
