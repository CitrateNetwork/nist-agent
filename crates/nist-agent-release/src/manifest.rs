//! `release.manifest.toml` — the canonical declaration of what
//! ships in a release bundle.
//!
//! Shape (target):
//!
//! ```toml
//! version = "0.1.0"
//! git_rev = "ca181f9..."
//!
//! [signature]
//! algorithm = "ed25519"
//! public_key_hex = "..."
//! signature_hex = "..."
//!
//! [[artifact]]
//! kind = "daemon"
//! path = "bin/citrate-agent"
//! sha256 = "0x..."
//!
//! [[artifact]]
//! kind = "model"
//! path = "models/gemma-4-e2b-it-Q4_K_M.gguf"
//! sha256 = "0x..."
//!
//! [[artifact]]
//! kind = "model-license"
//! path = "models/LICENSE.txt"
//! sha256 = "0x..."
//!
//! [[artifact]]
//! kind = "sbom"
//! path = "sbom.spdx.json"
//! sha256 = "0x..."
//! ```
//!
//! The detached signature in `[signature]` covers the manifest
//! body with the `[signature]` block excluded (set to its zero
//! state during signing). The verifier reconstructs that
//! pre-signature form when checking.

use serde::{Deserialize, Serialize};

use crate::error::ReleaseError;

/// What kind of artifact a manifest entry describes. The
/// installer / doctor use this to dispatch to the right verifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    /// The signed daemon binary.
    Daemon,
    /// The bundled concierge GGUF (Gemma 4 E2B Q4_K_M in v1.0).
    Model,
    /// The bundled model's upstream license file.
    ModelLicense,
    /// A capsule artifact shipped in the bundle.
    Capsule,
    /// SBOM in SPDX or CycloneDX form.
    Sbom,
    /// An overlay deployment runbook.
    Runbook,
    /// Other / future kinds. Operators can ship custom kinds; the
    /// installer simply hashes them and confirms the recorded
    /// sha256 matches.
    Other,
}

/// One entry in the manifest's artifact list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactEntry {
    pub kind: ArtifactKind,
    /// Relative path inside the extracted bundle. Forward slashes
    /// regardless of host OS so manifests are portable.
    pub path: String,
    /// Hex-encoded SHA-256 of the artifact bytes. May be prefixed
    /// with `0x` — both forms accepted on decode.
    pub sha256: String,
}

impl ArtifactEntry {
    /// Decoded 32-byte sha256. Surfaces a stable error if the
    /// recorded string isn't valid hex.
    pub fn sha256_bytes(&self) -> Result<[u8; 32], ReleaseError> {
        let trimmed = self.sha256.strip_prefix("0x").unwrap_or(&self.sha256);
        let bytes = hex::decode(trimmed).map_err(|e| ReleaseError::HexDecode {
            field: format!("artifact[{}].sha256", self.path),
            reason: e.to_string(),
        })?;
        if bytes.len() != 32 {
            return Err(ReleaseError::HexDecode {
                field: format!("artifact[{}].sha256", self.path),
                reason: format!("expected 32 bytes, got {}", bytes.len()),
            });
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

/// The signature block. `signature_hex` is set to all-zeros (128
/// hex chars) during signing so the signing payload is stable
/// across signers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignatureEnvelope {
    /// `"ed25519"` in v1.0. Reserved for algorithm upgrades.
    pub algorithm: String,
    /// Hex-encoded Ed25519 public key (32 bytes).
    pub public_key_hex: String,
    /// Hex-encoded Ed25519 signature (64 bytes). Pre-signature
    /// state is 128 zero hex chars.
    pub signature_hex: String,
}

impl SignatureEnvelope {
    pub const PRE_SIGNATURE_SIGNATURE_HEX: &'static str =
        "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    pub fn empty_ed25519() -> Self {
        Self {
            algorithm: "ed25519".to_string(),
            public_key_hex: String::new(),
            signature_hex: Self::PRE_SIGNATURE_SIGNATURE_HEX.to_string(),
        }
    }

    /// True iff this envelope is in pre-signature state (zeroed
    /// signature). The signing payload uses this form so the
    /// hash that gets signed is independent of any prior
    /// signature.
    pub fn is_pre_signature(&self) -> bool {
        self.signature_hex == Self::PRE_SIGNATURE_SIGNATURE_HEX
    }
}

/// Top-level manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    /// Semver of the release. Drives compat checks in doctor.
    pub version: String,
    /// Git rev of nist-agent at release time. Surfaced in
    /// `doctor` output.
    pub git_rev: String,
    /// One `[[artifact]]` per shipped file.
    #[serde(rename = "artifact", default)]
    pub artifacts: Vec<ArtifactEntry>,
    /// Detached signature envelope. See module docs for the
    /// pre-signature convention.
    pub signature: SignatureEnvelope,
}

impl ReleaseManifest {
    /// Decode from the canonical TOML wire form.
    pub fn from_toml(s: &str) -> Result<Self, ReleaseError> {
        toml::from_str(s).map_err(|e| ReleaseError::ManifestDecode(e.to_string()))
    }

    /// Encode to TOML.
    pub fn to_toml(&self) -> Result<String, ReleaseError> {
        toml::to_string(self).map_err(|e| ReleaseError::ManifestDecode(e.to_string()))
    }

