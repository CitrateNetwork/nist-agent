//! `PolicyBundle` — the signed configuration object.
//!
//! Encodes as canonical CBOR (RFC 8949 §4.2.1 deterministic
//! encoding) so the signed bytes are byte-stable across machines.
//! The SecurityOfficer signs the CBOR encoding directly; verifiers
//! recompute the canonical encoding and check the signature against
//! the bytes they produce.
//!
//! Bundles are versioned via `bundle_version`; the harness refuses
//! to load a bundle whose version is newer than it understands
//! (forward-compat is opt-in per future RFC revisions).

use crate::error::PolicyError;
use crate::overlay_state::ActiveOverlays;
use crate::types::{AnchorStrategy, EgressPosture, RiskTier, Role};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Bundle schema version. Bumped when the wire shape changes in a
/// breaking way (additive fields use serde `#[serde(default)]`).
pub const BUNDLE_VERSION: u32 = 1;

/// DID prefix stamped into every role by [`PolicyBundle::minimal_template`].
/// A deployable bundle MUST have rotated all of these to real
/// identities; `activate` refuses any bundle that still carries one
/// (NA2-B-028).
pub const PLACEHOLDER_DID_PREFIX: &str = "did:placeholder:";

/// The signed configuration. All fields are part of the canonical
/// signature payload; ordering of map keys is enforced by ciborium's
/// deterministic encoder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyBundle {
    /// Schema version. Matches `BUNDLE_VERSION` at write time.
    pub bundle_version: u32,
    /// Operator-chosen name (for audit-trail readability).
    pub bundle_name: String,
    /// Unix-epoch seconds when this bundle takes effect.
    pub not_before: i64,
    /// Unix-epoch seconds when this bundle ceases to be valid.
    /// Doctor flags expiration approaching as a WARN (S-7 check).
    pub expires_at: i64,
    /// Active overlay set + the activation history. Constructor
    /// pre-seeds CMMC-L3 baseline.
    pub overlays: ActiveOverlays,
    /// Capsule-name → tier mapping. May ESCALATE a capsule's
    /// declared tier but never de-escalates (enforced at
    /// `activate` time, not at parse time).
    pub risk_tier_map: BTreeMap<String, RiskTier>,
    /// Role assignment: role → list of DID strings of identities
    /// that hold the role. All five roles in `Role::ALL` MUST be
    /// non-empty at activation time.
    pub role_assignments: BTreeMap<Role, Vec<String>>,
    /// Operator-chosen anchor strategy (RFC §6.3).
    pub anchor_strategy: AnchorStrategy,
    /// Operator-chosen network posture (RFC §3.3).
    pub egress_posture: EgressPosture,
}

impl PolicyBundle {
    /// The minimum-viable bundle for a fresh deployment: CMMC-L3
    /// baseline only, no overlays added, all five roles assigned
    /// to placeholder DIDs the caller will rotate before deploy.
    pub fn minimal_template() -> Self {
        let mut role_assignments = BTreeMap::new();
        for r in Role::ALL {
            role_assignments.insert(*r, vec![format!("did:placeholder:{:?}", r)]);
        }
        Self {
            bundle_version: BUNDLE_VERSION,
            bundle_name: "minimal-template".into(),
            not_before: 0,
            expires_at: i64::MAX,
            overlays: ActiveOverlays::new_with_cmmc_baseline(),
            risk_tier_map: BTreeMap::new(),
            role_assignments,
            anchor_strategy: AnchorStrategy::HybridNightlyPlusCapsule,
            egress_posture: EgressPosture::Disabled,
        }
    }

    /// Canonical CBOR encoding. Pure function; same input always
    /// produces the same bytes. This is what the SecurityOfficer
    /// signs and what verifiers re-encode to check signatures.
    pub fn encode_canonical(&self) -> Result<Vec<u8>, PolicyError> {
        let mut out = Vec::new();
        ciborium::into_writer(self, &mut out)
            .map_err(|e| PolicyError::CborDecode(format!("encode: {e}")))?;
        Ok(out)
    }

    /// Hash of the canonical encoding (sha256). Audit records refer
    /// to bundles by this hash.
    pub fn canonical_hash(&self) -> Result<[u8; 32], PolicyError> {
        use sha2::{Digest, Sha256};
        let encoded = self.encode_canonical()?;
        Ok(Sha256::digest(&encoded).into())
    }

