---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-7
---

# Sprint S-7: Doctor checks 6–11

**Goal.** Land the remaining six RFC §10.2 doctor checks:
network-posture, model-hash, TLA-status, role-lattice coverage,
queue SLA, break-glass affirmation window. PR upstream.

**Why now.** Without all eleven checks, we cannot claim CA-7
continuous-monitoring evidence completeness. Doctor is the central
NIST audit artifact.

**Predecessors.** S-5 (model resolver hash check), S-6 (PolicyBundle
network posture), S-2 (TLA status reader).

**Features owned.**
- `features/core/doctor-preflight.feature`
- `features/core/network-posture.feature`

**Cross-repo.** Upstream PR to runtime.

**Exit criteria.**
- All eleven checks implemented and unit-tested.
- Signed TOML report contains all eleven check entries on a reference deployment.
- Each Phase-1 overlay's reference deployment passes (no BLOCKER) on a fresh install.
- `doctor` failure with BLOCKER prevents the harness from starting.
