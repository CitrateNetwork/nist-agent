---
created: 2026-05-19T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
planset: nist-sidecar-v1
---

# Alignment — nist-agent ↔ citrate-agent-runtime

> nist-agent is a **sidecar consumer** of `citrate-agent-runtime`.
> This file is the crosswalk that prevents duplication: for every RFC
> subsystem we identify what's already built in runtime, what's
> planned upstream, and what nist-agent owns end-to-end.

## Ownership rule

If the work is **inside `citrate-agent-core`'s normative surface**
(the Rust library crate that the RFC names canonical), it gets done
in `citrate-agent-runtime` and PR'd upstream. nist-agent depends on
the resulting `citrate_agent_core` crate.

If the work is **above the core** (distribution, packaging, EVM
adapter, overlay bundle authorship, Slint UI, deployment runbooks,
pilot onboarding), nist-agent owns it end-to-end.

This rule has one exception: when runtime's planset has not yet
sequenced a piece of normative-core work that nist-agent's roadmap
needs, nist-agent **may land it locally in a feature crate**, then
open an upstream PR. The local crate stays in nist-agent only until
the upstream PR merges; then it deletes.

## Crosswalk by RFC subsystem

| RFC subsystem | Runtime status | This-repo action |
|---|---|---|
| `agent/core::hitl::*` (HIC queue, 5-role lattice, quorum, SoD, break-glass framework) | **DONE.** `agent/core/src/hitl/*`, TLA-verified `ApprovalStateMachine.tla` (336k states). | **Consume.** Add overlay-specific quorum tests (FERPA/HIPAA/FedRAMP variants) under `tests/`. |
| `agent/core::audit::*` (hash-chained log, CBOR canonical, 3 anchor strategies) | **DONE.** `agent/core/src/audit/*`, TLA-verified `AuditChainIntegrity.tla` (35k states). | **Consume.** Add `Sink` impls for NFS/S3 and WORM (RFC §6.2) if runtime doesn't ship them. |
| `agent/core::capsule::*` (WIT/WASM loader, manifest, 3 signing tiers) | **DONE through CIT-AGENT-3a–3d.** 10 production capsules in `capsules/`. | **Consume.** Author overlay-specific capsules (FERPA redaction, HIPAA de-id, CMMC L3 enumeration). |
| `agent/core::chain::anchor` (AnchorRegistry adapter) | **DONE.** `agent/core/src/chain/anchor.rs` (CIT-AGENT-6a). | **Generalize.** **Owned here.** Wrap in `trait ChainClient`; default impl wraps runtime's adapter; generic EVM impl is new (S-4). PR adapter trait back upstream if it's clean. |
| `agent/core::doctor::*` (pre-flight framework + 5/11 checks) | **PARTIAL.** Framework + 5 checks (CIT-AGENT-7a). | **Co-execute.** Land checks 6–11 here in S-7; PR back upstream once green. |
| `agent/core::agent::*` (token-streaming loop, state-managed interrupt) | **SCAFFOLDED.** Empty `mod.rs`. Scheduled CIT-AGENT-3 in runtime. | **Co-execute.** Either wait on runtime's CIT-AGENT-3, or land in nist-agent S-5 and upstream the PR. Decision in S-5 kickoff. |
| `agent/core::policy::*` (signed PolicyBundle, data-class lattice) | **SCAFFOLDED.** Empty `mod.rs`. Scheduled CIT-AGENT-4 in runtime. | **Co-execute.** S-6 lands it (CBOR bundle, lattice, ratchet); PR back upstream. |
| `agent/core::model::*` (Ollama / llama.cpp / embedded GGUF resolver) | **SCAFFOLDED.** Empty `mod.rs`. Scheduled CIT-AGENT-3 alongside agent loop. | **Co-execute.** S-5 lands it; PR back upstream. |
| Five normative TLA+ specs (`ApprovalStateMachine`, `AuditChainIntegrity`, `DataClassLattice`, `CapsuleInstallGate`, `BreakGlassPath`) | **PARTIAL.** ApprovalStateMachine and AuditChainIntegrity verified in runtime; others referenced in module docs but archive-only. | **Owned here.** S-2 ports the verified two locally and authors the remaining three. CI ratchet via `scripts/ci/check_spec_ratchet.py`. |
| On-chain contracts (`OrganizationSBT`, `AgentSBT`, `CapsuleRegistry`, `AnchorRegistry`, `BenchmarkRegistry`) | **Lives in `citrate-chain`.** Not in runtime. | **Do not duplicate.** nist-agent links to `citrate-chain` for ABIs; ships deployment runbook addenda per overlay. |
| Slint app (concierge, HIC UI, capsule inspector, marketplace) | **ABSENT in runtime.** Separate UI repo or unbuilt. | **Owned here.** S-10. |
| Mobile companion (iOS + Android) | **ABSENT in runtime.** | **Owned here.** S-11. |
| Distribution (reproducible builds, signed releases, GGUF bundling) | **PARTIAL.** Runtime has `deny.toml`, supply-chain governance, but no signed-release pipeline. | **Owned here.** S-12. |
| Overlay policy bundles (FERPA, HIPAA, FedRAMP High, etc.) | **ABSENT in runtime.** Policy bundle schema not yet land. | **Owned here.** S-8 + S-9. Depends on S-6 landing the schema. |
| EVM-chain-agnostic anchoring | **ABSENT.** Runtime is Citrate-specific. | **Owned here.** S-4. |

