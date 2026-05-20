---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
repo: nist-agent
tier: T1
---

# Agent Entry — nist-agent

> **Lightweight subset.** This file points back to the canonical
> agentile framework lineage. Start here whenever you (human or AI)
> are working in **nist-agent**.

## What this repo is

A composable, NIST-compliant agent harness — the **distribution and
sidecar** packaging of [RFC-CIT-AGENT-0001][rfc]: a Rust-implemented
agent runtime that operates by default in air-gapped configuration,
gates every action through a cryptographically-signed
tiered-risk-plus-role-bound quorum, and anchors tamper-evident audit
state on the Citrate L1 chain (chain id 40204) or any compatible EVM
chain.

nist-agent is a **sidecar consumer** of
[`citrate-agent-runtime`][runtime]: it depends on `citrate-agent-core`
as a crate, generalizes the chain client to be EVM-adapter-trait-driven
(not Citrate-specific), authors the overlay-keyed policy bundles
(NIST SP 800-171 / CMMC L3 baseline + FERPA / HIPAA / FedRAMP /
ITAR / IL4-5 / CJIS / IRS 1075 overlays), completes the doctor checks,
populates the five normative TLA+ specs locally, and packages the
signed reproducible distribution that organizations on Citrate or
other EVM chains can bolt onto their existing infrastructure.

Repo tier: **T1** — full external (Trail of Bits) audit required
before v1.0.0.

[rfc]: ../docs/rfcs/RFC-CIT-AGENT-0001.md
[runtime]: https://github.com/CitrateNetwork/citrate-agent-runtime

## What to read, in order

