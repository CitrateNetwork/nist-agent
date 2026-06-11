//! `citrate-agent install <bundle>` — verify + install a signed
//! release bundle. Pinned-wording refusal on signature failure
//! per `dist-signed-releases.feature` ("release signature
//! invalid").
//!
//! The bundle layout assumed:
//!
//! ```text
//! bundle/
//!   release.manifest.toml
//!   bin/citrate-agent
//!   models/gemma-4-e2b-it-Q4_K_M.gguf
//!   models/LICENSE.txt
//!   sbom.spdx.json
//!   ...
//! ```
//!
//! The CLI verifies the manifest signature FIRST (no artifact
//! path named by an unauthenticated manifest is ever opened —
//! NIST_AGENT-2026-05-31-006), refuses artifact paths that
//! escape the bundle dir, then hashes each artifact and calls
//! `Installer::verify_and_install()`. On success it prints a
//! verdict line. The actual *extraction* (copy from bundle dir
//! to operator's install prefix) is operator-side; this binary
//! only validates.

use anyhow::{Context, Result};
use clap::Args;
use nist_agent_release::{Installer, ReleaseError, ReleaseManifest, ReleaseVerifier};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Path to the unpacked release bundle directory.
    pub bundle_dir: PathBuf,

    /// Hex-encoded Ed25519 public key the operator trusts as the
    /// release signer's root. Ships in
    /// `CitrateNetwork/.github/SECURITY.md` per ADR-011.
    #[arg(long, env = "NIST_AGENT_RELEASE_PUBKEY_HEX")]
    pub release_pubkey_hex: String,

    /// Don't actually verify; just print what would be checked.
    /// Useful for the air-gap test runbook to confirm the bundle
    /// is laid out as expected.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn run(args: InstallArgs) -> Result<i32> {
    let manifest_path = args.bundle_dir.join("release.manifest.toml");
    let toml_str = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    let manifest = ReleaseManifest::from_toml(&toml_str)
        .with_context(|| format!("parse manifest {}", manifest_path.display()))?;

    if args.dry_run {
        println!(
            "dry-run: {} artifact(s) would be hashed:",
            manifest.artifacts.len()
        );
        for a in &manifest.artifacts {
            println!("  {} ({})", a.path, format!("{:?}", a.kind).to_lowercase());
        }
        return Ok(0);
    }

    let verifier = match ReleaseVerifier::from_hex(&args.release_pubkey_hex) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid --release-pubkey-hex: {e}");
            return Ok(2);
        }
    };

    // Signature FIRST (NIST_AGENT-2026-05-31-006): no artifact
    // path named by the manifest is opened or hashed until the
    // manifest itself is authenticated — otherwise a tampered
    // manifest directs pre-auth reads at arbitrary local files.
    if let Err(e) = verifier.verify(&manifest) {
        match e {
            ReleaseError::ReleaseSignatureInvalid => {
                // Pinned wording per dist-signed-releases.feature.
                eprintln!("release signature invalid");
            }
            other => eprintln!("{other}"),
        }
        return Ok(2);
    }

    // Artifact paths must resolve inside the bundle dir — refuse
    // absolute paths and any `..` component even on a validly-
    // signed manifest (defense in depth).
    let mut artifact_paths: Vec<(String, PathBuf)> = Vec::with_capacity(manifest.artifacts.len());
    for artifact in &manifest.artifacts {
        match safe_bundle_path(&args.bundle_dir, &artifact.path) {
            Some(p) => artifact_paths.push((artifact.path.clone(), p)),
            None => {
                eprintln!("artifact path escapes bundle dir: {}", artifact.path);
                return Ok(2);
            }
        }
    }

    let mut measured: HashMap<String, [u8; 32]> = HashMap::new();
    for (rel, path) in &artifact_paths {
        let digest = stream_sha256(path)
            .with_context(|| format!("hash bundle artifact {}", path.display()))?;
        measured.insert(rel.clone(), digest);
    }

    let installer = Installer {
        manifest: &manifest,
        verifier: &verifier,
    };

    match installer.verify_and_install(&measured) {
        Ok(v) => {
            println!(
                "install verified: v{} ({}) — {} artifact(s)",
                v.manifest_version, v.git_rev, v.artifacts_verified,
            );
            Ok(0)
        }
        Err(ReleaseError::ReleaseSignatureInvalid) => {
            // Pinned wording per dist-signed-releases.feature.
            eprintln!("release signature invalid");
            Ok(2)
        }
        Err(e) => {
            eprintln!("{e}");
            Ok(2)
        }
    }
}

