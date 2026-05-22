//! `gen-release-key` — one-shot helper that produces a fresh
//! Ed25519 keypair for the release-signing soft-key seam.
//!
//! Stdout: the 32-byte secret key as hex (set as the
//! `RELEASE_SIGNING_KEY_HEX` GitHub Actions secret; never
//! commit). Stderr: human-readable banner with both the
//! private + public key for one-time inspection.
//!
//! Usage:
//!
//!   cargo run --manifest-path scripts/release/Cargo.toml \
//!     --bin gen-release-key 2>/dev/null \
//!     | gh secret set RELEASE_SIGNING_KEY_HEX -R CitrateNetwork/nist-agent
//!
//! The public key half should be published in operator-facing
//! docs (CitrateNetwork/.github/SECURITY.md) so installers can
//! configure their trust root via `--release-pubkey-hex`.

use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::TryRngCore;

fn main() {
    let mut bytes = [0u8; 32];
    OsRng
        .try_fill_bytes(&mut bytes)
        .expect("OsRng must succeed");
    let sk = SigningKey::from_bytes(&bytes);
    let pk = sk.verifying_key();

    eprintln!("nist-agent release-signing soft key (Ed25519)");
    eprintln!("===============================================");
    eprintln!("Private (set as RELEASE_SIGNING_KEY_HEX secret):");
    eprintln!("  {}", hex::encode(bytes));
    eprintln!("");
    eprintln!("Public  (publish to SECURITY.md / operator trust root):");
    eprintln!("  {}", hex::encode(pk.to_bytes()));
    eprintln!("");
    eprintln!("Posture: SOFT KEY (pre-production). HSM swap-in:");
    eprintln!("  docs/audit/HSM_SEAM.md");

    // Stdout = just the private hex so `gh secret set` can pipe.
    println!("{}", hex::encode(bytes));
}
