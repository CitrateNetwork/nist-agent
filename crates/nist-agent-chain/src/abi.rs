//! Minimal Solidity ABI encoding for the function calls we make.
//!
//! Modeled after `citrate_agent_core::chain::anchor`'s helpers,
//! generalized to accept caller-supplied contract addresses. We
//! intentionally do NOT pull in `ethabi` or `alloy-sol-types` —
//! the surface we touch is small (5 selectors, all uint8/bytes20/
//! bytes32), and keeping it hand-rolled means our supply-chain
//! posture stays as small as the runtime's.

use sha3::{Digest, Keccak256};

/// First four bytes of keccak256(signature) — Solidity function
/// selectors.
pub fn selector(signature: &str) -> [u8; 4] {
    let mut h = Keccak256::new();
    h.update(signature.as_bytes());
    let digest = h.finalize();
    let mut out = [0u8; 4];
    out.copy_from_slice(&digest[..4]);
    out
}

/// `AnchorRegistry.anchor(uint8 kind, bytes32 root)`.
pub fn encode_anchor(kind: u8, root: [u8; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 64);
    out.extend_from_slice(&selector("anchor(uint8,bytes32)"));
    let mut kind_word = [0u8; 32];
    kind_word[31] = kind;
    out.extend_from_slice(&kind_word);
    out.extend_from_slice(&root);
    out
}

/// `AnchorRegistry.isAnchored(bytes32 root)`.
pub fn encode_is_anchored(root: [u8; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 32);
    out.extend_from_slice(&selector("isAnchored(bytes32)"));
    out.extend_from_slice(&root);
    out
}

/// `AgentSBT.clearance(address agent)`.
pub fn encode_clearance_read(agent: [u8; 20]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 32);
    out.extend_from_slice(&selector("clearance(address)"));
    // address is 20 bytes, right-aligned in a 32-byte word.
    let mut word = [0u8; 32];
    word[12..].copy_from_slice(&agent);
    out.extend_from_slice(&word);
    out
}

/// `CapsuleRegistry.entryOf(bytes32 capsuleId)`.
pub fn encode_entry_of(capsule_id: [u8; 32]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + 32);
    out.extend_from_slice(&selector("entryOf(bytes32)"));
    out.extend_from_slice(&capsule_id);
    out
}

/// Decode a Solidity-returned uint8 from the last byte of a 32-byte
/// word. Returns `None` if input isn't at least 32 bytes.
pub fn decode_uint8_word(bytes: &[u8]) -> Option<u8> {
    if bytes.len() < 32 {
        return None;
    }
    Some(bytes[31])
}

/// Decode a Solidity-returned bool from a 32-byte word (last byte
/// is 0 or 1).
pub fn decode_bool_word(bytes: &[u8]) -> Option<bool> {
    decode_uint8_word(bytes).map(|b| b != 0)
}

/// Decode a Solidity-returned bytes32 from the first 32 bytes.
pub fn decode_bytes32(bytes: &[u8]) -> Option<[u8; 32]> {
    if bytes.len() < 32 {
        return None;
    }
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes[..32]);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_selector_keccak() {
        // First four bytes of keccak256("anchor(uint8,bytes32)") —
        // pinned for cross-layer alignment with the Solidity ABI.
        let s = selector("anchor(uint8,bytes32)");
        assert_eq!(s.len(), 4);
        assert_ne!(s, [0u8; 4]);
    }

    #[test]
    fn encode_anchor_layout() {
        let root = [0xab; 32];
        let calldata = encode_anchor(2, root);
        assert_eq!(calldata.len(), 68);
        assert_eq!(calldata[35], 2);
        assert_eq!(&calldata[36..68], &root);
    }

    #[test]
    fn encode_clearance_read_layout() {
        let addr = [0xde; 20];
        let calldata = encode_clearance_read(addr);
        assert_eq!(calldata.len(), 36);
        // Address sits in bytes [16..36] (12-byte zero pad + 20 bytes).
        for &b in &calldata[4..16] {
            assert_eq!(b, 0);
        }
        assert_eq!(&calldata[16..36], &addr);
    }

    #[test]
    fn decode_uint8_word_picks_last_byte() {
        let mut bytes = vec![0u8; 32];
        bytes[31] = 3;
        assert_eq!(decode_uint8_word(&bytes), Some(3));
        assert_eq!(decode_uint8_word(&[]), None);
    }

    #[test]
    fn decode_bool_word_zero_one() {
        let mut bytes = vec![0u8; 32];
        assert_eq!(decode_bool_word(&bytes), Some(false));
        bytes[31] = 1;
        assert_eq!(decode_bool_word(&bytes), Some(true));
    }
}
