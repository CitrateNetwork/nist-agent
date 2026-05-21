//! Ed25519 detached-signature wrap for release manifests.
//!
//! The release pipeline (deferred per ADR-011) holds the signing
//! key on an HSM. This module exposes:
//!
//! - `ReleaseSigner` — a soft-key signer used for testing and
//!   for the (deferred) integration-test fixture. The production
//!   path replaces this with the HSM-backed signer; the
//!   `Verifier` side does not change.
//! - `ReleaseVerifier` — what the operator runs at install time.
//!   Pure-Rust, no network, no HSM.

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

use crate::error::ReleaseError;
use crate::manifest::{ReleaseManifest, SignatureEnvelope};

/// Soft-key signer for tests + fixtures. The production HSM-
/// backed signer implements the same contract from a different
/// path; this is only the verifier-side proof that the contract
/// is implementable.
pub struct ReleaseSigner {
    key: SigningKey,
}

impl ReleaseSigner {
    pub fn new(key: SigningKey) -> Self {
        Self { key }
    }

    /// Build a signed manifest from a pre-signature manifest.
    /// Overwrites the manifest's `signature` envelope with the
    /// produced Ed25519 signature + the signer's public key.
    pub fn sign(&self, mut manifest: ReleaseManifest) -> Result<ReleaseManifest, ReleaseError> {
        manifest.signature = SignatureEnvelope::empty_ed25519();
        manifest.signature.public_key_hex = hex::encode(self.key.verifying_key().to_bytes());
        let payload = manifest.signing_payload()?;
        let sig: Signature = self.key.sign(&payload);
        manifest.signature.signature_hex = hex::encode(sig.to_bytes());
        Ok(manifest)
    }
}

/// Verifier the operator's installer runs. Accepts a fully-signed
/// manifest + an expected public key (the release cert that
/// ships in the operator's `CitrateNetwork/.github` org
/// defaults).
#[derive(Debug)]
pub struct ReleaseVerifier {
    expected_key: VerifyingKey,
}

impl ReleaseVerifier {
    /// Construct from a known-trusted public key.
    pub fn new(expected_key: VerifyingKey) -> Self {
        Self { expected_key }
    }

    /// Construct from the hex-encoded public key shipped in
    /// operator config. Surfaces a stable error if the hex is
    /// malformed.
    pub fn from_hex(public_key_hex: &str) -> Result<Self, ReleaseError> {
        let trimmed = public_key_hex.strip_prefix("0x").unwrap_or(public_key_hex);
        let bytes = hex::decode(trimmed).map_err(|e| ReleaseError::HexDecode {
            field: "release.public_key_hex".into(),
            reason: e.to_string(),
        })?;
        if bytes.len() != 32 {
            return Err(ReleaseError::HexDecode {
                field: "release.public_key_hex".into(),
                reason: format!("expected 32 bytes, got {}", bytes.len()),
            });
        }
        let mut k = [0u8; 32];
        k.copy_from_slice(&bytes);
        let vk = VerifyingKey::from_bytes(&k).map_err(|e| ReleaseError::HexDecode {
            field: "release.public_key_hex".into(),
            reason: e.to_string(),
        })?;
        Ok(Self::new(vk))
    }

    /// Verify the manifest's detached signature. Returns
    /// `Err(ReleaseSignatureInvalid)` on every failure path —
    /// per `dist-signed-releases.feature` the operator gets one
    /// stable error class regardless of which sub-check failed.
    pub fn verify(&self, manifest: &ReleaseManifest) -> Result<(), ReleaseError> {
        // 1. Public-key match. The manifest carries the public
        // key for transparency, but the operator's trust root is
        // their pre-configured cert; if those disagree, refuse.
        let manifest_pk = self.decode_manifest_public_key(manifest);
        match manifest_pk {
            Ok(pk) if pk == self.expected_key => {}
            _ => return Err(ReleaseError::ReleaseSignatureInvalid),
        }

        // 2. Signature decode.
        let sig = self
            .decode_signature(manifest)
            .ok_or(ReleaseError::ReleaseSignatureInvalid)?;

        // 3. Build the signing payload (pre-signature form) and
        // verify.
        let payload = manifest
            .signing_payload()
            .map_err(|_| ReleaseError::ReleaseSignatureInvalid)?;
        self.expected_key
            .verify(&payload, &sig)
            .map_err(|_| ReleaseError::ReleaseSignatureInvalid)
    }

