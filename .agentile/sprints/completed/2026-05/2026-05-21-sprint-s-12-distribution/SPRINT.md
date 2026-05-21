---
created: 2026-05-20T00:00:00Z
branch: feat/s-12-distribution
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-12
---

# Sprint S-12: Distribution and runbooks

**Goal.** Make every artifact byte-reproducible, every release HSM-
signed, the bundled GGUF separately hashable, the air-gap install
working from a single signed bundle, and Phase-1 overlay runbooks
landed.

**Why now.** Until S-12 closes, "we ship a sidecar" is aspirational.
Operators need an installer they can verify offline.

**Predecessors.** S-3, S-5 (model bundling), S-8 + S-9 (overlay
bundles to ship), S-10 (Slint to ship), S-11 (mobile to ship).

**Features owned.**
- `features/distribution/dist-reproducible-builds.feature`
- `features/distribution/dist-signed-releases.feature`
- `features/distribution/dist-bundled-gemma4.feature`
- `features/distribution/dist-airgap-install.feature`
- `features/distribution/dist-overlay-runbooks.feature`
- `features/surfaces/surface-cli.feature`
- `features/surfaces/surface-daemon.feature`
- `features/surfaces/surface-wasm-publish.feature`

**Cross-repo.** None unless the GGUF artifact moves to a separate
release repo (revisit at sprint kickoff).

**Exit criteria.**
- Two-machine reproducibility test passes.
- Release pipeline produces signed bundle + SBOM on tag push.
- Installer verifies release signature; refuses on failure.
- Doctor verifies running daemon hash against release manifest.
- Six overlay runbooks landed under `docs/compliance/<overlay>/RUNBOOK.md`.
- Air-gap install completes end-to-end on a host with no network.

## Close note — 2026-05-21

Status: **COMPLETE** (Rust verifier layer; CI/HSM infra deferred).

ADR-011 records the explicit scope split: this sprint lands the
**Rust verification logic** the operator runs on the target host
— manifest, signature verifier, installer, GGUF SI-7 check,
daemon-hash doctor check, egress-posture default. The producer
side (HSM-held key, GitHub Actions tag-push workflow, GGUF mirror
distribution, two-machine reproducibility test infrastructure) is
deferred to S-12b / external infra work.

**Delivered.**
- `crates/nist-agent-release` (13th workspace crate).
- `manifest.rs` — `ReleaseManifest` TOML schema with `[[artifact]]`
  entries (Daemon / Model / ModelLicense / Capsule / Sbom /
  Runbook / Other), detached `[signature]` envelope, deterministic
  signing payload (pre-signature form).
- `signature.rs` — `ReleaseSigner` (soft-key, for tests + the
  deferred fixture corpus) + `ReleaseVerifier` (operator side,
  pure-Rust). Single stable error class
  `ReleaseSignatureInvalid` regardless of sub-failure.
- `model.rs` — `ModelIntegrity::verify_gguf()` SI-7 check.
  Refuses with `"SI-7: model hash mismatch"` verbatim per
  `dist-bundled-gemma4.feature`.
- `install.rs` — `Installer::verify_and_install()`. Refuses
  with `"release signature invalid"` verbatim per
  `dist-signed-releases.feature`. Per-artifact hash check
  layered after the signature check.
- `egress.rs` — `EgressPosture::Disabled` default per RFC §3.3
  G1; SecurityOfficer-signed `EgressDirective` flips to
  `Enabled` via injected verifier callback.
- `daemon_hash.rs` — `DaemonHashCheck::verdict()` returns
  `Ok / Mismatch / NoDaemonEntry`. Doctor consumes; BLOCKER per
  `dist-reproducible-builds.feature`.
- All six overlay RUNBOOKs now carry `## Rollback` sections per
  `dist-overlay-runbooks.feature` (5 backfills: ferpa/coppa/
  cipa/hipaa/fedramp-high). Three-step shape verified by
  `tests/runbook_coverage.rs`.

**Metrics.**
- `cargo test --workspace` 167 → 202 (+35).
- Workspace crates 12 → 13.
- ADRs 10 → 11 (added ADR-011).

**Exit criteria — final state.**
- ⏸ Two-machine reproducibility test — DEFERRED (CI infra).
  Rust code is reproducibility-clean (no embedded build
  timestamps).
- ⏸ Release pipeline produces signed bundle + SBOM on tag push —
  DEFERRED (CI workflow + HSM provisioning).
- ✅ Installer verifies release signature; refuses on failure
  with pinned wording.
- ✅ Doctor verifies running daemon hash against release manifest
  (`DaemonHashCheck`).
- ✅ Six overlay runbooks present + all carry Rollback sections
  with the three-step shape.
- ⏸ Air-gap install completes end-to-end — DEFERRED (requires
  built bundle + offline test host).

**Carried into S-12b / S-13.**
- GitHub Actions workflow on `v\d+\.\d+\.\d+` tag push: build,
  produce SBOM, sign via HSM, upload bundle. Mechanical once HSM
  is provisioned; consumes the verifier this sprint shipped.
- Reproducible-build VM provisioning (rust-toolchain pinned,
  hermetic container).
- GGUF download path + license bundling. Operator-facing — pick
  USB-stick vs. internal-mirror at packaging time.
- Two-machine bit-for-bit comparison test.
- Trail of Bits engagement (S-13) gets a focused ~1 kLOC
  verifier crate to review.
