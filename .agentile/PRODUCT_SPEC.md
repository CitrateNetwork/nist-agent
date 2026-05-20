---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
---

# PRODUCT_SPEC.md — What the Finished nist-agent Sidecar Does

> **Scope contract.** If a feature isn't in this file, it's out of
> scope until added explicitly. Updates to this file are sprint-grade
> events — when a sprint changes the surface, it edits this file as
> part of its close criteria.

## Vision

nist-agent ships **RFC-CIT-AGENT-0001** — a Rust-implemented,
NIST-compliant agent harness — as a **composable sidecar** that
organizations on the Citrate Network or any compatible EVM chain can
bolt onto their existing infrastructure without re-architecting it.

The product is the *distribution surface* over `citrate-agent-core`
(the canonical library in `citrate-agent-runtime`): it bundles the
core, generalizes the chain client to be EVM-adapter-trait-driven,
authors the overlay-keyed policy bundles, finishes the doctor checks,
populates the five normative TLA+ specs, and packages the signed
reproducible release.

## Primary user

An IT-or-compliance officer at one of:

- a defense industrial base subcontractor handling CUI under DFARS
  252.204-7012;
- a healthcare provider or covered-entity vendor handling PHI under
  HIPAA / HITECH;
- a public-sector deployment (school district under FERPA / COPPA /
  CIPA, agency under FedRAMP / IRS 1075, law enforcement under CJIS);
- a research org handling export-controlled work under ITAR;
- a Citrate Network operator (`Organization` SBT holder) running the
  agent inside their node.

This user can install, configure, attest, and audit the sidecar
without writing Rust. They can extend it (new capsules, new overlay
bundles) under the bundled / managed / workspace signing pipeline
without re-architecting the host.

## What v1.0 ships

**Surfaces** (RFC §3.2, §8):

1. **`citrate-agentd`** — daemon binary; Unix-domain-socket gRPC for
   ops, optional mTLS endpoint for mobile companion.
2. **`citrate-agent-cli`** — one-shot CLI.
3. **`citrate-agent-slint`** — desktop operator app: concierge
   onboarding wizard, HITL approval queue UI, chat surface, capsule
   marketplace browser. Bundles Gemma 4 E2B as concierge model.
4. **`citrate-agent-wasm`** — Component Model publish target for
   embedding in browsers / other hosts.
5. **Mobile companion** — iOS + Android signing surface.

**Core capabilities** (RFC §3–6, §10):

- **Air-gapped by default.** No outbound traffic at first run or any
  subsequent run unless egress is explicitly policy-enabled (RFC G1).
- **Capsule sandbox.** Every capability invocation comes from a signed
  WIT/WASM `.cps` bundle with manifest, gherkin scenarios, optional
  TLA+ spec, three signing tiers (bundled / managed / workspace).
  Capability enforcement is **load-time** (wasmtime linker), not
  runtime advisory.
- **Five-role HITL.** Operator, Reviewer, Compliance Officer, Security
  Officer, Auditor — every action passes a tiered-risk + role-bound
  quorum gate. Hardware-backed signing via FIDO2 / PIV-CAC /
  Secure-Enclave-or-TPM. Break-glass path with 72-hour post-hoc
  affirmation.
- **Hash-chained audit log.** Per-record CBOR-canonical signed
  entries; four storage backends (local FS, NFS/S3, WORM, chain
  anchors); three anchor strategies (per-capsule, per-approval,
  nightly Merkle); operator-selectable retention by overlay.
- **EVM-adapter-trait chain client.** Pluggable adapter — default
  pin is Citrate L1 (chain id 40204) with the five RFC §7.1
  contracts; alternate adapters work against any EVM chain that hosts
  compatible contract bytecode. nist-agent does **not** ship the
  contracts; it ships the adapter and the deployment guide.
- **Doctor pre-flight.** Eleven checks per RFC §10.2; signed report
  output + optional on-chain commitment; serves as CA-7 continuous
  monitoring artifact.
- **Overlay bundles.** Signed policy bundles for the v1.0 overlay
  set (CMMC L3 baseline, FERPA, COPPA, CIPA, HIPAA / HITECH, FedRAMP
  High). One-way ratchet on activation.

**Formal verification** (RFC §9.1):

- Five normative TLA+ specs verify in CI as a BLOCKER:
  `ApprovalStateMachine.tla`, `AuditChainIntegrity.tla`,
  `DataClassLattice.tla`, `CapsuleInstallGate.tla`,
  `BreakGlassPath.tla`.
- Tier-high and tier-critical capsules MUST ship both Gherkin and TLA+
  (Rule 11 in our local rule set, derived from federation Rule 10).

**Distribution** (RFC §11):

- Reproducible builds (`build_reproducible = true` in every shipped
  manifest); signed releases; bundled Gemma 4 E2B GGUF as a separately
  hashable artifact (NIST SI-7).
- `Apache-2.0` source license; commercial license available for the
  FedRAMP package (RFC §8.4).

## What v1.0 explicitly does NOT ship

Mirrors RFC §1.3 plus a few sidecar-specific carve-outs:

- **N1.** Free-form subagent delegation by capsules.
- **N2.** Autonomous capsule creation at runtime.
- **N3.** Persistent cross-session learning that mutates the capability
  surface.
- **N4.** SVM-native (Solana) skills.
- **N5.** Multi-tenant cloud hosting operated by Citrate Network Inc.
  (Third parties may operate it under the commercial license.)
- **N-sidecar-1.** **nist-agent does not deploy the on-chain
  contracts.** Those are in `citrate-chain`. nist-agent ships the
  Rust-side adapter and a deployment-runbook addendum.
- **N-sidecar-2.** **nist-agent does not fork `citrate-agent-core`.**
  We consume it as a Cargo dependency on a federation-pinned rev.
  Bug fixes / features that belong in core get PR'd upstream, not
  copied here.

## What v1.1 adds (Q1 2027)

- DoD IL5 + ITAR overlays graduate to production.
- PIV-only enforcement mode (global policy).
- CJIS overlay.
- NIAP / Common Criteria evaluation initiated.

## What v2.0 adds (target 2027)

- Policy-gated subagent delegation.
- SVM-namespace `chain_calls` capability.
- Federated learning between sidecar instances on the Citrate
  cooperative (signed-capsule channel only).

## Out-of-scope indefinitely

- Free-form runtime skill creation by the agent itself.
- Cross-org agent collaboration outside the cooperative's
  signed-capsule channels.
- Cloud-hosted multi-tenant deployment operated by Citrate Network Inc.

## How "done" is judged for v1.0

- Trail of Bits engagement closed with no Tier-1 findings outstanding.
- All five TLA+ specs verify in CI.
- All eleven doctor checks return PASS / WARN (no BLOCKER) on a
  reference deployment for each Phase-1 overlay.
- Frontmatter coverage at 100 % under `scripts/ci/check_frontmatter.py`.
- Test count ratchet has been monotone non-decreasing across all v1.0
  sprints.
- A pilot operator from each Phase-1 overlay class (DIB, healthcare,
  K-12, FedRAMP) has completed onboarding via the Slint concierge.
- Federation `manifest.toml` pins nist-agent at a tagged
  `v1.0.0` and `drift-check.yml` passes for all consumers.