    fn decode_manifest_public_key(&self, manifest: &ReleaseManifest) -> Result<VerifyingKey, ()> {
        let bytes = hex::decode(&manifest.signature.public_key_hex).map_err(|_| ())?;
        if bytes.len() != 32 {
            return Err(());
        }
        let mut k = [0u8; 32];
        k.copy_from_slice(&bytes);
        VerifyingKey::from_bytes(&k).map_err(|_| ())
    }

    fn decode_signature(&self, manifest: &ReleaseManifest) -> Option<Signature> {
        let bytes = hex::decode(&manifest.signature.signature_hex).ok()?;
        if bytes.len() != 64 {
            return None;
        }
        let mut s = [0u8; 64];
        s.copy_from_slice(&bytes);
        Some(Signature::from_bytes(&s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ArtifactEntry, ArtifactKind, ReleaseManifest, SignatureEnvelope};

    fn fixture_signer() -> SigningKey {
        // Deterministic test key — fixture only.
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn pre_sig_manifest() -> ReleaseManifest {
        ReleaseManifest {
            version: "0.1.0".into(),
            git_rev: "deadbeef".into(),
            artifacts: vec![ArtifactEntry {
                kind: ArtifactKind::Daemon,
                path: "bin/citrate-agent".into(),
                sha256: format!("0x{}", "a".repeat(64)),
            }],
            signature: SignatureEnvelope::empty_ed25519(),
        }
    }

    #[test]
    fn signer_produces_a_manifest_the_verifier_accepts() {
        let signer = ReleaseSigner::new(fixture_signer());
        let signed = signer.sign(pre_sig_manifest()).expect("sign");
        let verifier = ReleaseVerifier::new(fixture_signer().verifying_key());
        verifier.verify(&signed).expect("verify");
    }

    #[test]
    fn verifier_refuses_when_signature_is_garbage() {
        let signer = ReleaseSigner::new(fixture_signer());
        let mut signed = signer.sign(pre_sig_manifest()).unwrap();
        signed.signature.signature_hex = "ff".repeat(64);
        let verifier = ReleaseVerifier::new(fixture_signer().verifying_key());
        let err = verifier.verify(&signed).unwrap_err();
        assert_eq!(err, ReleaseError::ReleaseSignatureInvalid);
    }

    #[test]
    fn verifier_refuses_when_artifact_list_tampered() {
        // The artifact list is part of the signing payload;
        // tampering with it must invalidate the signature.
        let signer = ReleaseSigner::new(fixture_signer());
        let mut signed = signer.sign(pre_sig_manifest()).unwrap();
        signed.artifacts.push(ArtifactEntry {
            kind: ArtifactKind::Other,
            path: "evil.so".into(),
            sha256: format!("0x{}", "e".repeat(64)),
        });
        let verifier = ReleaseVerifier::new(fixture_signer().verifying_key());
        assert_eq!(
            verifier.verify(&signed).unwrap_err(),
            ReleaseError::ReleaseSignatureInvalid
        );
    }

    #[test]
    fn verifier_refuses_when_signer_is_unknown() {
        let signer = ReleaseSigner::new(fixture_signer());
        let signed = signer.sign(pre_sig_manifest()).unwrap();
        // Verifier configured with a different public key.
        let other = SigningKey::from_bytes(&[9u8; 32]);
        let verifier = ReleaseVerifier::new(other.verifying_key());
        assert_eq!(
            verifier.verify(&signed).unwrap_err(),
            ReleaseError::ReleaseSignatureInvalid
        );
    }

    #[test]
    fn from_hex_constructs_a_verifier_with_matching_behavior() {
        let signer = ReleaseSigner::new(fixture_signer());
        let signed = signer.sign(pre_sig_manifest()).unwrap();
        let pk_hex = hex::encode(fixture_signer().verifying_key().to_bytes());
        let v = ReleaseVerifier::from_hex(&pk_hex).expect("decode");
        v.verify(&signed).expect("verify");
    }

    #[test]
    fn from_hex_rejects_malformed_input() {
        let err = ReleaseVerifier::from_hex("not-hex").unwrap_err();
        match err {
            ReleaseError::HexDecode { field, .. } => {
                assert_eq!(field, "release.public_key_hex");
            }
            other => panic!("expected HexDecode, got {other:?}"),
        }
    }
}
