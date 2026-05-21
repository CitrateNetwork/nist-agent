---
created: 2026-05-21T00:00:00Z
branch: feat/s-10b-hitl-ui
author: Saul Loveman + Claude Opus 4.7 (1M context)
status: completed
sprint: S-10b
---

# Sprint S-10b — Retrospective

## Outcome

| Field | Value |
|---|---|
| **Goal achieved?** | YES (render models + Slint scaffold + install gate); wizard callback wiring + tokio integration deferred to S-10c / S-12 |
| **WPs planned / closed** | 6 / 4 in scope; 2 deferred |
| **Carry-forward WPs** | WP-10b.5 (wizard callback wiring), WP-10b.6 (tokio integration) |
| **Closing branch** | `feat/s-10b-hitl-ui` (PR pending) |

## Metrics delta

| Axis | Start | End | Δ |
|---|---|---|---|
| Tests | 110 | 121 | **+11** |
| Workspace crates | 9 | 10 | +1 (`nist-agent-hitl`) |
| ADRs | 8 | 9 | +1 |
| Frontmatter coverage | 100% | 100% | maintained |
| Drift constraints | 11/11 green | 11/11 green | 0 |

## What worked

- **The S-10a pattern transferred verbatim.** Headless render
  models + `feat-ui` Slint scaffold + build.rs gating worked
  identically here. ~half the time was the cargo / clippy
  iteration; the actual rendering logic took a small fraction.
- **The install gate as a typed-error property.**
  `check_install_gate` returns `Err(HitlUiError::InstallGateUnseenFields)`
  with field-count math; the test walks through every field
  and asserts the gate stays closed until the last one. The
  AC-6 audit artifact behavior is now code-pinned, not
  prose-pinned.
- **Defensive severity parsing.** Upstream's
  `PendingView.risk_level` is free-form String. Mapping it to
  a typed enum with an explicit `Unknown` fallback (instead of
  panicking on unrecognized strings) means future risk-level
  additions don't crash the UI — they just display under the
  "Unknown" badge until the mapping is updated.
- **Scroll-tracking via `Vec<InspectorField>` with stable
  ids.** The same id across snapshots means the UI doesn't
  lose viewed-state on a re-render. `mark_viewed` is
  idempotent and tolerates unknown ids — no panic surface for
  the Slint integration to trip over.

## What didn't work

- **First-pass `from_str` clippy lint.** Naming the severity
  parser `from_str` triggers `clippy::should_implement_trait`
  because `std::str::FromStr` has the same name. Renamed to
  `parse`; one-line fix. Note for future:
  small-vocabulary mappers like this aren't `FromStr` impls
  (they don't return `Result<_, _>`), so they shouldn't
  shadow the trait method.
- **`from_pending` keys on `name` instead of a stable id.**
  Upstream's `PendingView` doesn't expose the `call_id` —
  that's internal to `PendingEntry`. We key on `name` which
  matches in practice but isn't guaranteed unique across two
  proposals targeting the same capsule function. Documented;
  tracked as an upstream-extension follow-up.

## What surprised us

- **The Capsule Inspector fixture is the right S-10c starting
  point.** It carries all 12 RFC §8.3 fields in deterministic
  order. When S-10c authors the live conversion from a
  parsed capsule manifest, it'll land alongside this fixture
  as the parallel test fixture.
- **`SigningTierBadge` could have been the same enum as
  `nist-agent-policy::types::SigningTier`** (referenced in
  the policy bundle / RFC §4.4). They're not unified today —
  the policy crate's enum is the on-chain wire form; the
  hitl crate's is the UI badge. Worth a future unification
  follow-up alongside the `DataClass`/`Clearance` unification
  flagged in ADR-005.

## Lessons for future sprints

1. **AC-6 gates encode as typed properties cheaply.** The
   install-gate test is ~30 lines and pins a load-bearing
   compliance behavior. Future "operator must X before Y"
   gates (e.g. trajectory export confirmation, overlay
   decommissioning workflow ack) should reuse the
   `mark_viewed`/`check_gate` shape.
2. **`Vec<TypedRow>` + `Vec<TypedRowVisual>` conversion is
   the Slint integration pattern.** Same in S-10a (PolicyBundle
   factory → wizard panes) and S-10b (PendingView →
   ApprovalRow → ApprovalRowVisual). S-10c (marketplace) will
   produce a third instance.
3. **Multiple small enums beat one big enum.** `SigningTierBadge`
   (3 variants), `ApprovalRowSeverity` (5 variants),
   `Overlay` (6 variants in prelude) — each is small, each
   serializes cleanly. Unifying them when their byte
   encodings need to match (DataClass/Clearance) is a
   follow-up, not a sprint-time concern.

## Pending follow-ups (S-10c + future)

1. **Wire wizard's deferred callbacks** (S-10c). Mechanical
   wiring inside the unified Slint app shell.
2. **Live tokio integration** with `ApprovalQueue` (S-12
   distribution). Bridges the Slint Sign/Reject events to
   `approve()` / `reject()` async methods.
3. **Extend upstream `PendingView` with `current_signatures` +
   `roles_still_required`** so `from_pending` doesn't need the
   parallel `signatures` slice (S-12b upstream-PR alongside
   other accumulated upstream prep).
4. **Unify `SigningTier` (policy) and `SigningTierBadge` (hitl).**
   Same shape, different display layer. Worth a single
   canonical home — likely `nist-agent-prelude` or a future
   `nist-agent-types` crate.
5. **Polish on the .slint pane styling** — colors are
   minimal today; S-12 distribution-track work adds real
   Slint theming.

## Next sprint(s)

- **S-10c (Marketplace browser + chat surface)** — last of the
  S-10 sub-sprints. Closes the Slint app's v1.0 surface area
  and wires the wizard's deferred callbacks at the same time.
- S-11 (mobile companion) can run in parallel.
