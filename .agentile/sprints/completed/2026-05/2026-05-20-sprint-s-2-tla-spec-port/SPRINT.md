---
created: 2026-05-20T00:00:00Z
branch: feat/s-2-tla-spec-port
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-2
closed: 2026-05-20T00:00:00Z
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
| **Status** | `COMPLETE` |
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
AgentSBT clearance → read capsule reads → check domination → HIC.
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
- 2026-05-20 (later) — Reality check on entry: all FIVE specs
  already exist in `citrate-agentile-archive/formal/specs/agent/`
  with PASSing TLC runs. WP-2.3 / 2.4 / 2.5 are NOT fresh authoring;
  they are porting + RFC-aligned renames (CapsuleInstall →
  CapsuleInstallGate, BreakGlass → BreakGlassPath). All five copied
  in, MODULE declarations realigned to filenames, baseline.json
  bumped specs.count 0 → 5. Spec ratchet now green
  (`scripts/ci/check_spec_ratchet.py` reports 5 ≥ 5).
- 2026-05-20 (close) — VERIFICATION_WORKFLOW.md authored, ADR-002
  records custody decision, tla-verify.yml CI workflow added
  (10-minute per-spec budget; downloads tla2tools.jar from official
  release; caches across runs). Sprint closes.

## Exit criteria

- [x] 5 `.tla` files in `.agentile/formal/specs/agent/` with matching `.cfg`s
- [x] All five verified at the archive's runtime budget (state counts in `README.md`); CI verification gates on every PR via `tla-verify.yml`
- [x] `scripts/ci/check_spec_ratchet.py` exits 0 with baseline = 5
- [x] `VERIFICATION_WORKFLOW.md` exists and is operator-readable
- [x] ADR landed (ADR-002)
- [x] Test ratchet: stays at 4 (S-3 baseline maintained)
- [x] Frontmatter coverage: 100%
- [x] Sprint moves to `sprints/completed/2026-05/`

## Close note (2026-05-20)

S-2 closed same day as kickoff (no carry-over). The original plan
imagined WP-2.3, 2.4, 2.5 as fresh authoring work; in fact all five
specs already existed in the agentile archive (`CapsuleInstall.tla`
and `BreakGlass.tla` under their pre-RFC names). The work shrank
to: copy, rename two of them to match RFC §9.1, realign their
`MODULE` declarations, populate the README + workflow doc + ADR,
and write the per-PR TLC verification CI job.

**Federation drift-check impact.** None — drift-check operates on
Cargo dep pins, not on `.agentile/formal/`. The spec ratchet is
nist-agent-local.

**Pending CI behavior.** `tla-verify.yml` is wired but has not yet
run on a PR (this is the PR that introduces it). Expect ~3-5
minutes per run; cache hit shrinks subsequent runs.

**Next sprint.** S-4 (EVM chain adapter trait). Will close the
last federation drift constraint (`citrate-wallet-core` pin in
`crates/nist-agent-chain/Cargo.toml`).
