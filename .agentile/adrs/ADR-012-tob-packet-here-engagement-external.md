---
created: 2026-05-21T00:00:00Z
branch: feat/s-13-tob-engagement
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 012
sprint: S-13
---

# ADR-012: TOB audit packet lives in `docs/audit/`; engagement runs externally; findings archive lives in `citrate-agentile-archive`

| Field | Value |
|---|---|
| **ADR Number** | ADR-012 |
| **Date** | 2026-05-21 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-13 |

## Context

The S-13 sprint goal is "engage Trail of Bits, deliver the audit
packet, receive findings, and close to no Tier-1 findings
outstanding." Three distinct phases:

1. **Packet authoring.** Scope document, threat model, deployment
   instructions, build instructions, contact info. This is what
   bootstraps the engagement.
2. **External engagement.** TOB's auditors read the packet, ask
   clarifying questions, exercise the code, produce findings.
   Multi-week elapsed time; paid third-party effort.
3. **Findings triage + remediation.** Once findings arrive, we
   classify (Tier-1 / Tier-2 / Tier-3), open follow-up sprints to
   remediate, and obtain sign-off.

These phases share no toolchain and produce different artifacts.
Trying to fit (2) into a Rust sprint is a category error — TOB
operates on their own schedule with their own infrastructure.

The federation already has an established pattern for cross-repo
audits: per-engagement findings live under
`citrate-agentile-archive/audits/<date>-<vendor>-<repo>-v<ver>/`.
That archive is the authoritative store; this repo carries only
the local lineage pointer in `.agentile/audits/`.

## Decision

**S-13 lands (1) — the packet — and stands up the local triage
home for (3). (2) is explicitly external and out of scope for
sprint close.**

### What S-13 ships

In this repo:

- `docs/audit/INDEX.md` — packet cover sheet linking the others.
- `docs/audit/SCOPE.md` — what's in scope (every nist-agent
  workspace crate at S-12's `30044d2`), what's out of scope
  (citrate-agent-runtime upstream, the chain contracts, the
  deferred CI workflow + HSM ceremony), file inventory + line
  counts so TOB can size the engagement.
- `docs/audit/THREAT_MODEL.md` — STRIDE / actor-asset crosswalk
  for the load-bearing surfaces: HIC approval queue, overlay
  activation ratchet, mobile-companion pairing, release
  verifier, audit-sink WORM guarantee, agent loop.
- `docs/audit/DEPLOYMENT.md` — how to stand up a reference
  deployment for hands-on exploration (workspace build + reference
  config + the bundled GGUF placeholder).
- `docs/audit/BUILD.md` — reproducible-build instructions (Rust
  toolchain pinned in `rust-toolchain.toml`, hermetic container
  recipe).
- `docs/audit/CONTACT.md` — escalation paths, secure
  communication channel (PGP / Signal), embargo policy, severity-
  driven response SLAs.
- `docs/audit/V1_READINESS.md` — exit-criteria cross-check
  against the Phase-1 planset.

Local triage scaffolding (no findings yet):

- `.agentile/audits/findings/RUBRIC.md` — tier classification
  (Tier-1 = critical/high; Tier-2 = medium; Tier-3 = low /
  informational) + per-tier remediation SLAs.
- `.agentile/audits/findings/TEMPLATE.md` — frontmatter +
  body shape for individual finding files.
- `.agentile/audits/findings/INDEX.md` — placeholder index that
  will collect per-finding links once the engagement starts.

### What S-13 does NOT ship

- The actual TOB engagement. Runs externally; lands findings on
  their schedule.
- Findings + remediation. Those become S-13a (per-finding
  remediation sprints) once findings arrive.
- v1.0 tag. The planset's Phase-1 exit criteria still include
  "Trail of Bits report received; no Tier-1 findings
  outstanding" — until that's true, we don't tag v1.0. S-13
  closing the *packet* phase is not the same as closing the
  *engagement* phase.

### Archive location

When TOB delivers a report, the per-engagement folder is
`citrate-agentile-archive/audits/2026-XX-XX-tob-nist-agent-v1/`
per the existing federation convention. This repo's
`.agentile/audits/` directory stays a lightweight pointer; the
findings tracker links each finding to its archive entry once
delivered.

## Consequences

**Positive.**

- Sprint scope matches what a Rust workspace can plausibly
  deliver in-session. The packet is the actual deliverable;
  marking the engagement "complete" before findings arrive would
  misrepresent state.
- TOB receives a single, browsable entry point at
  `docs/audit/INDEX.md` — no scavenger hunt across the
  repo to figure out what they're auditing.
- Findings triage has a stable home before the engagement
  starts; when findings arrive, we drop them into the existing
  tree rather than improvising under deadline pressure.

**Negative.**

- The sprint title's optimism ("Trail of Bits engagement
  preparation **+ remediation**") is honest about phase 1 only.
  We close S-13 with phase 2 in flight; a follow-up sprint
  (S-13a) carries remediation. Document this prominently in the
  sprint close note to avoid the v1.0 release being marketed
  as "TOB audited" before it actually is.
- We commit to maintaining the packet documents through code
  changes. A drifted threat-model is worse than no threat-model;
  every future sprint that touches a load-bearing surface should
  re-read the relevant THREAT_MODEL section and update if needed.

**Neutral.**

- The packet docs are markdown, not Rust. No new workspace
  crate. Test count stays flat across S-13; Rule 3 (monotone
  non-decreasing) is satisfied trivially (no regression).
- Frontmatter coverage gains 6 new dated docs under `docs/audit/`
  + 3 under `.agentile/audits/findings/`; baseline ratchet
  unchanged.

## Alternatives considered

1. **Wait until findings are in hand, then run a single S-13.**
   Rejected — the packet bootstraps the engagement; without it,
   the engagement can't start. Waiting is a deadlock.
2. **Skip the packet entirely; let TOB read the source.**
   Rejected — TOB's engagements are scoped by the packet; a
   threat-model-free engagement is a wildly expensive
   exploration rather than a focused audit. The packet is the
   investment that controls audit cost.
3. **Put the packet upstream in `citrate-agent-runtime`.**
   Rejected — the packet describes *this* product's audit
   surface. Upstream has its own audit lineage (separate per-
   engagement folder in the archive).

## Reversal conditions

If TOB engagements become a recurring, multi-product fixture and
multiple sidecars share the same threat-model boilerplate, a
shared `docs/audit/SHARED_*` set can move to the federation
repo. For v1.0, the packet stays here.

## References

- RFC-CIT-AGENT-0001 §11.1 (audit posture).
- `citrate-agentile-archive/audits/AUDIT_INDEX.md` — the
  federation-wide audit index this engagement will append to.
- ADR-011 — the verifier scope-split; the audit surface this
  packet describes consumes the ADR-011 deferred items as
  "out of scope, infrastructure side."
- `.agentile/audits/` — local audit lineage notes.
