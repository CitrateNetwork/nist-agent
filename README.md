# nist-agent

> Composable, NIST-compliant agent harness — sidecar for the Citrate
> Network and other EVM chains. The distribution and packaging
> surface for [RFC-CIT-AGENT-0001][rfc].

[rfc]: docs/rfcs/RFC-CIT-AGENT-0001.md

## What this is

A Rust-implemented agent harness that:

- Operates **air-gapped by default** — zero outbound connectivity
  unless explicitly enabled by a signed Security Officer policy
  directive.
- Gates every action through a **tiered-risk, role-bound quorum** of
  five roles (Operator, Reviewer, Compliance Officer, Security
  Officer, Auditor), with cryptographic sign-off from hardware-backed
  keys (FIDO2 / PIV-CAC / Secure-Enclave-or-TPM).
- Loads capabilities as **Capsules**: signed WIT/WASM Component
  Model bundles whose manifest is enforced at the wasmtime linker —
  capability enforcement is load-time, not advisory.
- Maintains a **hash-chained, signed audit log** with four storage
  backends and three on-chain anchor strategies; no operator content
  ever reaches the chain — only commitments.
- Verifies its safety-critical control flow with **five normative
  TLA+ specifications** — Approval state machine, audit chain
  integrity, data-class lattice, capsule install gate, break-glass
  path.
- Ships a **Slint operator app** with a Gemma 4 E2B bundled concierge
  for first-run onboarding, HITL approval queue, Capsule Inspector,
  marketplace browser.

## Compliance posture

- **Baseline:** NIST SP 800-171 Rev 3 + CMMC Level 3.
- **Overlays (v1.0):** FERPA, COPPA, CIPA, HIPAA / HITECH,
  FedRAMP High.
- **Overlays (v1.1):** DoD IL4 / IL5, ITAR, CJIS, IRS 1075.
- **Default network posture:** air-gapped, opt-in egress.

## Relationship to the Citrate federation

nist-agent is a **sidecar consumer** of
[`citrate-agent-runtime`][runtime]: it depends on
`citrate-agent-core` as a Cargo crate, adds the
EVM-chain-adapter trait, authors overlay-keyed policy bundles,
completes the doctor pre-flight checks, populates the five normative
TLA+ specs locally, and packages the signed distribution.

It does **not** fork the runtime. Anything in the normative core
surface gets done upstream and PR'd back; this repo owns the
chain-agnostic packaging, overlay bundles, formal specs, the Slint
UI, mobile companion, and the distribution pipeline.

[runtime]: https://github.com/CitrateNetwork/citrate-agent-runtime

## Start here

If you are a contributor (human or AI):

1. **[`.agentile/AGENT_ENTRY.md`](.agentile/AGENT_ENTRY.md)** — the
   entry point for every contributor.
2. **[`docs/rfcs/RFC-CIT-AGENT-0001.md`](docs/rfcs/RFC-CIT-AGENT-0001.md)**
   — the architecture reference. Everything in this repo is
   downstream of it.
3. **[`.agentile/planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](.agentile/planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md)**
   — the v1.0 workstream.
4. **[`features/`](features/)** — Gherkin BDD scenarios, one file per
   RFC normative section.
5. **[`.agentile/sprints/active/`](.agentile/sprints/active/)** — the
   sprints currently in flight.

## Status

Pre-alpha. Bootstrap complete (Sprint S-0). RFC v0.1 in quorum
review. Sprint S-1 (RFC canonization, planset land, feature
inventory, backlog seed) is **active**. Sprints S-2 through S-13 are
sequenced in [`ROADMAP.md`](.agentile/planset/2026-05-19-nist-sidecar-v1/ROADMAP.md).

## Methodology

This repo uses **Agentile** — the federation-wide methodology
documented at
`citrate-federation/agentile/` (maintainers only; private)
and prototyped at
[`github.com/citratenetwork/agentile`](https://github.com/citratenetwork/agentile).
The 13 rules are summarized in
[`.agentile/AGENT_ENTRY.md`](.agentile/AGENT_ENTRY.md) and detailed
in [`.agentile/rules/CORE_RULES.md`](.agentile/rules/CORE_RULES.md).

Notable rules to be aware of before contributing:

- **Rule 1.** No mocks / stubs / TODOs in production paths.
- **Rule 2.** Test count monotone non-decreasing across a sprint.
- **Rule 5 / 12.** Every doc carries Rule-12 frontmatter
  (`created`, `branch`, `author`, `status`).
- **Rule 10.** Authorization before destructive ops; TLA+ specs
  verify in CI as a BLOCKER.

## License

Source-available under the Business Source License 1.1 (see [`LICENSE`](LICENSE));
converts to Apache-2.0 on the Change Date stated in the license. This is the
commercial application-layer / core tier of Citrate's open-core model; the
infrastructure tier is Apache-2.0. Licensor: Citrate Inc. A commercial license is
also available for the FedRAMP package per RFC §8.4.

The bundled Gemma 4 E2B GGUF model file is distributed under its
own license alongside the binary as a separately hashable artifact
(NIST SI-7).

The Slint operator app is built on the Slint UI toolkit, which is
triple-licensed (GPLv3 / Royalty-Free / Commercial). The open-source
distribution falls under Slint's GPLv3 terms.

## Custodian

© 2026 Citrate Inc. Custody and authorship pipeline
governed by Citrate Inc.
