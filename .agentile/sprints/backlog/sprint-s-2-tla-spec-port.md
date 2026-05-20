---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-2
---

# Sprint S-2: TLA+ spec port

**Goal.** Port the five normative TLA+ specs locally (
`ApprovalStateMachine`, `AuditChainIntegrity`, `DataClassLattice`,
`CapsuleInstallGate`, `BreakGlassPath`) and wire
`scripts/ci/check_spec_ratchet.py` so spec verification is a Rule-10
BLOCKER on every merge.

**Why now.** The RFC names these five specs as normative for v1.0
(§9.1). Two of them are already verified inside `citrate-agent-runtime`
but live in the agentile archive; the other three exist only as
referenced names. We take canonical custody locally so the product
compliance posture (this repo) owns the safety arguments.

**Predecessors.** S-1.

**Features owned.**
- `features/capsule/capsule-install-gate.feature` (paired spec: `CapsuleInstallGate.tla`)
- `features/capsule/capsule-data-class-lattice.feature` (paired spec: `DataClassLattice.tla`)
- `features/core/hitl-break-glass.feature` (paired spec: `BreakGlassPath.tla`)

**Cross-repo.** None expected if specs land here canonically; runtime
references our copies via path.

**Exit criteria.**
- Five `.tla` and `.cfg` files at `.agentile/formal/`.
- `scripts/ci/check_spec_ratchet.py` exits 0 locally and in CI.
- Each spec has a one-paragraph README under `.agentile/formal/` linking the matching `.feature` file.
- `.agentile/formal/VERIFICATION_WORKFLOW.md` updated with the state counts for each spec.
