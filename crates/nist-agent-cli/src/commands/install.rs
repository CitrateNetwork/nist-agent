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
//! The CLI hashes each artifact named in the manifest, calls
//! `Installer::verify_and_install()`, and on success prints a
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

    let mut measured: HashMap<String, [u8; 32]> = HashMap::new();
    for artifact in &manifest.artifacts {
        let path = args.bundle_dir.join(&artifact.path);
        let digest = stream_sha256(&path)
            .with_context(|| format!("hash bundle artifact {}", path.display()))?;
        measured.insert(artifact.path.clone(), digest);
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
}
