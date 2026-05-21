---
created: 2026-05-21T00:00:00Z
branch: feat/s-12b-ci-infra-and-cli
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: complete
sprint: S-12b
---

# Sprint S-12b: CI infrastructure + operator CLI for on-prem air-gap test

**Goal.** Stand up the deferred S-12 infrastructure that turns
the v1.0-rc audit-boundary code into an *operationally usable*
release pipeline: an operator-facing CLI binary, real
llama.cpp inference behind a feature flag, an air-gap on-prem
test runbook, a GitHub Actions tag-push release workflow, a
hermetic-builder Dockerfile, and the HSM-seam documentation
that closes the loop on ADR-011.

**Why now.** S-12 closed the *Rust verifier* (ADR-011) but
deferred the producer-side machinery — no CLI binary entry
point, no signed release pipeline, no on-prem test path. The
operator needs to validate the workspace on an air-gapped
host before pilots (S-14) or TOB engagement scoping (S-13a)
expose it externally.

**Predecessors.** S-12 (release verifier + manifest schema +
SI-7 check), S-7 (doctor checks), S-5 (model resolver +
backend trait).

**Cross-repo.** None. The federation pin moves to capture
the S-12b close commit per existing cadence.

**Features owned.**
- `features/surfaces/surface-cli.feature`
- `features/distribution/dist-airgap-install.feature`
- `features/distribution/dist-signed-releases.feature` (CI half)
- `features/distribution/dist-reproducible-builds.feature` (container half)

**Exit criteria.**
- New `nist-agent-cli` bin crate landed with subcommands
  `doctor`, `model verify`, `model infer` (feat-flagged),
  `install`, `wizard` (feat-ui), `daemon`, `status`.
- llama.cpp embedded backend wired into `nist-agent-model`
  behind `feat-model-llamacpp`. SI-7 hash check unconditional;
  inference behind feature flag.
- `docs/audit/AIRGAP_TEST.md` runbook lets an operator run
  the full air-gap validation on prem.
- `.github/workflows/release.yml` triggers on `v\d+\.\d+\.\d+`
  tag push, builds inside the hermetic container, hashes
  artifacts, produces SBOM, signs with a soft key (HSM seam
  documented), uploads to GitHub release.
- `Dockerfile.builder` pins the rust toolchain + system deps
  for reproducible builds.
- `docs/audit/HSM_SEAM.md` documents the HSM swap-in path.
- `cargo test --workspace` ratchet monotone non-decreasing
  (adds CLI + llama.cpp scaffolding tests).

**Deferred (still S-12b followups / S-14 / v1.0-tag sprint):**
- Real HSM provisioning + key custody.
- Two-machine bit-for-bit reproducibility comparison in CI.
- A bundled Gemma 4 E2B GGUF in the release artifact set
  (license bundling + mirror distribution path).
- The `daemon` subcommand's full event loop — this sprint
  ships a minimal version that boots and exposes status.

## Close note — 2026-05-21

Status: **COMPLETE**. Rust workspace + CI scaffold + air-gap
runbook all in place. Operator can now run the air-gap
on-prem test per `docs/audit/AIRGAP_TEST.md`.

**Delivered.**
- `crates/nist-agent-cli` — 14th workspace crate. Bin name
  `citrate-agent`. Subcommands `doctor`, `model verify|infer`,
  `install`, `status`, `daemon`, `wizard`. Exit codes pinned:
  0 = pass, 1 = warn, 2 = blocker/refusal.
- `crates/nist-agent-model::EmbeddedLlamaCpp` — llama.cpp
  backend behind `feat-model-llamacpp`. SI-7 hash check
  unconditional; inference behind feature flag. Default build
  stays light.
- `docs/audit/AIRGAP_TEST.md` — full step-by-step for operator
  on-prem validation: pre-cache (network on) → bring up
  (network off) → build → test → doctor → SI-7 verify → embedded
  inference → install-refusal smoke.
- `.github/workflows/release.yml` — fires on `v\d+\.\d+\.\d+`
  tag push. Builds in hermetic container, runs full
  test/lint/fmt, builds `citrate-agent` bin, generates
  CycloneDX SBOM, hashes artifacts, signs via
  `scripts/release/sign-manifest` (soft-key via repo secret;
  HSM-swap seam documented), uploads draft pre-release.
- `Dockerfile.builder` — Debian-slim + pinned toolchain +
  native deps for llama.cpp + cargo-audit / cargo-deny
  pre-installed.
- `scripts/release/build_manifest.py` — deterministic
  `release.manifest.toml` builder.
- `scripts/release/sign-manifest` — soft-key Ed25519 signer
  (standalone crate; not a workspace member to keep dep tree
  separable).
- `docs/audit/HSM_SEAM.md` — concrete swap-in guide for
  PKCS#11 + Cloud KMS, 30-line wrapper example, checklist.

**Smoke tests run during the sprint.**
- `cargo build --workspace` clean.
- `cargo test --workspace` 202 → 211 (+9).
- `cargo clippy --workspace --all-targets -- -D warnings` clean.
- `cargo fmt --all -- --check` clean.
- `citrate-agent --help` lists every subcommand.
- `citrate-agent status` boots with feature readout.
- `citrate-agent doctor` runs the 5 NIST §10.2 checks (Pass
  on TLA-specs-current at ≥ 5; other 4 skip on empty context).
- `citrate-agent model verify` happy + tamper paths exit 0 / 2
  with the pinned wording.
- `build_manifest.py` produces a deterministic TOML matching
  the `ReleaseManifest` shape.
- `sign-manifest` (soft-key) produces a signature the verifier
  accepts.

**Metrics.**
- Workspace crates: 13 → 14 (+1 `nist-agent-cli`).
- `cargo test --workspace`: 202 → 211 (+9; Rule 3 satisfied).
- ADRs: 12 → 12 (no new ADR; CLI placement is implicit per
  the bin/library split convention).
- New workflows: 1 (`release.yml`).
- New runbooks: 2 (`AIRGAP_TEST.md`, `HSM_SEAM.md`).

**Exit criteria — final state.**
- ✅ `nist-agent-cli` bin crate with all 7 subcommands.
- ✅ `feat-model-llamacpp` wires llama-cpp-2; SI-7
  unconditional.
- ✅ `AIRGAP_TEST.md` operator runbook lands.
- ✅ `.github/workflows/release.yml` triggers on tag push.
- ✅ `Dockerfile.builder` pins toolchain + native deps.
- ✅ `docs/audit/HSM_SEAM.md` documents the swap path.
- ✅ Test ratchet monotone non-decreasing.

**Carried into S-12c / v1.0-tag.**
- Full daemon event loop (HITL queue processing, audit anchor
  cadence, IPC socket).
- Slint wizard window-up integration (render models exist;
  runtime window not yet).
- Two-machine bit-for-bit reproducibility comparison in CI.
- HSM ceremony + key publication.
- First production tag push exercising the workflow end-to-end.