## Decision protocol when ownership is unclear

If during sprint kickoff there's ambiguity about whether a piece
belongs upstream or here, the rule is:

1. **Default to upstream.** If it's a normative-core change, file an
   issue in `citrate-agent-runtime` and open a federation sprint.
2. **Exception: roadmap-critical and runtime-unsequenced.** If the
   work is on the nist-agent v1.0 path and runtime has not yet
   sequenced it in their planset, land in a feature crate locally
   in nist-agent. Open the upstream PR within the same sprint.
3. **Never copy code.** If a normative-core helper would be useful
   here, add it to the upstream PR and consume it.

## Open ownership questions

The following were open at planset opening; resolution status is
tracked here as each sprint closes.

- **`trait ChainClient` placement.** Is the trait an upstream
  concept (lives in `citrate-agent-core`) or a sidecar concept
  (lives in a nist-agent crate that *uses* `citrate-agent-core`)?
  Working assumption: upstream — the trait is core-shaped, the
  generic EVM impl is sidecar-shaped.
  **RESOLVED 2026-05-20 (S-4 close, ADR-003)**: trait lives in
  `nist-agent-chain` (sidecar concept). The working assumption
  was wrong — the trait's call shape is operator-policy-driven,
  the existing `AnchorRegistryClient` is Citrate-coupled, and
  upstream placement would have created release-cadence
  coupling. ADR-003 documents the full reasoning. Reversal
  conditions named there.
- **Doctor checks 6–11.** Land in runtime first (and consume), or
  land here first (and upstream)? Working assumption: land here in
  S-7, upstream in S-7's exit week. **OPEN — decision in S-7.**
- **TLA+ spec custody.** The two specs that are "DONE in runtime"
  — do they live canonically in runtime or in nist-agent? Working
  assumption: in **nist-agent** (since this repo asserts the
  product compliance posture); runtime references our copies.
  **RESOLVED 2026-05-20 (S-2 close, ADR-002)**: all five
  normative specs hold canonical custody in
  `nist-agent/.agentile/formal/specs/agent/`. Working assumption
  confirmed. ADR-002 documents reasoning and reversal conditions.
  The actual sprint also discovered that *all five* specs already
  existed in the archive (not just two), so the original
  "fresh authoring" framing for three of them was wrong.
