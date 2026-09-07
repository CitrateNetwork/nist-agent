//! Overlay activation state + one-way ratchet — RFC §2.3.
//!
//! "Overlay activation is a one-way ratchet within a deployment:
//! an overlay may be added by the Security Officer through the
//! standard policy bundle update flow, but removal requires a
//! documented decommissioning workflow and a final audit export of
//! the period during which the overlay applied."
//!
//! This module implements the ratchet as an `ActiveOverlays`
//! wrapper around a `BTreeSet<Overlay>`. Insertion is free.
//! Removal is gated by a `DecommissioningWorkflow` proof; the
//! ratchet refuses removal without it.

use crate::error::PolicyError;
use crate::Overlay;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The active overlay set + the activation history. The history is
/// append-only — once an overlay activates, removal records it as
/// decommissioned but the entry stays so auditors can verify the
/// period during which the overlay applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActiveOverlays {
    /// Currently-active overlays. CMMC-L3 baseline is always
    /// present; the constructor enforces that.
    active: BTreeSet<Overlay>,
    /// Overlays previously activated and then formally
    /// decommissioned. They stay in this set forever.
    decommissioned: BTreeSet<Overlay>,
}

/// Proof token that a decommissioning workflow has been documented
/// (RFC §2.3). In production this is signed by SecurityOfficer +
/// ComplianceOfficer and points to a workflow file under
/// `.agentile/sprints/active/`.
///
/// The proof is only meaningful if it is *bound* to a concrete
/// document: `remove_with_workflow` re-hashes the document the
/// caller supplies and refuses removal unless the digest matches
/// `workflow_sha256` (NA2-B-029). A default/zero token therefore
/// no longer authorizes anything.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecommissioningWorkflow {
    /// Sprint file path or URL where the workflow is documented.
    pub workflow_ref: String,
    /// SHA-256 of the workflow document at the time of activation.
    pub workflow_sha256: [u8; 32],
}

impl DecommissioningWorkflow {
    /// Verify this proof against the actual bytes of the referenced
    /// workflow document. Rejects an empty reference, an all-zero
    /// digest, or a digest that does not match `document`.
    pub fn verify(&self, document: &[u8]) -> Result<(), PolicyError> {
        use sha2::{Digest, Sha256};
        if self.workflow_ref.trim().is_empty() {
            return Err(PolicyError::WorkflowProofInvalid(
                "empty workflow_ref".into(),
            ));
        }
        if self.workflow_sha256 == [0u8; 32] {
            return Err(PolicyError::WorkflowProofInvalid(
                "all-zero workflow_sha256".into(),
            ));
        }
        let actual: [u8; 32] = Sha256::digest(document).into();
        if actual != self.workflow_sha256 {
            return Err(PolicyError::WorkflowProofInvalid(format!(
                "document digest does not match workflow_sha256 for '{}'",
                self.workflow_ref
            )));
        }
        Ok(())
    }
}

impl ActiveOverlays {
    /// The v1 default: CMMC-L3 baseline active, nothing else.
    /// Per RFC §2.1, CMMC-L3 is the baseline that applies to every
    /// deployment regardless of overlay selection.
    pub fn new_with_cmmc_baseline() -> Self {
        let mut active = BTreeSet::new();
        active.insert(Overlay::CmmcL3);
        Self {
            active,
            decommissioned: BTreeSet::new(),
        }
    }

    /// Snapshot of currently-active overlays.
    pub fn active(&self) -> &BTreeSet<Overlay> {
        &self.active
    }

    /// Snapshot of decommissioned overlays.
    pub fn decommissioned(&self) -> &BTreeSet<Overlay> {
        &self.decommissioned
    }

    /// Whether an overlay is currently active.
    pub fn is_active(&self, overlay: Overlay) -> bool {
        self.active.contains(&overlay)
    }

    /// Add an overlay. Idempotent — adding an already-active
    /// overlay is a no-op.
    ///
    /// Activation is free (no workflow required) because adding
    /// an overlay only TIGHTENS the policy — it never weakens the
    /// audit trail. The PolicyBundle update that introduces the
    /// new active set is itself dual-signed (SecurityOfficer +
    /// ComplianceOfficer), so the activation is auditable
    /// regardless.
    pub fn add(&mut self, overlay: Overlay) {
        // If the overlay was previously decommissioned, reactivating
        // it is allowed but is recorded as a fresh activation;
        // history-wise it shows up in both sets.
        self.active.insert(overlay);
    }

    /// Remove an overlay. The decommissioning workflow proof is
    /// REQUIRED — Rule of RFC §2.3.
    ///
    /// On success: overlay moves from `active` to `decommissioned`
    /// (the entry is preserved in history). On failure: returns
    /// `PolicyError::OverlayRemovalForbidden` and the state is
    /// unchanged.
    ///
    /// The harness MUST also append a "OverlayDecommissioned"
    /// AuditRecord at the same time; that's a caller concern, not
    /// this module's.
    pub fn remove_with_workflow(
        &mut self,
        overlay: Overlay,
        workflow: &DecommissioningWorkflow,
        workflow_document: &[u8],
    ) -> Result<(), PolicyError> {
        // CMMC-L3 baseline is non-removable. RFC §2.1: baseline
        // applies to every deployment regardless.
        if overlay == Overlay::CmmcL3 {
            return Err(PolicyError::OverlayRemovalForbidden(overlay));
        }
        // NA2-B-029: the decommissioning proof must actually bind to
        // the workflow document. A default/zero token is refused.
        workflow.verify(workflow_document)?;
        if !self.active.remove(&overlay) {
            // Not active — nothing to remove. Caller likely has a
            // stale bundle; treat as a no-op rather than error.
            return Ok(());
        }
        self.decommissioned.insert(overlay);
        Ok(())
    }

