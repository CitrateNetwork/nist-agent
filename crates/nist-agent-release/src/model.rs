//! SI-7 model integrity check.
//!
//! `dist-bundled-gemma4.feature` pins the rule: when the harness
//! attempts to load the bundled GGUF and the measured sha256
//! does not match the release manifest's `[model].sha256`, load
//! must fail with `Err("SI-7: model hash mismatch")`. This module
//! is the canonical home of that check.

use sha2::{Digest, Sha256};

use crate::error::ReleaseError;
use crate::manifest::ReleaseManifest;

pub struct ModelIntegrity;

impl ModelIntegrity {
    /// Verify a GGUF byte slice against the manifest's `[model]`
    /// entry. Returns `Err(Si7ModelHashMismatch)` on mismatch
    /// (the exact wording the feature scenario pins). Returns
    /// `Err(ArtifactMissing)` if the manifest carries no model
    /// entry — operationally that means the bundle is mis-built
    /// and the harness should refuse to load any GGUF.
    pub fn verify_gguf(manifest: &ReleaseManifest, gguf_bytes: &[u8]) -> Result<(), ReleaseError> {
        let entry = manifest
            .model_entry()
            .ok_or_else(|| ReleaseError::ArtifactMissing {
                path: "kind=model".into(),
            })?;
        let expected = entry.sha256_bytes()?;
        let measured = sha256_bytes(gguf_bytes);
        if expected == measured {
            Ok(())
        } else {
            Err(ReleaseError::Si7ModelHashMismatch)
        }
    }

    /// Verify a measured (already-hashed) digest against the
    /// manifest. Useful when the harness has already hashed the
    /// file while computing some other metric (e.g. doctor).
    pub fn verify_gguf_digest(
        manifest: &ReleaseManifest,
        measured_digest: [u8; 32],
    ) -> Result<(), ReleaseError> {
        let entry = manifest
            .model_entry()
            .ok_or_else(|| ReleaseError::ArtifactMissing {
                path: "kind=model".into(),
            })?;
        let expected = entry.sha256_bytes()?;
        if expected == measured_digest {
            Ok(())
        } else {
            Err(ReleaseError::Si7ModelHashMismatch)
        }
    }
}

pub(crate) fn sha256_bytes(bytes: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    arr
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ArtifactEntry, ArtifactKind, SignatureEnvelope};

    fn manifest_with_model(gguf_bytes: &[u8]) -> ReleaseManifest {
        let digest = sha256_bytes(gguf_bytes);
        ReleaseManifest {
            version: "0.1.0".into(),
            git_rev: "x".into(),
            artifacts: vec![ArtifactEntry {
                kind: ArtifactKind::Model,
                path: "models/gemma-4-e2b-it-Q4_K_M.gguf".into(),
                sha256: format!("0x{}", hex::encode(digest)),
            }],
            signature: SignatureEnvelope::empty_ed25519(),
        }
    }

    #[test]
    fn matching_gguf_bytes_pass() {
        let bytes = b"GGUF stub contents".to_vec();
        let m = manifest_with_model(&bytes);
        ModelIntegrity::verify_gguf(&m, &bytes).expect("matching bytes verify");
    }

    #[test]
    fn tampered_gguf_fails_with_exact_si7_wording() {
        // The feature scenario pins the wording so a serde
        // refactor or a `Display` change can't drift the
        // operator-facing message. The Display impl on
        // ReleaseError is the source of truth.
        let original = b"GGUF stub contents".to_vec();
        let manifest = manifest_with_model(&original);
        let tampered = b"GGUF stub contents (tampered)".to_vec();
        let err = ModelIntegrity::verify_gguf(&manifest, &tampered).unwrap_err();
        assert_eq!(err, ReleaseError::Si7ModelHashMismatch);
        assert_eq!(err.to_string(), "SI-7: model hash mismatch");
    }

    #[test]
    fn missing_model_entry_surfaces_artifact_missing() {
        let mut m = manifest_with_model(b"x");
        m.artifacts.clear();
        let err = ModelIntegrity::verify_gguf(&m, b"x").unwrap_err();
        match err {
            ReleaseError::ArtifactMissing { path } => assert_eq!(path, "kind=model"),
            other => panic!("expected ArtifactMissing, got {other:?}"),
        }
    }

    #[test]
    fn verify_gguf_digest_short_circuits_on_match() {
        let bytes = b"x".to_vec();
        let m = manifest_with_model(&bytes);
        let measured = sha256_bytes(&bytes);
        ModelIntegrity::verify_gguf_digest(&m, measured).expect("matches");
    }
}
