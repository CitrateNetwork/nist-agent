---
created: 2026-05-19T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
planset: nist-sidecar-v1
---

# Feature Inventory — nist-sidecar-v1

> Every RFC normative section → a `.feature` file under `features/`.
> Every feature maps to one or more sprints in
> [`ROADMAP.md`](ROADMAP.md). This file is the canonical join between
> RFC sections, Gherkin BDD scenarios, and sprint backlog stubs.

## Conventions

- Feature files live at `features/<area>/<slug>.feature`.
- Each feature begins with `# Maps RFC-CIT-AGENT-0001 §<section>`.
- Each feature ends with `# Sprint: S-<n> <slug>`.
- TLA+ specs that pair with a feature are noted in the feature's
  `Background:` block.
- Scenarios use `Given / When / Then / And / But` per Gherkin 6.

## Inventory by area

### Core (`features/core/`)

| File | RFC §  | TLA+ | Sprint |
|---|---|---|---|
| `agent-loop.feature` | §3.1, §5.4 | `ApprovalStateMachine.tla` | S-5 |
| `policy-bundle.feature` | §2.3, §3.1 | `DataClassLattice.tla` | S-6 |
| `hitl-quorum.feature` | §5.1–5.3 | `ApprovalStateMachine.tla` | (consume runtime) |
| `hitl-state-managed-interrupt.feature` | §5.4 | `ApprovalStateMachine.tla` | (consume runtime) |
| `hitl-break-glass.feature` | §5.5 | `BreakGlassPath.tla` | S-2 (spec author) + S-6 |
| `hitl-signing-surfaces.feature` | §5.6 | — | (consume runtime) + S-11 (mobile) |
| `audit-chain-append.feature` | §6.1 | `AuditChainIntegrity.tla` | (consume runtime) |
| `audit-storage-backends.feature` | §6.2 | — | S-9 (WORM, NFS/S3) |
| `audit-anchor-strategies.feature` | §6.3 | `AuditChainIntegrity.tla` | (consume runtime) + S-4 |
| `audit-retention.feature` | §6.4 | — | S-8/S-9 (overlay-driven) |
| `model-resolver.feature` | §3.3, §8.2 | — | S-5 |
| `network-posture.feature` | §3.3 | — | S-7 (doctor check) |
| `doctor-preflight.feature` | §10.1–10.3 | — | S-7 |

### Capsule (`features/capsule/`)

| File | RFC § | TLA+ | Sprint |
|---|---|---|---|
| `capsule-composition.feature` | §4.1–4.2 | — | (consume runtime) |
| `capsule-manifest-schema.feature` | §4.3 | — | (consume runtime) |
| `capsule-signing-tiers.feature` | §4.4 | — | (consume runtime) |
| `capsule-install-gate.feature` | §4.5, §7.2 | `CapsuleInstallGate.tla` | S-2 (spec author) |
| `capsule-wit-wasm-binding.feature` | §4.5 | — | (consume runtime) |
| `capsule-data-class-lattice.feature` | §7.2 | `DataClassLattice.tla` | S-2 + S-6 |

### Chain (`features/chain/`)

| File | RFC § | TLA+ | Sprint |
|---|---|---|---|
| `chain-organizationsbt.feature` | §7.1 | — | (link `citrate-chain`) |
| `chain-agentsbt.feature` | §7.1, §7.2 | — | (link `citrate-chain`) |
| `chain-capsuleregistry.feature` | §7.1 | — | (link `citrate-chain`) |
| `chain-anchorregistry.feature` | §7.1, §6.3 | — | (consume runtime) |
| `chain-benchmarkregistry.feature` | §7.1, §9.2 | — | (link `citrate-chain`) |
| `chain-privacy-guarantees.feature` | §7.3 | — | S-4 |
| `chain-evm-adapter.feature` | §3.2, §11 sidecar-N1 | — | **S-4 (new, this repo)** |

### Overlays (`features/overlays/`)