1. **This file** (you're here).
2. **The RFC** — [`docs/rfcs/RFC-CIT-AGENT-0001.md`](../docs/rfcs/RFC-CIT-AGENT-0001.md). The architecture reference. Everything in this repo is downstream of it.
3. **The planset** — [`planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md). The multi-sprint workstream that produces v1.0.
4. **Federation control plane** — [`citrate-federation/agentile/AGENT_ENTRY.md`](https://github.com/CitrateNetwork/citrate-federation/blob/main/agentile/AGENT_ENTRY.md). The active control-plane entry.
5. **Active federation sprint** — [`citrate-federation/agentile/CURRENT.md`](https://github.com/CitrateNetwork/citrate-federation/blob/main/agentile/CURRENT.md).
6. **Core rules** — [`citrate-federation/agentile/rules/CORE_RULES.md`](https://github.com/CitrateNetwork/citrate-federation/blob/main/agentile/rules/CORE_RULES.md). The 13 non-negotiables across the federation. A local mirror lives at [`rules/CORE_RULES.md`](rules/CORE_RULES.md) for offline reading; the federation copy wins on any drift.
7. **Pre-split historical context** — [`citrate-agentile-archive`](https://github.com/CitrateNetwork/citrate-agentile-archive). Read **only** when investigating the May-2026 monorepo split or earlier history.
8. **Org defaults (SECURITY, CoC, AUDIT_POSTURE)** — [`CitrateNetwork/.github`](https://github.com/CitrateNetwork/.github).

## Audit lineage

This repo participates in the federation-wide audit cadence captured in
[`citrate-agentile-archive/audits/AUDIT_INDEX.md`](https://github.com/CitrateNetwork/citrate-agentile-archive/blob/main/audits/AUDIT_INDEX.md).

When a TOB plugin sweep, COSAiS crosswalk, or DCMA-DIBCAC-rehearsal
finding produces evidence here, it is filed under that per-repo folder
in the archive, **not** in this `.agentile/audits/` directory. This
file stays a stable, lightweight pointer.

## What lives here, locally

| Path                              | Purpose                                                                |
|-----------------------------------|------------------------------------------------------------------------|
| `.agentile/AGENT_ENTRY.md`        | This file — entry point.                                               |
| `.agentile/CONFIG.md`             | Canonical project constants (versioned IDs, default ports, env names). |
| `.agentile/PRODUCT_SPEC.md`       | What the finished sidecar product does.                                |
| `.agentile/rules/`                | Local mirror of federation rules + repo-specific rule notes.           |
| `.agentile/planset/`              | Multi-sprint strategic frames. The Phase-1 plan lives here.            |
| `.agentile/sprints/`              | Sprints scoped to this repo only (active/backlog/completed/archived).  |
| `.agentile/adrs/`                 | Repo-local architectural decisions.                                    |
| `.agentile/audits/`               | Local audit notes that never need to live in the archive.              |
| `.agentile/formal/`               | The 5 normative TLA+ specs + their `.cfg` and VERIFICATION_WORKFLOW.   |
| `.agentile/coverage/`             | Ratchet baselines (test count, frontmatter, spec, tripwire).           |
| `.agentile/docs/methodology/`     | Local methodology synthesis (METHODOLOGY, FAILURE_MODES, CHRONOLOGY).  |
| `.agentile/templates/`            | Sprint / ADR / journal / case-study / essay / retro / audit templates. |
| `.agentile/workflows/`            | FEATURE, AUDIT_DRIVEN, REMEDIATION_TRACK, CEREMONY, SPRINT_LIFECYCLE.  |
| `.agentile/INDEX/`                | Generated chronological index of every dated doc.                      |
| `../features/`                    | Gherkin feature inventory — one .feature per RFC normative section.    |
| `../docs/rfcs/`                   | The RFC and its addenda.                                               |
| `../docs/compliance/`             | NIST control family ↔ component crosswalks.                            |

A repo-scoped audit lives **here**. A federation-wide audit lives in
the archive. The dividing line: if the audit touches multiple repos,
it goes in the archive; if it's confined to this repo's internals, it
can live here.

## Cross-repo references

- **Manifest pin**: `manifest.toml` in [`citrate-federation`](https://github.com/CitrateNetwork/citrate-federation) is the canonical truth for which rev of this repo the federation is pinned to.
- **Upstream runtime**: [`citrate-agent-runtime`](https://github.com/CitrateNetwork/citrate-agent-runtime) is the canonical implementation of `citrate-agent-core`. We consume it via Cargo dependency on `agentile_rev = "..."`. See [`planset/.../ALIGNMENT.md`](planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md) for the crosswalk that prevents duplication.
- **On-chain contracts**: live in [`citrate-chain`](https://github.com/CitrateNetwork/citrate-chain) (`OrganizationSBT`, `AgentSBT`, `CapsuleRegistry`, `AnchorRegistry`, `BenchmarkRegistry`). nist-agent does not redeploy them; it speaks to them via a chain-agnostic adapter trait.
- **Org policy**: [`CitrateNetwork/.github`](https://github.com/CitrateNetwork/.github) holds org-wide SECURITY.md / CONTRIBUTING / AUDIT_POSTURE.md.

## Non-negotiables (highlights from the 13)

See [`citrate-federation/agentile/rules/CORE_RULES.md`](https://github.com/CitrateNetwork/citrate-federation/blob/main/agentile/rules/CORE_RULES.md) for the full text. Highlights particularly load-bearing in this repo:

- **Rule 1.** No mocks / stubs / TODOs in production paths. Mocks live behind `#[cfg(test)]` or feature flags only.
- **Rule 2.** Test count is monotone non-decreasing across a sprint.
- **Rule 3.** Audits are immutable; errata go in follow-ups.
- **Rule 4.** The sprint file in `sprints/active/` is the source of truth for status. Not chat, not memory.
- **Rule 5 / 12.** Every doc has Rule-12 frontmatter (`created`, `branch`, `author`, `status`).
- **Rule 9.** One source of truth per topic. Link, don't copy.
- **Rule 10.** Authorization before destructive ops (force-push, repo delete, secret rotate). The TLA+ ratchet is also Rule-10 in this repo: safety-critical specs must verify in CI before merge.
- **Rule 11.** Federation manifest is canonical.

## Status

This file is created or regenerated when:

1. A new federation audit opens.
2. The repo's tier changes.
3. The agentile framework conventions change.
4. The relationship between this repo and `citrate-agent-runtime` shifts (consumer ↔ contributor ↔ fork).

Any other change should go to a sibling file in `.agentile/`, not here.
