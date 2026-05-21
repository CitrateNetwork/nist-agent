---
created: 2026-05-21T00:00:00Z
branch: feat/s-12-distribution
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-12
---

# Retro — Sprint S-12 (distribution: Rust verifier + runbook backfills)

## What landed

The Rust contract for "what counts as a valid release" — 13th
workspace crate `nist-agent-release`. Operators who run
`citrate-agent install <bundle>` get pinned-wording refusal
messages on every failure path (`"release signature invalid"`,
`"SI-7: model hash mismatch"`), pure-Rust, no network, no HSM.

ADR-011 sets the scope: this sprint is the verifier, S-12b is
the CI workflow + HSM ceremony, S-13 is Trail of Bits on the
verifier crate. Trying to do all three in S-12 would mix four
toolchains and four audit boundaries; the split keeps each
piece small enough to ship clean.

The runbook backfill closed the second half of the sprint:
five of the six overlay RUNBOOKs were missing the `## Rollback`
section the feature scenario requires. The new
`tests/runbook_coverage.rs` integration test pins both the
existence and the three-step shape, so a future doc edit can't
silently strip the rollback procedure.

## Metrics delta

| Metric | Before (S-11) | After (S-12) | Delta |
|---|---|---|---|
| `cargo test --workspace` | 167 | 202 | +35 |
| Workspace crates | 12 | 13 | +1 |
| ADRs | 10 | 11 | +1 (ADR-011) |
| TLA+ specs | 5 | 5 | 0 |
| Overlay runbooks with Rollback | 1/6 | 6/6 | +5 |

## What went well

- **Pinned-wording approach paid off again.** Same shape S-11
  used for the mobile error strings: a test that compares
  `err.to_string()` against the exact phrase from the feature
  scenario catches drift before it reaches an operator. Trivial
  cost, large insurance value.
- **Detached-signature with pre-signature payload form is the
  right shape.** Sign over a payload where the signature field
  is zeroed → any verifier reconstructing the payload computes
  the same bytes regardless of how the manifest was serialized
  upstream. Avoids a class of "sig over sig" bugs without
  needing a canonical-JSON / canonical-CBOR specification.
- **Single error class for all signature failures
  (`ReleaseSignatureInvalid`).** The feature scenario specifies
  one operator-facing reason; the verifier maps every internal
  failure (key mismatch, decode error, verify failure) to that
  one error. Operators can't disambiguate, but they shouldn't —
  any sub-failure means "don't install this bundle."
- **Soft-key signer as a fixture for the verifier.** Production
  uses an HSM; the test path uses a deterministic Ed25519 key.
  The verifier sees both as the same trait; the only divergent
  code is the signer construction. Means the HSM integration
  can land in S-12b without re-touching the verifier.

## What was tricky

- **Clippy's `derivable_impls` pushed `impl Default` into the
  derive.** Same pattern that hit S-10a (Step) and S-10c
  (MarketplaceSource). The lesson: any enum whose first
  variant is the "off" state should get
  `#[derive(Default)] + #[default] Off`. Worth landing in a
  shared `#[cfg(test)]` lint policy at some point.
- **`Result<ReleaseVerifier, ReleaseError>::unwrap_err` requires
  `Debug` on `T`.** Caught by clippy. Added `#[derive(Debug)]`
  to `ReleaseVerifier`. Small, but reminded me that public
  structs should default to Debug-derive unless there's a
  specific reason not to (private key material would be a
  reason; a verifier is not).
- **Runbook content-checking integration test was too strict
  initially.** First pass required every runbook to contain
  literal `content_hash` and `anchor` strings. Three runbooks
  inherit those concepts from the CMMC-L3 baseline without
  re-stating them. Loosened to `capsule` + `anchor|worm` which
  catches the load-bearing references without over-specifying.
  Lesson: integration tests over docs should check intent, not
  surface form, when the docs have a layered inheritance
  pattern.
- **Test count jumped +35.** Larger than recent sprints (+28
  for S-11, +18 for S-10c). Five modules each carry their own
  serde + happy-path + refusal tests; the per-module count is
  in line with prior sprints, the workspace delta just reflects
  the wider module surface this sprint owned.

## What to carry into S-12b / S-13

- **S-12b.** GitHub Actions workflow on tag push. Mechanical:
  build, hash, produce SBOM, hand the manifest off to the HSM
  for the detached signature, upload the bundle. The verifier
  contract is what the workflow targets.
- **S-12b.** Reproducible-build VM. Hermetic container with
  pinned Rust toolchain + locked deps. Two-machine bit-for-bit
  test runs inside CI.
- **S-12b.** GGUF distribution path. The manifest references a
  sha256; how the file reaches the operator (CDN mirror,
  internal artifact store, USB) is operator-side. Document the
  recommendation in `docs/compliance/<overlay>/RUNBOOK.md`'s
  "Deployment artifacts" section.
- **S-13.** Trail of Bits engagement scope can include the
  `nist-agent-release` crate as a focused ~1 kLOC review. Pure
  Rust, no FFI, no async, no network — small audit surface.
- **Doctor wiring.** The doctor crate should consume
  `DaemonHashCheck::verdict()` as a new `Severity::Blocker`
  check on every start. Landing this in S-12b avoids touching
  the doctor crate twice.

## Open follow-ups

- **Canonical TOML encoding.** `toml::to_string` produces stable
  output for our types, but a strictly-deterministic encoder
  would harden the signing-payload reconstruction story.
  Consider `toml_edit` for v1.1 if a non-stable case appears.
- **Algorithm upgrade path.** `SignatureEnvelope.algorithm` is
  carried for the inevitable post-quantum transition; we should
  decide on the v1.1 algorithm (Dilithium? hybrid Ed25519 +
  Dilithium?) before we ship v1.0 and lock the schema.
- **SBOM signing.** Feature scenario says "SBOM is itself
  signed". S-12b handles this; this crate's manifest already
  carries an `ArtifactKind::Sbom` entry with its own sha256,
  which is the verifier's anchor — the producer side decides
  whether to sign the SBOM separately or rely on the manifest
  signature covering it transitively.
