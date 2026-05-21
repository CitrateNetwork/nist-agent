//! nist-agent prelude — the public re-export surface for the
//! agent core, frozen at RFC-CIT-AGENT-0001 §3.2.
//!
//! This crate is consumed by every other crate in the nist-agent
//! workspace. It does two things:
//!
//! 1. Re-exports the `citrate_agent_core` types that the RFC names
//!    as the v1.0 frozen public surface (`ApprovalQueue`,
//!    `RecorderClient`, and the BFR-INT-12b view/tool-call types).
//! 2. Provides the `overlay` module — a thin enum capturing the
//!    NIST-compliance overlay set that the sidecar product layers
//!    on top of the agent core.
//!
//! Per
//! [`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`],
//! nist-agent does NOT fork `citrate-agent-core`. If a piece of the
//! public surface needs to change, the change lands upstream in
//! `citrate-agent-runtime` and propagates here via a manifest rev
//! bump.

pub use citrate_agent_core::{
    ApprovalOutcomePublic, ApprovalQueue, PendingView, RecorderClient, ToolCall, ToolResult,
};

/// Compliance overlays nist-agent ships at v1.0 per RFC §2.3.
///
/// Activation is a one-way ratchet within a deployment lifetime: an
/// overlay may be added by the Security Officer through a signed
/// PolicyBundle update; removal requires a documented decommissioning
/// workflow plus a final audit export. See the matching feature
/// scenarios in `features/overlays/overlay-activation-ratchet.feature`
/// and the safety property in `.agentile/formal/DataClassLattice.tla`
/// (authored in S-2).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "kebab-case")]
pub enum Overlay {
    /// NIST SP 800-171 Rev 3 + CMMC L3 baseline. Active in every
    /// deployment; the other overlays augment it.
    CmmcL3,
    /// Family Educational Rights and Privacy Act.
    Ferpa,
    /// Children's Online Privacy Protection Act.
    Coppa,
    /// Children's Internet Protection Act.
    Cipa,
    /// Health Insurance Portability and Accountability Act + HITECH.
    HipaaHitech,
    /// FedRAMP Rev 5 High baseline.
    FedrampHigh,
}

impl Overlay {
    /// All overlays shipped in v1.0. Used by the doctor pre-flight
    /// check and the Slint concierge's overlay-selector pane.
    pub const V1_SET: &'static [Overlay] = &[
        Overlay::CmmcL3,
        Overlay::Ferpa,
        Overlay::Coppa,
        Overlay::Cipa,
        Overlay::HipaaHitech,
        Overlay::FedrampHigh,
    ];

    /// Whether this overlay forbids mobile signing surfaces per
    /// RFC §5.6 (overlay-driven signing-surface eligibility).
    pub fn forbids_mobile_signing(self) -> bool {
        matches!(self, Overlay::FedrampHigh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v1_overlay_set_matches_rfc_section_2_3() {
        // RFC §2.3 enumerates exactly six v1.0 overlays.
        assert_eq!(Overlay::V1_SET.len(), 6);
        assert!(Overlay::V1_SET.contains(&Overlay::CmmcL3));
        assert!(Overlay::V1_SET.contains(&Overlay::FedrampHigh));
    }

    #[test]
    fn fedramp_high_forbids_mobile_per_rfc_section_5_6() {
        // Tested explicitly because the negative for FERPA / HIPAA
        // is what the mobile-companion sprint (S-11) will rely on.
        assert!(Overlay::FedrampHigh.forbids_mobile_signing());
        assert!(!Overlay::Ferpa.forbids_mobile_signing());
        assert!(!Overlay::HipaaHitech.forbids_mobile_signing());
        assert!(!Overlay::CmmcL3.forbids_mobile_signing());
    }

    #[test]
    fn overlay_serializes_kebab_case() {
        // PolicyBundle round-trip stability (S-6) depends on the
        // canonical form. Pin it here so a future serde derive
        // refactor can't silently break overlay activation.
        let json = serde_json::to_string(&Overlay::HipaaHitech).unwrap();
        assert_eq!(json, "\"hipaa-hitech\"");
        let round: Overlay = serde_json::from_str(&json).unwrap();
        assert_eq!(round, Overlay::HipaaHitech);
    }

    #[test]
    fn citrate_agent_core_reexports_are_constructible() {
        // Smoke test: confirms our git-pinned citrate-agent-core dep
        // is reachable. If this stops compiling, the manifest rev is
        // out of sync with the runtime's public surface.
        let _: Option<ApprovalOutcomePublic> = None;
        let _: Option<ToolCall> = None;
        let _: Option<ToolResult> = None;
        let _: Option<PendingView> = None;
    }
}