/// Join an artifact's manifest-declared path onto the bundle
/// dir, refusing absolute paths and any traversal component
/// (`Path::join` replaces the base on an absolute component, and
/// `..` walks out of the bundle). Returns `None` on refusal.
/// NIST_AGENT-2026-05-31-006.
fn safe_bundle_path(bundle_dir: &Path, artifact_path: &str) -> Option<PathBuf> {
    use std::path::Component;
    let rel = Path::new(artifact_path);
    if rel.is_absolute() {
        return None;
    }
    for component in rel.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(bundle_dir.join(rel))
}

/// Compute SHA-256 of a file by streaming 1 MiB chunks; avoids
/// loading the whole file into memory. Pinned by the
/// `streams_match_in_memory_hash` test below.
fn stream_sha256(path: &Path) -> std::io::Result<[u8; 32]> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1024 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let out = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&out);
    Ok(arr)
}

#[cfg(test)]
mod tests {
    // The end-to-end success path is exercised by
    // crates/nist-agent-release/src/install.rs::tests; the CLI
    // wrapper is a thin layer over that and doesn't need
    // duplicate coverage. We pin the operator-facing wording
    // shape (pinned refusal == "release signature invalid")
    // by re-asserting via the Display impl.
    use nist_agent_release::ReleaseError;

    use super::stream_sha256;
    use sha2::{Digest, Sha256};
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn refusal_wording_remains_stable() {
        assert_eq!(
            ReleaseError::ReleaseSignatureInvalid.to_string(),
            "release signature invalid"
        );
    }

    #[test]
    fn streams_match_in_memory_hash() {
        // Streaming and in-memory hash must agree byte-for-byte
        // across the 1 MiB chunk boundary — pin a 3-chunk input
        // so the loop body is exercised.
        let mut f = NamedTempFile::new().unwrap();
        let payload = vec![0xA5u8; 1024 * 1024 * 3 + 17];
        f.write_all(&payload).unwrap();
        f.flush().unwrap();

        let streamed = stream_sha256(f.path()).unwrap();
        let in_memory = {
            let mut h = Sha256::new();
            h.update(&payload);
            let mut a = [0u8; 32];
            a.copy_from_slice(&h.finalize());
            a
        };
        assert_eq!(streamed, in_memory);
    }

