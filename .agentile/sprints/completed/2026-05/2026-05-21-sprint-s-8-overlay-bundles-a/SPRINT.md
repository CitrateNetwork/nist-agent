---
created: 2026-05-21T00:00:00Z
branch: feat/s-8-overlay-bundles-a
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-8
closed: 2026-05-21T00:00:00Z
---

# Sprint S-8: Overlay bundles A (CMMC-L3 + FERPA + COPPA + CIPA)

## Sprint Metadata

| Field | Value |
|---|---|
| **Sprint ID** | `S-8` |
| **Sprint Name** | Overlay bundles A — PolicyBundle factories + retention floor table + runbooks |
| **Goal** | Land signed PolicyBundle factory functions for the four v1.0 overlays — CMMC-L3 baseline, FERPA, COPPA, CIPA — plus the RFC §6.4 retention floor table. Each factory's output passes signature verification, activation, and the relevant doctor checks. Compliance runbooks at `docs/compliance/<overlay>/RUNBOOK.md`. |
| **Branch** | `feat/s-8-overlay-bundles-a` |
| **Start Date** | 2026-05-21 |
| **End Date** | 2026-05-21 |
| **Status** | `COMPLETE` |
| **Planset** | [`../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md`](../../planset/2026-05-19-nist-sidecar-v1/OVERVIEW.md) |
| **Predecessors** | S-6 (PolicyBundle), S-7 (doctor checks) |

## Why this sprint

S-6 landed the PolicyBundle *type*; S-7 landed the doctor checks
that *validate* a bundle. S-8 lands the actual *content* — typed
factory functions that produce the canonical bundle for each
v1.0 overlay. Operators in K-12, defense industrial base, and
small government tiers can construct a deploy-ready bundle in
one function call:

```rust
let bundle = nist_agent_overlays::ferpa(OverlayBuilder { ... });
```

Plus the retention floor table per RFC §6.4, cross-pinned to
`features/core/audit-retention.feature` so the harness's
retention scheduler (downstream) has a typed source.

## Deliverables

- `crates/nist-agent-overlays/` — new workspace member.
  - `OverlayBuilder` — shared options struct (role assignments +
    validity window).
  - `cmmc_l3_baseline(opts) -> PolicyBundle` — RFC §2.1 baseline.
  - `ferpa(opts) -> PolicyBundle` — CMMC + FERPA.
  - `coppa(opts) -> PolicyBundle` — CMMC + COPPA.
  - `cipa(opts) -> PolicyBundle` — CMMC + CIPA.
  - `retention::retention_floor(overlay) -> Duration` per RFC §6.4.
  - `retention::effective_floor(active_set) -> Duration` —
    MAX of each member's floor.
- `docs/compliance/{cmmc-l3,ferpa,coppa,cipa}/RUNBOOK.md` —
  per-overlay deployment runbooks with Rule-12 frontmatter.
- Test count 80 → 93 (+13 covering construction, canonical CBOR
  round-trip, activate, doctor PolicyBundleValidity check, doctor
  RoleLattice check, retention table, effective-floor MAX).

## Test Baseline (start of sprint)

| Metric | Count | Captured |
|---|---|---|
| Tests | 80 | 2026-05-21 |
| Formal specs | 5 | 2026-05-21 |
| CI tripwires | 7 | 2026-05-21 |
| Frontmatter coverage | 100% | 2026-05-21 |
| Drift constraints | 11/11 green | 2026-05-21 |
| Workspace crates | 6 | 2026-05-21 |

## Method

Each overlay factory is a thin wrapper over `cmmc_l3_baseline`
that adds the appropriate variant to `ActiveOverlays` via the
one-way ratchet's `add()` method (S-6's API). Bundle round-trip
goes through the canonical-CBOR + Ed25519 path landed in S-6.
Doctor verification reuses S-7's checks.

The retention table cross-pins against
`features/core/audit-retention.feature` to keep the constants
honest: a test asserts the 6/6/3/5/5 mapping pinned by the
feature scenarios.

## Work Packages

### WP-8.1 — `OverlayBuilder` + cmmc_l3_baseline factory (DONE)

The base. Pre-seeds CMMC-L3 via
`ActiveOverlays::new_with_cmmc_baseline()`. Defaults:
HybridNightlyPlusCapsule anchor strategy, Disabled egress
posture, empty risk_tier_map, role_assignments from the builder.

