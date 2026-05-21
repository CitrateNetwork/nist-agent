//! nist-agent-chain — chain-agnostic adapter for the agent harness.
//!
//! RFC-CIT-AGENT-0001 §3.2 names "compose as a library, deploy as a
//! daemon, embed as a WASM component" as one of the v1.0 goals. The
//! product proposition is that nist-agent works on Citrate L1 *or
//! any compatible EVM chain*. This crate is the seam: `trait
//! ChainClient` abstracts the five RFC §7.1 contracts so the rest
//! of the harness binds to the trait, not a chain.
//!
//! Per ADR-003, the trait owns the abstraction; two implementations
//! ship in v1.0:
//!
//! 1. [`citrate::CitrateChainClient`] — the default, wraps
//!    `citrate_agent_core::chain::anchor::AnchorRegistryClient` and
//!    pins all five contract addresses to their canonical Citrate
//!    Mainnet deployments (chain id 40204).
//! 2. [`evm_generic::GenericEvmChainClient`] — operator-configured;
//!    contract addresses come from the signed PolicyBundle, RPC URL
//!    from the same. Lets operators deploy nist-agent against any
//!    EVM chain that hosts compatible contract bytecode.
//!
//! Both impls share the ABI encoding helpers in
//! [`abi`] and the [`types`] module's enums for AnchorKind,
//! Clearance, and CapsuleEntry.

pub mod abi;
pub mod citrate;
pub mod error;
pub mod evm_generic;
pub mod types;

use async_trait::async_trait;

pub use error::ChainError;
pub use types::{AnchorKind, CapsuleEntry, Clearance, ContractAddresses, TxHash};

/// The chain-agnostic adapter trait.
///
/// All methods are async and `Send + Sync` so the trait works as
/// `Arc<dyn ChainClient>` inside the agent loop. Method semantics
/// match the RFC §7 contract interfaces; impls translate to their
/// chain's RPC/transaction model.
#[async_trait]
pub trait ChainClient: Send + Sync {
    /// EVM chain id this client speaks (Citrate Mainnet = 40204; any
    /// other EVM chain = its assigned id).
    fn chain_id(&self) -> u64;

    /// Submit `AnchorRegistry.anchor(kind, root)`. Returns the tx
    /// hash on success.
    async fn anchor(&self, kind: AnchorKind, root: [u8; 32]) -> Result<TxHash, ChainError>;

    /// `AnchorRegistry.isAnchored(root)` — read-only eth_call.
    async fn is_anchored(&self, root: [u8; 32]) -> Result<bool, ChainError>;

    /// `AgentSBT.clearance(agent_did)` — read-only eth_call. Returns
    /// the on-chain clearance level for the named agent identity.
    /// The data-class lattice in RFC §7.2 dominates this value at
    /// install time.
    async fn read_clearance(&self, agent_address: [u8; 20]) -> Result<Clearance, ChainError>;

    /// `CapsuleRegistry.entryOf(capsule_id)` — read-only eth_call.
    /// Returns the published capsule's manifest hash, signing tier,
    /// and endorsement signal.
    async fn read_capsule_registry(&self, capsule_id: [u8; 32])
        -> Result<CapsuleEntry, ChainError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::citrate::CitrateChainClient;
    use crate::evm_generic::GenericEvmChainClient;

    // The smoke test of the abstraction: both impls implement the
    // trait, and a function taking `&dyn ChainClient` accepts either.
    // This is the load-bearing property — if the trait shape ever
    // diverges between impls, the next test breaks at compile time.
    fn _trait_object_accepts_both(c: &dyn ChainClient) -> u64 {
        c.chain_id()
    }

    #[test]
    fn both_impls_satisfy_the_trait() {
        let citrate = CitrateChainClient::new_for_tests();
        let generic = GenericEvmChainClient::new_for_tests();
        let id_a = _trait_object_accepts_both(&citrate);
        let id_b = _trait_object_accepts_both(&generic);
        assert_eq!(id_a, 40204, "Citrate Mainnet chain id is fixed in RFC §7.1");
        assert_ne!(id_a, id_b, "Generic test fixture uses a non-Citrate id");
    }
}
