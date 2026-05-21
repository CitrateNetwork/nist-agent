//! The `Wizard` state machine.
//!
//! Transitions are pure functions taking the current state +
//! step-specific input, returning the new state + the audit
//! record to append.

use crate::audit::{StepAudit, StepAuditKind};
use crate::error::WizardError;
use crate::state::{HardwareKey, RoleAssignment, WizardState};
use ed25519_dalek::{Signer, SigningKey};
use nist_agent_overlays::{
    cipa, cmmc_l3_baseline, coppa, fedramp_high, ferpa, hipaa, OverlayBuilder,
};
use nist_agent_policy::{types::Role, PolicyBundle, RawSignedBundle};
use nist_agent_prelude::Overlay;
use std::collections::BTreeMap;

/// Pane the operator is currently on. Forward-only — there's no
/// "back to step N" transition in v0.x (Slint UI can simulate it
/// by re-running transitions from a saved earlier state).
///
/// `PartialOrd` derived so callers can guard "are we past step
/// X?" naturally. The wizard doesn't persist its current step
/// across processes (the state ships to disk only after sign),
/// so we don't need Serialize/Deserialize on this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum Step {
    #[default]
    Idle = 0,
    OrgIdentity = 1,
    Roles = 2,
    HardwareKeys = 3,
    Overlays = 4,
    Review = 5,
    Signed = 6,
}

/// The state machine driving the wizard. Holds the current step
/// and the accumulated state; transitions advance both.
#[derive(Debug, Clone, Default)]
pub struct Wizard {
    step: Step,
    state: WizardState,
    audit_log: Vec<StepAudit>,
}

