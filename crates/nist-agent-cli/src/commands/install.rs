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
use std::path::PathBuf;

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
        let bytes =
            fs::read(&path).with_context(|| format!("read bundle artifact {}", path.display()))?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let out = hasher.finalize();
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&out);
        measured.insert(artifact.path.clone(), arr);
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

#[cfg(test)]
mod tests {
    // The end-to-end success path is exercised by
    // crates/nist-agent-release/src/install.rs::tests; the CLI
    // wrapper is a thin layer over that and doesn't need
    // duplicate coverage. We pin the operator-facing wording
    // shape (pinned refusal == "release signature invalid")
    // by re-asserting via the Display impl.
    use nist_agent_release::ReleaseError;

    #[test]
    fn refusal_wording_remains_stable() {
        assert_eq!(
            ReleaseError::ReleaseSignatureInvalid.to_string(),
            "release signature invalid"
        );
    }
}
