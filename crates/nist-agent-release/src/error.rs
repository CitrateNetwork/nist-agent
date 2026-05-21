//! `ReleaseError` — typed errors for manifest decode, signature
//! verification, install refusal, model integrity, and the
//! daemon-hash doctor check.

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ReleaseError {
    /// Wire-form decode of `release.manifest.toml` failed. The
    /// installer surfaces this with the underlying message so
    /// operators get a useful diagnostic when a corrupted
    /// manifest hits the offline target.
    #[error("manifest decode: {0}")]
    ManifestDecode(String),

    /// The detached signature on the release bundle did not
    /// verify against the configured release public key. The
    /// installer refuses with **exactly** this wording per the
    /// `dist-signed-releases.feature` scenario "Operator install
    /// verifies signatures before extracting" — the test pins
    /// the string.
    #[error("release signature invalid")]
    ReleaseSignatureInvalid,

    /// Bundled GGUF's measured hash did not match the
    /// `release.manifest.toml`'s `[model].sha256`. The harness
    /// refuses to load the model with **exactly** this wording
    /// per `dist-bundled-gemma4.feature` — pinned by test.
    #[error("SI-7: model hash mismatch")]
    Si7ModelHashMismatch,

    /// The running daemon executable's sha256 does not match the
    /// release manifest's daemon entry. Doctor reports this as
    /// `Severity::Blocker` per `dist-reproducible-builds.feature`.
    #[error("daemon executable hash mismatch: manifest {manifest_hex}, observed {observed_hex}")]
    DaemonHashMismatch {
        manifest_hex: String,
        observed_hex: String,
    },

    /// An artifact named in the manifest is missing from the
    /// extracted bundle. Either the bundle was incomplete or the
    /// manifest was tampered post-signature; the operator should
    /// re-acquire the bundle from a trusted mirror.
    #[error("artifact missing from bundle: {path}")]
    ArtifactMissing { path: String },

    /// An artifact present in the extracted bundle does not match
    /// the manifest's sha256 entry. Same remediation as
    /// `ArtifactMissing`.
    #[error("artifact hash mismatch: {path}: manifest {manifest_hex}, observed {observed_hex}")]
    ArtifactHashMismatch {
        path: String,
        manifest_hex: String,
        observed_hex: String,
    },

    /// An egress activation directive arrived but its
    /// SecurityOfficer signature did not verify. The harness
    /// keeps egress disabled and surfaces this error.
    #[error("egress activation directive: invalid SecurityOfficer signature")]
    EgressDirectiveInvalid,

    /// Hex decode error on a manifest sha256 field. Distinct
    /// from `ManifestDecode` because operators sometimes
    /// hand-edit hashes; this points them at the offending
    /// field.
    #[error("hex decode of {field}: {reason}")]
    HexDecode { field: String, reason: String },
}
