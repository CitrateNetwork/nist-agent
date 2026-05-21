//! Bell-LaPadula data-class lattice — RFC §7.2.
//!
//! Five levels, totally ordered: PUBLIC < CUI < PHI < FERPA < ITAR.
//! Domination is `(self as u8) >= (other as u8)`. The TLA+
//! `DataClassLattice.tla` (S-2) verifies the invariants that
//! depend on this ordering.
//!
//! **Cross-layer alignment.** The on-chain `AgentSBT.clearance`
//! enum (encoded as Solidity `uint8`) uses the same numbering.
//! `nist-agent-chain::types::Clearance` has the same variant
//! order; the alignment is asserted by a test in this module. If
//! either enum's byte encoding ever changes, both tests fail
//! simultaneously, surfacing the breakage early.

use serde::{Deserialize, Serialize};

/// The data-class lattice from RFC §7.2. Variants are
/// `#[repr(u8)]` so `as u8` produces the exact byte encoding the
/// chain uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
#[serde(rename_all = "kebab-case")]
pub enum DataClass {
    Public = 0,
    Cui = 1,
    Phi = 2,
    Ferpa = 3,
    Itar = 4,
}

impl DataClass {
    /// All five classes in canonical (low-to-high) order.
    pub const ALL: &'static [DataClass] = &[
        DataClass::Public,
        DataClass::Cui,
        DataClass::Phi,
        DataClass::Ferpa,
        DataClass::Itar,
    ];

    /// Bell-LaPadula "no read up": `self` (the subject's clearance)
    /// dominates `other` (the object's classification) iff
    /// `(self as u8) >= (other as u8)`.
    ///
    /// Used by:
    /// - Capsule install gate (RFC §4.5, §7.2): AgentSBT.clearance
    ///   MUST dominate every entry in the capsule manifest's
    ///   `data_class.reads`.
    /// - Per-call data-class check: argument's class must be
    ///   dominated by the capsule's declared reads.
    pub fn dominates(self, other: DataClass) -> bool {
        (self as u8) >= (other as u8)
    }

    /// Decode a u8 (typically from on-chain Solidity returndata).
    /// Returns `None` for unknown values; callers convert to a
    /// typed error variant of their domain (e.g. `ChainError` in
    /// nist-agent-chain).
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Public),
            1 => Some(Self::Cui),
            2 => Some(Self::Phi),
            3 => Some(Self::Ferpa),
            4 => Some(Self::Itar),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominates_is_reflexive_and_transitive() {
        // Reflexive: every class dominates itself.
        for c in DataClass::ALL {
            assert!(c.dominates(*c));
        }
        // Transitive: a >= b and b >= c implies a >= c. Pick a few
        // representative triples.
        assert!(DataClass::Itar.dominates(DataClass::Phi));
        assert!(DataClass::Phi.dominates(DataClass::Cui));
        assert!(DataClass::Itar.dominates(DataClass::Cui));
    }

    #[test]
    fn dominates_orders_per_rfc_section_7_2() {
        // The full chain from RFC §7.2: PUBLIC < CUI < PHI < FERPA < ITAR.
        assert!(DataClass::Cui.dominates(DataClass::Public));
        assert!(DataClass::Phi.dominates(DataClass::Cui));
        assert!(DataClass::Ferpa.dominates(DataClass::Phi));
        assert!(DataClass::Itar.dominates(DataClass::Ferpa));
        // And the negations.
        assert!(!DataClass::Public.dominates(DataClass::Cui));
        assert!(!DataClass::Cui.dominates(DataClass::Itar));
    }

    #[test]
    fn from_u8_round_trip() {
        for c in DataClass::ALL {
            let byte = *c as u8;
            assert_eq!(DataClass::from_u8(byte), Some(*c));
        }
        // Unknown bytes return None.
        assert_eq!(DataClass::from_u8(5), None);
        assert_eq!(DataClass::from_u8(255), None);
    }

    #[test]
    fn cross_layer_alignment_with_chain_clearance() {
        // Cross-layer invariant: nist-agent-chain::types::Clearance
        // MUST share the byte encoding with nist-agent-policy::
        // DataClass. We can't import Clearance here (would create a
        // policy→chain dep), so we pin the byte values explicitly
        // and rely on nist-agent-chain to keep its Clearance matching
        // these. The chain crate has a mirror test pinning the same
        // bytes; if either side drifts, both fail.
        assert_eq!(DataClass::Public as u8, 0);
        assert_eq!(DataClass::Cui as u8, 1);
        assert_eq!(DataClass::Phi as u8, 2);
        assert_eq!(DataClass::Ferpa as u8, 3);
        assert_eq!(DataClass::Itar as u8, 4);
    }

    #[test]
    fn lattice_serializes_kebab_case() {
        // PolicyBundle CBOR round-trip stability — pinning the form
        // here so the bundle's signed-bytes never drift on a serde
        // refactor.
        assert_eq!(serde_json::to_string(&DataClass::Phi).unwrap(), "\"phi\"");
        assert_eq!(
            serde_json::to_string(&DataClass::Public).unwrap(),
            "\"public\""
        );
        let round: DataClass = serde_json::from_str("\"itar\"").unwrap();
        assert_eq!(round, DataClass::Itar);
    }
}
