# features/ — Gherkin BDD Inventory

> One `.feature` file per RFC-CIT-AGENT-0001 normative section. These
> files are the **executable spec** for nist-agent: they drive sprint
> planning, they run under `cucumber-rs` (wired in S-1), and a
> feature's scenarios going green is a non-negotiable exit criterion
> for the sprint that owns it.

## Conventions

1. Each file opens with `# Maps RFC-CIT-AGENT-0001 §<section>` so the
   feature → RFC mapping is grep-able.
2. Each file ends with `# Sprint: S-<n> <slug>` linking the owning
   sprint (updated to the *completed* sprint path on close).
3. Each file with a paired TLA+ spec calls it out in the `Background:`
   block: `# TLA+: <spec-name>.tla`.
4. Gherkin keywords: `Feature / Background / Scenario / Scenario
   Outline / Given / When / Then / And / But / Examples`.
5. Use one verb per step; no compound steps.
6. Reference real RFC artifact names (`AgentSBT`, `AnchorRegistry`,
   `PolicyBundle`, etc.) so the feature stays current with the spec.

## Layout

| Directory | Coverage |
|---|---|
| `core/` | Agent loop, policy, HITL, audit chain, model resolver, network posture, doctor |
| `capsule/` | Capsule composition, manifest schema, signing tiers, install gate, WIT/WASM binding, data-class lattice |
| `chain/` | The five RFC §7.1 contracts + privacy guarantees + EVM adapter |
| `overlays/` | Per-overlay policy bundle behavior; overlay activation ratchet |
| `surfaces/` | Slint app surfaces (concierge, inspector), CLI, daemon, WASM publish, mobile companion |
| `distribution/` | Reproducible builds, signed releases, bundled Gemma 4 E2B, air-gap install, overlay runbooks |

See [`.agentile/planset/2026-05-19-nist-sidecar-v1/FEATURE_INVENTORY.md`](../.agentile/planset/2026-05-19-nist-sidecar-v1/FEATURE_INVENTORY.md)
for the canonical join table.
