//! Adversarial tests for NIST_AGENT-2026-05-31-001 (HIGH):
//! "Daemon never loads/verifies the PolicyBundle and never
//! enforces egress posture at runtime."
//!
//! RED protocol (SECREM-02 6.1): these tests reproduce the
//! finding against the pre-fix daemon — a daemon configured with
//! a policy bundle path booted without ever opening, verifying,
//! or activating the bundle. Fail-closed contract pinned here:
//!
//! 1. `bundle_path` set + file missing      → refuse to start.
//! 2. `bundle_path` set + untrusted signer  → refuse to start.
//! 3. `bundle_path` set + garbage pubkey    → refuse to start.
//! 4. `bundle_path` set + valid bundle      → start, and hold the
//!    verified bundle in `DaemonState` for runtime gating.

use ed25519_dalek::{Signer, SigningKey};
use nist_agent_daemon::{Daemon, DaemonConfig};
use nist_agent_policy::{PolicyBundle, RawSignedBundle};
use std::path::Path;

fn signing_key(seed: u8) -> SigningKey {
    SigningKey::from_bytes(&[seed; 32])
}

/// Write a signed bundle in the wizard's wire form (JSON wrapper
/// over canonical CBOR + detached signature) — the shape
/// `citrate-agent wizard` emits and operators point
/// `policy.bundle_path` at.
fn write_signed_bundle(path: &Path, sk: &SigningKey) {
    // NA2-B-028: `minimal_template()` is not directly deployable
    // (placeholder DIDs + infinite expiry), so a bundle the daemon
    // will actually accept must rotate the DIDs and carry a finite
    // validity window.
    let mut bundle = PolicyBundle::minimal_template();
    for (role, dids) in bundle.role_assignments.iter_mut() {
        *dids = vec![format!("did:citrate:{:?}:0xabc", role)];
    }
    bundle.not_before = 0;
    bundle.expires_at = 4_102_444_800; // 2100-01-01, finite
    let canonical = bundle.encode_canonical().expect("encode");
    let signature = sk.sign(&canonical).to_bytes().to_vec();
    let raw = RawSignedBundle {
        bundle_cbor: canonical,
        signature,
    };
    let json = serde_json::json!({
        "bundle_cbor_hex": hex::encode(&raw.bundle_cbor),
        "signature_hex": hex::encode(&raw.signature),
    });
    std::fs::write(path, serde_json::to_vec_pretty(&json).expect("json")).expect("write bundle");
}

fn config_with_policy(scratch: &Path, bundle_path: &Path, so_pubkey_hex: &str) -> DaemonConfig {
    let mut cfg = DaemonConfig::fixture(scratch);
    cfg.policy.bundle_path = Some(bundle_path.to_path_buf());
    cfg.policy.so_pubkey_hex = Some(so_pubkey_hex.to_string());
    cfg
}

#[test]
fn daemon_refuses_to_start_when_configured_bundle_is_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sk = signing_key(7);
    let pubkey_hex = hex::encode(sk.verifying_key().to_bytes());
    let cfg = config_with_policy(
        tmp.path(),
        &tmp.path().join("nonexistent.cbor"),
        &pubkey_hex,
    );
    assert!(
        Daemon::prepare(cfg).is_err(),
        "daemon must refuse to start when the configured PolicyBundle cannot be read \
         (fail-closed; NIST_AGENT-2026-05-31-001)"
    );
}

#[test]
fn daemon_refuses_bundle_signed_by_untrusted_key() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle_path = tmp.path().join("policy.json");
    // Signed by key 7; daemon trusts key 9.
    write_signed_bundle(&bundle_path, &signing_key(7));
    let trusted_hex = hex::encode(signing_key(9).verifying_key().to_bytes());
    let cfg = config_with_policy(tmp.path(), &bundle_path, &trusted_hex);
    assert!(
        Daemon::prepare(cfg).is_err(),
        "daemon must refuse a bundle whose SecurityOfficer signature does not verify \
         against the configured trust root"
    );
}

#[test]
fn daemon_refuses_garbage_so_pubkey_hex() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle_path = tmp.path().join("policy.json");
    write_signed_bundle(&bundle_path, &signing_key(7));
    let cfg = config_with_policy(tmp.path(), &bundle_path, "not-hex-at-all");
    assert!(
        Daemon::prepare(cfg).is_err(),
        "daemon must refuse a trust root that does not decode to an Ed25519 key"
    );
}