    /// Build the *signing payload* — the manifest's TOML form
    /// with the signature field zeroed. Signers and verifiers
    /// both compute over this exact byte sequence so a sig
    /// produced on machine A verifies on machine B regardless of
    /// the byte form on the wire.
    pub fn signing_payload(&self) -> Result<Vec<u8>, ReleaseError> {
        let mut clone = self.clone();
        clone.signature.signature_hex = SignatureEnvelope::PRE_SIGNATURE_SIGNATURE_HEX.to_string();
        let s = clone.to_toml()?;
        Ok(s.into_bytes())
    }

    /// Find the `kind = "model"` entry, if any. The harness uses
    /// this to drive the SI-7 hash check before loading the GGUF.
    pub fn model_entry(&self) -> Option<&ArtifactEntry> {
        self.artifacts
            .iter()
            .find(|a| a.kind == ArtifactKind::Model)
    }

    /// Find the `kind = "daemon"` entry, if any. Doctor uses this
    /// to drive the running-daemon hash check.
    pub fn daemon_entry(&self) -> Option<&ArtifactEntry> {
        self.artifacts
            .iter()
            .find(|a| a.kind == ArtifactKind::Daemon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ReleaseManifest {
        ReleaseManifest {
            version: "0.1.0".into(),
            git_rev: "deadbeef".into(),
            artifacts: vec![
                ArtifactEntry {
                    kind: ArtifactKind::Daemon,
                    path: "bin/citrate-agent".into(),
                    sha256: format!("0x{}", "a".repeat(64)),
                },
                ArtifactEntry {
                    kind: ArtifactKind::Model,
                    path: "models/gemma-4-e2b-it-Q4_K_M.gguf".into(),
                    sha256: format!("0x{}", "b".repeat(64)),
                },
                ArtifactEntry {
                    kind: ArtifactKind::ModelLicense,
                    path: "models/LICENSE.txt".into(),
                    sha256: format!("0x{}", "c".repeat(64)),
                },
            ],
            signature: SignatureEnvelope {
                algorithm: "ed25519".into(),
                public_key_hex: "00".repeat(32),
                signature_hex: SignatureEnvelope::PRE_SIGNATURE_SIGNATURE_HEX.to_string(),
            },
        }
    }

    #[test]
    fn manifest_round_trips_via_toml() {
        let m = sample();
        let s = m.to_toml().expect("encode");
        let r = ReleaseManifest::from_toml(&s).expect("decode");
        assert_eq!(r, m);
    }

    #[test]
    fn from_toml_returns_manifest_decode_on_garbage() {
        let err = ReleaseManifest::from_toml("not toml = {{[]").unwrap_err();
        match err {
            ReleaseError::ManifestDecode(msg) => assert!(!msg.is_empty()),
            other => panic!("expected ManifestDecode, got {other:?}"),
        }
    }

    #[test]
    fn artifact_sha256_bytes_accepts_with_and_without_0x_prefix() {
        let with = ArtifactEntry {
            kind: ArtifactKind::Sbom,
            path: "sbom.spdx.json".into(),
            sha256: format!("0x{}", "1".repeat(64)),
        };
        let without = ArtifactEntry {
            kind: ArtifactKind::Sbom,
            path: "sbom.spdx.json".into(),
            sha256: "1".repeat(64),
        };
        assert_eq!(
            with.sha256_bytes().unwrap(),
            without.sha256_bytes().unwrap()
        );
    }

    #[test]
    fn artifact_sha256_bytes_rejects_short_hash() {
        let short = ArtifactEntry {
            kind: ArtifactKind::Sbom,
            path: "sbom.spdx.json".into(),
            sha256: "deadbeef".into(),
        };
        let err = short.sha256_bytes().unwrap_err();
        match err {
            ReleaseError::HexDecode { field, reason } => {
                assert!(field.contains("sha256"));
                assert!(reason.contains("32 bytes"));
            }
            other => panic!("expected HexDecode, got {other:?}"),
        }
    }

    #[test]
    fn signing_payload_zeroes_signature_field() {
        // The signing payload must be independent of any prior
        // signature, otherwise sig-over-sig drift creeps in.
        let mut m = sample();
        m.signature.signature_hex = "ff".repeat(64);
        let payload = m.signing_payload().unwrap();
        let payload_str = std::str::from_utf8(&payload).unwrap();
        assert!(payload_str.contains(SignatureEnvelope::PRE_SIGNATURE_SIGNATURE_HEX));
        assert!(!payload_str.contains(&"ff".repeat(64)));
    }

    #[test]
    fn model_entry_finds_the_gguf() {
        let m = sample();
        let e = m.model_entry().expect("model present");
        assert_eq!(e.path, "models/gemma-4-e2b-it-Q4_K_M.gguf");
    }

    #[test]
    fn model_entry_returns_none_when_no_model() {
        let mut m = sample();
        m.artifacts.retain(|a| a.kind != ArtifactKind::Model);
        assert!(m.model_entry().is_none());
    }

    #[test]
    fn daemon_entry_finds_the_binary() {
        let m = sample();
        let e = m.daemon_entry().expect("daemon present");
        assert_eq!(e.path, "bin/citrate-agent");
    }

    #[test]
    fn signature_envelope_default_is_pre_signature_state() {
        let env = SignatureEnvelope::empty_ed25519();
        assert!(env.is_pre_signature());
        assert_eq!(env.algorithm, "ed25519");
    }

    #[test]
    fn artifact_kind_serializes_kebab_case() {
        let s = serde_json::to_string(&ArtifactKind::ModelLicense).unwrap();
        assert_eq!(s, "\"model-license\"");
        let r: ArtifactKind = serde_json::from_str("\"runbook\"").unwrap();
        assert_eq!(r, ArtifactKind::Runbook);
    }
}
