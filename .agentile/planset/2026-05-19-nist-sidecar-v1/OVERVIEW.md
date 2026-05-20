---
created: 2026-05-19T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
planset: nist-sidecar-v1
---

# Planset — nist-sidecar-v1

> A planset is a multi-sprint strategic frame. It sits between a sprint
> (timeline-bound, has daily updates) and an ADR (single decision). A
> planset names a workstream, declares its phases, identifies the
> cross-repo dependencies, and breaks the work into sprintable epics.
> Sprints reference back to the planset; the planset does not have
> daily updates.

## Thesis

**nist-agent ships RFC-CIT-AGENT-0001 as a composable, chain-agnostic
sidecar.** It composes `citrate-agent-core` (already largely built in
`citrate-agent-runtime`) with the few pieces the runtime hasn't landed
yet (Agent loop, Policy engine, Model resolver, doctor checks 6–11),
generalizes the chain client to be EVM-adapter-trait-driven,
authors the overlay-keyed policy bundles, populates the five
normative TLA+ specs locally, and packages the signed reproducible
distribution that operators on Citrate or any compatible EVM chain
can bolt onto their infrastructure.

## Why this planset, why now

Two converging pressures make this planset urgent:

1. **Regulatory tailwind (RFC §1.1).** CMMC L3 enforcement under
   32 CFR § 170 is now in flight for DoD prime contractors; FedRAMP
   Rev 5 baselines are mandatory for new authorizations; FERPA and
   HIPAA jurisprudence is hardening around AI-system data-class
   handling. Organizations are deploying agents either out of
   compliance or not at all.
2. **Runtime is mostly built (`citrate-agent-runtime`).** The HITL
   queue, audit chain, capsule loader, AnchorRegistry adapter,
   Doctor framework, and 10 production capsules are already in
   `citrate-agent-runtime`. The runtime has scaffolded but unfilled
   slots (Agent loop / Policy / Model resolver) and a small list of
   missing pieces (5 of 11 doctor checks, the five normative TLA+
   specs locally, EVM-adapter abstraction). The gap from "runtime
   does most of it" to "shippable NIST sidecar" is closeable in a
   single phase.

This planset names that closing work and sequences it.

## What's in scope here

See [`FEATURE_INVENTORY.md`](FEATURE_INVENTORY.md) for the canonical
list. Summarized:

- **Core completion.** Agent loop, Policy engine, Model resolver
  (fill the empty `mod.rs` files upstream; either land in runtime or
  carry locally and PR back).
- **Chain abstraction.** A `trait ChainClient` so nist-agent works on
  any EVM chain, not just Citrate L1.
- **Doctor completion.** Checks 6–11 of RFC §10.2.
- **TLA+ port.** Five normative specs locally, wired into a CI ratchet.
- **Overlay bundles.** Signed policy bundles for the Phase-1 overlay
  set: CMMC L3 baseline, FERPA, COPPA, CIPA, HIPAA / HITECH,
  FedRAMP High.
- **Slint concierge.** Onboarding wizard, HITL queue UI, capsule
  inspector, chat surface, capsule marketplace browser.
- **Distribution.** Reproducible builds, signed releases, bundled
  Gemma 4 E2B GGUF, deployment runbooks per overlay.
- **Pilot onboarding.** One operator per overlay class
  (DIB, healthcare, K-12, FedRAMP).

## What's explicitly NOT in this planset

- Subagent delegation (deferred to v2 planset).
- SVM compatibility (deferred to v2 planset).
- IL5 / ITAR / CJIS / IRS 1075 (deferred to nist-sidecar-v1.1 planset).
- On-chain contract authorship (lives in `citrate-chain`).
- Multi-tenant SaaS hosting (out-of-scope indefinitely).

## How this planset relates to other plansets

- **Upstream:** `citrate-federation` planset(s) governing the
  monorepo-split aftermath. Any cross-repo work here opens a
  federation sprint as well.
- **Sibling:** the (not-yet-extant) `citrate-agent-runtime` planset
  for runtime's own work. We coordinate via
  [`ALIGNMENT.md`](ALIGNMENT.md) to avoid duplication.
- **Downstream:** future `nist-sidecar-v1.1` planset (Q1 2027) and
  `nist-sidecar-v2` planset (target 2027).

## Documents in this planset

| File | Purpose |
|---|---|
| `OVERVIEW.md` | this file — thesis and scope |
| [`ROADMAP.md`](ROADMAP.md) | three phases mapped to RFC §11 milestones |
| [`ALIGNMENT.md`](ALIGNMENT.md) | crosswalk against `citrate-agent-runtime` to prevent duplication |
| [`FEATURE_INVENTORY.md`](FEATURE_INVENTORY.md) | every RFC normative section → `.feature` file → sprint stub |
| [`DEPENDENCIES.md`](DEPENDENCIES.md) | what we consume, what consumes us, what changes when |

## Success criteria (planset close)

The planset is **closed** when:

1. All sprint stubs listed in `FEATURE_INVENTORY.md` have moved from
   `sprints/backlog/` to `sprints/completed/`.
2. `PRODUCT_SPEC.md`'s "How 'done' is judged for v1.0" criteria are
   met.
3. `citrate-federation/manifest.toml` pins this repo at a tagged
   `v1.0.0`.
4. A planset-close note is appended below summarizing what shipped
   versus what was deferred to the next planset.

(Close note will be filled in at end-of-planset.)
