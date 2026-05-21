---
created: 2026-05-21T00:00:00Z
branch: feat/s-12b-ci-infra-and-cli
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Release engineer rotating from soft-key to HSM-backed release signing
---

# HSM Seam — Production Release Signing

> How to swap the S-12b soft-key release signer for an HSM-
> backed one without touching the verifier (which the operator
> + the audit care about). Per ADR-011 the verifier is the
> contract; the producer side iterates independently.

## The seam

The release pipeline has exactly one signing primitive:

```rust
SigningKey::from_bytes(&[u8; 32])
    .sign(&payload)
```

That's the only line that changes when you move from soft key
to HSM. Everything upstream of it (the manifest builder, the
artifact hashing, the SBOM, the staging tarball) and
everything downstream of it (the verifier, the installer, the
operator's trust root) stays unchanged.

The S-12b workflow's signer is at
[`scripts/release/src/main.rs`](../../scripts/release/src/main.rs).
The HSM swap-in replaces its signing-key construction step.

## Where the seam lives

```text
                   ┌──────────────────────────────────────────┐
                   │  Hashing + manifest assembly              │
                   │  scripts/release/build_manifest.py        │
                   │  → release.manifest.toml (pre-signature)  │
                   └────────────────┬─────────────────────────┘
                                    │
                   ┌────────────────▼─────────────────────────┐
                   │  ★ SEAM: scripts/release/sign-manifest    │
                   │                                            │
                   │  TODAY (S-12b):                            │
                   │    let key = SigningKey::from_bytes(...);  │
                   │    let sig = key.sign(payload);            │
                   │                                            │
                   │  PRODUCTION (S-12c+):                      │
                   │    let key = HsmEd25519::open("slot 0");   │
                   │    let sig = key.sign(payload);            │  ◄── same trait
                   │                                            │
                   └────────────────┬─────────────────────────┘
                                    │
                   ┌────────────────▼─────────────────────────┐
                   │  Bundle + upload (workflow YAML)          │
                   │  unchanged — output is the same TOML       │
                   └──────────────────────────────────────────┘
```

`ed25519_dalek::SigningKey` and `ed25519_dalek::Signer` are
both trait-shaped, so a drop-in HSM signer that implements
`Signer<Signature>` over an opaque handle is a 30-line wrapper.

## What an HSM-backed signer looks like

Two common shapes:

### Option A — PKCS#11 (YubiHSM, SafeNet, etc.)

Use `pkcs11` or `cryptoki` crates. The pattern:

```rust
use cryptoki::{Pkcs11, slot::Slot, session::UserType, mechanism::Mechanism};
use ed25519_dalek::{Signature, Signer};

struct HsmEd25519 {
    session: cryptoki::session::Session,
    key_handle: cryptoki::object::ObjectHandle,
}

impl Signer<Signature> for HsmEd25519 {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, ed25519_dalek::SignatureError> {
        let bytes = self
            .session
            .sign(&Mechanism::Eddsa, self.key_handle, msg)
            .map_err(|_| ed25519_dalek::SignatureError::new())?;
        Signature::from_slice(&bytes)
            .map_err(|_| ed25519_dalek::SignatureError::new())
    }
}
```

Then in `sign-manifest`:

```rust
let signer = HsmEd25519::open(slot_id, pin)?;
let sig = signer.sign(&payload);
```

That's the whole swap.

### Option B — Cloud HSM (AWS KMS, GCP KMS)

KMS endpoints expose REST APIs that produce Ed25519 sigs from
a key-arn handle. Pattern:

```rust
struct KmsEd25519 {
    client: aws_sdk_kms::Client,
    key_arn: String,
}

impl KmsEd25519 {
    async fn sign(&self, msg: &[u8]) -> Result<Signature, anyhow::Error> {
        let resp = self.client.sign()
            .key_id(&self.key_arn)
            .message(aws_smithy_types::Blob::new(msg))
            .signing_algorithm(SigningAlgorithmSpec::EcdsaSha256)  // Ed25519 = ED25519 for KMS
            .send().await?;
        let sig_bytes = resp.signature.unwrap();
        Signature::from_slice(sig_bytes.as_ref())
    }
}
```

KMS is convenient for serverless CI; PKCS#11 is the reference
posture for FedRAMP / FIPS 140-3.

## Verifier side: no change

The operator's installer (`nist-agent-release::Installer`)
verifies the manifest's `signature_hex` against the manifest's
declared `public_key_hex` AND against the operator's
configured trust root. The verifier neither knows nor cares
how the signature was produced — it just runs
`ed25519_dalek::Verifier::verify()` over the
canonical signing payload.

This is the load-bearing audit property: the verifier surface
is the contract; the signer surface is the implementation.

## Steps to swap (concrete checklist)

1. **Provision the HSM.** Out of scope for this repo.
   FedRAMP/FIPS sites should follow the vendor's key-ceremony
   docs + record the ceremony as a `PolicyChange` audit event
   per RFC §6.

2. **Generate the release-signing keypair on the HSM.**
   Algorithm: Ed25519. Capability: sign + verify. Do **not**
   permit export. Annotate the slot with the v1.0-rc release
   purpose so a future engineer doesn't confuse it with the
   SecurityOfficer policy-bundle key.

3. **Capture the public key.** Export the verifying-key bytes
   (32 bytes); publish to
   `CitrateNetwork/.github/SECURITY.md` so operators have a
   durable trust root to configure
   `NIST_AGENT_RELEASE_PUBKEY_HEX` against.

4. **Author the `HsmEd25519` wrapper.** ~30 lines per the
   example above. Lands in `scripts/release/src/hsm.rs`.

5. **Add a feature flag** to `scripts/release/Cargo.toml`:

   ```toml
   [features]
   default = []
   hsm-pkcs11 = ["dep:cryptoki"]
   hsm-kms = ["dep:aws-sdk-kms"]
   ```

6. **Update `scripts/release/src/main.rs`** to branch on env
   var `RELEASE_SIGNER`:

   ```rust
   match std::env::var("RELEASE_SIGNER").as_deref() {
       Ok("soft") | Err(_) => sign_with_soft_key(...),
       Ok("hsm-pkcs11") => sign_with_hsm(...),
       Ok("hsm-kms") => sign_with_kms(...),
   }
   ```

7. **Update `.github/workflows/release.yml`** to use the HSM
   path on the production tag-push:

   ```yaml
   - name: sign release manifest (HSM)
     env:
       RELEASE_SIGNER: hsm-pkcs11
       PKCS11_MODULE: /opt/yubihsm/libyubihsm_pkcs11.so
       PKCS11_PIN: ${{ secrets.PKCS11_PIN }}
       PKCS11_SLOT: ${{ secrets.PKCS11_SLOT_ID }}
     run: |
       cargo run --release --bin sign-manifest \
         --features hsm-pkcs11 \
         -- --input release-staging/release.manifest.toml
   ```

8. **Remove the "pre-release / not for production" warning**
   from the release notes. Flip `prerelease: true` to
   `prerelease: false` in `softprops/action-gh-release@v2`
   for the first stable v1.0.0 tag.

9. **Update `docs/audit/V1_READINESS.md`** to mark
   exit criterion 6 (federation pin updated to `v1.0.0-rc`)
   as gated only on TOB sign-off + S-14 pilots — the HSM
   swap closes the v1.0-rc release-signing prerequisite.

## What happens during the swap window

Between S-12b close and the HSM swap-in, releases produced by
the workflow are signed by a **soft key stored as a GitHub
Actions secret** (`RELEASE_SIGNING_KEY_HEX`). These are
pre-production bundles marked `prerelease: true`. Operators
should treat them as "audit-track only" — fine for the air-
gap test, fine for TOB to receive, not fine for pilot
deployments.

The verifier does not distinguish — a soft-key signature
verifies identically to an HSM signature *given the same
public key*. Distinguishing happens at the trust-root level:
`CitrateNetwork/.github/SECURITY.md` publishes the production
trust root only once the HSM ceremony completes.

## Cross-references

- [`AIRGAP_TEST.md`](AIRGAP_TEST.md) — uses the verifier path
  which is identical on both sides of the seam.
- [`SCOPE.md`](SCOPE.md) — HSM key custody is out of scope for
  TOB engagement.
- ADR-011 — the verifier scope split.
- `scripts/release/src/main.rs` — the seam.
- `crates/nist-agent-release/src/signature.rs` — the verifier.
