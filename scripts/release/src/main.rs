//! `sign-manifest` — soft-key Ed25519 signer for
//! `release.manifest.toml`. Consumes the pre-signature manifest
//! produced by `build_manifest.py` and writes a signed manifest
//! in place (or to `--out`).
//!
//! The signing key is supplied via the `RELEASE_SIGNING_KEY_HEX`
//! environment variable (32-byte Ed25519 secret key, hex-
//! encoded). When unset, the tool emits a warning and writes
//! the manifest in its pre-signature form so the operator sees
//! the bundle is UNSIGNED.
//!
//! HSM swap-in: replace this binary with an HSM-backed signer
//! that implements the same input/output contract. See
//! `docs/audit/HSM_SEAM.md`.

use anyhow::{Context, Result};
use clap::Parser;
use ed25519_dalek::{Signer, SigningKey};
use nist_agent_release::{ReleaseManifest, SignatureEnvelope};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "sign-manifest", version, about)]
struct Args {
    /// Path to the pre-signature release.manifest.toml.
    #[arg(long)]
    input: PathBuf,

    /// Path to write the signed manifest. Defaults to overwrite
    /// `--input`.
    #[arg(long)]
    out: Option<PathBuf>,

    /// Hex-encoded Ed25519 secret key (32 bytes, 64 hex chars).
    /// Defaults to env var RELEASE_SIGNING_KEY_HEX.
    #[arg(long, env = "RELEASE_SIGNING_KEY_HEX")]
    key_hex: Option<String>,
}

fn main() -> Result<()> {
    // clap's #[arg(long, env=...)] doesn't see the env var
    // when parsing in `try_parse_from`; use Parser::parse so
    // the default env-var lookup runs.
    let args = Args::parse();
    let toml_in = fs::read_to_string(&args.input)
        .with_context(|| format!("read input {}", args.input.display()))?;
    let mut manifest = ReleaseManifest::from_toml(&toml_in)
        .with_context(|| format!("parse input {}", args.input.display()))?;

    let out_path = args.out.unwrap_or_else(|| args.input.clone());

    let key_hex = match args.key_hex {
        Some(k) if !k.is_empty() => k,
        _ => {
            eprintln!(
                "warning: RELEASE_SIGNING_KEY_HEX not set; writing UNSIGNED manifest. \
                 See docs/audit/HSM_SEAM.md for the production signing path."
            );
            // Reset signature to pre-signature form so consumers
            // can detect the unsigned state explicitly.
            manifest.signature = SignatureEnvelope::empty_ed25519();
            let s = manifest
                .to_toml()
                .context("re-encode pre-signature manifest")?;
            fs::write(&out_path, s)
                .with_context(|| format!("write {}", out_path.display()))?;
            return Ok(());
        }
    };

    let key_bytes = hex::decode(key_hex.trim().trim_start_matches("0x"))
        .context("decode RELEASE_SIGNING_KEY_HEX")?;
    if key_bytes.len() != 32 {
        anyhow::bail!(
            "RELEASE_SIGNING_KEY_HEX must be 32 bytes (64 hex chars); got {}",
            key_bytes.len()
        );
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&key_bytes);
    let signing_key = SigningKey::from_bytes(&arr);
    let verifying_key = signing_key.verifying_key();

    // Initialize signature envelope to pre-signature form, fill
    // in the public key, build the signing payload, then sign.
    manifest.signature = SignatureEnvelope::empty_ed25519();
    manifest.signature.public_key_hex = hex::encode(verifying_key.to_bytes());
    let payload = manifest
        .signing_payload()
        .context("build signing payload")?;
    let sig = signing_key.sign(&payload);
    manifest.signature.signature_hex = hex::encode(sig.to_bytes());

    let s = manifest.to_toml().context("encode signed manifest")?;
    fs::write(&out_path, s)
        .with_context(|| format!("write {}", out_path.display()))?;
    eprintln!(
        "signed {} → {} (pubkey {})",
        args.input.display(),
        out_path.display(),
        manifest.signature.public_key_hex
    );
    Ok(())
}
