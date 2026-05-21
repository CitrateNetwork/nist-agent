//! Wire formats exchanged between the daemon and a paired device.
//!
//! These are serde-pinned because both the daemon and the native
//! iOS / Android apps consume them. The native apps either embed
//! this crate via FFI or implement a contract-compatible decoder
//! against the JSON fixture corpus (TBD at packaging time per
//! ADR-010).

use serde::{Deserialize, Serialize};

use crate::error::MobilePairingError;
use crate::pairing::PairingId;

/// One row in the snapshot the daemon ships to the device when
/// the device polls for pending approvals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalSnapshotRow {
    pub call_id: String,
    pub call_display: String,
    pub description: String,
    pub severity: String,
    pub args_pretty: String,
    /// What role the device is being asked to sign as.
    pub required_role: String,
}

/// The full snapshot the device fetches over mTLS.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalSnapshot {
    pub pairing_id: PairingId,
    pub rows: Vec<ApprovalSnapshotRow>,
    /// ISO-8601 of when the daemon assembled this snapshot. The
    /// device shows it so the operator knows how stale the list
    /// is; the daemon does not act on stale data.
    pub snapshot_at_iso: String,
}

/// Whether the operator approved or rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SignedDecisionKind {
    Approve,
    Reject,
}

/// The signed decision the device sends back. The signature
/// itself is opaque to this module; the daemon's crypto layer
/// verifies it against the device's attestation chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedDecision {
    pub pairing_id: PairingId,
    pub call_id: String,
    pub kind: SignedDecisionKind,
    /// ISO-8601 timestamp at which the device signed. Used by
    /// `MobileEligibility::check_not_expired` to enforce the
    /// 1-hour TTL.
    pub signed_at_iso: String,
    /// Unix-time variant of `signed_at_iso` — duplicated so the
    /// daemon doesn't re-parse on the hot path.
    pub signed_at_unix: u64,
    /// Free-form reject reason. Empty for approves.
    pub reason: String,
    /// Hex-encoded device-produced signature over a deterministic
    /// digest of the rest of this struct. Format owned by the
    /// daemon's crypto layer; opaque here.
    pub signature_hex: String,
}

impl SignedDecision {
    /// Parse a JSON-encoded decision from the wire. Surfaces a
    /// stable error type so the daemon can return a 400 with a
    /// useful message.
    pub fn from_json(s: &str) -> Result<Self, MobilePairingError> {
        serde_json::from_str(s).map_err(|e| MobilePairingError::WireDecode(e.to_string()))
    }

    /// Encode for transport.
    pub fn to_json(&self) -> Result<String, MobilePairingError> {
        serde_json::to_string(self).map_err(|e| MobilePairingError::WireDecode(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap() -> ApprovalSnapshot {
        ApprovalSnapshot {
            pairing_id: PairingId::new("p1"),
            rows: vec![ApprovalSnapshotRow {
                call_id: "c1".into(),
                call_display: "redact_pii".into(),
                description: "redact PII from gradebook export".into(),
                severity: "medium".into(),
                args_pretty: "{}".into(),
                required_role: "Reviewer".into(),
            }],
            snapshot_at_iso: "2026-05-21T11:00:00Z".into(),
        }
    }

    fn dec() -> SignedDecision {
        SignedDecision {
            pairing_id: PairingId::new("p1"),
            call_id: "c1".into(),
            kind: SignedDecisionKind::Approve,
            signed_at_iso: "2026-05-21T11:05:00Z".into(),
            signed_at_unix: 1747825500,
            reason: String::new(),
            signature_hex: "deadbeef".into(),
        }
    }

    #[test]
    fn approval_snapshot_round_trips() {
        let s = snap();
        let json = serde_json::to_string(&s).unwrap();
        let r: ApprovalSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(r, s);
    }

    #[test]
    fn signed_decision_round_trips_via_helper_methods() {
        let d = dec();
        let json = d.to_json().expect("encode");
        let r = SignedDecision::from_json(&json).expect("decode");
        assert_eq!(r, d);
    }

    #[test]
    fn from_json_returns_wire_decode_error_on_garbage() {
        let err = SignedDecision::from_json("not json").unwrap_err();
        match err {
            MobilePairingError::WireDecode(msg) => assert!(!msg.is_empty()),
            other => panic!("expected WireDecode, got {other:?}"),
        }
    }

    #[test]
    fn decision_kind_serializes_kebab_case() {
        let s = serde_json::to_string(&SignedDecisionKind::Approve).unwrap();
        assert_eq!(s, "\"approve\"");
    }

    #[test]
    fn snapshot_carries_pairing_id_so_daemon_can_route() {
        let s = snap();
        assert_eq!(s.pairing_id.as_str(), "p1");
    }
}