### WP-8.2 — ferpa / coppa / cipa factories (DONE)

Each calls `cmmc_l3_baseline` then `overlays.add(Overlay::X)` and
sets a descriptive `bundle_name`. RFC-§2.3-prescribed posture
deltas (mobile signing TTL, ComplianceOfficer-mandatory emit,
parental-consent gating, CIPA content filter routing) are
enforced at the HIC gate and capsule level — the bundle just
declares the overlay set.

### WP-8.3 — Retention floor table (DONE)

`retention_floor(overlay) -> Duration` with the 6/5/5/5/6/3
year mapping cross-pinned to `audit-retention.feature`.
`effective_floor(active)` takes the MAX across active overlays
(longest-floor wins); empty set defaults to CMMC-L3's 6 years.

### WP-8.4 — Tests (DONE)

13 tests across `profiles.rs` + `retention.rs`:
- Construction shape per overlay
- Canonical CBOR sign / verify round-trip per overlay
- `activate(0, &empty)` passes per overlay
- Doctor's `PolicyBundleValidityCheck` returns Pass per overlay
- Doctor's `RoleLatticeCheck` returns Pass per overlay
- Retention table matches `audit-retention.feature` values
- COPPA + CIPA match FERPA's floor (school-context)
- Effective floor takes MAX across active set
- Empty set falls back to CMMC baseline
- CMMC baseline is never dropped by an overlay factory
  (cross-check of the type-system property)

### WP-8.5 — Runbooks (DONE)

Four `docs/compliance/<overlay>/RUNBOOK.md` files with Rule-12
frontmatter, deployment artifacts, pre-deploy checklists,
decommissioning procedures, see-also cross-references.

### WP-8.6 — WIT/WASM supporting capsules (DEFERRED to S-8b)

The redact-and-attest, parental-consent-verifier, and
CIPA-content-filter capsules need cargo-component + WASM build
infrastructure that's its own sprint. Tracked in RETRO.

### WP-8.7 — Bundle v2 with embedded retention (DEFERRED)

RFC §6.4 implies the retention floor is in the bundle. S-8 puts
it in a constant table; v2 of the bundle moves it into the
signed payload. Tracked.

## Daily updates

- 2026-05-21 — Kickoff and close in one session. WP-8.1–8.5 done;
  WP-8.6 + 8.7 deferred. 13 new tests bring the workspace to 93.

## Exit criteria

- [x] `nist-agent-overlays` crate compiles + tests pass + clippy clean
- [x] Each factory produces a bundle that round-trips through
      canonical CBOR + Ed25519 signature
- [x] Each bundle activates against `now=0` + empty prior_tiers
- [x] Doctor PolicyBundleValidityCheck + RoleLatticeCheck pass
      for every factory output
- [x] Retention table cross-pinned to `audit-retention.feature`
- [x] Four runbooks under `docs/compliance/<overlay>/RUNBOOK.md`
- [x] Test ratchet up (80 → 93)
- [x] Frontmatter coverage stays at 100%
- [ ] WIT/WASM supporting capsules (DEFERRED to S-8b)
- [ ] Bundle v2 with embedded retention (DEFERRED)

## Close note (2026-05-21)

S-8 closed in a single session. The factories are small (each
~10 lines of Rust on top of `cmmc_l3_baseline`) because S-6's
PolicyBundle type and S-6's `ActiveOverlays::add` API did the
load-bearing work; this sprint mostly composes them. The
runbooks took more lines than the factories — appropriate, since
the runbooks are what auditors read.

**Coverage update.** The harness now has typed, signed
PolicyBundle factories for 4 of the 6 v1.0 overlays:

| Overlay | Factory | Doctor checks pass? |
|---|---|---|
| CMMC-L3 baseline | ✅ S-8 | ✅ |
| FERPA | ✅ S-8 | ✅ |
| COPPA | ✅ S-8 | ✅ |
| CIPA | ✅ S-8 | ✅ |
| HIPAA / HITECH | ⏸ S-9 | — |
| FedRAMP-High | ⏸ S-9 | — |

**Pending follow-ups** in RETRO.

**Next sprint.** S-9 (Overlay bundles B — HIPAA + FedRAMP-High +
WORM/NFS-S3 audit sinks) is now unblocked.
