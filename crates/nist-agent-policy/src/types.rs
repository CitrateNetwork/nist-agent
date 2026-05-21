//! Core enums for the PolicyBundle.
//!
//! `RiskTier` and `Role` are the small finite alphabets the bundle
//! draws from. `AnchorStrategy` and `EgressPosture` encode the
//! operator's choices for RFC §6.3 (anchor strategy) and §3.3
//! (network posture).

use serde::{Deserialize, Serialize};

/// RFC §5.2 — every action is assigned a risk tier that determines
/// the quorum size and the roles required to sign. Tiers form a
/// total order: Low < Medium < High < Critical. The policy bundle
/// MAY escalate (override a capsule's declared tier upward) but
/// never de-escalate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskTier {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}

/// RFC §5.3 — the five-role lattice. The harness MUST refuse to
/// operate without all five base roles assigned (PolicyBundle
/// activation enforces this via `PolicyError::RoleUnassigned`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    Operator,
    Reviewer,
    ComplianceOfficer,
    SecurityOfficer,
    Auditor,
}

impl Role {
    /// All five base roles in their canonical order. Used by
    /// `PolicyBundle::activate` to check the role-assignment map
    /// has every key.
    pub const ALL: &'static [Role] = &[
        Role::Operator,
        Role::Reviewer,
        Role::ComplianceOfficer,
        Role::SecurityOfficer,
        Role::Auditor,
    ];
}

/// RFC §6.3 — operator-chosen anchor strategy. The harness picks
/// one (or a hybrid); the bundle records the choice. Tier-high
/// deployments typically pick `HybridNightlyPlusCapsule`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnchorStrategy {
    /// Strategy A — on-chain events for capsule install +
    /// run-receipt only. No per-action chatter.
    PerCapsule,
    /// Strategy B — every approval emits its own on-chain event.
    PerApproval,
    /// Strategy C — daily Merkle root of the day's audit-log entries
    /// as a single on-chain event.
    NightlyMerkle,
    /// Default for FedRAMP / CMMC deployments — nightly Merkle root
    /// for baseline traffic plus per-capsule install event for
    /// tier-high actions.
    HybridNightlyPlusCapsule,
}

/// RFC §3.3 — operator-chosen network posture. The default at
/// first run is `Disabled` (air-gap). Operators in connected
/// environments configure their way out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EgressPosture {
    /// No outbound network connectivity. Capsules with
    /// `network = "none"` only.
    Disabled,
    /// All outbound traffic must traverse the configured egress
    /// broker; direct sockets are forbidden.
    BrokerOnly,
    /// Direct egress permitted per capsule manifest declarations.
    /// Requires a signed Security Officer policy directive
    /// (validated at bundle parse time, not at this enum level).
    Allowed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_tier_orders_total() {
        assert!(RiskTier::Low < RiskTier::Medium);
        assert!(RiskTier::Medium < RiskTier::High);
        assert!(RiskTier::High < RiskTier::Critical);
        // Round-trip ordering with explicit u8 encoding.
        assert!((RiskTier::Low as u8) < (RiskTier::Critical as u8));
    }

    #[test]
    fn role_all_contains_each_base_role_once() {
        assert_eq!(Role::ALL.len(), 5);
        for r in [
            Role::Operator,
            Role::Reviewer,
            Role::ComplianceOfficer,
            Role::SecurityOfficer,
            Role::Auditor,
        ] {
            assert_eq!(Role::ALL.iter().filter(|x| **x == r).count(), 1);
        }
    }

    #[test]
    fn enums_serialize_kebab_case() {
        // PolicyBundle round-trip stability depends on the canonical
        // form. Pin the kebab-case rendering here so a future serde
        // derive refactor can't silently break bundle parsing.
        let s = serde_json::to_string(&Role::ComplianceOfficer).unwrap();
        assert_eq!(s, "\"compliance-officer\"");
        let s = serde_json::to_string(&AnchorStrategy::HybridNightlyPlusCapsule).unwrap();
        assert_eq!(s, "\"hybrid-nightly-plus-capsule\"");
        let s = serde_json::to_string(&EgressPosture::BrokerOnly).unwrap();
        assert_eq!(s, "\"broker-only\"");
    }
}
