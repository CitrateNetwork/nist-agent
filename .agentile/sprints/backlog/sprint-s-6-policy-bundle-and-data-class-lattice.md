---
created: 2026-05-20T00:00:00Z
branch: main
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: backlog
sprint: S-6
---

# Sprint S-6: Policy bundle + data-class lattice

**Goal.** Land the signed `PolicyBundle` (canonical CBOR), the
five-level data-class lattice (PUBLIC < CUI < PHI < FERPA < ITAR),
risk-tier mapping, and the overlay one-way activation ratchet. PR
upstream to runtime.

**Why now.** Overlays (S-8, S-9) need the schema to exist. The
empty `policy::` mod must fill.

**Predecessors.** S-3, S-2 (DataClassLattice.tla).

**Features owned.**
- `features/core/policy-bundle.feature`
- `features/overlays/overlay-activation-ratchet.feature`
- (paired spec: `DataClassLattice.tla`)

**Cross-repo.** Upstream PR to runtime.

**Exit criteria.**
- `PolicyBundle` parses/encodes canonical CBOR (RFC 8949 §4.2.1).
- SecurityOfficer signature verified at load.
- Lattice enforced on every capsule `call`.
- Overlay activation enforced as one-way ratchet; decommission requires workflow.
- TLA+ `DataClassLattice.tla` verifies and is wired into ratchet.
