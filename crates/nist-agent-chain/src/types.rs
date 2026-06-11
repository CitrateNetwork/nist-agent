//! Shared types for both ChainClient implementations.
//!
//! `AnchorKind` MUST stay in sync with the Solidity enum order in
//! `AnchorRegistry.sol` (citrate-chain) — verified by the
//! cross-layer test in citrate-agent-runtime's CIT-AGENT-6a sprint.
//! We re-export the runtime's enum here rather than re-define it so
//! Rule 9 ("one source of truth per topic") holds.

use serde::{Deserialize, Serialize};

pub use citrate_agent_core::audit::AnchorKind;

/// Ethereum transaction hash returned by `anchor()` writes.
pub type TxHash = String;

/// Five-level clearance lattice from RFC §7.2 (Bell-LaPadula
/// adapted). Encoded as `uint8` on `AgentSBT.clearance`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clearance {
    Public = 0,
    Cui = 1,
    Phi = 2,
    Ferpa = 3,
    Itar = 4,
}

impl Clearance {
    /// Decode a contract-returned uint8 to a Clearance variant.
    /// Returns `None` for unknown values; callers convert to
    /// `ChainError::UnknownEnumValue`.
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

    /// Whether `self` dominates `other` under the lattice (per
    /// `DataClassLattice.tla` §7.2).
    pub fn dominates(self, other: Clearance) -> bool {
        (self as u8) >= (other as u8)
    }
}

/// CapsuleRegistry entry shape — what `entryOf(capsule_id)` returns.
/// Mirrors the v1 ERC-1155 metadata RFC §7.1 names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapsuleEntry {
    /// Manifest hash (sha256, encoded as bytes32 on chain).
    pub manifest_hash: [u8; 32],
    /// Signing tier: 0=bundled, 1=managed, 2=workspace per RFC §4.4.
    pub signing_tier: SigningTier,
    /// Whether the cooperative has endorsed this capsule (RFC §4.4
    /// notes endorsement as a separate event from publish).
    pub cooperative_endorsed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningTier {
    Bundled = 0,
    Managed = 1,
    Workspace = 2,
}

impl SigningTier {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Bundled),
            1 => Some(Self::Managed),
            2 => Some(Self::Workspace),
            _ => None,
        }
    }
}

/// Operator-supplied contract addresses for the five RFC §7.1
/// contracts. The Citrate impl hard-codes mainnet addresses; the
/// generic EVM impl reads these from the operator's signed
/// PolicyBundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub organization_sbt: [u8; 20],
    pub agent_sbt: [u8; 20],
    pub capsule_registry: [u8; 20],
    pub anchor_registry: [u8; 20],
    pub benchmark_registry: [u8; 20],
}

impl ContractAddresses {
    /// Canonical Citrate Mainnet addresses (placeholder values until
    /// citrate-chain confirms the v1 deployment). When the real
    /// addresses ship, this function gets the only update — every
    /// other call site reads it.
    pub fn citrate_mainnet() -> Self {
        // TBC: real addresses come from citrate-chain's v1 deploy.
        // Filling with deterministic placeholders for now; replaced
        // when citrate-chain tags v1 contracts. While these are
        // placeholders, `is_placeholder()` returns true and the
        // production constructor refuses to consume them
        // (NIST_AGENT-2026-05-31-007 ship-guard).
        Self {
            organization_sbt: [0x01; 20],
            agent_sbt: [0x02; 20],
            capsule_registry: [0x03; 20],
            anchor_registry: [0x04; 20],
            benchmark_registry: [0x05; 20],
        }
    }

    /// True while this set is the pre-deployment placeholder
    /// pattern (`[0x01..0x05; 20]`). Ship-guard
    /// (NIST_AGENT-2026-05-31-007): production constructors
    /// refuse placeholder addresses so a pre-v1 build can never
    /// silently anchor against nonexistent or squatted contracts.
    /// Flips to false the day `citrate_mainnet()` carries the
    /// real v1 deployment.
    pub fn is_placeholder(&self) -> bool {
        self.organization_sbt == [0x01; 20]
            && self.agent_sbt == [0x02; 20]
            && self.capsule_registry == [0x03; 20]
            && self.anchor_registry == [0x04; 20]
            && self.benchmark_registry == [0x05; 20]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citrate_mainnet_is_flagged_placeholder_until_v1_deploy() {
        // NIST_AGENT-2026-05-31-007: while the canonical mainnet
        // set is the deterministic [0x01..0x05; 20] pattern, the
        // ship-guard must report it as a placeholder. When the
        // real v1 addresses land, update citrate_mainnet() and
        // this pin flips with it.
        assert!(ContractAddresses::citrate_mainnet().is_placeholder());
    }

    #[test]
    fn real_looking_addresses_are_not_flagged_placeholder() {
        let mut a = ContractAddresses::citrate_mainnet();
        a.anchor_registry = [0xAB; 20];
        assert!(!a.is_placeholder());
    }
}