| File | RFC § | TLA+ | Sprint |
|---|---|---|---|
| `overlay-cmmc-l3-baseline.feature` | §2.1, §2.2 | — | S-8 |
| `overlay-ferpa.feature` | §2.3 | — | S-8 |
| `overlay-coppa.feature` | §2.3 | — | S-8 |
| `overlay-cipa.feature` | §2.3 | — | S-8 |
| `overlay-hipaa.feature` | §2.3 | — | S-9 |
| `overlay-fedramp-high.feature` | §2.3 | — | S-9 |
| `overlay-activation-ratchet.feature` | §2.3 | — | S-6 |

### Surfaces (`features/surfaces/`)

| File | RFC § | TLA+ | Sprint |
|---|---|---|---|
| `surface-slint-concierge.feature` | §8.1, §8.2 | — | S-10a (wizard) + S-10c (chat) |
| `surface-slint-capsule-inspector.feature` | §8.3 | — | S-10b |
| `surface-cli.feature` | §3.2 | — | S-12 |
| `surface-daemon.feature` | §3.2 | — | S-12 |
| `surface-wasm-publish.feature` | §3.2 | — | S-12 |
| `surface-mobile-companion.feature` | §5.6, §8 | — | S-11 |

### Distribution (`features/distribution/`)

| File | RFC § | TLA+ | Sprint |
|---|---|---|---|
| `dist-reproducible-builds.feature` | §11.1, capsule manifest `build_reproducible` | — | S-12 |
| `dist-signed-releases.feature` | §11.1 | — | S-12 |
| `dist-bundled-gemma4.feature` | §8.2 | — | S-12 |
| `dist-airgap-install.feature` | §3.3, G1 | — | S-12 |
| `dist-overlay-runbooks.feature` | §11.1 | — | S-12 |

## Backlog mapping

The set of sprint stubs created in `sprints/backlog/` during S-1 close:

| Sprint | Slug | Features delivered |
|---|---|---|
| S-1 | rfc-canonization-feature-inventory | (this file + Gherkin authoring) |
| S-2 | tla-spec-port | five TLA+ specs + ratchet wiring |
| S-3 | cargo-workspace-consume-runtime | dependency on `citrate-agent-core`; `deny.toml`; rust-toolchain |
| S-4 | evm-chain-adapter-trait | `trait ChainClient` + generic EVM impl + chain-privacy tests |
| S-5 | agent-loop-and-model-resolver | fill empty `agent/` + `model/` mods; PR back to runtime |
| S-6 | policy-bundle-and-data-class-lattice | signed CBOR PolicyBundle, lattice, activation ratchet |
| S-7 | doctor-checks-6-through-11 | finish RFC §10.2 |
| S-8 | overlay-bundles-a | CMMC L3, FERPA, COPPA, CIPA |
| S-9 | overlay-bundles-b | HIPAA, HITECH, FedRAMP High; WORM + NFS/S3 sinks |
| S-10a | slint-concierge-wizard | first-run wizard (org identity, roles, hardware keys, overlay selection, signed PolicyBundle) |
| S-10b | slint-hitl-and-inspector | HIC approval queue UI + Capsule Inspector pane |
| S-10c | slint-marketplace-and-chat | capsule marketplace browser + chat surface |
| S-11 | mobile-companion | iOS + Android signing surface |
| S-12 | distribution-and-runbooks | reproducible builds, signed releases, GGUF bundle, runbooks |
| S-13 | trail-of-bits-engagement | external audit + remediation |

## How a feature gets to "done"

For a Phase-1 feature:

1. Sprint kickoff references the feature file by path.
2. The sprint's plan-table includes a step "feature scenarios green
   under `cucumber-rs`".
3. If the feature has a paired TLA+ spec, the sprint's exit criteria
   include "TLA+ spec verifies under `scripts/ci/check_spec_ratchet.py`".
4. Sprint close (Rule 4) records the feature as done in the
   SPRINT.md plan-table.
5. The feature file's footer comment (`# Sprint: S-<n>`) is updated
   to reference the *completed* sprint path (not the active one).
