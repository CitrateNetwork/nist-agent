//! Pairing state machine + persistable record.
//!
//! Pairing a phone is itself an HIC-gated proposal per the feature
//! scenario "Pairing is HIC-gated": a SecurityOfficer must sign
//! before the pairing transitions from `SecurityOfficerSigned` to
//! `DeviceAttested` and on to `Active`. The persisted
//! `PairingRecord` is structured as an `AuditRecord` so it lands
//! in the WORM audit sink alongside every other policy decision.
//!
//! State machine:
//!
//! ```text
//! Initiated
//!    │  (SecurityOfficer signs)
//!    ▼
//! SecurityOfficerSigned
//!    │  (device presents attestation, allowlist permits)
//!    ▼
//! DeviceAttested
//!    │  (token verified)
//!    ▼
//! Active                     Rejected (terminal)
//!    │  (operator revokes)
//!    ▼
//! Revoked (terminal)
//! ```
//!
//! Transitions other than the listed ones are programmer errors
//! and refuse with `InvalidPairingTransition`.

use serde::{Deserialize, Serialize};

use crate::attestation::DeviceAttestation;
use crate::error::MobilePairingError;

/// Stable pairing id. Hex-encoded random bytes; treated opaquely.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PairingId(pub String);

impl PairingId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Short-lived pairing nonce the daemon prints to the operator
/// screen + the device captures (typically as a QR). Constant-time
/// compared at transition time.
///
/// NA2-B-027: this is a live bearer credential and `PairingRecord`
/// is written to the immutable WORM audit sink at several
/// transitions while the token is still valid. Both `Debug` and
/// `Serialize` are therefore REDACTED — the plaintext token must
/// never reach a log line or the persisted audit record. The
/// in-memory value is still available via [`PairingToken::matches`]
/// for the one comparison that needs it.
#[derive(Clone, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct PairingToken(pub String);

impl std::fmt::Debug for PairingToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PairingToken(<redacted>)")
    }
}

impl Drop for PairingToken {
    fn drop(&mut self) {
        // NA2-B-007: wipe the bearer token from memory on drop rather
        // than leaving it recoverable in a freed heap allocation.
        use zeroize::Zeroize;
        self.0.zeroize();
    }
}

impl Serialize for PairingToken {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Never emit the plaintext token into a persisted record.
        serializer.serialize_str("<redacted>")
    }
}

impl PairingToken {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    /// Constant-time-ish equality. `String` `PartialEq` is
    /// length-checked first which short-circuits, but the daemon
    /// is the only consumer and it sees one token per pairing
    /// flow, so the constant-time guarantee is principally for
    /// hygiene rather than to defeat a side channel.
    pub fn matches(&self, other: &PairingToken) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        let mut diff = 0u8;
        for (a, b) in self.0.as_bytes().iter().zip(other.0.as_bytes().iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }
}

/// Current state of a pairing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PairingState {
    Initiated,
    SecurityOfficerSigned,
    DeviceAttested,
    Active,
    Rejected,
    Revoked,
}

impl PairingState {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Initiated => "initiated",
            Self::SecurityOfficerSigned => "security-officer-signed",
            Self::DeviceAttested => "device-attested",
            Self::Active => "active",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}

/// One persisted pairing. The daemon writes this as an
/// `AuditRecord` at every state transition so the audit log
/// carries the full lifecycle of every paired device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairingRecord {
    pub pairing_id: PairingId,
    pub state: PairingState,
    /// SecurityOfficer DID that signed the pairing.
    /// `None` while state is `Initiated`.
    pub security_officer_did: Option<String>,
    /// Device attestation envelope. `None` until the device has
    /// transitioned past `SecurityOfficerSigned`.
    pub attestation: Option<DeviceAttestation>,
    /// Daemon-generated, short-lived token used to bind the
    /// device's first contact to the pairing the operator just
    /// initiated. Cleared once the device transitions to
    /// `Active` so subsequent state changes don't carry the
    /// secret.
    pub token: Option<PairingToken>,
    /// Audit-trail creation time (ISO-8601). Harness fills in.
    pub created_at_iso: String,
    /// ISO-8601 of the last transition.
    pub last_transition_at_iso: String,
    /// Unix-epoch seconds after which the token can no longer be
    /// used to activate the pairing (NA2-B-006). A stale pairing
    /// must be re-initiated rather than activated indefinitely.
    #[serde(default)]
    pub expires_at_unix: i64,
    /// Count of failed token presentations. The pairing is
    /// terminated once this reaches [`PairingRecord::MAX_TOKEN_ATTEMPTS`]
    /// (NA2-B-006), so a same-host attacker cannot retry without limit.
    #[serde(default)]
    pub failed_attempts: u32,
}

