//! GenericEvmChainClient — operator-configured ChainClient.
//!
//! Shares the ABI encoding and RPC-call machinery with the
//! [`crate::citrate::CitrateChainClient`] but reads chain id and
//! all five contract addresses from operator configuration. The
//! operator's signed PolicyBundle is the source of truth for this
//! configuration in production (RFC §3.1); the constructor here
//! takes the values directly so the trait can be wired in tests and
//! by alternative bundle implementations.

use crate::abi;
use crate::error::ChainError;
use crate::types::{AnchorKind, CapsuleEntry, Clearance, ContractAddresses, SigningTier, TxHash};
use crate::ChainClient;
use async_trait::async_trait;
use citrate_wallet_core::chain::{RpcClient, TransactionBuilder};
use k256::ecdsa::SigningKey;

pub struct GenericEvmChainClient {
    signing_key: SigningKey,
    from_address_hex: String,
    rpc_url: String,
    chain_id: u64,
    addresses: ContractAddresses,
}

impl GenericEvmChainClient {
    /// Build from an operator-supplied private key + RPC URL +
    /// chain id + full contract address set.
    pub fn new(
        hex_key: &str,
        rpc_url: impl Into<String>,
        chain_id: u64,
        addresses: ContractAddresses,
    ) -> Option<Self> {
        let stripped = hex_key.trim().trim_start_matches("0x");
        let bytes = hex::decode(stripped).ok()?;
        if bytes.len() != 32 {
            tracing::warn!(
                "GenericEvmChainClient key must be 32 bytes; got {}",
                bytes.len()
            );
            return None;
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        let signing_key = SigningKey::from_bytes(&arr.into()).ok()?;
        let from_address_hex = derive_address_hex(&signing_key);
        Some(Self {
            signing_key,
            from_address_hex,
            rpc_url: rpc_url.into(),
            chain_id,
            addresses,
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_tests() -> Self {
        let hex_key = "0x1feffc85883856c384f497cf057d38da863eb9b89c545e72fbfd35631eaf4a58";
        // Non-Citrate chain id so the trait-object test can
        // distinguish.
        Self::new(
            hex_key,
            "http://localhost:8545",
            31337,
            ContractAddresses {
                organization_sbt: [0x11; 20],
                agent_sbt: [0x22; 20],
                capsule_registry: [0x33; 20],
                anchor_registry: [0x44; 20],
                benchmark_registry: [0x55; 20],
            },
        )
        .expect("test fixture key is valid")
    }
}

#[async_trait]
impl ChainClient for GenericEvmChainClient {
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    async fn anchor(&self, kind: AnchorKind, root: [u8; 32]) -> Result<TxHash, ChainError> {
        if self.addresses.anchor_registry == [0u8; 20] {
            return Err(ChainError::ZeroAddress("AnchorRegistry"));
        }
        let kind_byte = anchor_kind_byte(kind);
        let calldata = abi::encode_anchor(kind_byte, root);
        let rpc = RpcClient::new(&self.rpc_url);
        let nonce = rpc
            .get_nonce(&self.from_address_hex)
            .await
            .map_err(|e| ChainError::Rpc(format!("nonce: {e}")))?;
        let signed = TransactionBuilder::new()
            .to(&hex::encode(self.addresses.anchor_registry))
            .data(calldata)
            .nonce(nonce)
            .gas_limit(200_000)
            .chain_id(self.chain_id)
            .sign_secp256k1(&self.signing_key, nonce)
            .map_err(|e| ChainError::Sign(e.to_string()))?;
        rpc.send_raw_transaction(&signed.raw)
            .await
            .map_err(|e| ChainError::Rpc(format!("send_raw_transaction: {e}")))
    }

    async fn is_anchored(&self, root: [u8; 32]) -> Result<bool, ChainError> {
        if self.addresses.anchor_registry == [0u8; 20] {
            return Err(ChainError::ZeroAddress("AnchorRegistry"));
        }
        let calldata = abi::encode_is_anchored(root);
        let result = RpcClient::new(&self.rpc_url)
            .eth_call(&hex::encode(self.addresses.anchor_registry), &calldata)
            .await
            .map_err(|e| ChainError::Rpc(format!("is_anchored: {e}")))?;
        abi::decode_bool_word(&result)
            .ok_or_else(|| ChainError::AbiDecode("is_anchored return too short".into()))
    }

    async fn read_clearance(&self, agent_address: [u8; 20]) -> Result<Clearance, ChainError> {
        if self.addresses.agent_sbt == [0u8; 20] {
            return Err(ChainError::ZeroAddress("AgentSBT"));
        }
        let calldata = abi::encode_clearance_read(agent_address);
        let result = RpcClient::new(&self.rpc_url)
            .eth_call(&hex::encode(self.addresses.agent_sbt), &calldata)
            .await
            .map_err(|e| ChainError::Rpc(format!("clearance: {e}")))?;
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
            .map_err(|e| ChainError::Rpc(format!("capsule_registry: {e}")))?;
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

fn anchor_kind_byte(kind: AnchorKind) -> u8 {
    // The mapping mirrors citrate_agent_core::chain::anchor —
    // the Solidity enum order is the source of truth.
    match kind {
        AnchorKind::PerCapsule => 0,
        AnchorKind::PerApproval => 1,
        AnchorKind::NightlyMerkle => 2,
    }
}

/// Derive the 20-byte EVM address from a secp256k1 signing key,
/// hex-encoded with no 0x prefix. Matches the runtime's
/// `audit::recorder::derive_address` shape so the same fixture key
/// produces the same address across crates.
fn derive_address_hex(signing_key: &SigningKey) -> String {
    use sha3::{Digest, Keccak256};
    let verifying = signing_key.verifying_key();
    // Uncompressed public-key bytes minus the 0x04 prefix.
    let point = verifying.to_encoded_point(false);
    let pubkey = &point.as_bytes()[1..];
    let mut h = Keccak256::new();
    h.update(pubkey);
    let digest = h.finalize();
    hex::encode(&digest[12..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_is_configurable() {
        let c = GenericEvmChainClient::new_for_tests();
        assert_eq!(c.chain_id(), 31337);
    }

    #[test]
    fn refuses_invalid_hex_key() {
        let addresses = ContractAddresses::citrate_mainnet();
        assert!(
            GenericEvmChainClient::new("not-hex", "http://localhost:8545", 1, addresses).is_none()
        );
    }

    #[test]
    fn derive_address_matches_runtime_fixture() {
        // The runtime's anchor.rs test fixture maps
        // 0x1feffc...4a58 → 0x4250675f9015e65fc866f3a373f82bb9dfc000c6.
        // Our derive_address_hex MUST agree.
        let hex_key = "0x1feffc85883856c384f497cf057d38da863eb9b89c545e72fbfd35631eaf4a58";
        let bytes = hex::decode(hex_key.trim_start_matches("0x")).unwrap();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&bytes);
        let signing_key = SigningKey::from_bytes(&arr.into()).unwrap();
        assert_eq!(
            derive_address_hex(&signing_key).to_lowercase(),
            "4250675f9015e65fc866f3a373f82bb9dfc000c6"
        );
    }
}
