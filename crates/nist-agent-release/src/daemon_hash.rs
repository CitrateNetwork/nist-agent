//! Daemon-executable hash check.
//!
//! `dist-reproducible-builds.feature` pins the rule: "Doctor
//! verifies the running daemon's hash matches the published
//! manifest; mismatch is BLOCKER." This module owns the
//! comparison. The doctor crate consumes this verdict and surfaces
//! it with its own `Severity::Blocker`.
//!
//! Wiring the actual "measure the running daemon's executable
//! sha256" — on Linux, read `/proc/self/exe`'s bytes — lives in
//! the doctor's host-OS bridge. This module accepts the measured
//! digest so the verdict computation is host-agnostic and
//! testable in CI.

use crate::error::ReleaseError;
use crate::manifest::ReleaseManifest;

/// Verdict from comparing a running daemon's executable against
/// the release manifest. `Ok` means the running binary is the one
/// the manifest says it should be; `Mismatch` is the BLOCKER the
/// feature scenario pins.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DaemonHashVerdict {
    Ok {
        manifest_hex: String,
        observed_hex: String,
    },
    Mismatch {
        manifest_hex: String,
        observed_hex: String,
    },
    /// The manifest carried no daemon entry. Doctor should treat
    /// this as a misconfiguration: a release without a daemon
    /// manifest entry is malformed.
    NoDaemonEntry,
}

pub struct DaemonHashCheck;

impl DaemonHashCheck {
    /// Compute the verdict. The doctor invokes this with the
    /// measured digest of the running executable.
    pub fn verdict(
        manifest: &ReleaseManifest,
        observed_digest: [u8; 32],
    ) -> Result<DaemonHashVerdict, ReleaseError> {
        let entry = match manifest.daemon_entry() {
            Some(e) => e,
            None => return Ok(DaemonHashVerdict::NoDaemonEntry),
        };
        let expected = entry.sha256_bytes()?;
        if expected == observed_digest {
            Ok(DaemonHashVerdict::Ok {
                manifest_hex: hex::encode(expected),
                observed_hex: hex::encode(observed_digest),
            })
        } else {
            Ok(DaemonHashVerdict::Mismatch {
                manifest_hex: hex::encode(expected),
                observed_hex: hex::encode(observed_digest),
            })
        }
    }

    /// Doctor's convenience wrapper: convert a `Mismatch` verdict
    /// into a `ReleaseError::DaemonHashMismatch` the doctor can
    /// box up as a BLOCKER finding.
    pub fn into_result(verdict: DaemonHashVerdict) -> Result<(), ReleaseError> {
        match verdict {
            DaemonHashVerdict::Ok { .. } | DaemonHashVerdict::NoDaemonEntry => Ok(()),
            DaemonHashVerdict::Mismatch {
                manifest_hex,
                observed_hex,
            } => Err(ReleaseError::DaemonHashMismatch {
                manifest_hex,
                observed_hex,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{ArtifactEntry, ArtifactKind, SignatureEnvelope};
    use crate::model::sha256_bytes;

    fn manifest_with_daemon(daemon_bytes: &[u8]) -> ReleaseManifest {
        ReleaseManifest {
            version: "0.1.0".into(),
            git_rev: "x".into(),
            artifacts: vec![ArtifactEntry {
                kind: ArtifactKind::Daemon,
                path: "bin/citrate-agent".into(),
                sha256: format!("0x{}", hex::encode(sha256_bytes(daemon_bytes))),
            }],
            signature: SignatureEnvelope::empty_ed25519(),
        }
    }

    #[test]
    fn matching_daemon_hash_returns_ok_verdict() {
        let bytes = b"daemon".to_vec();
        let m = manifest_with_daemon(&bytes);
        let v = DaemonHashCheck::verdict(&m, sha256_bytes(&bytes)).unwrap();
        assert!(matches!(v, DaemonHashVerdict::Ok { .. }));
        DaemonHashCheck::into_result(v).expect("ok verdict -> ok result");
    }

    #[test]
    fn mismatched_daemon_hash_is_blocker() {
        let original = b"daemon".to_vec();
        let m = manifest_with_daemon(&original);
        let other = sha256_bytes(b"tampered");
        let v = DaemonHashCheck::verdict(&m, other).unwrap();
        match v {
            DaemonHashVerdict::Mismatch { .. } => {}
            other => panic!("expected Mismatch, got {other:?}"),
        }
    }

    #[test]
    fn mismatch_converts_to_daemon_hash_mismatch_error() {
        let m = manifest_with_daemon(b"x");
        let v = DaemonHashCheck::verdict(&m, sha256_bytes(b"y")).unwrap();
        let err = DaemonHashCheck::into_result(v).unwrap_err();
        match err {
            ReleaseError::DaemonHashMismatch {
                manifest_hex,
                observed_hex,
            } => {
                assert_ne!(manifest_hex, observed_hex);
            }
            other => panic!("expected DaemonHashMismatch, got {other:?}"),
        }
    }

    #[test]
    fn missing_daemon_entry_is_a_recognized_state() {
        let mut m = manifest_with_daemon(b"x");
        m.artifacts.clear();
        let v = DaemonHashCheck::verdict(&m, [0u8; 32]).unwrap();
        assert_eq!(v, DaemonHashVerdict::NoDaemonEntry);
        // `into_result` treats this as Ok — it's the doctor's
        // job to surface this as a separate finding ("release
        // manifest is malformed"), not the hash check's.
        DaemonHashCheck::into_result(v).expect("no-daemon-entry -> ok");
    }
}