impl PairingRecord {
    /// Default pairing-token lifetime: 5 minutes (the "short-lived"
    /// window the token doc-comment promises).
    pub const DEFAULT_TTL_SECS: i64 = 300;
    /// Failed token presentations tolerated before the pairing is
    /// terminated.
    pub const MAX_TOKEN_ATTEMPTS: u32 = 5;

    /// Construct a freshly-initiated pairing. The token is
    /// daemon-generated; the caller supplies the wall-clock time so
    /// the token's expiry (`now_unix + ttl_secs`) is bound at
    /// creation (NA2-B-006). Pass `PairingRecord::DEFAULT_TTL_SECS`
    /// for the standard 5-minute window.
    pub fn initiate(
        pairing_id: PairingId,
        token: PairingToken,
        created_at_iso: impl Into<String>,
        now_unix: i64,
        ttl_secs: i64,
    ) -> Self {
        let ts: String = created_at_iso.into();
        Self {
            pairing_id,
            state: PairingState::Initiated,
            security_officer_did: None,
            attestation: None,
            token: Some(token),
            created_at_iso: ts.clone(),
            last_transition_at_iso: ts,
            expires_at_unix: now_unix.saturating_add(ttl_secs.max(0)),
            failed_attempts: 0,
        }
    }

    /// Apply a SecurityOfficer signature. Refuses unless the
    /// current state is `Initiated`.
    pub fn apply_security_officer(
        &mut self,
        security_officer_did: impl Into<String>,
        now_iso: impl Into<String>,
    ) -> Result<(), MobilePairingError> {
        if self.state != PairingState::Initiated {
            return Err(MobilePairingError::InvalidPairingTransition {
                expected: "initiated",
                observed: self.state.as_str(),
            });
        }
        self.security_officer_did = Some(security_officer_did.into());
        self.state = PairingState::SecurityOfficerSigned;
        self.last_transition_at_iso = now_iso.into();
        Ok(())
    }

    /// Attach a device attestation once it has been verified.
    /// Refuses unless the current state is `SecurityOfficerSigned`.
    ///
    /// NA2-B-005: the envelope is no longer stored on the device's
    /// say-so. It is routed through `verifier`, which must approve
    /// it before the state machine advances. The workspace default
    /// ([`crate::attestation::DenyingChainVerifier`]) fails closed.
    pub fn apply_attestation(
        &mut self,
        attestation: DeviceAttestation,
        verifier: &dyn crate::attestation::AttestationChainVerifier,
        now_iso: impl Into<String>,
    ) -> Result<(), MobilePairingError> {
        if self.state != PairingState::SecurityOfficerSigned {
            return Err(MobilePairingError::InvalidPairingTransition {
                expected: "security-officer-signed",
                observed: self.state.as_str(),
            });
        }
        verifier.verify(&attestation)?;
        self.attestation = Some(attestation);
        self.state = PairingState::DeviceAttested;
        self.last_transition_at_iso = now_iso.into();
        Ok(())
    }

    /// Activate the pairing once the device presents a matching
    /// token. Clears the token from the record so it isn't
    /// persisted with subsequent transitions.
    pub fn activate(
        &mut self,
        presented_token: &PairingToken,
        now_iso: impl Into<String>,
        now_unix: i64,
    ) -> Result<(), MobilePairingError> {
        if self.state != PairingState::DeviceAttested {
            return Err(MobilePairingError::InvalidPairingTransition {
                expected: "device-attested",
                observed: self.state.as_str(),
            });
        }
        // NA2-B-006: refuse an expired token window. A "short-lived"
        // token that never expires is a standing bearer credential.
        if now_unix > self.expires_at_unix {
            self.state = PairingState::Rejected;
            self.token = None;
            self.last_transition_at_iso = now_iso.into();
            return Err(MobilePairingError::PairingExpired {
                expires_at_unix: self.expires_at_unix,
                now_unix,
            });
        }
        let token = self
            .token
            .as_ref()
            .ok_or(MobilePairingError::PairingTokenMismatch)?;
        if !token.matches(presented_token) {
            // NA2-B-006: count failures and terminate the pairing
            // after too many, rather than allowing unlimited retries.
            self.failed_attempts += 1;
            self.last_transition_at_iso = now_iso.into();
            if self.failed_attempts >= Self::MAX_TOKEN_ATTEMPTS {
                self.state = PairingState::Rejected;
                self.token = None;
                return Err(MobilePairingError::PairingTooManyAttempts {
                    attempts: self.failed_attempts,
                });
            }
            return Err(MobilePairingError::PairingTokenMismatch);
        }
        self.token = None;
        self.state = PairingState::Active;
        self.last_transition_at_iso = now_iso.into();
        Ok(())
    }

