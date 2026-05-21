---
created: 2026-05-20T00:00:00Z
branch: feat/s-2-tla-spec-port
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: accepted
adr: 002
sprint: S-2
---

# ADR-002: TLA+ custody lives in nist-agent, not citrate-agent-runtime

| Field | Value |
|---|---|
| **ADR Number** | ADR-002 |
| **Date** | 2026-05-20 |
| **Status** | ACCEPTED |
| **Author** | Saul Loveman + Claude Opus 4.7 (1M context) |
| **Sprint** | S-2 |

## Context

RFC-CIT-AGENT-0001 §9.1 names five normative TLA+ specifications
that v1.0 MUST verify in CI as a Rule-10 BLOCKER:

1. `ApprovalStateMachine` — RFC §5 HITL state machine
2. `AuditChainIntegrity` — RFC §6 hash-chained audit log
3. `DataClassLattice` — RFC §7.2 Bell-LaPadula adapted
4. `CapsuleInstallGate` — RFC §4.5 / §7.2 (archived as `CapsuleInstall`)
5. `BreakGlassPath` — RFC §5.5 (archived as `BreakGlass`)

All five exist in `citrate-agentile-archive/formal/specs/agent/`
and PASS at runtime's 180s × 4 workers × 2.5GB heap budget. The
question this ADR resolves is: which repo holds canonical custody?

[`planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md)
listed the question explicitly:

> **TLA+ spec custody.** The two specs that are "DONE in runtime"
> — do they live canonically in runtime or in nist-agent? Working
> assumption: in nist-agent (since this repo asserts the product
> compliance posture); runtime references our copies.

Three forces in tension:

1. **Engine vs. product distinction.** `citrate-agent-runtime` is
   the engine — it implements the eight RFC subsystems as a
   library. nist-agent is the product — it asserts the compliance
   posture, packages the distribution, ships overlays, runs the
   Trail of Bits engagement. The compliance assertion is what TLA+
   verifies; therefore the product owns the assertion.
2. **Where the audit lives.** When DCMA-DIBCAC or Trail of Bits
   reviews the assessment evidence, they walk into the *product*
   repo's `.agentile/formal/` directory. If the specs are in
   runtime, the auditor follows a link to a different repo with
   different release cadence and different audit tier — friction
   and confusion. Holding the specs in nist-agent keeps the
   evidence walk in one place.
3. **Cargo dependency direction.** nist-agent depends on
   `citrate-agent-core`, not the other way around. If specs lived
   in runtime, nist-agent would have to fetch them via federation
   manifest pin to verify them locally — adding a build-time
   dependency on the engine for what should be a static check.

## Decision

**We will hold the canonical copies of all five normative TLA+
specs in `nist-agent/.agentile/formal/specs/agent/`.**
`citrate-agent-runtime` references our copies via path or link;
the archive's prior copies are now historical only.

Concretely, S-2 lands:

1. The five specs ported into
   `.agentile/formal/specs/agent/` with their `.cfg` files. Two of
   them (`CapsuleInstall`, `BreakGlass`) are renamed to match the
   RFC §9.1 names (`CapsuleInstallGate`, `BreakGlassPath`); their
   `MODULE` declarations are updated to match the new filenames.
   No behavioral changes.
2. `.agentile/coverage/baseline.json` `specs.count` bumped to 5.
3. `.github/workflows/tla-verify.yml` runs TLC against every spec
   on every PR — this is the Rule-10 BLOCKER, not just the
   file-count ratchet.
4. `.agentile/formal/{README,VERIFICATION_WORKFLOW}.md` document
   the inventory and the operator path.

## Consequences

**Positive.**

- A Trail of Bits assessor (S-13) walks into nist-agent and finds
  the safety-property evidence alongside the policy bundle, the
  doctor outputs, the overlay capsules. One repo, one audit trail.
- `citrate-agent-runtime`'s release cadence stays decoupled from
  formal-verification cadence. A spec tweak no longer needs a
  runtime release.
- nist-agent's CI now has a real Rule-10 BLOCKER. Previously the
  spec ratchet was a no-op (count = 0).

**Negative.**

- If `citrate-agent-runtime` lands a behavior change that requires
  a spec update, the workflow is: PR the spec update to nist-agent
  first (verify), then PR the behavior change to runtime
  referencing the verified spec. One PR becomes two. Acceptable —
  this is what Rule-10-first looks like.
- Cross-repo `git log` is now needed to trace a spec change's
  motivation if the motivation lives in a runtime commit. The
  spec's commit message must name the runtime commit it responds
  to (a new convention introduced by this ADR).

**Neutral.**

- The archive's copies (`citrate-agentile-archive/formal/specs/agent/`)
  remain as the pre-split historical record. They are not
  authoritative going forward. ALIGNMENT.md and the spec README
  both link to nist-agent's copies as canonical.

## Alternatives considered

1. **Custody in `citrate-agent-runtime`.** Rejected: the
   compliance assertion is the product's, not the engine's. See
   "Forces in tension" #1–3 above.
2. **Custody in `citrate-federation`.** Rejected: the federation
   holds cross-repo *coordination state*, not per-repo content.
   Putting specs in the federation would create a third bookkeeping
   surface (manifest pin → federation copy → runtime/nist-agent
   consumers).
3. **Custody split: 3 specs in nist-agent, 2 in runtime.** Rejected:
   the audit walk benefits from co-location. Splitting forces the
   assessor (and tools like Trail of Bits' analysis pipeline) to
   resolve two locations.

## Reversal conditions

This ADR is reversed if any of:

- A second product builds on `citrate-agent-core` and asserts a
  *different* compliance posture (e.g., a hypothetical
  `gdpr-agent` for EU-only deployments). The specs that are
  shared between products move to runtime; product-specific specs
  stay in their respective product repos.
- The federation introduces a `formal/` corpus that's authoritative
  across all consumers (new monorepo-style invariant). Unlikely
  but possible.

If reversed, a successor ADR supersedes this one and the specs
move with one commit per affected repo, preserving git history.

## References

- RFC-CIT-AGENT-0001 §9.1 (the five normative specs).
- [`.agentile/planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md) — "Open ownership questions" section.
- [`.agentile/formal/README.md`](../formal/README.md) — current inventory.
- Federation rule 10 (Authorization before destructive ops; safety-critical specs verify in CI as BLOCKER).
- Federation rule 9 (One source of truth per topic. Link, don't copy.).
