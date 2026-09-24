---
created: 2026-05-19T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
planset: nist-sidecar-v1
---

# Roadmap — nist-sidecar-v1

> Phased milestones mapping RFC §11 to actual sprint sequencing. Each
> phase contains a set of sprint stubs (created in `sprints/backlog/`).
> Phase boundaries are not dates — they're sets of exit criteria.
> Dates listed are *targets*, not commitments.

## Phase 0 — Bootstrap and inventory (2026-05, ~1 sprint)

**Goal.** Get the agentile scaffolding right, freeze the RFC for
quorum review, author the feature inventory and BDD scenarios, seed
the backlog.

**Sprints.**
- `S-0` — Bootstrap (in flight; landed automatically by
  `bootstrap.sh`).
- `S-1` — RFC canonization, planset land, feature inventory and
  Gherkin BDD scenarios written, backlog seeded.

**Exit criteria.**
- `.agentile/` matches federation conventions; AGENT_ENTRY points to
  control plane.
- RFC v0.1 lives at `docs/rfcs/RFC-CIT-AGENT-0001.md` with Rule-12
  frontmatter.
- Every Phase-1 sprint exists as a stub in `sprints/backlog/`.
- One `.feature` per RFC normative section under `features/`.
- Federation manifest opens an entry for `nist-agent`.

## Phase 1 — v1.0-rc readiness (2026-Q3 → 2026-Q4)

Maps RFC §11.1 "v1.0 Mainnet-Ready (Target Q2 2027)" minus
non-Phase-1 overlays (which slip to Phase 1.5 in §11.2).

**Sprints (each gets a backlog stub seeded by S-1):**

- `S-2` — TLA+ port. Author / port the five normative specs locally;
  wire `scripts/ci/check_spec_ratchet.py`. (RFC §9.1)
- `S-3` — Cargo workspace + dependency on `citrate-agent-core`.
  Pin to a federation-canonical rev. Mirror `deny.toml` posture from
  runtime. (RFC §3.2)
- `S-4` — EVM chain adapter trait. Generalize `chain::anchor::*`
  behind a `trait ChainClient` with a default Citrate impl and a
  generic EVM impl. (RFC §7, §11 sidecar-N1)
- `S-5` — Agent loop + Model resolver. Land token-streaming loop with
  state-managed interrupt and the local-Ollama / llama.cpp / embedded
  GGUF resolver. PR back to runtime upstream. (RFC §3.1, §8.2)
- `S-6` — Policy bundle and data-class lattice. Signed CBOR bundle,
  five-level lattice (PUBLIC | CUI | PHI | FERPA | ITAR), overlay
  one-way ratchet. (RFC §2.3, §5.2)
- `S-7` — Doctor completion. Land checks 6–11 (network posture, model
  hash, TLA verification status, role-lattice coverage, queue SLA,
  break-glass affirmation window). (RFC §10.2)
- `S-8` — Overlay bundles A: CMMC L3 baseline + FERPA + COPPA + CIPA.
  (RFC §2.3)
- `S-9` — Overlay bundles B: HIPAA / HITECH + FedRAMP High. (RFC §2.3)
- `S-10a` — Slint concierge wizard. First-run setup: org identity,
  five-role enrollment, hardware-key enrollment, overlay selection,
  PolicyBundle authoring. Gemma 4 E2B drives the conversation copy.
  (RFC §8.1, §8.2) — split from original S-10 on 2026-05-21.
- `S-10b` — Slint HITL queue UI + Capsule Inspector pane. The two
  surfaces every operator action passes through. (RFC §8.3) —
  split from original S-10.
- `S-10c` — Slint marketplace browser + chat surface. Browse and
  install capsules from bundled / site-mirror / on-chain sources;
  interactive chat drives the agent loop. — split from original
  S-10.
- `S-11` — Mobile companion. iOS + Android signing surface; overlay-
  driven enable/disable + TTL. (RFC §5.6, §8)
- `S-12` — Distribution. Reproducible builds, signed releases, GGUF
  bundling, deployment runbooks per overlay. (RFC §11.1)
- `S-13` — Trail of Bits engagement preparation + remediation.

**Exit criteria.**
- All five TLA+ specs verify in CI as BLOCKER.
- All 11 doctor checks PASS or WARN (no BLOCKER) on a reference
  deployment per Phase-1 overlay.
- `cargo test --workspace` ratchet has been monotone non-decreasing
  across S-3 → S-13.
- One pilot operator per overlay class has completed onboarding via
  the Slint concierge end-to-end (capsule install, action proposed,
  HITL gate cleared, audit anchor written).
- Trail of Bits report received; no Tier-1 findings outstanding.
- Federation manifest pin for `nist-agent` updated to `v1.0.0-rc`.

## Phase 2 — v1.0 mainnet anchor (2027-Q2)

**Sprints.**
- `S-14` — Pilot onboarding closeout; capture lessons learned in
  `docs/case_studies/`.
- `S-15` — Mainnet anchor: first production deployment of nist-agent
  on Citrate Mainnet, anchored on AnchorRegistry per operator-chosen
  strategy.
- `S-16` — v1.0 release: tag, federation manifest bump to `v1.0.0`,
  publish reproducible-build SBOMs.

**Exit criteria.**
- `v1.0.0` tag exists.
- Federation `drift-check.yml` green across all consumers.
- Mainnet-anchored doctor report published.
- Planset closes; next planset (`nist-sidecar-v1.1`) opens.

## Out-of-roadmap (subsequent plansets)

- **`nist-sidecar-v1.1` (2027-Q1).** IL5 + ITAR + CJIS + IRS 1075
  overlays, PIV-only mode, NIAP / CC initiation. Maps RFC §11.2.
- **`nist-sidecar-v2` (target 2027).** Policy-gated subagent
  delegation, SVM namespace for `chain_calls`, federated learning
  channel. Maps RFC §11.3.