#[test]
fn daemon_refuses_tampered_bundle_bytes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle_path = tmp.path().join("policy.json");
    let sk = signing_key(7);
    // Sign one bundle, then flip the signed bytes on disk.
    let bundle = PolicyBundle::minimal_template();
    let mut canonical = bundle.encode_canonical().expect("encode");
    let signature = sk.sign(&canonical).to_bytes().to_vec();
    // Tamper AFTER signing.
    let last = canonical.len() - 1;
    canonical[last] ^= 0xFF;
    let json = serde_json::json!({
        "bundle_cbor_hex": hex::encode(&canonical),
        "signature_hex": hex::encode(&signature),
    });
    std::fs::write(&bundle_path, serde_json::to_vec(&json).expect("json")).expect("write");

    let pubkey_hex = hex::encode(sk.verifying_key().to_bytes());
    let cfg = config_with_policy(tmp.path(), &bundle_path, &pubkey_hex);
    assert!(
        Daemon::prepare(cfg).is_err(),
        "daemon must refuse a bundle whose bytes were tampered after signing"
    );
}

#[test]
fn daemon_with_valid_bundle_starts_and_holds_policy() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle_path = tmp.path().join("policy.json");
    let sk = signing_key(7);
    write_signed_bundle(&bundle_path, &sk);
    let pubkey_hex = hex::encode(sk.verifying_key().to_bytes());
    let cfg = config_with_policy(tmp.path(), &bundle_path, &pubkey_hex);
    let daemon = Daemon::prepare(cfg).expect("valid bundle must be accepted");
    let bundle = daemon
        .state()
        .policy
        .clone()
        .expect("verified bundle must be held in DaemonState for runtime gating");
    assert_eq!(bundle.bundle_name, "minimal-template");
}

/// Write a deployable signed bundle carrying an explicit overlay set.
fn write_signed_bundle_with_overlays(
    path: &Path,
    sk: &SigningKey,
    overlays: nist_agent_policy::ActiveOverlays,
) {
    let mut bundle = PolicyBundle::minimal_template();
    for (role, dids) in bundle.role_assignments.iter_mut() {
        *dids = vec![format!("did:citrate:{:?}:0xabc", role)];
    }
    bundle.not_before = 0;
    bundle.expires_at = 4_102_444_800;
    bundle.overlays = overlays;
    let canonical = bundle.encode_canonical().expect("encode");
    let signature = sk.sign(&canonical).to_bytes().to_vec();
    let raw = RawSignedBundle {
        bundle_cbor: canonical,
        signature,
    };
    let json = serde_json::json!({
        "bundle_cbor_hex": hex::encode(&raw.bundle_cbor),
        "signature_hex": hex::encode(&raw.signature),
    });
    std::fs::write(path, serde_json::to_vec_pretty(&json).expect("json")).expect("write bundle");
}

#[test]
fn daemon_refuses_bundle_that_silently_drops_active_overlay() {
    // NA2-B-004: the overlay ratchet is enforced across restarts.
    // First start activates HIPAA; a second bundle that drops HIPAA
    // without decommissioning it must refuse startup.
    use nist_agent_policy::{ActiveOverlays, Overlay};
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle_path = tmp.path().join("policy.json");
    let sk = signing_key(7);
    let pubkey_hex = hex::encode(sk.verifying_key().to_bytes());

    let mut with_hipaa = ActiveOverlays::new_with_cmmc_baseline();
    with_hipaa.add(Overlay::HipaaHitech);
    write_signed_bundle_with_overlays(&bundle_path, &sk, with_hipaa);
    let cfg = config_with_policy(tmp.path(), &bundle_path, &pubkey_hex);
    Daemon::prepare(cfg).expect("first activation with HIPAA succeeds");

    // Second bundle: CMMC baseline only (HIPAA silently dropped).
    write_signed_bundle_with_overlays(&bundle_path, &sk, ActiveOverlays::new_with_cmmc_baseline());
    let cfg2 = config_with_policy(tmp.path(), &bundle_path, &pubkey_hex);
    assert!(
        Daemon::prepare(cfg2).is_err(),
        "dropping an active overlay without decommissioning must refuse startup"
    );
}

#[test]
fn daemon_without_bundle_path_starts_in_minimal_mode() {
    // The documented air-gap smoke posture: no bundle configured
    // boots minimal mode. This stays legal; the finding is about
    // a CONFIGURED bundle being silently ignored.
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg = DaemonConfig::fixture(tmp.path());
    assert!(Daemon::prepare(cfg).is_ok());
}