    /// Bare removal without a workflow — always errors with
    /// `OverlayRemovalForbidden`. Provided so callers can offer a
    /// "force remove" branch in tests without accidentally enabling
    /// it in production (the branch returns the error).
    pub fn remove_without_workflow(&mut self, overlay: Overlay) -> Result<(), PolicyError> {
        Err(PolicyError::OverlayRemovalForbidden(overlay))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORKFLOW_DOC: &[u8] = b"overlay decommissioning workflow: HIPAA export complete";

    fn workflow() -> DecommissioningWorkflow {
        use sha2::{Digest, Sha256};
        DecommissioningWorkflow {
            workflow_ref: ".agentile/sprints/active/overlay-decom-stub.md".into(),
            workflow_sha256: Sha256::digest(WORKFLOW_DOC).into(),
        }
    }

    #[test]
    fn remove_with_default_proof_is_refused() {
        // NA2-B-029 tripwire: a zero/default proof token, or a
        // document that does not hash to the recorded digest, must
        // NOT authorize removal.
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        s.add(Overlay::Ferpa);
        let zero = DecommissioningWorkflow {
            workflow_ref: String::new(),
            workflow_sha256: [0u8; 32],
        };
        assert!(matches!(
            s.remove_with_workflow(Overlay::Ferpa, &zero, b"anything"),
            Err(PolicyError::WorkflowProofInvalid(_))
        ));
        assert!(s.is_active(Overlay::Ferpa));
        // Right token, wrong document: still refused.
        assert!(matches!(
            s.remove_with_workflow(Overlay::Ferpa, &workflow(), b"tampered document"),
            Err(PolicyError::WorkflowProofInvalid(_))
        ));
        assert!(s.is_active(Overlay::Ferpa));
    }

    #[test]
    fn default_carries_cmmc_l3_baseline() {
        let s = ActiveOverlays::new_with_cmmc_baseline();
        assert!(s.is_active(Overlay::CmmcL3));
        assert!(!s.is_active(Overlay::Ferpa));
        assert!(s.decommissioned().is_empty());
    }

    #[test]
    fn add_is_idempotent() {
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        s.add(Overlay::Ferpa);
        s.add(Overlay::Ferpa);
        assert_eq!(s.active().len(), 2); // CMMC + FERPA
    }

    #[test]
    fn cmmc_l3_baseline_is_non_removable() {
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        let err = s
            .remove_with_workflow(Overlay::CmmcL3, &workflow(), WORKFLOW_DOC)
            .expect_err("CMMC baseline must be non-removable");
        match err {
            PolicyError::OverlayRemovalForbidden(o) => assert_eq!(o, Overlay::CmmcL3),
            other => panic!("expected OverlayRemovalForbidden, got {other:?}"),
        }
        assert!(s.is_active(Overlay::CmmcL3));
    }

    #[test]
    fn remove_without_workflow_always_errors() {
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        s.add(Overlay::Ferpa);
        let err = s
            .remove_without_workflow(Overlay::Ferpa)
            .expect_err("removal without workflow must error");
        match err {
            PolicyError::OverlayRemovalForbidden(o) => assert_eq!(o, Overlay::Ferpa),
            other => panic!("expected OverlayRemovalForbidden, got {other:?}"),
        }
        assert!(s.is_active(Overlay::Ferpa));
    }

    #[test]
    fn remove_with_workflow_moves_to_decommissioned() {
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        s.add(Overlay::Ferpa);
        s.add(Overlay::HipaaHitech);
        s.remove_with_workflow(Overlay::Ferpa, &workflow(), WORKFLOW_DOC)
            .expect("remove with workflow ok");
        assert!(!s.is_active(Overlay::Ferpa));
        assert!(s.decommissioned().contains(&Overlay::Ferpa));
        // The other overlays are untouched.
        assert!(s.is_active(Overlay::CmmcL3));
        assert!(s.is_active(Overlay::HipaaHitech));
    }

    #[test]
    fn remove_of_inactive_overlay_is_noop_not_error() {
        // The overlay isn't active, so there's nothing to remove.
        // Returning Ok lets the harness treat the bundle as already
        // converged rather than blocking on a stale bundle.
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        s.remove_with_workflow(Overlay::Coppa, &workflow(), WORKFLOW_DOC)
            .expect("removing inactive overlay is a no-op");
        assert!(!s.is_active(Overlay::Coppa));
        assert!(!s.decommissioned().contains(&Overlay::Coppa));
    }

    #[test]
    fn ratchet_is_documented_in_serialization() {
        // PolicyBundle CBOR round-trip pins the activation state.
        let mut s = ActiveOverlays::new_with_cmmc_baseline();
        s.add(Overlay::Ferpa);
        s.remove_with_workflow(Overlay::Ferpa, &workflow(), WORKFLOW_DOC)
            .expect("remove");
        let json = serde_json::to_string(&s).expect("ser");
        let round: ActiveOverlays = serde_json::from_str(&json).expect("de");
        assert_eq!(round, s);
        assert!(round.decommissioned().contains(&Overlay::Ferpa));
    }
}
