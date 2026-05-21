---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
audience: Trail of Bits engagement team
---

# Build — nist-agent v1.0-rc

> How to build the workspace from source against the audit
> boundary commit. The Rust code is reproducibility-clean (no
> embedded build timestamps in our code paths); the producer-
> side CI workflow that delivers byte-reproducible bundles is
> deferred to S-12b per ADR-011.

## Audit boundary

Clone at:

```
git clone --recurse-submodules ssh://git@github.com/CitrateNetwork/nist-agent.git
cd nist-agent
git checkout 30044d26201daaabdeaaab8d7f25d950b336f023
```

The S-12 close commit is the v1.0-rc audit boundary. Any S-13
remediation commits will land on top of this and are tagged
with `S-13a` in commit messages.

## Toolchain

Pinned in `rust-toolchain.toml` at the workspace root. The
workspace MSRV is `1.78`; CI builds against the toolchain file's
exact rev to avoid silent drift. To install:

```sh
rustup show     # honors rust-toolchain.toml automatically
rustc --version # expect 1.78.x or later as pinned
```

## Cross-repo dependencies (pinned by rev)

| Crate | Source repo | Pinned rev |
|---|---|---|
| `citrate-agent-core` | `CitrateNetwork/citrate-agent-runtime` | `2591ed2d28780ca7938befe853be7cd1620029cc` |
| `citrate-wallet-core` | `CitrateNetwork/citrate-chain` | `0f2d16b486a9ec1ed8a0394440dac480075d6942` |

These match the federation manifest entries; drift-check
enforces equality in CI. The federation's
[`drift-check.yml`](https://github.com/CitrateNetwork/citrate-federation/blob/main/.github/workflows/drift-check.yml)
runs on every PR.

SSH config (operator must have `~/.ssh/config` entries) is
documented at the workspace's `INSTALL.md`. Auditors with
read-only deploy keys: contact per [`CONTACT.md`](CONTACT.md).

## Build commands

```sh
# Full workspace build.
cargo build --workspace

# Workspace tests (canonical command; pinned in baseline.json).
cargo test --workspace
# Expected: 202 tests pass at S-12 boundary.

# Lint (zero warnings policy).
cargo clippy --workspace --all-targets -- -D warnings

# Format check.
cargo fmt --all -- --check

# Build with Slint feature enabled.
cargo build --workspace --features feat-ui
# Two known Window-inheritance warnings on MarketplacePane /
# ChatPane — cosmetic, integration site wraps panes in a Window.
```

## Formal verification

TLC verification of the 5 normative TLA+ specs:

```sh
cd .agentile/formal
./scripts/verify-all.sh
```

Each spec runs with `BOUNDED_EXPLORATION` parameters; current
bounds are documented in each `.cfg` file. The ratchet
(`scripts/ci/check_specs.py`) verifies that no spec count
regresses across a sprint.

## Reproducibility status

What's reproducible **today** without S-12b:

- The Rust workspace itself: same `cargo build --workspace`
  on two machines with the same toolchain + `Cargo.lock`
  produces functionally-identical binaries. Strict byte-
  reproducibility requires the deferred hermetic-container
  CI (S-12b).
- The 5 TLA+ specs verify deterministically under TLC.
- The 6 overlay runbooks render deterministically as Markdown.

What requires S-12b:

- Strict byte-for-byte reproducible binary across two clean
  hermetic build environments.
- HSM-signed release manifest + detached `.sig` files for the
  bundled tarball.
- SBOM in SPDX or CycloneDX, itself signed.
- Verified install on an air-gapped host.

The verifier the deferred CI hands off to is the audit
boundary's `nist-agent-release` crate; an auditor can exercise
the verifier locally against a soft-key-signed test manifest
(see `crates/nist-agent-release/src/signature.rs`'s test module
for the fixture pattern).

## Toolchain hashes (reproducibility evidence)

Recorded in `Cargo.lock` (deterministic resolution) and the
`rust-toolchain.toml` pin. Auditors who want to validate the
deps tree:

```sh
cargo tree --workspace --duplicates  # surface any duplicate-version drift
cargo audit                          # known-vulnerability scan via RustSec
cargo deny check                     # license/ban policy (cargo-deny config TBD)
```

Auditor-facing recommendation: run `cargo audit` against
this commit and flag any advisories. We track advisories at
`.agentile/audits/findings/` (currently empty; populated as
TOB delivers findings).

## Test surface map

Where each acceptance criterion is tested:

| Criterion | Test location |
|---|---|
| HITL install-gate (AC-6) | `crates/nist-agent-hitl/src/inspector.rs::install_gate_blocks_until_every_field_viewed` |
| Overlay ratchet refuses CMMC-L3 removal | `crates/nist-agent-policy/src/overlay_state.rs::tests` |
| Pairing HITL-gated | `crates/nist-agent-mobile-pairing/src/pairing.rs::out_of_order_security_officer_is_refused` |
| Mobile TTL 1-hour boundary | `crates/nist-agent-mobile-pairing/src/eligibility.rs::signature_at_ttl_boundary_is_accepted` |
| Samsung Knox refused | `crates/nist-agent-mobile-pairing/src/attestation.rs::samsung_knox_refused_by_default_per_feature_scenario` |
| FedRAMP High forbids mobile | `crates/nist-agent-mobile-pairing/src/eligibility.rs::fedramp_high_forbids_mobile_per_feature_table` |
| GGUF SI-7 wording pinned | `crates/nist-agent-release/src/model.rs::tampered_gguf_fails_with_exact_si7_wording` |
| Release sig refusal wording | `crates/nist-agent-release/src/install.rs::install_refuses_with_pinned_wording_on_bad_signature` |
| Egress disabled by default | `crates/nist-agent-release/src/egress.rs::default_posture_is_disabled_per_rfc_section_3_3_g1` |
| Overlay-certification pre-filter | `crates/nist-agent-marketplace/src/browser.rs::constructor_filters_listings_not_certified_for_any_active_overlay` |
| Trajectory export forbidden under FedRAMP High | `crates/nist-agent-marketplace/src/chat.rs::export_forbidden_when_fedramp_high_active` |
| Runbook Rollback present (all 6) | `crates/nist-agent-release/tests/runbook_coverage.rs::every_runbook_has_a_rollback_section` |

## See also

- [`SCOPE.md`](SCOPE.md) — what to build, what to ignore.
- [`DEPLOYMENT.md`](DEPLOYMENT.md) — once built, how to run it.
- [`INSTALL.md`](../../INSTALL.md) — operator-facing install walkthrough (precedes this audit packet).
