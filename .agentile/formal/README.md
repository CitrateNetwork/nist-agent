---
created: 2026-05-20T00:00:00Z
branch: feat/s-2-tla-spec-port
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
---

# .agentile/formal/ — Normative TLA+ specifications

> The five safety-critical TLA+ specs RFC-CIT-AGENT-0001 §9.1 names
> as normative for v1.0. Each spec models one subsystem of the
> harness; each is paired with a Gherkin `.feature` file under
> `features/`. Spec verification is a Rule-10 BLOCKER on every PR.

## Inventory

| Spec | RFC § | Maps to feature | TLC states (archive, runtime budget) |
|---|---|---|---|
| [`ApprovalStateMachine`](specs/agent/ApprovalStateMachine.tla) | §5 | [`features/core/hitl-quorum.feature`](../../features/core/hitl-quorum.feature) + [`hitl-state-managed-interrupt.feature`](../../features/core/hitl-state-managed-interrupt.feature) | 336,292 |
| [`AuditChainIntegrity`](specs/agent/AuditChainIntegrity.tla) | §6 | [`features/core/audit-chain-append.feature`](../../features/core/audit-chain-append.feature) + [`audit-anchor-strategies.feature`](../../features/core/audit-anchor-strategies.feature) | 35,435 |
| [`DataClassLattice`](specs/agent/DataClassLattice.tla) | §7.2 | [`features/capsule/capsule-data-class-lattice.feature`](../../features/capsule/capsule-data-class-lattice.feature) + [`features/core/policy-bundle.feature`](../../features/core/policy-bundle.feature) | BOUNDED_EXPLORATION ≥3.4M states |
| [`CapsuleInstallGate`](specs/agent/CapsuleInstallGate.tla) | §4.5, §7.2 | [`features/capsule/capsule-install-gate.feature`](../../features/capsule/capsule-install-gate.feature) | 2,600 |
| [`BreakGlassPath`](specs/agent/BreakGlassPath.tla) | §5.5 | [`features/core/hitl-break-glass.feature`](../../features/core/hitl-break-glass.feature) | BOUNDED_EXPLORATION |

The `CapsuleInstallGate` and `BreakGlassPath` specs were ported from
`citrate-agentile-archive/formal/specs/agent/{CapsuleInstall,BreakGlass}.tla`
and renamed to match the RFC §9.1 normative names. The behavior is
unchanged; only the `MODULE` declaration was edited.

## Custody

Per [`ADR-002`](../adrs/ADR-002-tla-custody-in-nist-agent.md) and
[`planset/.../ALIGNMENT.md`](../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md),
**nist-agent holds the canonical copy** of these five specs.
`citrate-agent-runtime` references our copies. When a spec changes,
the change lands here first.

## How to verify locally

See [`VERIFICATION_WORKFLOW.md`](VERIFICATION_WORKFLOW.md). The
short version:

```bash
# install tla2tools.jar (one-time)
curl -L https://github.com/tlaplus/tlaplus/releases/latest/download/tla2tools.jar \
  -o ~/.local/share/tla2tools.jar

# verify a spec
java -jar ~/.local/share/tla2tools.jar -workers auto \
  -config .agentile/formal/specs/agent/ApprovalStateMachine.cfg \
  .agentile/formal/specs/agent/ApprovalStateMachine.tla
```

## Ratchet

`scripts/ci/check_spec_ratchet.py` enforces Rule 10's file-count
floor (currently 5; baseline in `.agentile/coverage/baseline.json`).
`.github/workflows/tla-verify.yml` runs full TLC verification on
every PR — that is the load-bearing BLOCKER per Rule 10.

### BOUNDED_EXPLORATION

`BreakGlassPath` and `DataClassLattice` exceed the per-PR CI budget
(10 min × 2 cores × 2.5GB heap on a GitHub Actions hosted runner).
TLC's progressive search runs for the full budget without finding
any invariant violation, then the wall-clock kills it. Per
[`VERIFICATION_WORKFLOW.md`](VERIFICATION_WORKFLOW.md) option 4 and
the archive's `TLC_BASELINE.md` pattern, this is recorded as
"bounded exploration with zero violations" — tolerated by CI, not
ignored. Three follow-up paths are tracked in the S-2 RETRO:

1. Tighten the `.cfg` constants (lower `CONSTANTS` cardinalities)
   so the bounded model fits within the CI budget.
2. Provision a slow-track CI workflow (`tla-verify-slow.yml`)
   running these specs nightly with a 60-minute budget per spec.
3. Use a self-hosted runner with more CPU/RAM that completes the
   full search.

The `tla-verify.yml` workflow has a hard-coded allow-list of
`BOUNDED_EXPLORATION_SPECS`. Adding a new entry to that list is a
sprint-level decision — surface it in the PR description and the
relevant sprint's RETRO.