    /// Activate this bundle on the running harness. Returns
    /// `Ok(())` if every invariant holds, an error variant if not.
    ///
    /// Invariants checked here:
    /// 1. Every `Role::ALL` role has at least one identity assigned.
    /// 2. `bundle_version` is supported by this harness build.
    /// 3. Validity window is consistent (`not_before <= expires_at`).
    /// 4. Validity window contains `now` (operator-supplied; pass
    ///    a deterministic value in tests, `time::now()` in prod).
    /// 5. The risk-tier map only ESCALATES from `prior_tiers`; never
    ///    de-escalates. Pass empty `prior_tiers` for a first-load.
    ///
    /// The Bell-LaPadula lattice and overlay activation ratchet
    /// are NOT checked here — they're per-call concerns enforced
    /// downstream (capsule install for the lattice, the
    /// ActiveOverlays type for the ratchet).
    pub fn activate(
        &self,
        now_unix_seconds: i64,
        prior_tiers: &BTreeMap<String, RiskTier>,
    ) -> Result<(), PolicyError> {
        if self.bundle_version > BUNDLE_VERSION {
            return Err(PolicyError::CborDecode(format!(
                "bundle_version {} exceeds harness max {}",
                self.bundle_version, BUNDLE_VERSION
            )));
        }
        for role in Role::ALL {
            let count = self
                .role_assignments
                .get(role)
                .map(|v| v.len())
                .unwrap_or(0);
            if count == 0 {
                return Err(PolicyError::RoleUnassigned(*role));
            }
        }
        // NA2-B-028: refuse un-rotated placeholder identities. A
        // bundle straight out of `minimal_template()` would otherwise
        // satisfy the role-lattice check above with `did:placeholder:*`
        // DIDs and report PASS on doctor. Checked after every role is
        // confirmed non-empty so a truly unassigned role is reported
        // as such first.
        for role in Role::ALL {
            if let Some(dids) = self.role_assignments.get(role) {
                for did in dids {
                    if did.starts_with(PLACEHOLDER_DID_PREFIX) {
                        return Err(PolicyError::PlaceholderDidNotRotated {
                            role: *role,
                            did: did.clone(),
                        });
                    }
                }
            }
        }
        // NA2-B-028: a deployable bundle MUST carry a finite validity
        // window. `expires_at == i64::MAX` (the template default)
        // makes the RFC §10.2 expiry check unfalsifiable.
        if self.expires_at == i64::MAX {
            return Err(PolicyError::NoFiniteValidity);
        }
        if self.not_before > self.expires_at {
            return Err(PolicyError::Expired(format!(
                "not_before {} > expires_at {}",
                self.not_before, self.expires_at
            )));
        }
        if now_unix_seconds < self.not_before {
            return Err(PolicyError::NotYetValid(format!(
                "{} < not_before {}",
                now_unix_seconds, self.not_before
            )));
        }
        if now_unix_seconds > self.expires_at {
            return Err(PolicyError::Expired(format!(
                "{} > expires_at {}",
                now_unix_seconds, self.expires_at
            )));
        }
        // Per-capsule tier non-de-escalation. New entries are free;
        // existing entries may stay the same or escalate, never go
        // lower.
        for (capsule, declared) in prior_tiers {
            if let Some(proposed) = self.risk_tier_map.get(capsule) {
                if proposed < declared {
                    return Err(PolicyError::TierDeEscalation {
                        capsule: capsule.clone(),
                        declared: *declared,
                        proposed: *proposed,
                    });
                }
            }
        }
        Ok(())
    }
}

/// A bundle as it travels on the wire: the canonical CBOR-encoded
/// bytes plus the SecurityOfficer's detached Ed25519 signature.
/// Verifiers receive `(bytes, sig, pubkey)` and reconstitute the
/// `PolicyBundle` only after `verify` succeeds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSignedBundle {
    /// Canonical CBOR bytes of the `PolicyBundle`.
    pub bundle_cbor: Vec<u8>,
    /// 64-byte Ed25519 signature over `bundle_cbor`. Stored as
    /// `Vec<u8>` rather than `[u8; 64]` because serde's default
    /// derive doesn't support fixed arrays larger than 32 bytes
    /// without a helper crate. The length is enforced at the call
    /// to `Signature::from_slice` in `verify_and_decode`.
    pub signature: Vec<u8>,
}

