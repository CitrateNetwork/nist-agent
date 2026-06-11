//! PolicyBundle startup gate — fix for NIST_AGENT-2026-05-31-001
//! (HIGH: "Daemon never loads/verifies the PolicyBundle").
//!
//! `Daemon::prepare` calls [`load_policy`] before any IPC
//! bring-up. When `policy.bundle_path` is configured, the bundle
//! MUST read, verify against the SecurityOfficer trust root
//! (`policy.so_pubkey_hex`), and activate — any failure refuses
//! startup (fail-closed). The verified bundle is held in
//! `DaemonState` so runtime surfaces gate on it. No bundle path
//! configured boots the documented minimal/air-gap smoke mode.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::VerifyingKey;
use nist_agent_policy::{PolicyBundle, RawSignedBundle};

use crate::config::DaemonConfig;
use crate::error::DaemonError;

/// Load + verify + activate the configured PolicyBundle. Returns
/// `Ok(None)` only when no bundle path is configured; every
/// failure on a configured bundle is a startup-refusing error.
pub(crate) fn load_policy(config: &DaemonConfig) -> Result<Option<PolicyBundle>, DaemonError> {
    let Some(bundle_path) = &config.policy.bundle_path else {
        return Ok(None);
    };
    let so_pubkey_hex =
        config
            .policy
            .so_pubkey_hex
            .as_deref()
            .ok_or_else(|| DaemonError::ConfigInvalid {
                field: "policy.so_pubkey_hex".into(),
                reason: "required when policy.bundle_path is set".into(),
            })?;
    let trusted = decode_so_pubkey(so_pubkey_hex)?;
    let raw = read_raw_bundle(bundle_path)?;
    let bundle = raw.verify_and_decode(&trusted).map_err(|e| {
        DaemonError::PolicyRejected(format!("verify {}: {e}", bundle_path.display()))
    })?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    bundle.activate(now, &BTreeMap::new()).map_err(|e| {
        DaemonError::PolicyRejected(format!("activate {}: {e}", bundle_path.display()))
    })?;
    Ok(Some(bundle))
}

fn decode_so_pubkey(hex_str: &str) -> Result<VerifyingKey, DaemonError> {
    let trimmed = hex_str.trim_start_matches("0x");
    let bytes = hex::decode(trimmed)
        .map_err(|e| DaemonError::PolicyRejected(format!("so_pubkey_hex decode: {e}")))?;
    let arr: [u8; 32] = bytes.as_slice().try_into().map_err(|_| {
        DaemonError::PolicyRejected(format!(
            "so_pubkey_hex: expected 32 bytes, got {}",
            bytes.len()
        ))
    })?;
    VerifyingKey::from_bytes(&arr)
        .map_err(|e| DaemonError::PolicyRejected(format!("so_pubkey_hex: {e}")))
}

/// Read the wire form the wizard emits — a JSON wrapper carrying
/// `bundle_cbor_hex` + `signature_hex` — falling back to a raw
/// CBOR-encoded `RawSignedBundle`. Anything else is refused.
fn read_raw_bundle(path: &Path) -> Result<RawSignedBundle, DaemonError> {
    let bytes = std::fs::read(path)
        .map_err(|e| DaemonError::PolicyRejected(format!("read {}: {e}", path.display())))?;
    if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
        let cbor_hex = v.get("bundle_cbor_hex").and_then(|x| x.as_str());
        let sig_hex = v.get("signature_hex").and_then(|x| x.as_str());
        if let (Some(c), Some(s)) = (cbor_hex, sig_hex) {
            let bundle_cbor = hex::decode(c)
                .map_err(|e| DaemonError::PolicyRejected(format!("bundle_cbor_hex: {e}")))?;
            let signature = hex::decode(s)
                .map_err(|e| DaemonError::PolicyRejected(format!("signature_hex: {e}")))?;
            return Ok(RawSignedBundle {
                bundle_cbor,
                signature,
            });
        }
    }
    ciborium::from_reader::<RawSignedBundle, _>(bytes.as_slice()).map_err(|e| {
        DaemonError::PolicyRejected(format!(
            "decode {}: neither wizard JSON wrapper nor CBOR RawSignedBundle: {e}",
            path.display()
        ))
    })
}
