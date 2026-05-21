---
created: 2026-05-21T00:00:00Z
branch: feat/s-12-distribution
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 011
sprint: S-12
---

# ADR-011: Distribution sprint lands the Rust verifier; CI/HSM infra is deferred

| Field | Value |
|---|---|
| **ADR Number** | ADR-011 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-12 |

## Context

S-12 is the "make every artifact byte-reproducible, every release
HSM-signed, the bundled GGUF separately hashable, the air-gap
install working from a single signed bundle, and Phase-1 overlay
runbooks landed" sprint. That ambition spans four kinds of work:

1. **Rust verification logic.** The code that *consumes* a signed
   release: parses the manifest, verifies detached signatures
   before extracting, hash-checks the GGUF before loading,
   compares the running daemon's executable against the manifest.
   Pure Rust; testable in headless CI.
2. **Release-pipeline CI infrastructure.** A GitHub Actions
   workflow (or equivalent) that runs on `v\d+\.\d+\.\d+` tag
   push, builds reproducibly on two machines, produces an SBOM,
   uploads the bundle, and obtains a detached signature from an
   HSM-held release key.
3. **HSM provisioning + key custody.** The actual hardware
   security module the release engineer holds. Procurement +
   physical-security policy + key-ceremony documentation.
4. **The bundled GGUF artifact.** The Gemma 4 E2B Q4_K_M GGUF
   itself (~1.6 GB), its license bundling, and its CDN / mirror
   distribution path.

Trying to land (1) through (4) in one sprint mixes Rust work,
GitHub Actions tinkering, procurement, and gigabyte binary
handling. They don't share a toolchain, a release cadence, or an
audit boundary.

## Decision

**S-12 lands (1) — the Rust verifier. (2), (3), (4) are
deferred to S-12b / external infra work, tracked but not part of
this sprint's exit criteria.**

The verifier code is the contract the operator runs on the
target host; everything else is a producer of the inputs that
verifier consumes. Once the verifier is shipped and audit-able,
the producer side can iterate (different CI vendors, key-ceremony
revisions, GGUF mirror changes) without touching the
nist-agent codebase.

### What S-12 ships

A new crate `nist-agent-release` with:

- `ReleaseManifest` — serde-pinned struct describing every
  shipped artifact (daemon binary, GGUF model, capsule .cps
  files, SBOM, runbooks) by SHA-256, plus a detached signature
  envelope.
- `Signer` / `Verifier` — Ed25519 signature wrappers (re-export
  from upstream's existing crypto layer) that produce / verify
  the detached signatures the release pipeline attaches.
- `ModelIntegrity::verify_gguf_hash()` — the SI-7 check from
  `dist-bundled-gemma4.feature`. Refuses to load a tampered GGUF
  with exactly `Err("SI-7: model hash mismatch")`.
- `Installer::verify_and_install()` — runs the signature check
  before extracting; refuses with exactly
  `Err("release signature invalid")` on bad sig.
- `EgressPosture::default()` — `Disabled`; opt-in via a signed
  SecurityOfficer directive (matches RFC §3.3 G1).
- `DaemonHashCheck` — doctor-side check that compares the
  running daemon executable's sha256 against the manifest.
- The five S-12 surface features (`dist-*`) pinned by tests.

The six overlay runbooks under `docs/compliance/<overlay>/` are
backfilled where missing (notably the `Rollback` section the
feature scenario requires) so all six pass
`dist-overlay-runbooks.feature`.

### What S-12 defers

- The release CI workflow itself. Authoring a Yaml file that
  invokes the verifier on tag push is mechanical once the
  verifier exists; lands in S-12b alongside any
  reproducible-build VM provisioning.
- HSM provisioning. Out of scope for a Rust workspace.
- The GGUF binary's storage + delivery path. The verifier reads a
  sha256 — *how* the file gets to the operator (USB stick, S3
  bucket, internal mirror) is an operator concern. We pin the
  hash via the manifest; ship the manifest in this repo's
  release; let downstream choose.
- The actual SBOM generator. Standard tooling (`cargo cyclonedx`
  / `cargo-sbom`) plugs in at CI time; we accept any SPDX or
  CycloneDX document the manifest references.
- End-to-end "two clean machines produce identical bytes"
  validation. The Rust code is reproducibility-clean (no
  embedded build timestamps in our code paths); the actual
  byte-for-byte check needs the CI workflow that's deferred.

## Consequences

**Positive.**

- The verifier is testable in CI today: hash check, signature
  verification, manifest decode, install refusal paths all run
  under `cargo test`.
- The exact error strings the feature scenarios specify
  (`"SI-7: model hash mismatch"`, `"release signature invalid"`)
  are pinned by test, so a serde / refactor can't drift them.
- Audit boundary is small. Trail of Bits reviews ~1 kLOC of
  pure-Rust verifier code; the CI workflow becomes a smaller,
  separate engagement (or accepts the verifier's contract).

**Negative.**

- The marketing line "S-12 closes distribution" is shifted: the
  *Rust contract* of distribution is closed; the *operational
  reality* needs S-12b for CI and S-12c for HSM ceremony.
- A consumer who reads only the sprint title might expect a
  workable `cargo release` flow. We document the deferred parts
  prominently in the sprint close note.

**Neutral.**

- The crate depends on `nist-agent-prelude` (for `Overlay`),
  `nist-agent-policy` (for `SecurityOfficer` signing-key types),
  and `nist-agent-doctor` (only as a downstream consumer for the
  daemon-hash check — the check itself lives in this crate so
  the doctor doesn't grow a new direct dependency).

## Alternatives considered

1. **Land everything in S-12.** Rejected — five different
   toolchains, audit boundaries, and release cadences.
   Sprint-shaped work fights the work.
2. **Skip the verifier; rely on operators using `cosign` /
   `sigstore`.** Rejected — the contract is product-shaped
   (specific overlay defaults, specific GGUF SI-7 wording, a
   specific manifest schema) and the verifier owns it. Generic
   tooling can produce the signatures upstream of our verifier;
   we don't owe operators a re-implementation.
3. **Put the verifier upstream in `citrate-agent-core`.**
   Rejected per the same logic as ADR-008/-009/-010: the
   manifest and runbook contract are product-shaped. The engine
   owns generic crypto primitives (`Ed25519` signing); the
   product owns the manifest format and the install flow.

## Reversal conditions

If a second product in the federation ships a similarly shaped
signed-bundle install flow and the two manifests diverge in
unsustainable ways, the verifier migrates to a
`citrate-release-kit` upstream crate. Until that's concrete,
the verifier stays here.

## References

- RFC-CIT-AGENT-0001 §3.3 (G1 egress posture), §8.2 (bundled
  model), §11.1 (release engineering posture).
- ADR-008 / ADR-009 / ADR-010 — sibling
  product-stays-here decisions.
- `features/distribution/*.feature` — the five distribution
  scenarios this crate's tests pin.
- `crates/nist-agent-release/` — the crate this ADR governs.
- Federation rule 9 (one source of truth — verifier is the
  product's source of truth for what a valid release looks
  like).