impl RawSignedBundle {
    /// Verify the SecurityOfficer signature against the supplied
    /// public key. On success, decode and return the bundle. On
    /// failure, return a typed error.
    ///
    /// The verifier also re-encodes the decoded bundle and compares
    /// bytes against the input — this catches non-canonical CBOR
    /// (signed bytes that *parse* to a bundle but aren't the
    /// deterministic encoding). Non-canonical CBOR is a Rule-10
    /// concern: a non-canonical encoding could let an attacker
    /// substitute equivalent-but-different bytes the signature
    /// wouldn't catch.
    pub fn verify_and_decode(
        &self,
        trusted_so_pubkey: &VerifyingKey,
    ) -> Result<PolicyBundle, PolicyError> {
        let sig =
            Signature::from_slice(&self.signature).map_err(|_| PolicyError::SignatureInvalid)?;
        trusted_so_pubkey
            .verify(&self.bundle_cbor, &sig)
            .map_err(|_| PolicyError::SignatureInvalid)?;
        let bundle: PolicyBundle = ciborium::from_reader(self.bundle_cbor.as_slice())
            .map_err(|e| PolicyError::CborDecode(format!("decode: {e}")))?;
        // Re-encode and compare for canonical-CBOR enforcement.
        let reencoded = bundle.encode_canonical()?;
        if reencoded != self.bundle_cbor {
            return Err(PolicyError::NonCanonicalCbor);
        }
        Ok(bundle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    fn fixture_signing_key() -> SigningKey {
        // Deterministic 32-byte seed so the test reproduces.
        SigningKey::from_bytes(&[7u8; 32])
    }

    /// A bundle the operator could actually deploy: every
    /// placeholder DID rotated to a concrete identity and a finite
    /// validity window. `minimal_template()` is deliberately NOT
    /// deployable (NA2-B-028); tests that exercise a *valid*
    /// activation must start from this.
    fn deployable_template() -> PolicyBundle {
        let mut b = PolicyBundle::minimal_template();
        for (role, dids) in b.role_assignments.iter_mut() {
            *dids = vec![format!("did:citrate:{:?}:0xabc", role)];
        }
        b.not_before = 0;
        b.expires_at = 4_102_444_800; // 2100-01-01, finite
        b
    }

    #[test]
    fn minimal_template_is_not_deployable() {
        // NA2-B-028 tripwire: the template ships un-rotated
        // placeholder DIDs and an infinite expiry, so it must NOT
        // activate cleanly.
        let b = PolicyBundle::minimal_template();
        let err = b
            .activate(0, &BTreeMap::new())
            .expect_err("template must not be directly deployable");
        assert!(
            matches!(err, PolicyError::PlaceholderDidNotRotated { .. }),
            "expected PlaceholderDidNotRotated, got {err:?}"
        );
        // And even with DIDs rotated, an infinite expiry is refused.
        let mut b2 = deployable_template();
        b2.expires_at = i64::MAX;
        assert!(matches!(
            b2.activate(0, &BTreeMap::new()),
            Err(PolicyError::NoFiniteValidity)
        ));
    }

    #[test]
    fn minimal_template_round_trips_through_canonical_cbor() {
        let b = PolicyBundle::minimal_template();
        let cbor = b.encode_canonical().expect("encode");
        let round: PolicyBundle = ciborium::from_reader(cbor.as_slice()).expect("decode");
        assert_eq!(round, b);
    }

    #[test]
    fn canonical_encoding_is_deterministic() {
        let a = PolicyBundle::minimal_template();
        let b = PolicyBundle::minimal_template();
        assert_eq!(a.encode_canonical().unwrap(), b.encode_canonical().unwrap());
        assert_eq!(a.canonical_hash().unwrap(), b.canonical_hash().unwrap());
    }

    #[test]
    fn activate_requires_all_five_roles() {
        let mut b = PolicyBundle::minimal_template();
        b.role_assignments.remove(&Role::SecurityOfficer);
        let err = b.activate(0, &BTreeMap::new()).expect_err("missing SO");
        match err {
            PolicyError::RoleUnassigned(r) => assert_eq!(r, Role::SecurityOfficer),
            other => panic!("expected RoleUnassigned, got {other:?}"),
        }
    }

    #[test]
    fn activate_refuses_de_escalation() {
        let mut b = deployable_template();
        b.risk_tier_map.insert("cap1".into(), RiskTier::Low);
        let mut prior = BTreeMap::new();
        prior.insert("cap1".to_string(), RiskTier::High);
        let err = b
            .activate(0, &prior)
            .expect_err("Low < High should de-escalate");
        match err {
            PolicyError::TierDeEscalation {
                capsule,
                declared,
                proposed,
            } => {
                assert_eq!(capsule, "cap1");
                assert_eq!(declared, RiskTier::High);
                assert_eq!(proposed, RiskTier::Low);
            }
            other => panic!("expected TierDeEscalation, got {other:?}"),
        }
    }

    #[test]
    fn activate_permits_escalation_and_new_entries() {
        let mut b = deployable_template();
        b.risk_tier_map.insert("cap1".into(), RiskTier::Critical); // escalation
        b.risk_tier_map.insert("cap-new".into(), RiskTier::Medium); // new entry
        let mut prior = BTreeMap::new();
        prior.insert("cap1".to_string(), RiskTier::Low);
        b.activate(0, &prior).expect("escalation + new entry ok");
    }

    #[test]
    fn activate_refuses_bundle_outside_validity_window() {
        let mut b = deployable_template();
        b.not_before = 100;
        b.expires_at = 200;
        assert!(matches!(
            b.activate(50, &BTreeMap::new()),
            Err(PolicyError::NotYetValid(_))
        ));
        assert!(matches!(
            b.activate(250, &BTreeMap::new()),
            Err(PolicyError::Expired(_))
        ));
        b.activate(150, &BTreeMap::new()).expect("inside window");
    }

    #[test]
    fn sign_and_verify_round_trip() {
        let sk = fixture_signing_key();
        let pk = sk.verifying_key();
        let bundle = PolicyBundle::minimal_template();
        let cbor = bundle.encode_canonical().expect("encode");
        let signature = sk.sign(&cbor).to_bytes().to_vec();
        let raw = RawSignedBundle {
            bundle_cbor: cbor,
            signature,
        };
        let decoded = raw.verify_and_decode(&pk).expect("verify+decode");
        assert_eq!(decoded, bundle);
    }

    #[test]
    fn verify_rejects_wrong_key() {
        let sk = fixture_signing_key();
        let bundle = PolicyBundle::minimal_template();
        let cbor = bundle.encode_canonical().expect("encode");
        let sig = sk.sign(&cbor).to_bytes().to_vec();
        // Use a *different* key for verification.
        let other_pk = SigningKey::from_bytes(&[42u8; 32]).verifying_key();
        let raw = RawSignedBundle {
            bundle_cbor: cbor,
            signature: sig,
        };
        match raw.verify_and_decode(&other_pk) {
            Err(PolicyError::SignatureInvalid) => (),
            other => panic!("expected SignatureInvalid, got {other:?}"),
        }
    }

    #[test]
    fn verify_rejects_tampered_bytes() {
        let sk = fixture_signing_key();
        let pk = sk.verifying_key();
        let bundle = PolicyBundle::minimal_template();
        let cbor = bundle.encode_canonical().expect("encode");
        let sig = sk.sign(&cbor).to_bytes().to_vec();
        // Flip a byte in the middle of the signed bytes.
        let mut tampered = cbor.clone();
        let mid = tampered.len() / 2;
        tampered[mid] ^= 0xff;
        let raw = RawSignedBundle {
            bundle_cbor: tampered,
            signature: sig,
        };
        match raw.verify_and_decode(&pk) {
            Err(PolicyError::SignatureInvalid) => (),
            other => panic!("expected SignatureInvalid, got {other:?}"),
        }
    }

    #[test]
    fn verify_rejects_non_canonical_cbor() {
        // Build a non-canonical encoding by hand: a TWO-byte CBOR
        // integer for a value that fits in one byte. ciborium's
        // deterministic encoder always picks the shortest form;
        // we craft a longer one manually and sign it.
        //
        // Strategy: encode the bundle normally, then prepend extra
        // bytes that don't change the parse. Hardest path is just
        // to take the canonical bytes and append a no-op CBOR
        // break or trailing garbage — ciborium ignores trailing
        // bytes on decode, so the signed bytes parse fine, but
        // re-encoding doesn't produce them.
        let sk = fixture_signing_key();
        let pk = sk.verifying_key();
        let bundle = PolicyBundle::minimal_template();
        let canonical = bundle.encode_canonical().expect("encode");
        let mut non_canonical = canonical.clone();
        non_canonical.push(0xf6); // CBOR null — trailing, decoder may ignore
        let sig = sk.sign(&non_canonical).to_bytes().to_vec();
        let raw = RawSignedBundle {
            bundle_cbor: non_canonical,
            signature: sig,
        };
        // If ciborium ignores the trailing byte, we get NonCanonicalCbor;
        // if it rejects it, CborDecode. Both are valid hard-rejects.
        match raw.verify_and_decode(&pk) {
            Err(PolicyError::NonCanonicalCbor) | Err(PolicyError::CborDecode(_)) => (),
            other => panic!("expected NonCanonicalCbor or CborDecode, got {other:?}"),
        }
    }
}
