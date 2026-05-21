//! Installer flow.
//!
//! `dist-signed-releases.feature` pins the rule: the operator
//! runs `citrate-agent install <bundle>`; the installer verifies
//! the detached signature before extracting; refuses install on
//! failure with reason `"release signature invalid"`. This module
//! is the host of that flow.
//!
//! The installer here is bundle-content-agnostic: it accepts a
//! `ReleaseManifest`, an `expected_key`, and the *measured*
//! sha256 of each artifact in the bundle. Wiring the actual
//! filesystem extraction is a thin wrapper around this verifier
//! that the daemon binary owns; this module is the policy.

use std::collections::HashMap;

use crate::error::ReleaseError;
use crate::manifest::ReleaseManifest;
use crate::signature::ReleaseVerifier;

/// What an install attempt returned. Carries enough detail for
/// the CLI to render a useful operator message; the `Result` from
/// `Installer::verify` already short-circuits on the signature
/// check per the feature scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallVerdict {
    pub manifest_version: String,
    pub git_rev: String,
    pub artifacts_verified: usize,
}

pub struct Installer<'a> {
    pub manifest: &'a ReleaseManifest,
    pub verifier: &'a ReleaseVerifier,
}

impl<'a> Installer<'a> {
    /// Verify the manifest's signature THEN every artifact's
    /// recorded sha256 against the supplied measured digests.
    /// Per the feature scenario, signature failure must refuse
    /// install *before* any extraction happens — the caller
    /// constructs the `Installer`, calls `verify_and_install`,
    /// and only on `Ok` proceeds to extract.
    ///
    /// `measured` is a `path -> sha256_bytes` map the caller
    /// fills in by hashing each artifact in the staging area
    /// (extraction-to-temp + per-file hash is the usual approach).
    pub fn verify_and_install(
        &self,
        measured: &HashMap<String, [u8; 32]>,
    ) -> Result<InstallVerdict, ReleaseError> {
        // 1. Signature first. The scenario "refuses install on
        // failure with reason 'release signature invalid'" is the
        // load-bearing check; everything else is the additional
        // bundle-integrity layer.
        self.verifier.verify(self.manifest)?;

        // 2. Every manifest artifact must appear in `measured`
        // with matching sha256.
        for entry in &self.manifest.artifacts {
            let observed =
                measured
                    .get(&entry.path)
                    .ok_or_else(|| ReleaseError::ArtifactMissing {
                        path: entry.path.clone(),
                    })?;
            let expected = entry.sha256_bytes()?;
            if expected != *observed {
                return Err(ReleaseError::ArtifactHashMismatch {
                    path: entry.path.clone(),
                    manifest_hex: hex::encode(expected),
                    observed_hex: hex::encode(observed),
                });
            }
        }

        Ok(InstallVerdict {
            manifest_version: self.manifest.version.clone(),
            git_rev: self.manifest.git_rev.clone(),
            artifacts_verified: self.manifest.artifacts.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ArtifactEntry, ArtifactKind, SignatureEnvelope};
    use crate::model::sha256_bytes;
    use crate::signature::{ReleaseSigner, ReleaseVerifier};
    use ed25519_dalek::SigningKey;

    fn signer() -> SigningKey {
        SigningKey::from_bytes(&[7u8; 32])
    }

    fn build_signed_manifest(daemon_bytes: &[u8], gguf_bytes: &[u8]) -> ReleaseManifest {
        let pre = ReleaseManifest {
            version: "0.1.0".into(),
            git_rev: "deadbeef".into(),
            artifacts: vec![
                ArtifactEntry {
                    kind: ArtifactKind::Daemon,
                    path: "bin/citrate-agent".into(),
                    sha256: format!("0x{}", hex::encode(sha256_bytes(daemon_bytes))),
                },
                ArtifactEntry {
                    kind: ArtifactKind::Model,
                    path: "models/gemma-4-e2b-it-Q4_K_M.gguf".into(),
                    sha256: format!("0x{}", hex::encode(sha256_bytes(gguf_bytes))),
                },
            ],
            signature: SignatureEnvelope::empty_ed25519(),
        };
        ReleaseSigner::new(signer()).sign(pre).unwrap()
    }

    #[test]
    fn happy_path_verifies_and_returns_verdict() {
        let daemon = b"daemon bytes".to_vec();
        let gguf = b"gguf bytes".to_vec();
        let manifest = build_signed_manifest(&daemon, &gguf);
        let v = ReleaseVerifier::new(signer().verifying_key());
        let installer = Installer {
            manifest: &manifest,
            verifier: &v,
        };
        let mut measured: HashMap<String, [u8; 32]> = HashMap::new();
        measured.insert("bin/citrate-agent".into(), sha256_bytes(&daemon));
        measured.insert(
            "models/gemma-4-e2b-it-Q4_K_M.gguf".into(),
            sha256_bytes(&gguf),
        );
        let verdict = installer
            .verify_and_install(&measured)
            .expect("happy path verifies");
        assert_eq!(verdict.manifest_version, "0.1.0");
        assert_eq!(verdict.artifacts_verified, 2);
    }

    #[test]
    fn install_refuses_with_pinned_wording_on_bad_signature() {
        // Feature scenario verbatim:
        //   "refuses install on failure with reason
        //    'release signature invalid'"
        let daemon = b"d".to_vec();
        let gguf = b"g".to_vec();
        let mut manifest = build_signed_manifest(&daemon, &gguf);
        manifest.signature.signature_hex = "00".repeat(64);
        let v = ReleaseVerifier::new(signer().verifying_key());
        let installer = Installer {
            manifest: &manifest,
            verifier: &v,
        };
        let err = installer.verify_and_install(&HashMap::new()).unwrap_err();
        assert_eq!(err, ReleaseError::ReleaseSignatureInvalid);
        assert_eq!(err.to_string(), "release signature invalid");
    }

    #[test]
    fn install_refuses_when_artifact_missing() {
        let daemon = b"d".to_vec();
        let gguf = b"g".to_vec();
        let manifest = build_signed_manifest(&daemon, &gguf);
        let v = ReleaseVerifier::new(signer().verifying_key());
        let installer = Installer {
            manifest: &manifest,
            verifier: &v,
        };
        let mut measured: HashMap<String, [u8; 32]> = HashMap::new();
        // Only the daemon present; GGUF missing.
        measured.insert("bin/citrate-agent".into(), sha256_bytes(&daemon));
        let err = installer.verify_and_install(&measured).unwrap_err();
        match err {
            ReleaseError::ArtifactMissing { path } => {
                assert_eq!(path, "models/gemma-4-e2b-it-Q4_K_M.gguf")
            }
            other => panic!("expected ArtifactMissing, got {other:?}"),
        }
    }

    #[test]
    fn install_refuses_when_artifact_hash_mismatches() {
        let daemon = b"d".to_vec();
        let gguf = b"g".to_vec();
        let manifest = build_signed_manifest(&daemon, &gguf);
        let v = ReleaseVerifier::new(signer().verifying_key());
        let installer = Installer {
            manifest: &manifest,
            verifier: &v,
        };
        let mut measured: HashMap<String, [u8; 32]> = HashMap::new();
        measured.insert("bin/citrate-agent".into(), sha256_bytes(&daemon));
        measured.insert(
            "models/gemma-4-e2b-it-Q4_K_M.gguf".into(),
            sha256_bytes(b"tampered gguf"),
        );
        let err = installer.verify_and_install(&measured).unwrap_err();
        match err {
            ReleaseError::ArtifactHashMismatch { path, .. } => {
                assert_eq!(path, "models/gemma-4-e2b-it-Q4_K_M.gguf")
            }
            other => panic!("expected ArtifactHashMismatch, got {other:?}"),
        }
    }
}
