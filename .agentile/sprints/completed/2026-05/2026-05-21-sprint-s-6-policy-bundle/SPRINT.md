---
created: 2026-05-21T00:00:00Z
branch: feat/s-6-policy-bundle
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-6
closed: 2026-05-21T00:00:00Z
---

# Sprint S-6: PolicyBundle + data-class lattice

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-6` |
| **Sprint Name** | PolicyBundle (canonical CBOR + Ed25519) + data-class lattice + overlay ratchet |
| **Goal** | Land the empty `citrate-agent-core::policy` slot locally in `nist-agent-policy` — signed `PolicyBundle`, Bell-LaPadula data-class lattice (PUBLIC < CUI < PHI < FERPA < ITAR), five-role lattice, risk-tier escalation rule, anchor strategy + egress posture enums, one-way overlay activation ratchet. Per ADR-005, upstream PR follows. |
| **Branch** | `feat/s-6-policy-bundle` |
| **Start Date** | 2026-05-21 |
| **End Date** | 2026-05-21 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-3 (Cargo workspace), S-2 (DataClassLattice.tla), S-4 (chain Clearance enum we cross-align with) |

## Why this sprint

PolicyBundle is the load-bearing configuration object — every
other sprint binds to it:

- S-7 (Doctor) reads it for signature freshness + role-lattice
  checks.
- S-8 / S-9 (Overlay bundles) author concrete PolicyBundle
  instances per overlay.
- S-10 (Slint concierge) presents `Role::ALL` and the
  `ActiveOverlays` ratchet in onboarding panes.
- S-12 (Distribution) bundles a signed default PolicyBundle with
  the air-gap install image.

Per ADR-005, the local landing follows the ALIGNMENT.md exception-
clause precedent (third such exercise after ADR-002/004); upstream
PR to `citrate-agent-runtime` is a follow-up operator action.

## Deliverables

- `crates/nist-agent-policy/` — workspace member.
  - `PolicyBundle` struct with canonical CBOR encode/decode +
    `RawSignedBundle` Ed25519 verification with non-canonical-CBOR
    defense-in-depth.
  - `DataClass` lattice (PUBLIC..ITAR) with `dominates()`
    operator + cross-layer alignment test with `nist-agent-chain::Clearance`.
  - `ActiveOverlays` one-way ratchet — `add()` is free,
    `remove_with_workflow()` requires a `DecommissioningWorkflow`
    proof token, `remove_without_workflow()` always errors,
    CMMC-L3 baseline is non-removable.
  - `RiskTier` (Low/Medium/High/Critical) with non-de-escalation
    rule enforced at `activate()` time.
  - `Role` enum (the five base roles) + `Role::ALL` constant +
    role-assignment completeness check at `activate()`.
  - `AnchorStrategy` (4 variants per RFC §6.3) + `EgressPosture`
    (3 variants per RFC §3.3) enums.
  - `PolicyError` typed errors.
- ADR-005 records the placement decision + cross-layer
  alignment strategy for the `DataClass`/`Clearance` duplication.
- Workspace deps: ciborium 0.2, ed25519-dalek 2.1 (std).
- Test count: 41 → 66 (+25 new policy tests).

## Test Baseline (start of sprint)

| Metric | Count | Captured |
|---|---|---|
| Tests | 41 | 2026-05-21 |
| Formal specs | 5 | 2026-05-21 |
| CI tripwires | 7 | 2026-05-21 |
| Frontmatter coverage | 100% | 2026-05-21 |
| Drift constraints | 11/11 green | 2026-05-21 |
| Workspace crates | 4 | 2026-05-21 |

## Method

Bind to the RFC §2.3 / §3.1 / §5.2 / §5.3 / §7.2 surface as the
shape contract; canonical CBOR per RFC 8949 §4.2.1; Ed25519 via
dalek 2.1; cross-layer alignment test for the lattice/clearance
byte-mapping.

## Work Packages

### WP-6.1 — `DataClass` lattice (DONE)

Five-variant enum, `dominates()` operator, `from_u8()` decoder,
cross-layer alignment test pinning byte values 0..4 against
`nist-agent-chain::Clearance`. 5 unit tests.

### WP-6.2 — `Role`, `RiskTier`, `AnchorStrategy`, `EgressPosture` (DONE)

The four core enums. `Role::ALL` constant for activation-time
completeness check. Serde kebab-case rendering pinned by test.
3 unit tests.

### WP-6.3 — `ActiveOverlays` one-way ratchet (DONE)

`add()` idempotent and free. `remove_with_workflow()` consumes
`DecommissioningWorkflow` proof; CMMC-L3 baseline is
non-removable; removing an inactive overlay is a no-op (not an
error). `remove_without_workflow()` always errors. 6 unit tests.

### WP-6.4 — `PolicyBundle` + canonical CBOR + Ed25519 (DONE)

CBOR encoding deterministic via ciborium. `canonical_hash()` for
audit-record bundle references. `activate()` checks role
completeness, bundle version, validity window, and risk-tier
non-de-escalation. `RawSignedBundle::verify_and_decode()` does
signature check AND non-canonical-CBOR re-encode defense. 8 unit
tests covering round-trip, role enforcement, escalation rule,
validity window, wrong-key rejection, tampered-bytes rejection,
non-canonical encoding rejection.

### WP-6.5 — `Overlay` derives `Ord` (in prelude) (DONE)

PolicyBundle's `ActiveOverlays` stores `BTreeSet<Overlay>`. The
prelude's `Overlay` enum gained `PartialOrd, Ord` derives so the
BTreeSet works. No behavioral change; cosmetic.

### WP-6.6 — ADR-005 (DONE)

Records placement decision, cross-layer-alignment strategy for
the lattice/clearance duplication, reversal conditions.

### WP-6.7 — Upstream PR (DEFERRED to S-6b)

Same shape as ADR-004's WP-5.5. Branch on runtime, lift
`nist-agent-policy/src/*` into `agent/core/src/policy/`, update
runtime Cargo.toml deps. Tracked in RETRO.

### WP-6.8 — Wire PolicyBundle into agent loop + model resolver (DEFERRED)

The loop should honor the risk-tier map; the model resolver
should read `egress_posture` to block egress-required paths.
Both are cross-crate wiring concerns — sized as a separate small
sprint, doesn't block S-7.

## Daily updates

- 2026-05-21 — Kickoff and close in one session. WP-6.1 → 6.6
  done; WP-6.7 + 6.8 deferred. 25 new tests bring the workspace
  to 66. fmt + clippy + deny all clean locally.

## Exit criteria

- [x] `nist-agent-policy` crate compiles + 25 tests pass + clippy clean
- [x] Canonical CBOR round-trip is byte-deterministic
- [x] SecurityOfficer Ed25519 signature verifies on round-trip; wrong
      key + tampered bytes + non-canonical bytes all reject hard
- [x] `activate()` enforces role completeness, validity window, and
      risk-tier non-de-escalation
- [x] `ActiveOverlays::add()` is the only ratchet-forward operation;
      `remove_with_workflow()` is the ONLY ratchet-backward path
- [x] Cross-layer alignment test pins `DataClass` byte values to
      `nist-agent-chain::Clearance`
- [x] ADR-005 landed
- [x] Test ratchet up (41 → 66)
- [x] Frontmatter coverage 100%
- [ ] Upstream PR on `citrate-agent-runtime` (DEFERRED to S-6b)

## Close note (2026-05-21)

S-6 closed in a single session. Three crate-internal modules
went in lock-step (lattice, overlay_state, bundle) because they
share the `PolicyBundle` struct shape. The hardest call was the
`DataClass`/`Clearance` duplication — ADR-005 records the
cross-layer test pinning as the temporary discipline, with the
unification follow-up tracked.

**Pending follow-ups.**

1. **Upstream PR on `citrate-agent-runtime`** lifting
   `nist-agent-policy/src/*` into
   `citrate_agent_core::policy`. Same shape as ADR-004's WP-5.5.
2. **Wire PolicyBundle into loop + model resolver.** Loop reads
   the risk-tier map for proposed-action escalation; model
   resolver reads `egress_posture` to refuse egress-required
   sources when disabled. Cross-crate wiring, half-sprint.
3. **Unify `DataClass`/`Clearance` enums.** Either a new
   `nist-agent-types` crate or move both into upstream's
   `citrate_agent_core::types`. Tracked at the same time as the
   upstream PRs so the move is one commit.
4. **Larger overlay-state history.** Today `ActiveOverlays`
   tracks `active` + `decommissioned` as sets. Production
   audit-trail needs a *timeline* (when each overlay activated /
   decommissioned with dates and workflow refs). Sized as
   S-6b-extension or absorbed into S-8 when the first real
   overlay bundle gets authored.

**Next sprint.** S-7 (Doctor checks 6-11) is now unblocked. It
will read the PolicyBundle for its check #3 (policy bundle
signed + in validity window).