    #[test]
    fn streams_handles_empty_file() {
        let f = NamedTempFile::new().unwrap();
        let streamed = stream_sha256(f.path()).unwrap();
        // SHA-256 of empty input.
        let expected =
            hex::decode("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
                .unwrap();
        assert_eq!(&streamed[..], &expected[..]);
    }

    // ---- NIST_AGENT-2026-05-31-006 (order-of-operations /
    // path-control-before-authentication) adversarial tests ----
    //
    // RED protocol: pre-fix, `run()` hashed every manifest-named
    // artifact path BEFORE the manifest signature was verified,
    // and `Path::join` let an absolute or `../` artifact.path
    // escape the bundle dir. Pinned contract:
    //   1. a bad signature aborts before ANY artifact IO;
    //   2. traversal / absolute artifact paths are refused even
    //      on a validly-signed manifest.

    use super::{run, InstallArgs};
    use ed25519_dalek::SigningKey;
    use nist_agent_release::{
        ArtifactEntry, ArtifactKind, ReleaseManifest, ReleaseSigner, SignatureEnvelope,
    };

    fn release_key() -> SigningKey {
        SigningKey::from_bytes(&[11u8; 32])
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        let mut h = Sha256::new();
        h.update(bytes);
        format!("0x{}", hex::encode(h.finalize()))
    }

    fn manifest_with_artifact(path: &str, content: &[u8]) -> ReleaseManifest {
        ReleaseManifest {
            version: "0.1.0".into(),
            git_rev: "deadbeef".into(),
            artifacts: vec![ArtifactEntry {
                kind: ArtifactKind::Other,
                path: path.into(),
                sha256: sha256_hex(content),
            }],
            signature: SignatureEnvelope::empty_ed25519(),
        }
    }

    fn write_bundle(dir: &std::path::Path, manifest: &ReleaseManifest) {
        std::fs::write(
            dir.join("release.manifest.toml"),
            manifest.to_toml().expect("toml"),
        )
        .expect("write manifest");
    }

    fn args_for(bundle_dir: std::path::PathBuf) -> InstallArgs {
        InstallArgs {
            bundle_dir,
            release_pubkey_hex: hex::encode(release_key().verifying_key().to_bytes()),
            dry_run: false,
        }
    }

    #[test]
    fn safe_bundle_path_allows_normal_refuses_escape() {
        use super::safe_bundle_path;
        let base = std::path::Path::new("/bundle");
        assert!(safe_bundle_path(base, "bin/citrate-agent").is_some());
        assert!(safe_bundle_path(base, "./models/m.gguf").is_some());
        assert!(safe_bundle_path(base, "../escape").is_none());
        assert!(safe_bundle_path(base, "a/../../escape").is_none());
        assert!(safe_bundle_path(base, "/etc/shadow").is_none());
    }

    #[test]
    fn bad_signature_aborts_before_any_artifact_io() {
        // Artifact deliberately ABSENT from the bundle: pre-fix
        // the CLI tried to hash it (artifact IO on an unverified
        // manifest) and errored on the missing file; post-fix the
        // signature check refuses first and run() exits 2 without
        // touching any artifact path.
        let tmp = tempfile::tempdir().unwrap();
        let manifest = manifest_with_artifact("bin/citrate-agent", b"never written");
        // Unsigned (zeroed envelope) == invalid signature.
        write_bundle(tmp.path(), &manifest);
        let code = run(args_for(tmp.path().to_path_buf()))
            .expect("signature refusal must precede artifact IO");
        assert_eq!(code, 2, "invalid signature must refuse install");
    }

    #[test]
    fn traversal_artifact_path_refused_even_when_signed() {
        // Validly-signed manifest naming "../escape" with the
        // CORRECT hash of the escaped file: pre-fix this verified
        // and exited 0 (reading outside the bundle); post-fix the
        // path is refused.
        let outer = tempfile::tempdir().unwrap();
        let bundle = outer.path().join("bundle");
        std::fs::create_dir(&bundle).unwrap();
        let secret = b"outside the bundle";
        std::fs::write(outer.path().join("escape"), secret).unwrap();

        let manifest = manifest_with_artifact("../escape", secret);
        let signed = ReleaseSigner::new(release_key()).sign(manifest).unwrap();
        write_bundle(&bundle, &signed);

        let code = run(args_for(bundle)).expect("traversal refusal must not be an IO error");
        assert_eq!(code, 2, "`..` artifact path must be refused");
    }

    #[test]
    fn absolute_artifact_path_refused_even_when_signed() {
        let outer = tempfile::tempdir().unwrap();
        let bundle = outer.path().join("bundle");
        std::fs::create_dir(&bundle).unwrap();
        let secret = b"absolute target";
        let abs = outer.path().join("victim.txt");
        std::fs::write(&abs, secret).unwrap();

        let manifest = manifest_with_artifact(abs.to_str().unwrap(), secret);
        let signed = ReleaseSigner::new(release_key()).sign(manifest).unwrap();
        write_bundle(&bundle, &signed);

        let code = run(args_for(bundle)).expect("absolute-path refusal must not be an IO error");
        assert_eq!(code, 2, "absolute artifact path must be refused");
    }
}
