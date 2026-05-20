---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: active
sprint: S-2
---

# Sprint S-2: TLA+ spec port

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-2` |
| **Sprint Name** | TLA+ spec port — five normative specs land locally |
| **Goal** | Port the five normative TLA+ specs (`ApprovalStateMachine`, `AuditChainIntegrity`, `DataClassLattice`, `CapsuleInstallGate`, `BreakGlassPath`) into `.agentile/formal/`, verify each under TLC, and wire `scripts/ci/check_spec_ratchet.py` as a Rule-10 BLOCKER on every merge. |
| **Branch** | `main` |
| **Start Date** | 2026-05-20 |
| **End Date (target)** | 2026-06-03 |
| **Status** | `KICKED OFF` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-1 (closed 2026-05-20) |

## Why this sprint

The RFC names five TLA+ specs as normative for v1.0 (§9.1) and
Rule 10 makes their CI verification a BLOCKER on every merge. Two
(`ApprovalStateMachine`, `AuditChainIntegrity`) are already verified
inside `citrate-agent-runtime` but live in `citrate-agentile-archive`;
the other three (`DataClassLattice`, `CapsuleInstallGate`,
`BreakGlassPath`) exist only as referenced names in module docs.

Per [`ALIGNMENT.md`](../../planset/2026-05-19-nist-sidecar-v1/ALIGNMENT.md),
nist-agent takes canonical custody of all five specs because this
repo asserts the product compliance posture; runtime references our
copies.

Without S-2, every subsequent sprint that touches safety-critical
control flow (S-5 agent loop, S-6 policy, S-7 doctor checks, S-11
mobile) has no machine-checkable safety contract to verify against.

## Deliverables

- `.agentile/formal/ApprovalStateMachine.tla` + `.cfg` (ported from runtime, re-verified locally)
- `.agentile/formal/AuditChainIntegrity.tla` + `.cfg` (ported, re-verified)
- `.agentile/formal/DataClassLattice.tla` + `.cfg` (new, authored from RFC §7.2)
- `.agentile/formal/CapsuleInstallGate.tla` + `.cfg` (new, authored from RFC §4.5 + §7.2)
- `.agentile/formal/BreakGlassPath.tla` + `.cfg` (new, authored from RFC §5.5)
- `.agentile/formal/VERIFICATION_WORKFLOW.md` — operator-facing doc explaining how to run TLC locally and how the CI ratchet works.
- `.agentile/formal/README.md` per spec — links to the matching `.feature` file, names the safety invariant in English, records state count + verification date.
- `scripts/ci/check_spec_ratchet.py` — wired and green (skeleton already ships it; this sprint validates it works in our config).
- ADR: `.agentile/adrs/2026-XX-XX-tla-custody-in-nist-agent.md` — captures the ownership decision per ALIGNMENT.md.

## Test Baseline (start of sprint)

| Metric | Count | Captured | Canonical command |
|---|---|---|---|
| **Tests** | 0 | 2026-05-20 | (deferred to S-3) |
| **Formal specs** | 0 | 2026-05-20 | `scripts/ci/check_spec_ratchet.py` |
| **CI tripwires** | 7 | 2026-05-20 | `scripts/ci/check_tripwire_ratchet.py` |
| **Frontmatter coverage** | 50/50 | 2026-05-20 | `scripts/ci/check_frontmatter.py` |

## Method

Per Agentile sprint method: TLA+ first (this is the TLA+ sprint),
BDD/Gherkin already present (features land in S-1), failing test +
tripwire come next, then code (S-5 onward). For each spec:

1. Read the paired `.feature` file under `features/`.
2. Re-read the RFC section it maps to.
3. Author the TLA+ spec: variables, init, next-state, safety
   invariant, liveness property if applicable.
4. Author the `.cfg` with `SPECIFICATION`, `INVARIANTS`, model
   bounds.
5. Run `tlc -workers auto <spec>.tla` locally; record state count.
6. Author a README.md alongside the spec explaining the invariant in
   English.
7. Wire into `scripts/ci/check_spec_ratchet.py` baseline.
8. Open upstream PR adding TLC artifact pin to `citrate-federation`
   if needed for cross-repo verification.

## Work Packages

### WP-2.1 — Port `ApprovalStateMachine.tla` from runtime

Source: `citrate-agent-runtime` archive. Goal: byte-identical port
plus local TLC run (target: 336k+ states, matches runtime).
Acceptance: `.tla` + `.cfg` + README in `.agentile/formal/`;
`scripts/ci/check_spec_ratchet.py` includes it; verification logged.

### WP-2.2 — Port `AuditChainIntegrity.tla` from runtime

Same shape as WP-2.1. Target: 35k+ states.

### WP-2.3 — Author `DataClassLattice.tla`

New. Models the PUBLIC < CUI < PHI < FERPA < ITAR lattice plus
per-capsule read/write/emit constraint. Safety invariant: no
emission's class is dominated-by-but-not-equal-to the union of read
classes minus the redact-and-attest set. State bounds: 4 clearance
levels, 3 capsule slots, 5 actions per slot. Target state count:
research yields this.

### WP-2.4 — Author `CapsuleInstallGate.tla`

New. Models the four-step install ladder from RFC §7.2: read
AgentSBT clearance → read capsule reads → check domination → HITL.
Safety invariant: no capsule reaches `installed` state with reads
that exceed clearance under the lattice. State bounds: 3 capsules,
4 clearance levels, 2 chain reachability states (online / cached).

### WP-2.5 — Author `BreakGlassPath.tla`

New. Models the 72-hour affirmation window from RFC §5.5. Safety
invariants: (a) break-glass actions always have at least one SO
signature; (b) an action remains flagged on every doctor report
until affirmed or formally remediated; (c) ITAR-classified data is
never reachable from a break-glass-eligible state. Liveness:
eventually every break-glass action is either affirmed or
remediated.

### WP-2.6 — Write `VERIFICATION_WORKFLOW.md`

Operator-facing: how to install `tla2tools.jar`, how to run TLC
locally, how the CI ratchet works, how to add a new spec without
breaking the ratchet.

### WP-2.7 — ADR — TLA+ custody lives in nist-agent

Per ALIGNMENT.md. Acceptance: ADR file under `.agentile/adrs/`
with Rule-12 frontmatter, references the open ownership question
from ALIGNMENT.md, records the decision.

### WP-2.8 — CI wiring

Acceptance: `scripts/ci/check_spec_ratchet.py` is invoked by the
agentile CI workflow; baseline = 5; spec count ≥ 5 on every PR.

## Daily updates

- 2026-05-20 — Kickoff. Backlog stub promoted to active; WPs
  enumerated. Approach for WP-2.1 / WP-2.2: pull spec sources from
  `citrate-agent-runtime` and the agentile archive on first
  authoring session. WP-2.3–2.5 require fresh authoring.

## Exit criteria

- [ ] 5 `.tla` files in `.agentile/formal/` with matching `.cfg`s
- [ ] All five verify locally under TLC; state counts recorded in each spec's README.md
- [ ] `scripts/ci/check_spec_ratchet.py` exits 0 with baseline = 5
- [ ] `VERIFICATION_WORKFLOW.md` exists and is operator-readable
- [ ] ADR landed
- [ ] Test ratchet: still 0 (S-3 baselines this)
- [ ] Frontmatter coverage: 100%
- [ ] Sprint moves to `sprints/completed/2026-06/` (or later)

## Close note

(filled in at close)
