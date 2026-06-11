//! Adversarial tests for NIST_AGENT-2026-05-31-004 (egress
//! directive gate) and NIST_AGENT-2026-05-31-009 (divergent
//! EgressPosture enums).
//!
//! RED protocol (SECREM-02 6.1): pre-fix, `apply_directive` took
//! an injectable `impl FnOnce(..) -> bool` verifier (a caller
//! wiring `|_| true` flipped the air-gap off with no
//! cryptography) and the directive carried no nonce/expiry, so a
//! captured directive replayed forever. Pinned contract:
//!
//! 1. Signature verification happens IN-CRATE (Ed25519 over the
//!    directive's canonical payload) — no injectable verifier.
//! 2. A consumed nonce can never be replayed; nonces are strictly
//!    increasing.
//! 3. An expired directive is refused.
//! 4. The policy bundle's 3-state posture maps totally onto this
//!    crate's 2-state gate, and `BrokerOnly` NEVER maps to
//!    unrestricted direct egress.

use ed25519_dalek::{Signer, SigningKey};
use nist_agent_release::{EgressDirective, EgressPosture, ReleaseError};

fn so_key() -> SigningKey {
    SigningKey::from_bytes(&[9u8; 32])
}

fn signed_directive(nonce: u64, expires_at_unix: i64, sk: &SigningKey) -> EgressDirective {
    let mut d = EgressDirective {
        reason: "operator approved on 2026-06-11".into(),
        asserted_at_iso: "2026-06-11T10:00:00Z".into(),
        nonce,
        expires_at_unix,
        signature_hex: String::new(),
    };
    let sig = sk.sign(&d.signing_payload());
    d.signature_hex = hex::encode(sig.to_bytes());
    d
}

const NOW: i64 = 1_780_000_000; // fixed test clock
const LATER: i64 = NOW + 3600;

#[test]
fn valid_signed_directive_enables_egress_and_consumes_nonce() {
    let sk = so_key();
    let d = signed_directive(1, LATER, &sk);
    let (posture, consumed) = EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), None, NOW)
        .expect("valid directive accepted");
    assert_eq!(posture, EgressPosture::Enabled);
    assert_eq!(consumed, 1);
}

#[test]
fn garbage_signature_refused_in_crate() {
    let sk = so_key();
    let mut d = signed_directive(1, LATER, &sk);
    d.signature_hex = "deadbeef".into();
    let err = EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), None, NOW)
        .unwrap_err();
    assert_eq!(err, ReleaseError::EgressDirectiveInvalid);
}

#[test]
fn tampered_reason_refused() {
    let sk = so_key();
    let mut d = signed_directive(1, LATER, &sk);
    d.reason = "tampered after signing".into();
    assert!(EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), None, NOW)
        .is_err());
}

#[test]
fn wrong_trust_root_refused() {
    let sk = so_key();
    let other = SigningKey::from_bytes(&[13u8; 32]);
    let d = signed_directive(1, LATER, &sk);
    assert!(EgressPosture::Disabled
        .apply_directive(&d, &other.verifying_key(), None, NOW)
        .is_err());
}

#[test]
fn replayed_nonce_refused() {
    let sk = so_key();
    let d = signed_directive(5, LATER, &sk);
    let (_, consumed) = EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), None, NOW)
        .expect("first use accepted");
    // Replay the captured directive against the persisted nonce.
    let err = EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), Some(consumed), NOW)
        .unwrap_err();
    assert!(
        matches!(err, ReleaseError::EgressDirectiveReplayed { .. }),
        "captured directive must not replay: {err:?}"
    );
}

#[test]
fn stale_nonce_refused() {
    let sk = so_key();
    let d = signed_directive(3, LATER, &sk);
    assert!(EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), Some(7), NOW)
        .is_err());
}

#[test]
fn expired_directive_refused() {
    let sk = so_key();
    let d = signed_directive(1, NOW - 1, &sk);
    assert!(EgressPosture::Disabled
        .apply_directive(&d, &sk.verifying_key(), None, NOW)
        .is_err());
}

#[test]
fn disable_requires_no_directive() {
    // Fail-closed direction is always free.
    assert_eq!(EgressPosture::Enabled.disable(), EgressPosture::Disabled);
}

#[test]
fn injectable_verifier_removed_from_api() {
    // Source pin: the misuse-prone `impl FnOnce` verifier
    // injection must not return.
    let src = include_str!("../src/egress.rs");
    assert!(
        !src.contains("impl FnOnce"),
        "egress gate must verify signatures in-crate, not via injected closure"
    );
}

#[test]
fn policy_posture_maps_totally_and_broker_only_is_not_direct_egress() {
    // NIST_AGENT-2026-05-31-009: the PolicyBundle's 3-state
    // posture is the source of truth; this crate's 2-state gate
    // derives from it. BrokerOnly must NEVER collapse to
    // unrestricted direct egress.
    use nist_agent_policy::types::EgressPosture as PolicyPosture;
    assert_eq!(
        EgressPosture::from(PolicyPosture::Disabled),
        EgressPosture::Disabled
    );
    assert_eq!(
        EgressPosture::from(PolicyPosture::BrokerOnly),
        EgressPosture::Disabled,
        "broker-only means NO direct sockets; the 2-state gate must stay Disabled"
    );
    assert_eq!(
        EgressPosture::from(PolicyPosture::Allowed),
        EgressPosture::Enabled
    );
}
