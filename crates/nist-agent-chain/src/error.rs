//! `ChainError` — typed error for chain adapter operations.
//!
//! Per Rule 1 (no unwraps, no stubs in production paths), every
//! method on `ChainClient` returns `Result<_, ChainError>`. The
//! variants below cover the failure modes both implementations can
//! produce; consumers can match on them or fall through to the
//! `Display` rendering.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    /// RPC-level transport error (timeout, network unreachable,
    /// malformed response, etc.).
    #[error("rpc: {0}")]
    Rpc(String),

    /// The configured RPC returned data that doesn't match the
    /// expected ABI shape (e.g. eth_call returned empty bytes for a
    /// method that should return a 32-byte word).
    #[error("abi decode: {0}")]
    AbiDecode(String),

    /// Transaction signing failed (bad key, missing nonce, etc.).
    #[error("sign: {0}")]
    Sign(String),

    /// The configured contract address is the zero address — almost
    /// certainly a misconfiguration. Surfaced at first call rather
    /// than silently sending to 0x0.
    #[error("zero contract address for {0}")]
    ZeroAddress(&'static str),

    /// The on-chain value doesn't decode to a known enum variant
    /// (e.g. AgentSBT.clearance returns a uint8 that isn't 0..=4).
    /// Usually a sign the contract has been upgraded past nist-agent's
    /// version pin.
    #[error("unknown enum value {1} for {0}")]
    UnknownEnumValue(&'static str, u8),
}