impl Wizard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn step(&self) -> Step {
        self.step
    }

    pub fn state(&self) -> &WizardState {
        &self.state
    }

    pub fn audit_log(&self) -> &[StepAudit] {
        &self.audit_log
    }

    fn assert_step(&self, expected: Step) -> Result<(), WizardError> {
        if self.step != expected {
            return Err(WizardError::WrongStep {
                actual: self.step,
                expected,
            });
        }
        Ok(())
    }

    /// Begin: Idle → OrgIdentity.
    pub fn begin(&mut self) -> Result<(), WizardError> {
        self.assert_step(Step::Idle)?;
        self.step = Step::OrgIdentity;
        Ok(())
    }

    /// Submit org identity: OrgIdentity → Roles.
    pub fn set_org_identity(
        &mut self,
        now_unix: i64,
        org_name: impl Into<String>,
        org_did: impl Into<String>,
    ) -> Result<(), WizardError> {
        self.assert_step(Step::OrgIdentity)?;
        let org_name = org_name.into();
        let org_did = org_did.into();
        if org_name.trim().is_empty() {
            return Err(WizardError::EmptyOrgName);
        }
        self.state.org_name = org_name.clone();
        self.state.org_did = org_did.clone();
        self.audit_log.push(StepAudit::new(
            now_unix,
            StepAuditKind::OrgIdentitySet { org_did, org_name },
        ));
        self.step = Step::Roles;
        Ok(())
    }

    /// Assign one identity to a role. Stays on Roles until the
    /// operator calls `finish_roles()`. Idempotent for re-adding
    /// the same identity.
    pub fn assign_role(
        &mut self,
        now_unix: i64,
        role: Role,
        did: impl Into<String>,
        label: impl Into<String>,
    ) -> Result<(), WizardError> {
        self.assert_step(Step::Roles)?;
        let did = did.into();
        let label = label.into();
        let entries = self.state.roles.entry(role).or_default();
        if !entries.iter().any(|e| e.did == did) {
            entries.push(RoleAssignment {
                did: did.clone(),
                label,
            });
        }
        self.audit_log.push(StepAudit::new(
            now_unix,
            StepAuditKind::RoleAssigned { role, did },
        ));
        Ok(())
    }

    /// Conclude the Roles step. Errors if any of the five base
    /// roles has zero identities. Roles → HardwareKeys.
    pub fn finish_roles(&mut self) -> Result<(), WizardError> {
        self.assert_step(Step::Roles)?;
        if !self.state.roles_complete() {
            // Surface the first missing role so the UI can
            // highlight it.
            for r in Role::ALL {
                if self
                    .state
                    .roles
                    .get(r)
                    .map(|v| v.is_empty())
                    .unwrap_or(true)
                {
                    return Err(WizardError::RoleUnassigned(*r));
                }
            }
        }
        self.step = Step::HardwareKeys;
        Ok(())
    }

    /// Enroll a hardware key. Stays on HardwareKeys until
    /// `finish_hardware_keys()`. The shortness check protects
    /// against operator paste errors.
    pub fn enroll_hardware_key(
        &mut self,
        now_unix: i64,
        key: HardwareKey,
    ) -> Result<(), WizardError> {
        self.assert_step(Step::HardwareKeys)?;
        // Min length: 16 bytes of hex (8 bytes raw). Real FIDO2
        // attestation IDs are far longer; this is a sanity bound.
        if key.fingerprint.len() < 16 {
            return Err(WizardError::HardwareKeyTooShort {
                role: key.role,
                got: key.fingerprint.len(),
            });
        }
        self.audit_log.push(StepAudit::new(
            now_unix,
            StepAuditKind::HardwareKeyEnrolled {
                role: key.role,
                did: key.did.clone(),
                surface: key.surface.clone(),
                fingerprint: key.fingerprint.clone(),
            },
        ));
        self.state.hardware_keys.push(key);
        Ok(())
    }

    /// HardwareKeys → Overlays. No required-key count enforcement
    /// in v0.x (operators may defer enrollment for some roles to
    /// post-deploy); doctor's hardware-key check (S-11) catches
    /// the gap at first agent action attempt.
    pub fn finish_hardware_keys(&mut self) -> Result<(), WizardError> {
        self.assert_step(Step::HardwareKeys)?;
        self.step = Step::Overlays;
        Ok(())
    }

    /// Activate an overlay on top of the CMMC-L3 baseline.
    /// Idempotent — re-activating the same overlay is a no-op
    /// (mirrors `ActiveOverlays::add`).
    pub fn activate_overlay(&mut self, now_unix: i64, overlay: Overlay) -> Result<(), WizardError> {
        self.assert_step(Step::Overlays)?;
        if !self.state.additional_overlays.contains(&overlay) {
            self.state.additional_overlays.push(overlay);
        }
        self.audit_log.push(StepAudit::new(
            now_unix,
            StepAuditKind::OverlayActivated { overlay },
        ));
        Ok(())
    }

    /// Overlays → Review.
    pub fn finish_overlays(&mut self) -> Result<(), WizardError> {
        self.assert_step(Step::Overlays)?;
        self.step = Step::Review;
        Ok(())
    }

    /// Set the output path the signed bundle will be written to.
    /// Required before `sign_and_write`.
    pub fn set_output_path(&mut self, path: std::path::PathBuf) -> Result<(), WizardError> {
        // Allowed on Review or earlier (the UI may set this
        // before Overlays).
        if self.step >= Step::Signed {
            return Err(WizardError::WrongStep {
                actual: self.step,
                expected: Step::Review,
            });
        }
        self.state.output_path = Some(path);
        Ok(())
    }

    /// Construct + sign the PolicyBundle, write it to disk,
    /// emit the final audit record. Review → Signed.
    ///
    /// `now_unix` is the validity-window anchor. `validity_secs`
    /// is added to compute `expires_at`. The SecurityOfficer's
    /// signing key (`sk`) is consumed by reference; the wizard
    /// does NOT retain it.
    pub fn sign_and_write(
        &mut self,
        now_unix: i64,
        validity_secs: u64,
        sk: &SigningKey,
    ) -> Result<RawSignedBundle, WizardError> {
        self.assert_step(Step::Review)?;
        let output_path = self
            .state
            .output_path
            .clone()
            .ok_or_else(|| WizardError::Io("output_path not set".into()))?;
        if output_path.exists() {
            return Err(WizardError::OutputPathExists(
                output_path.display().to_string(),
            ));
        }

        // Construct role-assignment map for the factory.
        let mut role_assignments = BTreeMap::new();
        for (role, entries) in &self.state.roles {
            role_assignments.insert(*role, entries.iter().map(|e| e.did.clone()).collect());
        }
        let opts = OverlayBuilder {
            role_assignments,
            not_before: now_unix,
            expires_at: now_unix + validity_secs as i64,
        };

        // Pick the most-specific base factory; layer the rest on.
        // The first overlay in `additional_overlays` (if any)
        // chooses the base; subsequent ones get added.
        let mut bundle = self.pick_bundle(opts);

        // Hash + signature.
        let canonical = bundle.encode_canonical()?;
        let canonical_hash: [u8; 32] = bundle.canonical_hash()?;
        let signature = sk.sign(&canonical).to_bytes().to_vec();
        let raw = RawSignedBundle {
            bundle_cbor: canonical,
            signature,
        };

        // Write the wire form (CBOR + signature) as a JSON
        // wrapper so operators can inspect it without a CBOR
        // viewer. The harness loads either shape.
        let json = serde_json::json!({
            "bundle_cbor_hex": hex::encode(&raw.bundle_cbor),
            "signature_hex": hex::encode(&raw.signature),
            "canonical_hash_hex": hex::encode(canonical_hash),
        });
        let body = serde_json::to_vec_pretty(&json)
            .map_err(|e| WizardError::Io(format!("serialize: {e}")))?;
        std::fs::write(&output_path, body)
            .map_err(|e| WizardError::Io(format!("write {}: {e}", output_path.display())))?;

        self.audit_log.push(StepAudit::new(
            now_unix,
            StepAuditKind::PolicyBundleSigned {
                bundle_name: bundle.bundle_name.clone(),
                canonical_hash,
            },
        ));
        // Re-bind `bundle` to silence the unused-mut warning when
        // the bundle isn't mutated after the canonical step.
        let _ = &mut bundle;
        self.step = Step::Signed;
        Ok(raw)
    }

    /// Build the PolicyBundle from the accumulated state by
    /// picking the right overlay factory and layering additional
    /// overlays via the one-way ratchet.
    fn pick_bundle(&self, opts: OverlayBuilder) -> PolicyBundle {
        let extras = &self.state.additional_overlays;

        // Priority order: the most-restrictive overlay wins as
        // the base. Operator-chosen extras layer via add().
        // Note: CMMC-L3 baseline is always active regardless.
        let base: PolicyBundle = if extras.contains(&Overlay::FedrampHigh) {
            fedramp_high(opts)
        } else if extras.contains(&Overlay::HipaaHitech) {
            hipaa(opts)
        } else if extras.contains(&Overlay::Ferpa) {
            ferpa(opts)
        } else if extras.contains(&Overlay::Coppa) {
            coppa(opts)
        } else if extras.contains(&Overlay::Cipa) {
            cipa(opts)
        } else {
            cmmc_l3_baseline(opts)
        };

        // Layer remaining extras (each factory already added one
        // overlay; the others go here).
        let mut bundle = base;
        for o in extras {
            if !bundle.overlays.is_active(*o) {
                bundle.overlays.add(*o);
            }
        }
        bundle
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use nist_agent_policy::types::Role;
    use tempfile::TempDir;

    fn sk() -> SigningKey {
        SigningKey::from_bytes(&[9u8; 32])
    }

    /// Helper: walk the wizard through every step with placeholder
    /// inputs. Used as the smoke-path skeleton for several tests.
    fn run_happy_path(out: std::path::PathBuf) -> (Wizard, RawSignedBundle) {
        let mut w = Wizard::new();
        w.begin().expect("begin");
        w.set_org_identity(0, "Test Co", "did:test:org:0xab")
            .expect("org");
        for (i, r) in Role::ALL.iter().enumerate() {
            w.assign_role(0, *r, format!("did:test:role:{i}"), format!("Role {i}"))
                .expect("role");
        }
        w.finish_roles().expect("finish_roles");
        w.enroll_hardware_key(
            0,
            HardwareKey {
                role: Role::SecurityOfficer,
                did: "did:test:role:3".into(),
                surface: "piv-cac".into(),
                fingerprint: "0123456789abcdef0123456789abcdef".into(),
            },
        )
        .expect("hwkey");
        w.finish_hardware_keys().expect("finish_hwkeys");
        w.finish_overlays().expect("finish_overlays");
        w.set_output_path(out).expect("output_path");
        let raw = w.sign_and_write(0, 365 * 24 * 3600, &sk()).expect("sign");
        (w, raw)
    }

    // ── happy path ──

    #[test]
    fn full_walkthrough_produces_signed_bundle() {
        let tmp = TempDir::new().expect("tempdir");
        let out = tmp.path().join("bundle.json");
        let (w, raw) = run_happy_path(out.clone());
        assert_eq!(w.step(), Step::Signed);
        assert!(out.exists());
        // 6 audit records (1 org + 5 role + 1 hwkey + 0 overlay + 1 sign = 8 actually)
        let kinds: Vec<&StepAuditKind> = w.audit_log().iter().map(|a| &a.kind).collect();
        assert!(matches!(kinds[0], StepAuditKind::OrgIdentitySet { .. }));
        assert!(matches!(
            kinds.last().unwrap(),
            StepAuditKind::PolicyBundleSigned { .. }
        ));
        // The signature MUST verify against the SK's public key.
        let pk = sk().verifying_key();
        let decoded = raw.verify_and_decode(&pk).expect("verify");
        assert_eq!(decoded.bundle_name, "cmmc-l3-baseline");
    }

    #[test]
    fn picks_fedramp_high_when_active() {
        let tmp = TempDir::new().expect("tempdir");
        let out = tmp.path().join("b.json");
        let mut w = Wizard::new();
        w.begin().unwrap();
        w.set_org_identity(0, "Org", "did:o").unwrap();
        for r in Role::ALL {
            w.assign_role(0, *r, format!("did:r:{r:?}"), "x").unwrap();
        }
        w.finish_roles().unwrap();
        w.finish_hardware_keys().unwrap();
        w.activate_overlay(0, Overlay::FedrampHigh).unwrap();
        w.finish_overlays().unwrap();
        w.set_output_path(out).unwrap();
        let raw = w.sign_and_write(0, 100_000, &sk()).unwrap();
        let pk = sk().verifying_key();
        let bundle = raw.verify_and_decode(&pk).unwrap();
        assert_eq!(bundle.bundle_name, "fedramp-high");
    }

    // ── transition guards ──

    #[test]
    fn begin_only_from_idle() {
        let mut w = Wizard::new();
        w.begin().unwrap();
        let err = w.begin().expect_err("double-begin must error");
        match err {
            WizardError::WrongStep { actual, expected } => {
                assert_eq!(actual, Step::OrgIdentity);
                assert_eq!(expected, Step::Idle);
            }
            other => panic!("expected WrongStep, got {other:?}"),
        }
    }

    #[test]
    fn finish_roles_errors_on_unfilled_role() {
        let mut w = Wizard::new();
        w.begin().unwrap();
        w.set_org_identity(0, "Org", "did:o").unwrap();
        // Assign only 4 of 5 roles.
        for r in [
            Role::Operator,
            Role::Reviewer,
            Role::ComplianceOfficer,
            Role::SecurityOfficer,
        ] {
            w.assign_role(0, r, format!("did:r:{r:?}"), "x").unwrap();
        }
        let err = w.finish_roles().expect_err("missing Auditor must error");
        match err {
            WizardError::RoleUnassigned(Role::Auditor) => {}
            other => panic!("expected RoleUnassigned(Auditor), got {other:?}"),
        }
    }

    #[test]
    fn org_identity_rejects_empty() {
        let mut w = Wizard::new();
        w.begin().unwrap();
        let err = w
            .set_org_identity(0, "  ", "did:o")
            .expect_err("empty must error");
        match err {
            WizardError::EmptyOrgName => {}
            other => panic!("expected EmptyOrgName, got {other:?}"),
        }
    }

    #[test]
    fn hardware_key_rejects_short_fingerprint() {
        let mut w = Wizard::new();
        w.begin().unwrap();
        w.set_org_identity(0, "Org", "did:o").unwrap();
        for r in Role::ALL {
            w.assign_role(0, *r, format!("did:r:{r:?}"), "x").unwrap();
        }
        w.finish_roles().unwrap();
        let err = w
            .enroll_hardware_key(
                0,
                HardwareKey {
                    role: Role::Operator,
                    did: "did".into(),
                    surface: "fido2".into(),
                    fingerprint: "tooshort".into(),
                },
            )
            .expect_err("short fingerprint must error");
        match err {
            WizardError::HardwareKeyTooShort {
                role: Role::Operator,
                ..
            } => {}
            other => panic!("expected HardwareKeyTooShort(Operator), got {other:?}"),
        }
    }

    #[test]
    fn assign_role_is_idempotent() {
        let mut w = Wizard::new();
        w.begin().unwrap();
        w.set_org_identity(0, "Org", "did:o").unwrap();
        w.assign_role(0, Role::Operator, "did:alice", "Alice")
            .unwrap();
        w.assign_role(0, Role::Operator, "did:alice", "Alice")
            .unwrap();
        let entries = w.state().roles.get(&Role::Operator).unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn output_path_collision_errors() {
        let tmp = TempDir::new().expect("tempdir");
        let out = tmp.path().join("bundle.json");
        std::fs::write(&out, b"prior").unwrap();
        let mut w = Wizard::new();
        w.begin().unwrap();
        w.set_org_identity(0, "Org", "did:o").unwrap();
        for r in Role::ALL {
            w.assign_role(0, *r, format!("did:r:{r:?}"), "x").unwrap();
        }
        w.finish_roles().unwrap();
        w.finish_hardware_keys().unwrap();
        w.finish_overlays().unwrap();
        w.set_output_path(out).unwrap();
        let err = w
            .sign_and_write(0, 100_000, &sk())
            .expect_err("collision must error");
        match err {
            WizardError::OutputPathExists(_) => {}
            other => panic!("expected OutputPathExists, got {other:?}"),
        }
    }

    #[test]
    fn audit_log_has_one_entry_per_completed_step() {
        let tmp = TempDir::new().expect("tempdir");
        let out = tmp.path().join("b.json");
        let (w, _) = run_happy_path(out);
        // 1 org + 5 role + 1 hwkey + 1 sign = 8.
        assert_eq!(w.audit_log().len(), 8);
    }
}