    /// Reject the pairing. Allowed from any pre-terminal state.
    pub fn reject(&mut self, now_iso: impl Into<String>) -> Result<(), MobilePairingError> {
        match self.state {
            PairingState::Initiated
            | PairingState::SecurityOfficerSigned
            | PairingState::DeviceAttested => {
                self.state = PairingState::Rejected;
                self.token = None;
                self.last_transition_at_iso = now_iso.into();
                Ok(())
            }
            _ => Err(MobilePairingError::InvalidPairingTransition {
                expected: "pre-terminal state",
                observed: self.state.as_str(),
            }),
        }
    }

    /// Revoke an active pairing. Operator-initiated; terminal.
    pub fn revoke(&mut self, now_iso: impl Into<String>) -> Result<(), MobilePairingError> {
        if self.state != PairingState::Active {
            return Err(MobilePairingError::InvalidPairingTransition {
                expected: "active",
                observed: self.state.as_str(),
            });
        }
        self.state = PairingState::Revoked;
        self.last_transition_at_iso = now_iso.into();
        Ok(())
    }

    /// True iff the pairing is in a state where the device may
    /// sign approvals.
    pub fn is_signable(&self) -> bool {
        self.state == PairingState::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attestation::{AttestationAllowlist, AttestationChainId, DeviceVendor};

    fn att() -> DeviceAttestation {
        DeviceAttestation {
            chain_id: AttestationChainId::new("apple-sep"),
            vendor: DeviceVendor::Apple,
            // A non-degenerate (≥32-byte) envelope so the allowlist
            // verifier accepts it in the happy path.
            chain_bytes_hex: "ab".repeat(48),
            display_label: "iPhone 17 Pro".into(),
        }
    }

    /// Allowlist-backed verifier used by the state-machine tests.
    /// `apple-sep` is allowlisted by default.
    fn verifier() -> AttestationAllowlist {
        AttestationAllowlist::v1_default()
    }

    fn fresh() -> PairingRecord {
        // Token valid from t=1000 for the default 5-minute window
        // (expires at 1300).
        PairingRecord::initiate(
            PairingId::new("p1"),
            PairingToken::new("abcd-1234"),
            "2026-05-21T10:00:00Z",
            1_000,
            PairingRecord::DEFAULT_TTL_SECS,
        )
    }

    #[test]
    fn happy_path_walks_full_state_machine() {
        let mut r = fresh();
        r.apply_security_officer("did:role:sec-officer", "2026-05-21T10:01:00Z")
            .unwrap();
        assert_eq!(r.state, PairingState::SecurityOfficerSigned);
        r.apply_attestation(att(), &verifier(), "2026-05-21T10:02:00Z")
            .unwrap();
        assert_eq!(r.state, PairingState::DeviceAttested);
        assert!(r.attestation.is_some());
        r.activate(
            &PairingToken::new("abcd-1234"),
            "2026-05-21T10:03:00Z",
            1_100,
        )
        .unwrap();
        assert_eq!(r.state, PairingState::Active);
        assert!(r.is_signable());
        // Token is cleared once active to avoid leaking it on
        // subsequent transitions.
        assert!(r.token.is_none());
    }

    #[test]
    fn out_of_order_security_officer_is_refused() {
        let mut r = fresh();
        r.apply_security_officer("did:role:sec-officer", "t1")
            .unwrap();
        // Cannot apply again.
        let err = r
            .apply_security_officer("did:role:sec-officer", "t2")
            .unwrap_err();
        match err {
            MobilePairingError::InvalidPairingTransition { expected, observed } => {
                assert_eq!(expected, "initiated");
                assert_eq!(observed, "security-officer-signed");
            }
            other => panic!("expected InvalidPairingTransition, got {other:?}"),
        }
    }

    #[test]
    fn attestation_before_security_officer_is_refused() {
        let mut r = fresh();
        let err = r.apply_attestation(att(), &verifier(), "t1").unwrap_err();
        assert!(matches!(
            err,
            MobilePairingError::InvalidPairingTransition { .. }
        ));
    }

    #[test]
    fn activate_with_wrong_token_is_refused() {
        let mut r = fresh();
        r.apply_security_officer("did:x", "t1").unwrap();
        r.apply_attestation(att(), &verifier(), "t2").unwrap();
        let err = r
            .activate(&PairingToken::new("not-the-token"), "t3", 1_100)
            .unwrap_err();
        assert_eq!(err, MobilePairingError::PairingTokenMismatch);
        // State is unchanged and a bounded retry is still allowed, but
        // the failure is now COUNTED (NA2-B-006) — retries are no
        // longer unlimited.
        assert_eq!(r.state, PairingState::DeviceAttested);
        assert_eq!(r.failed_attempts, 1);
        // A subsequent presentation of the right token within the
        // attempt budget still works.
        r.activate(&PairingToken::new("abcd-1234"), "t4", 1_100)
            .unwrap();
        assert_eq!(r.state, PairingState::Active);
    }

    #[test]
    fn activate_is_terminated_after_max_attempts() {
        // NA2-B-006 tripwire: unlimited wrong-token retries are no
        // longer allowed — the pairing self-terminates.
        let mut r = fresh();
        r.apply_security_officer("did:x", "t1").unwrap();
        r.apply_attestation(att(), &verifier(), "t2").unwrap();
        let mut last = MobilePairingError::PairingTokenMismatch;
        for _ in 0..PairingRecord::MAX_TOKEN_ATTEMPTS {
            last = r
                .activate(&PairingToken::new("wrong"), "t", 1_100)
                .unwrap_err();
        }
        assert!(
            matches!(last, MobilePairingError::PairingTooManyAttempts { .. }),
            "got {last:?}"
        );
        assert_eq!(r.state, PairingState::Rejected);
        assert!(r.token.is_none());
        // Even the correct token cannot revive a terminated pairing.
        let err = r
            .activate(&PairingToken::new("abcd-1234"), "t", 1_100)
            .unwrap_err();
        assert!(matches!(
            err,
            MobilePairingError::InvalidPairingTransition { .. }
        ));
    }

    #[test]
    fn activate_after_expiry_is_refused() {
        // NA2-B-006 tripwire: a token cannot be activated past its
        // TTL (created at 1000, +300s = expires 1300).
        let mut r = fresh();
        r.apply_security_officer("did:x", "t1").unwrap();
        r.apply_attestation(att(), &verifier(), "t2").unwrap();
        let err = r
            .activate(&PairingToken::new("abcd-1234"), "t3", 1_301)
            .unwrap_err();
        assert!(
            matches!(err, MobilePairingError::PairingExpired { .. }),
            "got {err:?}"
        );
        assert_eq!(r.state, PairingState::Rejected);
        assert!(r.token.is_none());
    }

    #[test]
    fn token_is_redacted_in_serialization_and_debug() {
        // NA2-B-027 tripwire: the live token must never reach the
        // persisted audit record or a debug log line.
        let r = fresh();
        let json = serde_json::to_string(&r).expect("serialize record");
        assert!(
            !json.contains("abcd-1234"),
            "plaintext token leaked into serialized record: {json}"
        );
        assert!(json.contains("<redacted>"));
        let dbg = format!("{r:?}");
        assert!(!dbg.contains("abcd-1234"), "token leaked into Debug: {dbg}");
    }

    #[test]
    fn reject_works_from_any_pre_terminal_state() {
        for setup in 0u8..3 {
            let mut r = fresh();
            if setup >= 1 {
                r.apply_security_officer("did:x", "t").unwrap();
            }
            if setup >= 2 {
                r.apply_attestation(att(), &verifier(), "t").unwrap();
            }
            r.reject("trj")
                .expect("reject from pre-terminal state should succeed");
            assert_eq!(r.state, PairingState::Rejected);
            assert!(r.token.is_none(), "token cleared on reject");
        }
    }

    #[test]
    fn reject_from_active_is_refused() {
        let mut r = fresh();
        r.apply_security_officer("did:x", "t1").unwrap();
        r.apply_attestation(att(), &verifier(), "t2").unwrap();
        r.activate(&PairingToken::new("abcd-1234"), "t3", 1_100)
            .unwrap();
        let err = r.reject("t4").unwrap_err();
        assert!(matches!(
            err,
            MobilePairingError::InvalidPairingTransition { .. }
        ));
    }

    #[test]
    fn revoke_active_is_terminal() {
        let mut r = fresh();
        r.apply_security_officer("did:x", "t1").unwrap();
        r.apply_attestation(att(), &verifier(), "t2").unwrap();
        r.activate(&PairingToken::new("abcd-1234"), "t3", 1_100)
            .unwrap();
        r.revoke("t4").unwrap();
        assert_eq!(r.state, PairingState::Revoked);
        assert!(!r.is_signable());
        // Revoke from revoked is refused.
        assert!(r.revoke("t5").is_err());
    }

    #[test]
    fn token_constant_time_compare_handles_unequal_lengths() {
        let a = PairingToken::new("abc");
        let b = PairingToken::new("abcd");
        assert!(!a.matches(&b));
        let c = PairingToken::new("abcd");
        assert!(b.matches(&c));
    }

    #[test]
    fn state_serializes_kebab_case() {
        // PairingRecord is persisted as an AuditRecord; the wire
        // form is load-bearing for audit compatibility. Pin it.
        let s = serde_json::to_string(&PairingState::DeviceAttested).unwrap();
        assert_eq!(s, "\"device-attested\"");
        let s = serde_json::to_string(&PairingState::SecurityOfficerSigned).unwrap();
        assert_eq!(s, "\"security-officer-signed\"");
    }
}
