//! CitrateChainClient — default ChainClient impl pinned to Citrate
//! Mainnet (chain id 40204) and the five RFC §7.1 contracts.
//!
//! Wraps `citrate_agent_core::chain::anchor::AnchorRegistryClient`
//! for the anchor + is_anchored methods. The clearance + capsule-
//! registry reads use the same `citrate_wallet_core::chain::RpcClient`
//! the underlying client uses, so we share the connection
//! abstraction with the runtime's existing test posture.

use crate::abi;
use crate::error::ChainError;
use crate::types::{AnchorKind, CapsuleEntry, Clearance, ContractAddresses, SigningTier, TxHash};
use crate::ChainClient;
use async_trait::async_trait;
use citrate_agent_core::chain::AnchorRegistryClient;
use citrate_wallet_core::chain::RpcClient;

pub struct CitrateChainClient {
    inner: AnchorRegistryClient,
    rpc_url: String,
    addresses: ContractAddresses,
}

impl CitrateChainClient {
    /// Build for production use. `hex_key` is the operator's signing
    /// key for `anchor()` writes; `rpc_url` points at a Citrate node.
    /// Contract addresses come from `ContractAddresses::citrate_mainnet()`.
    pub fn from_hex_key(hex_key: &str, rpc_url: impl Into<String>) -> Option<Self> {
        let rpc_url = rpc_url.into();
        let addresses = ContractAddresses::citrate_mainnet();
        let inner = AnchorRegistryClient::from_hex_key(
            hex_key,
            rpc_url.clone(),
            hex::encode(addresses.anchor_registry),
            40204,
        )?;
        Some(Self {
            inner,
            rpc_url,
            addresses,
        })
    }

    /// Test fixture — no real RPC, but the trait shape is exercised.
    /// Used by the trait-object smoke test in lib.rs.
    #[cfg(test)]
    pub(crate) fn new_for_tests() -> Self {
        // Deterministic 32-byte test key (also used in the runtime's
        // anchor.rs tests).
        let hex_key = "0x1feffc85883856c384f497cf057d38da863eb9b89c545e72fbfd35631eaf4a58";
        Self::from_hex_key(hex_key, "http://localhost:8545").expect("test fixture key is valid")
    }
}

#[async_trait]
impl ChainClient for CitrateChainClient {
    fn chain_id(&self) -> u64 {
        40204
    }

    async fn anchor(&self, kind: AnchorKind, root: [u8; 32]) -> Result<TxHash, ChainError> {
        self.inner
            .anchor(kind, root)
            .await
            .map_err(|e| ChainError::Rpc(format!("anchor: {e}")))
    }

    async fn is_anchored(&self, root: [u8; 32]) -> Result<bool, ChainError> {
        self.inner
            .is_anchored(root)
            .await
            .map_err(|e| ChainError::Rpc(format!("is_anchored: {e}")))
    }

    async fn read_clearance(&self, agent_address: [u8; 20]) -> Result<Clearance, ChainError> {
        if self.addresses.agent_sbt == [0u8; 20] {
            return Err(ChainError::ZeroAddress("AgentSBT"));
        }
        let calldata = abi::encode_clearance_read(agent_address);
        let result = RpcClient::new(&self.rpc_url)
            .eth_call(&hex::encode(self.addresses.agent_sbt), &calldata)
            .await
            .map_err(|e| ChainError::Rpc(format!("clearance eth_call: {e}")))?;
        let raw = abi::decode_uint8_word(&result)
            .ok_or_else(|| ChainError::AbiDecode("clearance return too short".into()))?;
        Clearance::from_u8(raw).ok_or(ChainError::UnknownEnumValue("Clearance", raw))
    }

    async fn read_capsule_registry(
        &self,
        capsule_id: [u8; 32],
    ) -> Result<CapsuleEntry, ChainError> {
        if self.addresses.capsule_registry == [0u8; 20] {
            return Err(ChainError::ZeroAddress("CapsuleRegistry"));
        }
        let calldata = abi::encode_entry_of(capsule_id);
        let result = RpcClient::new(&self.rpc_url)
            .eth_call(&hex::encode(self.addresses.capsule_registry), &calldata)
            .await
            .map_err(|e| ChainError::Rpc(format!("capsule_registry eth_call: {e}")))?;
        // The contract returns (bytes32 manifest_hash, uint8 signing_tier, bool endorsed)
        // as a single tuple. Static-size tuples are encoded inline:
        //   [0..32]   = manifest_hash
        //   [32..64]  = signing_tier word (last byte)
        //   [64..96]  = endorsed word (last byte)
        if result.len() < 96 {
            return Err(ChainError::AbiDecode(format!(
                "capsule entry too short: {} bytes",
                result.len()
            )));
        }
        let manifest_hash = abi::decode_bytes32(&result[..32])
            .ok_or_else(|| ChainError::AbiDecode("manifest_hash".into()))?;
        let tier_byte = abi::decode_uint8_word(&result[32..64])
            .ok_or_else(|| ChainError::AbiDecode("signing_tier".into()))?;
        let signing_tier = SigningTier::from_u8(tier_byte)
            .ok_or(ChainError::UnknownEnumValue("SigningTier", tier_byte))?;
        let endorsed = abi::decode_bool_word(&result[64..96])
            .ok_or_else(|| ChainError::AbiDecode("endorsed".into()))?;
        Ok(CapsuleEntry {
            manifest_hash,
            signing_tier,
            cooperative_endorsed: endorsed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_is_citrate_mainnet() {
        let c = CitrateChainClient::new_for_tests();
        assert_eq!(c.chain_id(), 40204);
    }

    #[test]
    fn refuses_invalid_hex_key() {
        assert!(CitrateChainClient::from_hex_key("not-hex", "http://localhost:8545").is_none());
    }
}
